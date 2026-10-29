#!/usr/bin/env bash
# Build and package the native host architecture. No signing identity is required.
set -euo pipefail

fail() { printf 'package-macos: %s\n' "$*" >&2; exit 1; }
if [[ $# -ne 0 ]]; then
    printf 'Usage: %s\nBuild a host-architecture Schematic.app and ZIP in rust/dist.\n' "$0" >&2
    exit 2
fi
[[ "$(uname -s)" == Darwin ]] || fail 'Run this packager on macOS.'
for command in cargo rustc codesign ditto plutil xcrun shasum unzip sips iconutil; do
    command -v "$command" >/dev/null 2>&1 || fail "Missing required command: $command"
done

script_dir="$(cd -P -- "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
native_root="$(cd -P -- "$script_dir/.." && pwd)"
project_root="$(cd -P -- "$native_root/.." && pwd)"
cd "$native_root"
manifest="$native_root/Cargo.toml"
# Isolate the packaging cache from interactive development unless overridden.
package_target_dir="${CARGO_TARGET_DIR:-$native_root/target/package-macos}"
metadata="$(CARGO_TARGET_DIR="$package_target_dir" cargo metadata --locked --no-deps --format-version 1 --manifest-path "$manifest")"
package_name="$(printf '%s' "$metadata" | plutil -extract packages.0.name raw -o - -)"
package_version="$(printf '%s' "$metadata" | plutil -extract packages.0.version raw -o - -)"
target_dir="$(printf '%s' "$metadata" | plutil -extract target_directory raw -o - -)"
[[ "$package_name" == schematic-native ]] || fail 'Expected the schematic-native package.'
host_target="$(rustc -vV | awk '$1 == "host:" { print $2 }')"
case "$host_target" in
    aarch64-apple-darwin|x86_64-apple-darwin) ;;
    *) fail "Unsupported native host target: $host_target" ;;
esac

# Preserve Cargo's encoded flags, or translate its whitespace-separated flags.
# Encoded arguments retain repository paths containing spaces as single arguments.
separator="$(printf '\037')"
build_flags="${CARGO_ENCODED_RUSTFLAGS-}"
if [[ -z "$build_flags" && -n "${RUSTFLAGS-}" ]]; then
    while IFS= read -r flag; do
        [[ -n "$flag" ]] || continue
        build_flags="${build_flags:+$build_flags$separator}$flag"
    done < <(printf '%s\n' "$RUSTFLAGS" | tr -s '[:space:]' '\n')
fi
if [[ -n "${HOME-}" ]]; then
    build_flags="${build_flags:+$build_flags$separator}--remap-path-prefix=$HOME=build-home"
fi
build_flags="${build_flags:+$build_flags$separator}--remap-path-prefix=$project_root=schematic"
CARGO_TARGET_DIR="$package_target_dir" CARGO_ENCODED_RUSTFLAGS="$build_flags" \
    cargo build --locked --release --manifest-path "$manifest" --bin "$package_name" --target "$host_target"
binary="$target_dir/$host_target/release/$package_name"
[[ -x "$binary" ]] || fail 'Cargo did not produce the expected executable.'

# This bundle does not ship external dynamic libraries. Fail rather than create
# a package that silently depends on a developer's package manager or home folder.
library_listing="$(xcrun otool -L "$binary")"
while IFS= read -r dependency; do
    case "$dependency" in
        /System/Library/*|/usr/lib/*|/Library/Apple/System/Library/*) ;;
        *) fail "Unbundled dynamic library dependency: $dependency" ;;
    esac
done < <(printf '%s\n' "$library_listing" | awk 'NR > 1 { print $1 }')
minimum_macos="$(xcrun vtool -show-build "$binary" | awk '$1 == "minos" { print $2; exit }')"
[[ "$minimum_macos" =~ ^[0-9]+\.[0-9]+(\.[0-9]+)?$ ]] || fail 'Cannot determine the Mach-O minimum macOS version.'
bundle_version="${package_version%%[-+]*}"
[[ "$bundle_version" =~ ^[0-9]+\.[0-9]+\.[0-9]+$ ]] || fail 'Cargo version is not a three-component bundle version.'

dist="$native_root/dist"
[[ ! -L "$dist" ]] || fail 'Refusing to write through a dist symlink.'
mkdir -p "$dist"
staging="$(mktemp -d "$dist/.package.XXXXXX")"
final_app="$dist/Schematic.app"
previous_app_moved=0
cleanup() {
    cleanup_status=$?
    trap - EXIT
    if [[ "$previous_app_moved" == 1 && ! -e "$final_app" && -d "$staging/previous.app" ]]; then
        mv "$staging/previous.app" "$final_app" || cleanup_status=1
    fi
    rm -rf -- "$staging"
    exit "$cleanup_status"
}
trap cleanup EXIT

app="$staging/Schematic.app"
mkdir -p "$app/Contents/MacOS" "$app/Contents/Resources"
cp "$binary" "$app/Contents/MacOS/$package_name"
chmod 755 "$app/Contents/MacOS/$package_name"
cp "$project_root/LICENSE" "$app/Contents/Resources/LICENSE"
cp "$native_root/README.md" "$app/Contents/Resources/README.md"
cp "$native_root/PARITY.md" "$app/Contents/Resources/PARITY.md"
icon_png="$native_root/assets/icon.png"
[[ -f "$icon_png" ]] || fail 'Missing rust/assets/icon.png; generate the native icon first.'
icon_dimensions="$(sips -g pixelWidth -g pixelHeight "$icon_png")"
icon_width="$(printf '%s\n' "$icon_dimensions" | awk '$1 == "pixelWidth:" { print $2 }')"
icon_height="$(printf '%s\n' "$icon_dimensions" | awk '$1 == "pixelHeight:" { print $2 }')"
[[ "$icon_width" =~ ^[0-9]+$ && "$icon_width" == "$icon_height" && "$icon_width" -gt 0 ]] || fail 'The native icon must be a square PNG.'
iconset="$staging/AppIcon.iconset"
mkdir -p "$iconset"
for size in 16 32 128 256 512; do
    for scale in 1 2; do
        pixels=$((size * scale))
        suffix=''
        [[ "$scale" == 1 ]] || suffix='@2x'
        sips -z "$pixels" "$pixels" "$icon_png" --out "$iconset/icon_${size}x${size}${suffix}.png" >/dev/null
    done
done
iconutil -c icns -o "$app/Contents/Resources/AppIcon.icns" "$iconset"
cat > "$app/Contents/Info.plist" <<PLIST
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0"><dict>
  <key>CFBundleDevelopmentRegion</key><string>en</string>
  <key>CFBundleExecutable</key><string>$package_name</string>
  <key>CFBundleIdentifier</key><string>dev.schematic.native</string>
  <key>CFBundleInfoDictionaryVersion</key><string>6.0</string>
  <key>CFBundleIconFile</key><string>AppIcon.icns</string>
  <key>CFBundleName</key><string>Schematic</string>
  <key>CFBundleDisplayName</key><string>Schematic</string>
  <key>CFBundlePackageType</key><string>APPL</string>
  <key>CFBundleShortVersionString</key><string>$bundle_version</string>
  <key>CFBundleVersion</key><string>$bundle_version</string>
  <key>LSApplicationCategoryType</key><string>public.app-category.developer-tools</string>
  <key>LSMinimumSystemVersion</key><string>$minimum_macos</string>
  <key>NSHighResolutionCapable</key><true/>
  <key>NSSupportsAutomaticGraphicsSwitching</key><true/>
  <key>NSPrincipalClass</key><string>NSApplication</string>
</dict></plist>
PLIST
plutil -lint "$app/Contents/Info.plist"
codesign --force --sign - --timestamp=none "$app"
codesign --verify --strict --verbose=2 "$app"

archive_name="$package_name-$package_version-$host_target.zip"
ditto -c -k --sequesterRsrc --keepParent "$app" "$staging/$archive_name"
unzip -tq "$staging/$archive_name"
# Check that ZIP extraction preserves the actual signed, executable app bundle.
ditto -x -k "$staging/$archive_name" "$staging/unpacked"
[[ -x "$staging/unpacked/Schematic.app/Contents/MacOS/$package_name" ]] || fail 'ZIP lost executable permissions.'
codesign --verify --strict --verbose=2 "$staging/unpacked/Schematic.app"
(cd "$staging" && shasum -a 256 "$archive_name" > "$archive_name.sha256")

for output in "$dist/$archive_name" "$dist/$archive_name.sha256"; do
    [[ ! -L "$output" && ! -d "$output" ]] || fail "Refusing to replace a symlink or directory: $output"
done
if [[ -e "$final_app" || -L "$final_app" ]]; then
    [[ -d "$final_app" && ! -L "$final_app" ]] || fail 'Refusing to replace a non-directory or symlink named Schematic.app.'
    existing_id="$(plutil -extract CFBundleIdentifier raw -o - "$final_app/Contents/Info.plist")"
    [[ "$existing_id" == dev.schematic.native ]] || fail 'Refusing to replace an unrelated application.'
fi
mv "$staging/$archive_name" "$dist/$archive_name"
mv "$staging/$archive_name.sha256" "$dist/$archive_name.sha256"
if [[ -d "$final_app" ]]; then
    mv "$final_app" "$staging/previous.app"
    previous_app_moved=1
fi
mv "$app" "$final_app"
printf 'App: %s\nZIP: %s\nChecksum: %s\nArchitecture: %s\nSigning: ad-hoc (not Developer ID signed or notarized)\n' \
    "$final_app" "$dist/$archive_name" "$dist/$archive_name.sha256" "$host_target"

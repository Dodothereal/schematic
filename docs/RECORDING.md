# Recording the walkthrough

The public video records the actual generated `dist/schematic.html` in an
isolated Chrome profile. It does not contain third-party footage, a user desktop
or real analytics. Narration uses the local macOS Samantha voice. Captions,
chapters and a transcript accompany the MP4.

The optional recording path needs macOS, Node 22.13+, Chrome, FFmpeg and FFprobe.
These are media-production tools, not application dependencies.

```sh
npm run build
node tools/record-walkthrough.mjs
node tools/make-watch-page.mjs
```

Set `SCHEMATIC_CHROME`, `FFMPEG` or `FFPROBE` to installed executables when needed.
`SCHEMATIC_VOICE` can select another installed macOS voice. The script never
downloads a voice, opens user browser tabs or sends media to an external service.

`tools/walkthrough-script.mjs` contains chapter copy. The recorder drives real
controls, captures browser frames with their timestamps, synthesizes narration,
encodes an H.264/AAC MP4 and writes WebVTT captions, chapter metadata and a
Markdown transcript. Intermediate frames and audio stay in ignored `scratch/`.
Published assets live in `docs/assets/`.

Review the encoded result at the beginning, middle and end, check every chapter
against the visible interaction, and verify audio, captions and playback before
publishing. The watch page supplies a native player, seekable chapters and links
to both the MP4 and written transcript. GitHub's README uses a clickable poster
so the video stays reachable even where inline video tags are stripped.

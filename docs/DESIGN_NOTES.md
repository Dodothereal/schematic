# A reference for the response

The v0.2 collection studies how an interface explains a change. Its sixteen new
specimens are original HTML, CSS and JavaScript implementations, informed by
[Animate UI's primitive catalog](https://animate-ui.com/docs/primitives) and
[motion principles](https://animate-ui.com/docs#motion-that-feels-right).

The clearest influence is continuity: a selected item travels, a group makes
room, and a disclosure opens in place. See Animate UI's documentation for
[pinned lists](https://animate-ui.com/docs/primitives/animate-ui/pinned-list),
[morphing text](https://animate-ui.com/docs/primitives/animate-ui/morphing-text),
[auto height](https://animate-ui.com/docs/primitives/animate-ui/auto-height), and
[navigation menus](https://animate-ui.com/docs/primitives/base-ui/navigation-menu).
Schematic keeps its own tokens, visual language and offline runtime. It does
not bundle Animate UI, React, Motion or another component library.

## The new collections

**Primitives** explores disclosure, pinning and rearrangement, text and number
changes, and surfaces that reveal another state. **Navigation** explores menus,
searchable choice, wayfinding, trees, pagination, docks, a validated flow and a
carousel. Each specimen has a complete interaction rather than an autoplay-only
animation. State remains when visiting another page.

The overview puts three interaction studies beside the introduction and derives
its searchable catalog directly from the actual specimens. A result opens and
focuses its working example. Counts and names do not depend on a second manual
inventory.

## Visual direction

An open page, strong type, fine rules and clear insets replace the oversized
introductory card and repeated dot-grid stages. Component previews use quiet
surfaces, with a distinct caption area. The shell keeps a readable rhythm at
desktop and narrow widths. Headlines have no gradient. Color and Mono each
support light and dark.

## Motion and access

The shared reorder helper measures displayed positions before cancelling an
interrupted animation. New movement starts from those positions. Direction,
disclosure and selection each use the system's own timing and curves. Reduced
motion applies the same state change directly. Hidden collections cancel their
work; controls remain operable with a keyboard.

Browser checks inspect actual intermediate frames, interrupted actions, hidden
content, focus, state retention and narrow layouts. These checks establish the
tested behavior; they do not claim parity with Animate UI's full catalog or
independent Safari, Firefox or VoiceOver certification.

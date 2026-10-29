# Implementation notes

The home page starts with a component assembly, followed by search and examples.
Names describe the component or its action. Instructions cover operation and keyboard controls; repeated intro
banners and promotional copy have been removed.

## Component assembly

Five HTML components sit on an SVG drafting grid. Assembled, exploded and
wireframe views share the same elements, so switching views moves the parts
instead of replacing them. Selection updates the inspector and example link.
Radius changes update the actual geometry. A mobile picker keeps each selected
component at readable size. Entry and chart tracing stop offscreen, on navigation
and under reduced motion. No rendering package or external asset is required.

## Catalog

The catalog reads names, descriptions and icons from the actual specimens.
Results open and focus the corresponding example. A full application example
can register its application frame as one specimen without a second page header.

## Dragging

`S.sortable()` moves the original DOM rows. Pointer handles support dragging,
edge scrolling and movement between groups. Space or Enter picks up and drops;
arrow keys move; Escape restores the complete previous order. Existing move
buttons remain available. Route changes and hidden documents cancel pickup and
owned animation work.

## Coding workspace

The workspace is a local simulation with three seeded conversations. Each keeps
its own messages, draft, changed files, check results, approval and merge state.
The editor occupies the center, with files on the left, chat on the right and
check logs in a bottom console. File tabs, syntax coloring and split diffs show
the code directly. Replies stream locally. Tool cards expose reads, edits and
check logs. The diff compares the actual before and after files.

The initial shortcut PR has a failing check. Rerunning it without a fix still
fails. Applying the fix changes the displayed patch and reruns all three checks.
A merge requires passing checks and approval, then confirmation. Changing code
invalidates approval. Stop, conversation changes and navigation cancel pending
simulation work. There is no AI, Git or CI service connection.

## Icons and motion

The 82-icon SVG registry is reused by shell controls, specimens, drag handles and
tool results. Icons have individual animations triggered by hover, focus and
state changes. Loader animations pause offscreen. Checkbox paths use the same
registry while their checked state controls the draw.

Motion uses shared timing and curves. Reduced motion applies the same final
state directly. Browser checks cover intermediate frames, cancellation, focus,
retained state, narrow layouts and cleanup.

## References

The original motion studies were informed by
[Animate UI's catalog](https://animate-ui.com/docs/primitives), especially
[pinned lists](https://animate-ui.com/docs/primitives/animate-ui/pinned-list),
[auto height](https://animate-ui.com/docs/primitives/animate-ui/auto-height), and
[navigation menus](https://animate-ui.com/docs/primitives/base-ui/navigation-menu).
Schematic's examples are original vanilla implementations; Animate UI, React
and Motion are not bundled.

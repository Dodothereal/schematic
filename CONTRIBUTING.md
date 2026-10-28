# Contributing

Schematic is a reference for how an interface should look, move and respond.
A contribution should improve that experience and keep the offline build intact.

1. Read [the design guide](DESIGN_GUIDE.md) and [the page contract](prototype/CONTRACT.md).
2. Open `prototype/index.html` directly in a browser. No install or server is needed.
3. Keep changes focused. Page work belongs in that page's JavaScript and scoped CSS.
4. Use the shared tokens. Support both appearances, Color and Mono palettes,
   keyboard focus, and reduced motion. Keep chart values readable during hover.
5. Rebuild and check before opening a pull request:

   ```sh
   npm run build
   npm run check
   npm run test:browser
   ```

Node 22+ is required for tooling. Browser checks use an isolated Chrome process;
set `SCHEMATIC_CHROME` if Chrome is installed outside the usual location.
There are no npm dependencies to install.

For visual changes, describe the trigger and resulting behavior. Include a short
recording when timing matters, and check an intermediate animation frame as well
as the resting state. Test a narrow screen and both motion preferences. Never
commit temporary captures, credentials, local paths or personal recovery notes.

Report a bug with the browser/version, appearance, palette, motion preference,
page and steps to reproduce. Mark simulated data or actions clearly. A framework
port should be described as supported only after it works in that runtime.

# Schematic contributor instructions

- Read `README.md`, `DESIGN_GUIDE.md` and `prototype/CONTRACT.md` before changes.
- Preserve the shared visual language, native semantics and offline operation.
- Pages own their scoped CSS and JavaScript; promote genuinely shared behavior
  into the runtime deliberately. Keep `render()` free of layout reads and timers.
- Use semantic tokens, meaningful motion and interruptible state changes.
  Hover must reveal intent or information. Do not add generic button lift/grow.
- Support light/dark, Color/Mono, keyboard access and reduced motion. Stop
  recurring work on hidden pages. Keep action timing independent of motion scale.
- Prefer final-geometry chart tracing; wait until the plot is visible. Keep axes
  and path construction out of animation frame loops.
- New UI must be tested in the actual browser, including intermediate frames
  where timing matters. A syntax check is not visual verification.
- Rebuild with `npm run build`; run `npm run check` and relevant browser checks.
  Do not edit generated token names or bundled HTML by hand.
- Do not publish scratch captures, credentials, local recovery history or personal
  file paths. Keep source and required public assets only.
- Do not use emojis in prose, UI, commit messages or documentation.

- Use literal component and action names. Keep instructions operational. Do not add slogans, abstract taglines, promotional claims or repeated intro text.

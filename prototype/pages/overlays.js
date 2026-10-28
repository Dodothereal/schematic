(() => {
  const S = window.Schematic;
  const instances = new WeakMap();
  const ic = (name, size = 18) => S.icon(name, { size, anim: false });
  const badge = (text, tone = "") => `<span class="badge ${tone ? `badge-${tone}` : ""}">${text}</span>`;
  const closeButton = (label = "Close dialog") => `<button type="button" class="btn btn-ghost btn-icon" data-ov-close aria-label="${label}">${ic("x")}</button>`;
  const dialog = (id, title, desc, body, cls = "") => `<dialog class="ov-dialog ${cls}" id="ov-${id}" aria-labelledby="ov-${id}-title" aria-describedby="ov-${id}-desc"><div class="ov-dialog-panel"><div class="ov-dialog-head"><div class="stack ov-tight"><span class="t-eyebrow">Interactive specimen</span><h2 class="t-h2" id="ov-${id}-title">${title}</h2></div>${closeButton()}</div><p class="muted" id="ov-${id}-desc">${desc}</p>${body}</div></dialog>`;
  const menuItem = (icon, label, action, extra = "") => `<button type="button" class="ov-menu-item" role="menuitem" tabindex="-1" data-menu-action="${action}" ${extra}>${ic(icon, 16)}<span>${label}</span></button>`;
  const accordion = (mode, items) => `<div class="ov-accordion" data-accordion="${mode}">${items.map(([title, desc], i) => `<div class="ov-acc-item"><h3><button type="button" class="ov-acc-trigger" id="ov-${mode}-trigger-${i}" aria-expanded="${i === 0}" aria-controls="ov-${mode}-panel-${i}"><span class="ov-acc-index t-mono">0${i + 1}</span><span>${title}</span>${ic("chevron-down", 16)}</button></h3><div class="ov-collapse ${i === 0 ? "is-open" : ""}" id="ov-${mode}-panel-${i}" role="region" aria-labelledby="ov-${mode}-trigger-${i}" ${i ? 'inert aria-hidden="true"' : ""}><div class="ov-collapse-inner"><p class="ov-acc-copy">${desc}</p></div></div></div>`).join("")}</div>`;
  const tones = [
    ["success", "check-circle", "Changes saved", "Your local specimen is ready to review."],
    ["info", "info", "A little context", "This message adds information without blocking your work."],
    ["warning", "alert", "Review before sharing", "Two reviewers still need access to this draft."],
    ["danger", "x-circle", "Upload interrupted", "This is a demo error. No files have been sent."],
  ];

  S.page({
    id: "overlays", title: "Overlays & Feedback", group: "Build", icon: "window", order: 70,
    description: "Dialogs, edge surfaces, menus and feedback that respect your attention.",
    render: () => `<div class="pg-overlays">
      ${S.pageHead({ icon: "window", eyebrow: "Build / 07", title: "Overlays & Feedback", lead: "A pause. A choice. A quiet confirmation. Give every interruption just enough space, and a clear way back.", meta: [badge("Native dialog", "accent"), badge("Keyboard first"), badge("Local simulations")] })}

      ${S.section({ id: "ov-dialogs", num: "01", title: "A deliberate pause", desc: "One decision at a time. The background recedes, focus stays inside, and closing returns you to the exact starting point.", body: `<div class="grid cols-2">
        ${S.demo({ cls: "ov-morph-demo span-all", title: "Origin morph", desc: "A project card becomes its detail surface. The return journey carries the same spatial context.", specs: ["spring-smooth", "origin → surface"], body: `<div class="ov-origin-scene"><div class="ov-scene-heading"><span class="t-eyebrow">Your workspace</span><span class="t-caption">Select a project to inspect</span></div><button type="button" class="ov-project" data-ov-open="project" aria-haspopup="dialog"><span class="ov-project-art" aria-hidden="true"><svg viewBox="0 0 320 120" fill="none"><path class="ov-art-grid" d="M0 30h320M0 60h320M0 90h320M40 0v120M80 0v120M120 0v120M160 0v120M200 0v120M240 0v120M280 0v120"/><rect class="ov-art-back" x="100" y="16" width="118" height="72" rx="8"/><rect class="ov-art-front" x="72" y="34" width="118" height="72" rx="8"/><path class="ov-art-lines" d="M88 54h50M88 66h84M88 78h64"/><path class="ov-art-accent" d="m163 84 7 7 13-15"/></svg><span class="ov-project-open">${ic("arrow-up-right", 16)}</span></span><span class="ov-project-info"><span><strong>Atlas workspace</strong><span class="t-caption">Design foundations · 12 components</span></span>${badge("In review", "accent")}</span></button><div class="ov-origin-aside"><span class="ov-symbol">${ic("layers", 22)}</span><h3 class="t-h3">Keep the connection.</h3><p class="t-small muted">The surface starts where your attention already is.</p><span class="ov-key-note"><kbd class="kbd">Enter</kbd> to open <span aria-hidden="true">/</span> <kbd class="kbd">Esc</kbd> to return</span></div></div>` })}
        ${S.demo({ title: "Modal dialog", desc: "A focused form with native validation and a persistent local result.", specs: ["ease-out", "focus trap"], body: `<div class="ov-mini-spec"><span class="ov-symbol">${ic("pencil", 22)}</span><strong data-workspace-name>Atlas workspace</strong><button type="button" class="btn btn-secondary" data-ov-open="rename" aria-haspopup="dialog">Rename workspace</button><span class="t-caption" data-rename-result>Edits stay in this page.</span></div>` })}
        ${S.demo({ title: "Destructive confirmation", desc: "The safe action receives focus first. Removal affects only this specimen.", specs: ["explicit choice", "reversible demo"], body: `<div class="ov-mini-spec"><span class="ov-symbol ov-danger">${ic("trash", 22)}</span><strong data-draft-label>Exploration draft</strong><div class="row"><button type="button" class="btn btn-secondary" data-ov-open="delete" aria-haspopup="dialog">Remove draft</button><button type="button" class="btn btn-ghost" data-restore-draft hidden>Restore draft</button></div><span class="t-caption" data-draft-state>1 local demo draft</span></div>` })}
      </div>` })}

      ${S.section({ id: "ov-surfaces", num: "02", title: "Room at the edges", desc: "Bring related work alongside the task. A drawer follows an edge; a sheet follows your hand.", body: `<div class="grid cols-2">
        ${S.demo({ title: "Side drawer", desc: "A full-height inspector with real controls. Apply a local preference or dismiss without committing.", specs: ["ease-out-expo", "edge anchored"], body: `<div class="ov-edge-preview" aria-hidden="true"><div class="ov-preview-lines"><span></span><span></span><span></span></div><div class="ov-preview-drawer">${ic("sliders")}<span></span><span></span></div></div><button type="button" class="btn btn-secondary" data-ov-open="drawer" aria-haspopup="dialog">Open inspector ${ic("sidebar", 16)}</button>` })}
        ${S.demo({ title: "Bottom sheet", desc: "Drag the handle down to dismiss, flick for velocity, or pull up against a soft rubberband. Escape and Close work too.", specs: ["spring-smooth", "velocity aware"], body: `<div class="ov-sheet-preview" aria-hidden="true"><span class="ov-preview-grip"></span><div>${ic("layers", 22)}<span>Quick actions</span></div></div><button type="button" class="btn btn-secondary" data-ov-open="sheet" aria-haspopup="dialog">Open bottom sheet ${ic("chevrons-up-down", 16)}</button>` })}
      </div>` })}

      ${S.section({ id: "ov-menus", num: "03", title: "Close to the action", desc: "Small, contextual surfaces. Arrow keys explore, Escape steps back, and focus always has a way home.", body: `<div class="grid cols-2 ov-flyouts">
        ${S.demo({ cls: "ov-pop-demo", title: "Dropdown + submenu", desc: "Up / Down cycle items. Right opens the submenu; Left returns. Type an initial to jump.", specs: ["roving focus", "collision aware"], body: `<div class="ov-mini-spec"><span class="t-eyebrow">Project actions</span><button type="button" class="btn btn-secondary" id="ov-actions-trigger" data-menu-trigger="actions" aria-haspopup="menu" aria-expanded="false" aria-controls="ov-actions-menu">Actions ${ic("chevron-down", 16)}</button><span class="ov-menu-result t-caption" data-menu-result role="status">Choose an action to try it.</span></div><div class="ov-menu" id="ov-actions-menu" role="menu" aria-labelledby="ov-actions-trigger" popover="manual">${menuItem("pencil", "Rename workspace", "rename")}${menuItem("copy", "Duplicate specimen", "duplicate")}<button type="button" class="ov-menu-item" role="menuitem" tabindex="-1" id="ov-export-trigger" data-submenu aria-haspopup="menu" aria-expanded="false" aria-controls="ov-export-menu">${ic("download", 16)}<span>Export as</span>${ic("chevron-right", 14)}</button><hr class="ov-menu-divider">${menuItem("trash", "Remove draft…", "delete")}</div><div class="ov-menu ov-submenu" id="ov-export-menu" role="menu" aria-labelledby="ov-export-trigger" popover="manual">${menuItem("code", "SVG markup", "svg")}${menuItem("document", "Design tokens", "tokens")}${menuItem("copy", "Component spec", "spec")}</div>` })}
        ${S.demo({ cls: "ov-pop-demo", title: "Context menu", desc: "Right-click the canvas, use Shift + F10, or use the visible menu button.", specs: ["pointer + keyboard", "focus return"], body: `<div class="ov-context-target" tabindex="0" role="group" aria-label="Atlas specimen canvas. Press Shift F10 for its context menu." data-context-target><span class="ov-context-cross" aria-hidden="true">+</span><span class="t-mono t-small">atlas / specimen.svg</span><span class="t-caption">Right-click anywhere in this canvas</span><button type="button" class="btn btn-secondary btn-sm" id="ov-context-trigger" data-menu-trigger="context" aria-haspopup="menu" aria-expanded="false" aria-controls="ov-context-menu">${ic("more", 16)} Open context menu</button></div><div class="ov-menu" id="ov-context-menu" role="menu" aria-label="Specimen actions" popover="manual">${menuItem("eye", "Inspect specimen", "inspect")}${menuItem("copy", "Duplicate specimen", "duplicate")}${menuItem("star", "Mark as favorite", "favorite")}</div>` })}
        ${S.demo({ cls: "ov-pop-demo", title: "Intent hovercard", desc: "Wait briefly on the name or focus it with Tab. The pointer can travel into the card; Escape dismisses it.", specs: ["intent delay", "focus equivalent"], body: `<div class="ov-byline"><span class="ov-avatar" aria-hidden="true">MC</span><div><p class="t-small">Reviewed by <button type="button" class="ov-person" id="ov-person" aria-haspopup="dialog" aria-controls="ov-person-card" aria-expanded="false">Maya Chen</button></p><span class="t-caption">Motion designer · example profile</span></div></div><div class="ov-hovercard" id="ov-person-card" role="dialog" aria-label="Maya Chen, example profile" popover="manual"><div class="row"><span class="ov-avatar ov-avatar-lg" aria-hidden="true">MC</span><div><strong>Maya Chen</strong><p class="t-caption">@maya · example profile</p></div></div><p class="t-small muted">Designing calm interfaces, one deliberate transition at a time.</p><div class="ov-profile-stats"><span><strong>24</strong> components</span><span><strong>8</strong> reviews</span></div><button type="button" class="btn btn-secondary btn-sm" data-profile-action>Mark review as read ${ic("check", 14)}</button></div>` })}
        ${S.demo({ cls: "ov-pop-demo", title: "Tooltip toolbar", desc: "Intent on hover; immediate on keyboard focus. Labels remain accessible without the tooltip.", specs: ["shared tooltip", "arrow navigation"], body: `<div class="ov-mini-spec"><div class="ov-toolbar" role="toolbar" aria-label="Specimen tools">${[["copy", "Duplicate", "duplicate"], ["star", "Favorite", "favorite"], ["link", "Inspect link", "link"], ["info", "Show details", "inspect"]].map(([icon, label, action], i) => `<button type="button" class="btn btn-ghost btn-icon" data-tool="${action}" aria-label="${label}" data-tip="${label}" tabindex="${i ? -1 : 0}" ${action === "favorite" ? 'aria-pressed="false"' : ""}>${ic(icon)}</button>`).join("")}</div><span class="t-caption" data-tool-result role="status">Tab in, then use Left / Right.</span></div>` })}
      </div>` })}

      ${S.section({ id: "ov-notifications", num: "04", title: "Acknowledge, then get out of the way", desc: "Tone carries meaning alongside an icon and a plain-language message. Toasts use the shared stack in the corner.", body: `<div class="grid cols-2">
        ${S.demo({ title: "Toast tones", desc: "Trigger several, then hover the stack to fan it out. Each notification has a dismiss control.", specs: ["shared stack", "polite status"], body: `<div class="ov-tone-buttons">${tones.map(([tone, icon, title]) => `<button type="button" class="btn btn-secondary" data-toast-tone="${tone}"><span class="ov-tone-dot ov-tone-${tone}">${ic(icon, 16)}</span>${tone[0].toUpperCase() + tone.slice(1)}</button>`).join("")}</div>` })}
        ${S.demo({ title: "Promise feedback", desc: "A deterministic local delay. Pending is dismissed before success appears; nothing is uploaded.", specs: ["pending → success", "simulated"], body: `<div class="ov-mini-spec"><span class="ov-symbol">${ic("upload", 22)}</span><button type="button" class="btn btn-primary" data-promise>Simulate save ${ic("arrow-right", 16)}</button><span class="t-caption" data-promise-result role="status">Local simulation · no network</span></div>` })}
      </div><div class="ov-alerts">${tones.map(([tone, icon, title, desc]) => `<div class="ov-alert ov-tone-${tone}"><span>${ic(icon)}</span><div><strong>${title}</strong><p>${desc}</p></div></div>`).join("")}</div>
      <div class="ov-banner-wrap"><div class="row-between"><span class="t-eyebrow">Collapsing announcement</span><button type="button" class="btn btn-ghost btn-sm" data-banner-restore aria-controls="ov-banner" aria-expanded="true">Replay banner ${ic("refresh", 14)}</button></div><div class="ov-collapse is-open" id="ov-banner"><div class="ov-collapse-inner"><div class="ov-banner"><span class="ov-symbol">${ic("sparkles")}</span><div><strong>A smaller interruption.</strong><p class="t-small muted">Dismiss this banner and the page closes the gap.</p></div><button type="button" class="btn btn-ghost btn-icon" data-banner-close aria-label="Dismiss announcement">${ic("x")}</button></div></div></div></div>` })}

      ${S.section({ id: "ov-disclosure", num: "05", title: "Reveal at the right pace", desc: "Content expands into the document rather than floating over it. The icon points the way; the layout makes room.", body: `<div class="grid cols-2">
        ${S.demo({ title: "Single accordion", desc: "Opening a new answer closes the previous one. Up / Down move between headings.", specs: ["grid 0fr → 1fr", "single open"], stage: "is-plain is-col", body: accordion("single", [["When should I use a dialog?", "Use a dialog when the current task needs a decision before work can continue. Keep its title specific and its primary action unambiguous."], ["Where does focus go?", "Move focus inside on open. For destructive actions, start with the safe choice. Return focus to the initiating control on close."], ["How much motion is enough?", "Enough to explain the change in context. Reduced motion preserves every state and skips the spatial journey."]]) })}
        ${S.demo({ title: "Multiple accordion", desc: "Answers remain open independently. Their state survives a trip to another page.", specs: ["grid 0fr → 1fr", "multiple open"], stage: "is-plain is-col", body: accordion("multi", [["Keyboard", "Tab reaches each heading. Enter or Space toggles an answer. Arrow keys, Home and End move between headings."], ["Motion", "The content track interpolates from zero to one fraction. Opacity follows the disclosure; reversing mid-transition stays smooth."], ["Semantics", "Each real button exposes its expanded state and controls a named region. Closed content is inert and excluded from navigation."]]) })}
        ${S.demo({ cls: "span-all", title: "Show-more disclosure", desc: "A compact summary with an optional deeper explanation. No content is fetched.", specs: ["inline expansion", "aria-expanded"], stage: "is-plain", body: `<div class="ov-disclosure"><div class="row"><span class="ov-symbol">${ic("eye")}</span><div><h3 class="t-h3">Quiet by default. Complete on demand.</h3><p class="t-small muted">Keep the essential information visible, and let the reader choose the rest.</p></div></div><div class="ov-collapse" id="ov-more" inert aria-hidden="true"><div class="ov-collapse-inner"><p class="ov-more-copy">A disclosure belongs in the reading flow. It can hold rationale, technical notes, or a longer explanation without asking the reader to change context. The control remains in place, keeps focus, and describes its next action.</p></div></div><button type="button" class="btn btn-ghost btn-sm" data-show-more aria-expanded="false" aria-controls="ov-more"><span>Show more</span>${ic("chevron-down", 14)}</button></div>` })}
      </div>` })}

      ${S.section({ id: "ov-loading", num: "06", title: "Make waiting legible", desc: "A shaped skeleton reserves the final layout. Deterministic progress tells you what is happening, with no invented network activity.", body: S.demo({ cls: "ov-loading-demo", title: "Skeleton → content", desc: "Eight-second local simulation at 1×. Pause, replay, or change speed. Leaving this page or hiding the tab pauses it.", specs: ["deterministic", "no network"], stage: "is-plain", replay: true, attrs: 'data-loading-demo', body: `<div class="ov-loading-layout"><div class="ov-loading-preview" aria-busy="true" aria-label="Simulated project preview"><div class="ov-skeleton" aria-hidden="true"><div class="ov-skeleton-art"></div><div class="ov-skeleton-line ov-w-60"></div><div class="ov-skeleton-line ov-w-90"></div><div class="ov-skeleton-line ov-w-75"></div><div class="ov-skeleton-footer"><span></span><span></span></div></div><div class="ov-loaded" hidden><div class="ov-loaded-art">${ic("check-circle", 40)}<span class="t-mono">ATLAS / 001</span></div><strong>Atlas workspace</strong><p class="t-small muted">12 components. One coherent system.</p><div class="row">${badge("Ready", "success")}${badge("Local preview")}</div></div></div><div class="ov-progress-panel"><div class="row-between"><span class="t-eyebrow">Preview pipeline</span><span class="badge">Simulation</span></div><div class="ov-progress-heading"><strong data-progress-label>Ready when you are</strong><output class="t-num" data-progress-percent aria-hidden="true">0%</output></div><label class="ov-sr-only" for="ov-progress">Simulated preview progress</label><progress class="ov-sr-only" id="ov-progress" max="100" value="0">0%</progress><div class="ov-progress-track" aria-hidden="true"><span></span></div><ol class="ov-progress-steps">${["Read specimen", "Resolve tokens", "Compose preview"].map((name, i) => `<li data-step="${i}"><span class="ov-step-marker">${ic("check", 12)}<span>${i + 1}</span></span>${name}</li>`).join("")}</ol><div class="ov-loading-controls"><button type="button" class="btn btn-primary btn-sm" data-progress-toggle>Run simulation</button><button type="button" class="btn btn-secondary btn-sm" data-progress-replay aria-label="Replay loading simulation">${ic("refresh", 14)} Replay</button><label class="ov-speed-label"><span class="ov-sr-only">Simulation speed</span><select data-progress-speed><option value="0.5">0.5×</option><option value="1" selected>1×</option><option value="2">2×</option></select></label></div><p class="t-caption" data-progress-status role="status">Idle · press Run simulation to begin.</p></div></div>` }) })}

      <div class="ov-sr-only" role="status" aria-live="polite" data-ov-status></div>
      ${dialog("rename", "Rename workspace", "Choose a name that your team will recognize. This changes only the local demo.", `<form data-rename-form class="stack"><label class="ov-field">Workspace name<input name="workspace" required maxlength="40" value="Atlas workspace" autocomplete="off" autofocus></label><span class="t-caption">1–40 characters. Your edits stay here when you return.</span><div class="ov-dialog-actions"><button type="button" class="btn btn-secondary" data-ov-close>Cancel</button><button type="submit" class="btn btn-primary">Save name</button></div></form>`)}
      ${dialog("delete", "Remove exploration draft?", "This removes the example draft from this page. You can restore it immediately afterward.", `<div class="ov-delete-summary">${ic("document", 24)}<div><strong>Exploration draft</strong><p class="t-caption">Local specimen · 1 draft · no remote files</p></div></div><div class="ov-dialog-actions"><button type="button" class="btn btn-secondary" data-ov-close autofocus>Keep draft</button><button type="button" class="btn btn-danger" data-delete-confirm>${ic("trash", 16)} Remove draft</button></div>`, "ov-dialog-danger")}
      ${dialog("project", "Atlas workspace", "A reference kit for interfaces that feel calm, connected, and clear.", `<div class="ov-project-detail-art" aria-hidden="true">${ic("layers", 48)}<span class="t-mono">CONTEXT / CONTINUITY / CLARITY</span></div><div class="ov-detail-grid"><div><span class="t-eyebrow">Components</span><strong>12</strong></div><div><span class="t-eyebrow">Status</span><strong>In review</strong></div><div><span class="t-eyebrow">Updated</span><strong>Today</strong></div></div><p class="t-small muted">The small card and this surface share an origin. Closing returns the surface to the card, preserving your place in the workspace.</p><div class="ov-dialog-actions"><button type="button" class="btn btn-primary" data-ov-close>Back to workspace ${ic("arrow-left", 16)}</button></div>`, "ov-dialog-morph")}
      ${dialog("drawer", "Project inspector", "Edit a local preview preference. Your saved choice stays with this page.", `<form class="ov-drawer-form" data-inspector-form><div class="ov-inspector-section"><span class="t-eyebrow">Workspace</span><div class="ov-delete-summary">${ic("folder", 24)}<div><strong data-workspace-name>Atlas workspace</strong><p class="t-caption">12 components · design system</p></div></div></div><fieldset class="ov-inspector-section"><legend class="t-eyebrow">Preview density</legend><label class="ov-radio"><input type="radio" name="density" value="Comfortable" checked><span><strong>Comfortable</strong><small>More room to scan and compare.</small></span></label><label class="ov-radio"><input type="radio" name="density" value="Compact"><span><strong>Compact</strong><small>More components in the same space.</small></span></label></fieldset><label class="ov-checkbox"><input type="checkbox" name="annotations" checked>Show motion annotations</label><p class="t-caption" data-inspector-result>Current: Comfortable · annotations on</p><div class="ov-dialog-actions"><button type="button" class="btn btn-secondary" data-ov-close>Cancel</button><button type="submit" class="btn btn-primary">Apply to demo</button></div></form>`, "ov-dialog-drawer")}
      ${dialog("sheet", "Quick actions", "A tactile surface with a keyboard alternative. Drag down or flick the handle; pull up to feel resistance.", `<button type="button" class="ov-sheet-handle" data-sheet-handle aria-label="Close bottom sheet, or drag this handle down"><span></span></button><div class="ov-sheet-options">${[["copy", "Duplicate specimen", "Keep the original, make a local copy.", "duplicate"], ["star", "Mark as favorite", "A little emphasis for this example.", "favorite"], ["eye", "Inspect details", "Read the project without leaving the page.", "inspect"]].map(([icon, title, desc, action]) => `<button type="button" class="ov-sheet-option" data-sheet-action="${action}"><span class="ov-symbol">${ic(icon, 20)}</span><span><strong>${title}</strong><small>${desc}</small></span>${ic("chevron-right", 16)}</button>`).join("")}</div><div class="ov-dialog-actions"><span class="t-caption">Drag handle · Escape · Close</span><button type="button" class="btn btn-secondary" data-ov-close autofocus>Close sheet</button></div>`, "ov-dialog-sheet")}
    </div>`,

    mount(root) {
      const page = root.querySelector(".pg-overlays");
      const $ = (sel) => page.querySelector(sel);
      const $$ = (sel, context = page) => [...context.querySelectorAll(sel)];
      const state = { active: false, timers: new Set(), animations: new Set(), dialogs: new Map(), menus: [], toasts: new Set(), raf: 0, running: false, elapsed: 0, lastTime: 0, favorite: false, copies: 0, savedDensity: "Comfortable", savedAnnotations: true, drag: null, suppressSheetClick: false };
      const announce = (message) => { $("[data-ov-status]").textContent = message; };
      const later = (fn, duration) => {
        const timer = setTimeout(() => { state.timers.delete(timer); if (state.active && !document.hidden) fn(); }, S.dur(duration));
        state.timers.add(timer); return timer;
      };
      const clear = (timer) => { clearTimeout(timer); state.timers.delete(timer); };
      const animate = (el, frames, duration = 460, easing = S.curve.outExpo) => {
        const animation = el.animate(frames, { duration: S.reduced() ? 0 : S.dur(duration), easing });
        state.animations.add(animation);
        animation.finished.catch(() => {}).finally(() => state.animations.delete(animation));
        return animation;
      };
      const toast = (options) => {
        const handle = S.toast(options); state.toasts.add(handle); return handle;
      };
      const setCollapse = (panel, open) => {
        panel.classList.toggle("is-open", open); panel.inert = !open;
        panel.setAttribute("aria-hidden", String(!open));
      };

      // Native modality supplies focus containment; all dialog content stays under this page.
      const closeDialog = (dlg, immediate = false, restore = true) => {
        const record = state.dialogs.get(dlg);
        if (!dlg.open || record?.closing) return;
        const panel = dlg.firstElementChild;
        const currentTransform = getComputedStyle(panel).transform;
        const currentOpacity = getComputedStyle(panel).opacity;
        if (dlg.id === "ov-sheet" && state.drag) resetDrag();
        if (record) { record.closing = true; record.animation?.cancel(); }
        const finish = () => {
          if (!dlg.open || state.dialogs.get(dlg) !== record) return;
          state.dialogs.delete(dlg); dlg.close(); dlg.removeAttribute("data-closing");
          panel.style.transform = ""; panel.style.opacity = "";
          if (restore && state.active && !document.hidden && record?.origin?.isConnected) record.origin.focus({ preventScroll: true });
        };
        if (immediate || S.reduced()) { finish(); return; }
        dlg.setAttribute("data-closing", "");
        let target = "scale(.97)";
        if (dlg.classList.contains("ov-dialog-drawer")) target = "translateX(100%)";
        if (dlg.classList.contains("ov-dialog-sheet")) target = "translateY(110%)";
        if (dlg.classList.contains("ov-dialog-morph") && record.origin?.isConnected) {
          const a = record.origin.getBoundingClientRect(), b = panel.getBoundingClientRect();
          target = `translate(${a.left + a.width / 2 - b.left - b.width / 2}px, ${a.top + a.height / 2 - b.top - b.height / 2}px) scale(${a.width / b.width}, ${a.height / b.height})`;
        }
        const animation = animate(panel, [{ transform: currentTransform, opacity: currentOpacity }, { transform: target, opacity: 0 }], 320, S.curve.in);
        if (record) record.animation = animation;
        animation.finished.then(finish).catch(() => {});
      };
      const openDialog = (name, origin) => {
        if (!state.active || document.hidden) return;
        closeMenus(false); hideCard();
        const dlg = $(`#ov-${name}`);
        if (!dlg || dlg.open) return;
        if (name === "drawer") {
          $(`[name="density"][value="${state.savedDensity}"]`).checked = true;
          $('[name="annotations"]').checked = state.savedAnnotations;
        }
        const record = { origin: origin || document.activeElement, closing: false };
        state.dialogs.set(dlg, record); dlg.showModal();
        const panel = dlg.firstElementChild;
        let from = "translateY(16px) scale(.97)";
        if (name === "drawer") from = "translateX(100%)";
        if (name === "sheet") from = "translateY(100%)";
        if (name === "project" && origin && !S.reduced()) {
          const a = origin.getBoundingClientRect(), b = panel.getBoundingClientRect();
          from = `translate(${a.left + a.width / 2 - b.left - b.width / 2}px, ${a.top + a.height / 2 - b.top - b.height / 2}px) scale(${a.width / b.width}, ${a.height / b.height})`;
        }
        record.animation = animate(panel, S.reduced() ? [{ opacity: 0 }, { opacity: 1 }] : [{ opacity: 0, transform: from }, { opacity: 1, transform: "none" }], name === "project" ? 600 : 460, name === "project" ? S.curve.springSmooth : S.curve.outExpo);
        const focus = dlg.querySelector("[autofocus]") || dlg.querySelector("button");
        focus?.focus({ preventScroll: true });
        if (focus?.matches("input")) focus.select();
      };
      $$(".ov-dialog").forEach((dlg) => {
        dlg.addEventListener("cancel", (event) => { event.preventDefault(); closeDialog(dlg); });
        dlg.addEventListener("keydown", (event) => {
          if (event.key !== "Tab") return;
          const controls = $$("button:not(:disabled), input:not(:disabled), select:not(:disabled), textarea:not(:disabled), a[href], [tabindex]:not([tabindex='-1'])", dlg).filter((el) => el.getClientRects().length && !el.closest("[inert]"));
          const first = controls[0], last = controls.at(-1);
          if (event.shiftKey && document.activeElement === first) { event.preventDefault(); last?.focus(); }
          else if (!event.shiftKey && document.activeElement === last) { event.preventDefault(); first?.focus(); }
        });
        dlg.addEventListener("click", (event) => {
          if (event.target !== dlg) return;
          if (dlg.id === "ov-sheet" && state.suppressSheetClick && event.detail) { state.suppressSheetClick = false; return; }
          closeDialog(dlg);
        });
        dlg.addEventListener("close", () => { if (dlg.open) return; state.dialogs.get(dlg)?.animation?.cancel(); state.dialogs.delete(dlg); if (dlg.id === "ov-sheet") resetDrag(); });
      });
      page.addEventListener("click", (event) => {
        const opener = event.target.closest("[data-ov-open]");
        if (opener) openDialog(opener.dataset.ovOpen, opener);
        const closer = event.target.closest("[data-ov-close]");
        if (closer) closeDialog(closer.closest("dialog"));
      });
      $("[data-rename-form]").addEventListener("submit", (event) => {
        event.preventDefault();
        const input = event.currentTarget.elements.workspace;
        if (!input.value.trim()) { input.setCustomValidity("Enter a workspace name."); input.reportValidity(); return; }
        const name = input.value.trim(); input.value = name;
        $$("[data-workspace-name]").forEach((el) => { el.textContent = name; });
        $(".ov-project-info strong").textContent = name; $("#ov-project-title").textContent = name;
        $(".ov-loaded > strong").textContent = name;
        $("[data-rename-result]").textContent = "Name saved in this specimen.";
        closeDialog($("#ov-rename")); announce(`Workspace renamed to ${name}.`);
      });
      $('[name="workspace"]').addEventListener("input", (event) => event.target.setCustomValidity(""));
      $("[data-delete-confirm]").addEventListener("click", () => {
        $("[data-draft-label]").textContent = "Draft removed";
        $("[data-draft-state]").textContent = "No demo drafts · restore anytime";
        $('[data-ov-open="delete"]').hidden = true; $("[data-restore-draft]").hidden = false;
        const dlg = $("#ov-delete"); state.dialogs.get(dlg).origin = $("[data-restore-draft]");
        closeDialog(dlg); announce("Example draft removed. Restore draft is available.");
      });
      $("[data-restore-draft]").addEventListener("click", () => {
        $("[data-draft-label]").textContent = "Exploration draft"; $("[data-draft-state]").textContent = "1 local demo draft";
        $('[data-ov-open="delete"]').hidden = false; $("[data-restore-draft]").hidden = true;
        $('[data-ov-open="delete"]').focus(); announce("Example draft restored.");
      });
      $("[data-inspector-form]").addEventListener("submit", (event) => {
        event.preventDefault(); const form = event.currentTarget;
        state.savedDensity = form.elements.density.value; state.savedAnnotations = form.elements.annotations.checked;
        const text = `${state.savedDensity} · annotations ${state.savedAnnotations ? "on" : "off"}`;
        $("[data-inspector-result]").textContent = `Current: ${text}`;
        closeDialog($("#ov-drawer")); announce(`Inspector preference saved: ${text}.`);
      });

      // A native manual popover escapes demo clipping while retaining page ownership.
      const showFloating = (panel) => {
        panel.getAnimations().forEach((animation) => animation.cancel());
        panel.inert = false; panel.setAttribute("data-open", "");
        if (panel.showPopover && !panel.matches(":popover-open")) panel.showPopover();
      };
      const hideFloating = (panel, immediate = false) => {
        panel.inert = true;
        const finish = () => {
          if (!panel.inert) return;
          if (panel.hidePopover && panel.matches(":popover-open")) panel.hidePopover();
          panel.removeAttribute("data-open");
        };
        panel.getAnimations().forEach((animation) => animation.cancel());
        if (immediate || S.reduced()) finish();
        else animate(panel, [{ opacity: 1, transform: "none" }, { opacity: 0, transform: "translateY(-4px)" }], 140, S.curve.in).finished.then(finish).catch(() => {});
      };
      const placeFloating = (panel, anchor, point, submenu = false) => {
        const gap = parseFloat(S.token("--s-2")), inset = parseFloat(S.token("--s-3"));
        const a = anchor.getBoundingClientRect(), b = panel.getBoundingClientRect();
        let x = point ? point.x : submenu ? a.right + gap : a.left;
        let y = point ? point.y : submenu ? a.top : a.bottom + gap;
        if (submenu && x + b.width > innerWidth - inset) x = a.left - b.width - gap;
        if (!point && !submenu && y + b.height > innerHeight - inset) y = a.top - b.height - gap;
        x = S.clamp(x, inset, Math.max(inset, innerWidth - b.width - inset));
        y = S.clamp(y, inset, Math.max(inset, innerHeight - b.height - inset));
        panel.style.left = `${x}px`; panel.style.top = `${y}px`;
        panel.style.transformOrigin = `${S.clamp(a.left + a.width / 2 - x, 0, b.width)}px ${y < a.top ? "100%" : "0"}`;
      };
      const menuItems = (panel) => $$("[role=menuitem]", panel).filter((el) => !el.disabled);
      const closeMenus = (restore = true, immediate = false) => {
        const origin = state.menus[0]?.trigger;
        state.menus.splice(0).reverse().forEach(({ panel, trigger }) => { hideFloating(panel, immediate); trigger.setAttribute("aria-expanded", "false"); });
        if (restore && state.active && origin) origin.focus({ preventScroll: true });
      };
      const closeSubmenu = () => {
        if (state.menus.length < 2) return;
        const { panel, trigger } = state.menus.pop(); hideFloating(panel); trigger.setAttribute("aria-expanded", "false"); trigger.focus();
      };
      const openMenu = (name, trigger, point, last = false) => {
        closeMenus(false, true); hideCard();
        const panel = $(`#ov-${name}-menu`);
        showFloating(panel); placeFloating(panel, trigger, point);
        state.menus.push({ panel, trigger }); trigger.setAttribute("aria-expanded", "true");
        animate(panel, [{ opacity: 0, transform: S.reduced() ? "none" : "translateY(-4px) scale(.97)" }, { opacity: 1, transform: "none" }], 320);
        const items = menuItems(panel); (last ? items.at(-1) : items[0])?.focus({ preventScroll: true });
      };
      const openSubmenu = () => {
        if (state.menus.length > 1) { menuItems(state.menus[1].panel)[0]?.focus(); return; }
        const panel = $("#ov-export-menu"), trigger = $("[data-submenu]");
        showFloating(panel); placeFloating(panel, trigger, null, true);
        state.menus.push({ panel, trigger }); trigger.setAttribute("aria-expanded", "true");
        animate(panel, [{ opacity: 0 }, { opacity: 1 }], 220); menuItems(panel)[0]?.focus();
      };
      const runAction = (action, origin) => {
        if (action === "rename" || action === "delete") {
          if (action === "delete" && $('[data-ov-open="delete"]').hidden) { announce("The example draft is already removed. Use Restore draft."); return; }
          openDialog(action, origin); return;
        }
        if (action === "inspect") { openDialog("project", origin); return; }
        let message;
        if (action === "duplicate") message = `Local copy ${++state.copies} created for this interaction demo.`;
        else if (action === "favorite") {
          state.favorite = !state.favorite; message = state.favorite ? "Specimen marked as favorite." : "Favorite removed.";
          $('[data-tool="favorite"]').setAttribute("aria-pressed", String(state.favorite));
        } else if (action === "link") message = "Example path: atlas / specimen.svg. No link was shared.";
        else message = `${({ svg: "SVG markup", tokens: "Design tokens", spec: "Component spec" })[action]} selected. Export is simulated; no file was written.`;
        $("[data-menu-result]").textContent = message; $("[data-tool-result]").textContent = message; announce(message);
      };
      $$("[data-menu-trigger]").forEach((trigger) => {
        trigger.addEventListener("click", () => {
          state.menus[0]?.trigger === trigger ? closeMenus() : openMenu(trigger.dataset.menuTrigger, trigger);
        });
        trigger.addEventListener("keydown", (event) => {
          if (event.key === "ArrowDown" || event.key === "ArrowUp") { event.preventDefault(); openMenu(trigger.dataset.menuTrigger, trigger, null, event.key === "ArrowUp"); }
        });
      });
      const context = $("[data-context-target]");
      context.setAttribute("aria-haspopup", "menu"); context.setAttribute("aria-controls", "ov-context-menu");
      context.addEventListener("contextmenu", (event) => { event.preventDefault(); openMenu("context", event.target.closest("button") || context, { x: event.clientX, y: event.clientY }); });
      context.addEventListener("keydown", (event) => {
        if ((event.shiftKey && event.key === "F10") || event.key === "ContextMenu") { event.preventDefault(); openMenu("context", context); }
      });
      $("[data-submenu]").addEventListener("click", openSubmenu);
      $$("[data-menu-action]").forEach((item) => item.addEventListener("click", () => {
        const origin = state.menus[0]?.trigger; closeMenus(true); runAction(item.dataset.menuAction, origin);
      }));
      page.addEventListener("keydown", (event) => {
        const menu = event.target.closest(".ov-menu");
        if (!menu || !state.menus.length) return;
        const items = menuItems(menu), index = items.indexOf(document.activeElement);
        const key = event.key;
        if (["ArrowDown", "ArrowUp", "Home", "End"].includes(key)) {
          event.preventDefault();
          const next = key === "Home" ? 0 : key === "End" ? items.length - 1 : (index + (key === "ArrowDown" ? 1 : -1) + items.length) % items.length;
          items[next]?.focus();
        } else if (key === "ArrowRight" && event.target.closest("[data-submenu]")) { event.preventDefault(); openSubmenu(); }
        else if (key === "ArrowLeft" && state.menus.length > 1) { event.preventDefault(); closeSubmenu(); }
        else if (key === "Escape") { event.preventDefault(); event.stopPropagation(); state.menus.length > 1 ? closeSubmenu() : closeMenus(); }
        else if (key === "Tab") closeMenus();
        else if (key.length === 1 && !event.metaKey && !event.ctrlKey && /\S/.test(key)) {
          const next = items.slice(index + 1).concat(items.slice(0, index + 1)).find((item) => item.textContent.trim().toLowerCase().startsWith(key.toLowerCase()));
          if (next) { event.preventDefault(); next.focus(); }
        }
      });

      // Hover intent has equivalent focus/click entry and a short pointer-exit grace period.
      const person = $("#ov-person"), card = $("#ov-person-card");
      let cardTimer, cardOpen = false, suppressCardFocus = false;
      const hideCard = (immediate = false) => {
        clear(cardTimer); if (!cardOpen) return;
        cardOpen = false; person.setAttribute("aria-expanded", "false"); hideFloating(card, immediate);
      };
      const showCard = () => {
        clear(cardTimer); if (!state.active || document.hidden || cardOpen) return;
        closeMenus(false); cardOpen = true; person.setAttribute("aria-expanded", "true");
        showFloating(card); placeFloating(card, person);
        animate(card, [{ opacity: 0, transform: S.reduced() ? "none" : "translateY(6px)" }, { opacity: 1, transform: "none" }], 320);
      };
      [person, card].forEach((el) => {
        el.addEventListener("pointerenter", (event) => { if (event.pointerType === "touch") return; clear(cardTimer); if (!cardOpen) cardTimer = later(showCard, 450); });
        el.addEventListener("pointerleave", () => { clear(cardTimer); cardTimer = later(() => { if (!card.contains(document.activeElement) && document.activeElement !== person) hideCard(); }, 180); });
        el.addEventListener("focusout", (event) => { if (!card.contains(event.relatedTarget) && event.relatedTarget !== person) hideCard(); });
      });
      person.addEventListener("focus", () => { if (suppressCardFocus) suppressCardFocus = false; else showCard(); });
      person.addEventListener("click", showCard);
      $("[data-profile-action]").addEventListener("click", () => { hideCard(); suppressCardFocus = true; person.focus(); announce("Maya's example review marked as read."); });
      page.addEventListener("keydown", (event) => {
        if (event.key === "Escape" && cardOpen && (event.target === person || card.contains(event.target))) {
          event.preventDefault(); event.stopPropagation(); hideCard(); suppressCardFocus = document.activeElement !== person; person.focus();
        }
      });
      const outside = (event) => {
        if (!state.active) return;
        state.suppressSheetClick = false;
        if (state.menus.length && !state.menus.some(({ panel, trigger }) => panel.contains(event.target) || trigger.contains(event.target))) closeMenus(false);
        if (cardOpen && !person.contains(event.target) && !card.contains(event.target)) hideCard();
      };
      document.addEventListener("pointerdown", outside);
      const onGeometryChange = (event) => {
        if (!state.active || event.target?.closest?.(".ov-menu, .ov-hovercard")) return;
        closeMenus(false, true); hideCard(true);
      };
      document.addEventListener("scroll", onGeometryChange, true); window.addEventListener("resize", onGeometryChange);
      const toolbar = $(".ov-toolbar");
      toolbar.addEventListener("keydown", (event) => {
        if (!["ArrowRight", "ArrowLeft", "Home", "End"].includes(event.key)) return;
        event.preventDefault(); const buttons = $$("button", toolbar), index = buttons.indexOf(document.activeElement);
        const next = event.key === "Home" ? 0 : event.key === "End" ? buttons.length - 1 : (index + (event.key === "ArrowRight" ? 1 : -1) + buttons.length) % buttons.length;
        buttons.forEach((button, i) => { button.tabIndex = i === next ? 0 : -1; }); buttons[next].focus();
      });
      $$("[data-tool]").forEach((button) => button.addEventListener("click", () => runAction(button.dataset.tool, button)));

      // Drag only the explicit handle, so sheet scrolling and buttons retain native behavior.
      const handle = $("[data-sheet-handle]"), sheet = $("#ov-sheet"), sheetPanel = sheet.firstElementChild;
      const resetDrag = () => {
        const drag = state.drag; state.drag = null;
        if (drag && handle.hasPointerCapture(drag.id)) handle.releasePointerCapture(drag.id);
        sheetPanel.classList.remove("is-dragging"); sheetPanel.style.transform = "";
      };
      const springBack = () => {
        const transform = sheetPanel.style.transform; resetDrag();
        animate(sheetPanel, [{ transform }, { transform: "none" }], 600, S.curve.springSmooth);
      };
      handle.addEventListener("pointerdown", (event) => {
        if (event.button !== 0 || state.dialogs.get(sheet)?.closing) return;
        state.dialogs.get(sheet)?.animation?.cancel();
        state.drag = { id: event.pointerId, startY: event.clientY, y: 0, samples: [{ y: event.clientY, time: event.timeStamp }], moved: false };
        handle.setPointerCapture(event.pointerId); sheetPanel.classList.add("is-dragging");
      });
      document.addEventListener("pointermove", (event) => {
        const drag = state.drag; if (!drag || drag.id !== event.pointerId) return;
        const dy = event.clientY - drag.startY;
        drag.moved ||= Math.abs(dy) > 4; drag.y = dy;
        drag.samples.push({ y: event.clientY, time: event.timeStamp });
        while (drag.samples.length > 2 && event.timeStamp - drag.samples[0].time > 100) drag.samples.shift();
        const limit = parseFloat(S.token("--s-12"));
        const y = dy < 0 ? -limit * (1 - Math.exp(dy / (limit * 2))) : dy;
        sheetPanel.style.transform = S.reduced() ? "none" : `translateY(${y}px)`;
      });
      document.addEventListener("pointerup", (event) => {
        const drag = state.drag; if (!drag || drag.id !== event.pointerId) return;
        const first = drag.samples[0], velocity = (event.clientY - first.y) / Math.max(1, event.timeStamp - first.time);
        state.suppressSheetClick = drag.moved;
        if (drag.y > sheetPanel.offsetHeight * 0.28 || (drag.y > 16 && velocity > 0.65)) closeDialog(sheet);
        else springBack();
      });
      document.addEventListener("pointercancel", (event) => { if (state.drag?.id === event.pointerId) { state.suppressSheetClick = true; springBack(); } });
      handle.addEventListener("lostpointercapture", () => { if (state.drag) springBack(); });
      handle.addEventListener("click", (event) => { if (state.suppressSheetClick && event.detail) { state.suppressSheetClick = false; return; } closeDialog(sheet); });
      $$("[data-sheet-action]").forEach((button) => button.addEventListener("click", () => {
        const action = button.dataset.sheetAction, origin = state.dialogs.get(sheet)?.origin;
        closeDialog(sheet, true); runAction(action, origin);
      }));

      $$("[data-accordion]").forEach((group) => {
        const triggers = $$(".ov-acc-trigger", group);
        triggers.forEach((trigger) => trigger.addEventListener("click", () => {
          const opening = trigger.getAttribute("aria-expanded") !== "true";
          triggers.forEach((other) => {
            if (other !== trigger && group.dataset.accordion !== "single") return;
            const open = other === trigger && opening;
            other.setAttribute("aria-expanded", String(open)); setCollapse($(`#${other.getAttribute("aria-controls")}`), open);
          });
        }));
        group.addEventListener("keydown", (event) => {
          if (!event.target.matches(".ov-acc-trigger") || !["ArrowDown", "ArrowUp", "Home", "End"].includes(event.key)) return;
          event.preventDefault(); const i = triggers.indexOf(event.target);
          triggers[event.key === "Home" ? 0 : event.key === "End" ? triggers.length - 1 : (i + (event.key === "ArrowDown" ? 1 : -1) + triggers.length) % triggers.length].focus();
        });
      });
      $("[data-show-more]").addEventListener("click", (event) => {
        const button = event.currentTarget, open = button.getAttribute("aria-expanded") !== "true";
        button.setAttribute("aria-expanded", String(open)); button.querySelector("span").textContent = open ? "Show less" : "Show more"; setCollapse($("#ov-more"), open);
      });
      $("[data-banner-close]").addEventListener("click", () => {
        setCollapse($("#ov-banner"), false); $("[data-banner-restore]").setAttribute("aria-expanded", "false"); $("[data-banner-restore]").focus(); announce("Announcement dismissed.");
      });
      $("[data-banner-restore]").addEventListener("click", () => { setCollapse($("#ov-banner"), true); $("[data-banner-restore]").setAttribute("aria-expanded", "true"); });

      $$("[data-toast-tone]").forEach((button) => button.addEventListener("click", () => {
        const [tone, icon, title, desc] = tones.find(([tone]) => tone === button.dataset.toastTone);
        toast({ tone, icon, title, desc });
      }));
      let pendingToast;
      const stopPromise = () => {
        if (!pendingToast) return;
        pendingToast.dismiss(); pendingToast = null;
        $("[data-promise]").disabled = false; $("[data-promise]").removeAttribute("aria-busy");
        $("[data-promise-result]").textContent = "Simulation canceled while away. Replay anytime.";
      };
      $("[data-promise]").addEventListener("click", () => {
        const button = $("[data-promise]"); button.disabled = true; button.setAttribute("aria-busy", "true");
        $("[data-promise-result]").textContent = "Simulating a save…";
        pendingToast = toast({ title: "Saving example…", desc: "Local simulation. No data is sent.", icon: "loader", duration: Infinity });
        later(() => {
          pendingToast?.dismiss(); pendingToast = null;
          toast({ title: "Example saved", desc: "The local simulation completed successfully.", tone: "success" });
          button.disabled = false; button.removeAttribute("aria-busy"); $("[data-promise-result]").textContent = "Local simulation complete. Run it again anytime.";
        }, 1800);
      });

      // Functional simulation time stays readable with reduced motion; visual interpolation is removed.
      const points = [[0, 0], [0.14, 16], [0.3, 32], [0.48, 51], [0.63, 68], [0.85, 88], [1, 100]];
      let lastPhase = -1;
      const updateProgress = () => {
        const t = S.clamp(state.elapsed / 8000, 0, 1);
        let i = 1; while (i < points.length - 1 && t > points[i][0]) i++;
        const [at, av] = points[i - 1], [bt, bv] = points[i];
        const value = Math.min(100, Math.round(S.lerp(av, bv, (t - at) / (bt - at))));
        const phase = value >= 100 ? 3 : value >= 68 ? 2 : value >= 32 ? 1 : 0;
        $("#ov-progress").value = value; $("#ov-progress").textContent = `${value}%`;
        $("[data-progress-percent]").textContent = `${value}%`; $(".ov-progress-track span").style.transform = `scaleX(${value / 100})`;
        $("[data-progress-label]").textContent = value === 0 && !state.running ? "Ready when you are" : ["Reading specimen", "Resolving tokens", "Composing preview", "Preview ready"][phase];
        $$("[data-step]").forEach((step, n) => {
          step.classList.toggle("is-done", n < phase); step.classList.toggle("is-current", n === phase);
          n === phase ? step.setAttribute("aria-current", "step") : step.removeAttribute("aria-current");
        });
        const complete = value >= 100;
        $(".ov-skeleton").hidden = complete; $(".ov-loaded").hidden = !complete;
        $(".ov-loading-preview").setAttribute("aria-busy", String(!complete));
        if (phase !== lastPhase && state.running) { $("[data-progress-status]").textContent = complete ? "Complete · local preview is ready." : `${["Step 1 of 3: reading specimen.", "Step 2 of 3: resolving tokens.", "Step 3 of 3: composing preview."][phase]} Local simulation.`; }
        lastPhase = phase;
        page.toggleAttribute("data-progress-running", state.running && state.active && !document.hidden);
        $("[data-progress-toggle]").textContent = complete ? "Run again" : state.running ? "Pause" : state.elapsed ? "Resume" : "Run simulation";
        if (complete && state.running) { state.elapsed = 8000; state.running = false; page.removeAttribute("data-progress-running"); animate($(".ov-loaded"), [{ opacity: 0 }, { opacity: 1 }], 320); }
      };
      const tick = (now) => {
        state.raf = 0; if (!state.active || document.hidden || !state.running) return;
        if (state.lastTime) state.elapsed += Math.min(now - state.lastTime, 200) * Number($("[data-progress-speed]").value) / (S.reduced() ? 1 : S.motionScale());
        state.lastTime = now; updateProgress();
        if (state.running) state.raf = requestAnimationFrame(tick);
      };
      const pause = (reason = "Paused · resume when ready.") => {
        cancelAnimationFrame(state.raf); state.raf = 0; state.lastTime = 0;
        const wasRunning = state.running; state.running = false; updateProgress();
        if (wasRunning) $("[data-progress-status]").textContent = reason;
      };
      const start = (reset = false) => {
        if (!state.active || document.hidden) return;
        cancelAnimationFrame(state.raf); if (reset || state.elapsed >= 8000) { state.elapsed = 0; lastPhase = -1; }
        state.running = true; state.lastTime = 0; updateProgress(); state.raf = requestAnimationFrame(tick);
      };
      $("[data-progress-toggle]").addEventListener("click", () => state.running ? pause() : start());
      $("[data-progress-replay]").addEventListener("click", () => start(true));
      $("[data-loading-demo]").addEventListener("replay", () => start(true));
      $("[data-progress-speed]").addEventListener("change", () => { state.lastTime = 0; $("[data-progress-status]").textContent = `${state.running ? "Running" : "Paused"} at ${$("[data-progress-speed]").value}× · local simulation.`; });

      const suspend = (reason) => {
        pause(reason); state.timers.forEach(clearTimeout); state.timers.clear(); stopPromise();
        closeMenus(false, true); hideCard(true); resetDrag();
        [...state.dialogs.keys()].forEach((dlg) => {
          const record = state.dialogs.get(dlg); record.closing = false; closeDialog(dlg, true, false);
        });
        state.animations.forEach((animation) => animation.cancel()); state.animations.clear();
        state.toasts.forEach((handle) => handle.dismiss()); state.toasts.clear();
        page.removeAttribute("data-visible");
      };
      const syncMotion = () => {
        page.toggleAttribute("data-reduced", S.reduced());
        if (S.reduced()) state.animations.forEach((animation) => { try { animation.finish(); } catch {} });
      };
      const motionObserver = new MutationObserver(syncMotion);
      const motionPreference = matchMedia("(prefers-reduced-motion: reduce)");
      motionPreference.addEventListener("change", () => { if (state.active) syncMotion(); });
      document.addEventListener("visibilitychange", () => {
        if (!state.active) return;
        if (document.hidden) suspend("Paused · tab was hidden. Resume when ready.");
        else { syncMotion(); page.setAttribute("data-visible", ""); }
      });
      updateProgress();
      instances.set(root, {
        enter() { state.active = true; syncMotion(); motionObserver.observe(document.documentElement, { attributes: true, attributeFilter: ["data-motion", "style"] }); if (!document.hidden) page.setAttribute("data-visible", ""); },
        leave() { state.active = false; motionObserver.disconnect(); suspend("Paused · page was left. Resume when ready."); },
      });
    },
    enter(root) { instances.get(root)?.enter(); },
    leave(root) { instances.get(root)?.leave(); },
  });
})();

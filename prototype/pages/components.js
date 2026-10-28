(() => {
  "use strict";
  const S = window.Schematic;
  const instances = new WeakMap();
  // Action time is deliberately independent of motion preferences.
  const HOLD_MS = 1500;
  const COPY_TEXT = "Schematic / Components / Built to respond.";
  const people = [
    { initials: "AL", name: "Alex Lin", role: "Design systems", status: "Available", tone: "success" },
    { initials: "MR", name: "Morgan Reed", role: "Product design", status: "Focusing", tone: "warning" },
    { initials: "SJ", name: "Sam Jordan", role: "Engineering", status: "Away", tone: "neutral" },
    { initials: "PK", name: "Priya Kapoor", role: "Accessibility", status: "Available", tone: "info" },
  ];
  const tags = ["Interface", "Motion", "Accessible", "Tokens", "Prototyping"];
  const icon = (name, size = 16) => S.icon(name, { size });
  const note = (key, text) => `<p class="cp-note" data-note="${key}" role="status" aria-live="polite" aria-atomic="true">${text}</p>`;
  const demo = (title, desc, body, specs = [], cls = "") => S.demo({ title, desc, body, specs, cls, stage: "is-col" });

  const render = () => `<div class="pg-components">
    ${S.pageHead({ icon: "cube", eyebrow: "Build / Interaction library", title: "Components", lead: "Small details. Clear responses. A working collection of controls that make every action feel considered.", meta: ["<span class=\"badge badge-accent\">Interactive specimens</span>", "<span class=\"badge\">Keyboard ready</span>", "<span class=\"badge\">Local demo state</span>"] })}
    <div class="cp-intro"><span class="cp-intro-mark" aria-hidden="true">${icon("cursor", 20)}</span><p>Press, choose, hold, remove. Every specimen below responds.<br><span class="muted">Your choices stay here when you visit another page.</span></p><span class="cp-index t-mono" aria-hidden="true">[ 01 — 05 ]</span></div>

    ${S.section({ id: "cp-buttons", num: "01", title: "A language of actions", desc: "Hierarchy comes from emphasis. Motion comes from the action itself: a light, an icon, a decisive press.", body: `<div class="grid cols-2">
      ${demo("Button hierarchy", "Choose an emphasis to see its role. The button stays put on hover.", `<div class="cp-button-matrix">${[
        ["primary", "Continue", "arrow-right", "Primary · the next step in a flow."],
        ["secondary", "Save draft", "document", "Secondary · a supporting action."],
        ["soft", "Quick action", "bolt", "Soft · useful without competing for attention."],
        ["ghost", "View details", "eye", "Ghost · a quiet, optional action."],
        ["danger", "Remove sample", "trash", "Destructive · removes a sample, with an undo."],
      ].map(([v, label, ic, explanation]) => `<div class="cp-variant"><span class="t-caption">${v}</span><button type="button" class="btn btn-${v}" data-variant="${v}" data-explanation="${explanation}">${icon(ic)}<span>${label}</span></button></div>`).join("")}</div>${note("variant", "Choose a button to inspect its intent.")}<button type="button" class="btn btn-secondary btn-sm" data-undo-sample hidden>Undo sample removal</button>`, ["press / snappy", "hover / light"])}
      ${demo("A size for the moment", "Each size adds a block to the local composition below.", `<div class="cp-size-line">${[["sm", "S"], ["", "M"], ["lg", "L"], ["xl", "XL"]].map(([size, label]) => `<button type="button" class="btn btn-secondary ${size ? `btn-${size}` : ""}" data-add-block="${label}" aria-label="Add ${label} block">${icon("plus")} ${label}</button>`).join("")}</div><div class="cp-composition" aria-label="Block composition" data-composition><span class="cp-block" data-size="M">M</span><span class="cp-block" data-size="L">L</span><span class="cp-block" data-size="S">S</span></div><div class="cp-size-foot"><span class="t-caption" data-block-total>3 of 8 blocks</span><button type="button" class="btn btn-ghost btn-sm" data-reset-blocks>${icon("refresh")} Reset blocks</button></div>${note("size", "Add up to eight blocks. Reset starts a fresh composition.")}`, ["28 / 34 / 40 / 48"])}
    </div>` })}

    ${S.section({ id: "cp-states", num: "02", title: "Follow through", desc: "An action is a conversation. Pending, success, failure and cancellation each deserve a distinct answer.", body: `<div class="grid cols-2">
      ${demo("Pending → resolved", "Two deterministic simulations. The second fails once, then succeeds on retry. Nothing is sent.", `<div class="cp-async-row"><div class="cp-async-item"><span class="t-eyebrow">Success path</span><button type="button" class="btn btn-primary cp-async" data-async="save">${icon("document")} Save sample</button></div><div class="cp-async-item"><span class="t-eyebrow">Recovery path</span><button type="button" class="btn btn-secondary cp-async" data-async="retry">${icon("refresh")} Test recovery</button></div></div>${note("async", "Ready. Each simulated request takes about one second.")}`, ["idle / busy / result"])}
      ${demo("Copy with certainty", "Confirmation appears only if the shared clipboard helper reports success.", `<div class="cp-copy-field"><span class="t-eyebrow">Sample text</span><p data-copy-text>${COPY_TEXT}</p></div><button type="button" class="btn btn-secondary" data-copy>${icon("copy")} Copy text</button>${note("copy", "Copy the sample, or select the text above manually.")}`, ["real clipboard"])}
      ${demo("A deliberate confirmation", "Hold with a pointer, Space or Enter. Release early, leave the button or press Escape to cancel.", `<div class="cp-hold-object"><span class="cp-object-icon">${icon("folder", 24)}</span><div><strong>Explorations</strong><p class="t-caption">A local sample folder</p></div><span class="badge" data-folder-state>Active</span></div><button type="button" class="btn btn-danger btn-lg cp-hold" data-hold aria-describedby="cp-hold-help"><span class="cp-hold-fill" aria-hidden="true"></span><span class="cp-hold-label">${icon("lock")} <span data-hold-label>Hold to archive</span></span></button><p class="cp-hold-help" id="cp-hold-help">Hold for 1.5 seconds. Releasing early cancels.</p><progress class="cp-hold-progress" data-hold-progress max="100" value="0" aria-label="Archive confirmation progress"></progress><button type="button" class="btn btn-secondary btn-sm" data-restore-folder hidden>${icon("refresh")} Restore folder</button>${note("hold", "No action until the full hold completes.")}`, ["1.5s / fixed action time"])}
      ${demo("A little appreciation", "One favourite per visitor. The count and the button describe the same state.", `<div class="cp-favourite-card"><div class="cp-favourite-art" aria-hidden="true"><span></span><span></span><span></span><span></span></div><div class="cp-favourite-meta"><span class="t-eyebrow">Collection 004</span><strong>Useful little details</strong><span class="t-caption">Motion, made purposeful.</span></div><div class="cp-favourite-wrap"><button type="button" class="btn btn-secondary cp-favourite" data-favourite aria-pressed="false" aria-label="Favourite Useful little details">${icon("heart")} <span data-favourite-word>Favourite</span><span class="cp-fav-count t-num" data-favourite-count aria-hidden="true">128</span></button><div class="cp-particles" data-particles aria-hidden="true"></div></div></div>${note("favourite", "128 favourites. Yours is not added yet.")}`, ["count / particles"])}
    </div>` })}

    ${S.section({ id: "cp-menus", num: "03", title: "Tools, in reach", desc: "The common action stays one press away. Alternatives and labels arrive exactly where they are needed.", body: `<div class="grid cols-2">
      ${demo("One action, a few alternatives", "The main button adds a chosen block to this demo. The menu changes what it adds.", `<div class="cp-split-wrap"><div class="cp-split"><button type="button" class="btn btn-primary" data-insert>${icon("plus")} <span data-insert-label>Add note</span></button><button type="button" class="btn btn-primary btn-icon" data-menu-trigger aria-label="Choose a block type" aria-haspopup="menu" aria-controls="cp-insert-menu" aria-expanded="false">${icon("chevron-down")}</button></div><div class="pop cp-menu" id="cp-insert-menu" role="menu" aria-label="Block type" inert>${[["note", "document", "Note", "A thought, captured"], ["task", "check-circle", "Task", "A next step"], ["link", "link", "Link", "A place to return to"]].map(([value, ic, label, sub]) => `<button type="button" role="menuitemradio" aria-checked="${value === "note"}" tabindex="-1" data-insert-choice="${value}">${icon(ic)}<span><strong>${label}</strong><small>${sub}</small></span><span class="cp-menu-check">${icon("check")}</span></button>`).join("")}</div></div><div class="cp-insert-tray" data-insert-tray aria-label="Inserted sample blocks"><span class="cp-empty-tray">Your next idea goes here.</span></div><button type="button" class="btn btn-ghost btn-sm" data-clear-tray>Clear blocks</button>${note("split", "Note selected. Add a note or open the menu for another block.")}`, ["arrows / home / end", "esc / dismiss"], "cp-menu-demo")}
      ${demo("A toolbar that travels with you", "Choose a tool to change the local preview. Hover or focus to reveal each tool’s name.", `<div class="cp-toolbar" data-toolbar role="toolbar" aria-label="Preview tools">${[["cursor", "Select"], ["pencil", "Draw"], ["chat", "Comment"], ["ruler", "Measure"]].map(([ic, label], i) => `<button type="button" class="btn btn-ghost btn-icon" data-tool="${label}" data-tip="${label}" data-tip-side="top" aria-label="${label} tool" aria-pressed="${i === 0}" tabindex="${i === 0 ? 0 : -1}">${icon(ic, 18)}</button>`).join("")}</div><div class="cp-tool-preview" data-tool-preview data-mode="Select" aria-hidden="true"><div class="cp-tool-shape"><span class="cp-tool-mark">${icon("cursor", 20)}</span><span class="cp-tool-comment">Aa</span><span class="cp-tool-dimension">128 × 80</span></div></div>${note("tool", "Select tool. The shape’s selection boundary is visible.")}`, ["one gliding highlight"])}
    </div>` })}

    ${S.section({ id: "cp-settings", num: "04", title: "Choice, made tangible", desc: "Familiar native inputs with a considered skin. Every preference updates the miniature workspace next to it.", body: `<div class="grid cols-2">
      ${demo("Workspace preferences", "Switch, checkbox, mixed selection and radio controls retain their native keyboard behavior.", `<div class="cp-settings"><div class="cp-setting-line"><label class="cp-switch-label" for="cp-guides"><strong>Layout guides</strong><small>Show the construction grid.</small></label><label class="cp-switch"><input type="checkbox" role="switch" id="cp-guides" checked><span class="cp-switch-track" aria-hidden="true"><span></span></span><span class="cp-sr">Layout guides</span></label></div><fieldset class="cp-fieldset"><legend>Visible layers</legend><label class="cp-check cp-check-parent"><input type="checkbox" data-all-layers><span class="cp-check-box" aria-hidden="true">${icon("check", 12)}<span class="cp-mixed-line"></span></span><span>All layers <small data-layer-count>2 of 3 visible</small></span></label>${[["type", "Typography", true], ["shapes", "Shapes", true], ["notes", "Annotations", false]].map(([id, label, checked]) => `<label class="cp-check cp-layer-child"><input type="checkbox" data-layer="${id}" ${checked ? "checked" : ""}><span class="cp-check-box" aria-hidden="true">${icon("check", 12)}</span><span>${label}</span></label>`).join("")}</fieldset><fieldset class="cp-fieldset cp-radio-field"><legend>Preview canvas</legend>${[["paper", "Paper"], ["tinted", "Tinted"]].map(([value, label], i) => `<label class="cp-radio"><input type="radio" name="cp-canvas" value="${value}" ${i === 0 ? "checked" : ""}><span aria-hidden="true"></span>${label}</label>`).join("")}</fieldset></div><div class="cp-workspace-preview" data-workspace-preview data-guides="true" data-canvas="paper" aria-label="Workspace preview"><span class="cp-preview-note" data-preview-layer="notes">Leave room to breathe.</span><div data-preview-layer="type"><span class="t-eyebrow">FIELD NOTES / 001</span><h3>Make space<br>for the idea.</h3></div><div class="cp-preview-shapes" data-preview-layer="shapes" aria-hidden="true"><span></span><span></span><span></span></div></div>${note("settings", "Guides on. 2 of 3 layers visible. Paper canvas.")}`, ["native controls"], "cp-settings-demo")}
      ${demo("Segmented, connected", "Choose a density. The selection moves once; the sample list reorganizes around it.", `<div class="seg cp-density-seg" data-density role="group" aria-label="List density"><span class="seg-thumb" aria-hidden="true"></span>${["Compact", "Comfort", "Roomy"].map((label, i) => `<button type="button" class="seg-btn" data-density-value="${label}" aria-pressed="${i === 1}">${label}</button>`).join("")}</div><div class="cp-density-list" data-density-list data-density="Comfort">${[["document", "Getting started", "8 min read"], ["cube", "Component anatomy", "12 min read"], ["wave", "Motion principles", "6 min read"]].map(([ic, title, sub], i) => `<div class="cp-density-item"><span class="cp-density-num t-mono">0${i + 1}</span><span class="cp-density-ic">${icon(ic)}</span><span><strong>${title}</strong><small>${sub}</small></span><span class="cp-density-end" aria-hidden="true">${icon("check", 14)}</span></div>`).join("")}</div>${note("density", "Comfort density. Balanced spacing for everyday reading.")}`, ["selection / spring", "layout / FLIP"])}
    </div>` })}

    ${S.section({ id: "cp-collections", num: "05", title: "A place for everything", desc: "Content, labels and people stay connected as you switch context. No position is lost without a reason.", body: `<div class="stack">
      ${demo("Tabs with a sense of direction", "Left and right arrows choose a tab. Home and End jump to the edges. Tab moves into the selected panel.", `<div class="cp-tabs-shell"><div class="cp-tablist" role="tablist" aria-label="Component collection" data-tabs>${[["overview", "Overview", "grid"], ["anatomy", "Anatomy", "layers"], ["behavior", "Behavior", "cursor"]].map(([id, label, ic], i) => `<button type="button" role="tab" id="cp-tab-${id}" aria-controls="cp-panel-${id}" aria-selected="${i === 0}" tabindex="${i === 0 ? 0 : -1}" data-tab="${id}">${icon(ic)}${label}</button>`).join("")}<span class="cp-tab-underline" aria-hidden="true"></span></div><div class="cp-tab-panels"><div role="tabpanel" id="cp-panel-overview" aria-labelledby="cp-tab-overview" tabindex="0" data-panel="overview"><div class="cp-tab-feature"><div class="cp-tab-drawing" aria-hidden="true"><span class="cp-drawing-button">${icon("plus")} Create something</span><span class="cp-drawing-caption t-mono">intent → action → feedback</span></div><div><span class="t-eyebrow">01 / The whole picture</span><h3>One familiar gesture.<br>A complete response.</h3><p>A control is more than its resting state. It has a focus, a press, a pending moment and a result.</p><span class="badge badge-accent">Made to be used</span></div></div></div><div role="tabpanel" id="cp-panel-anatomy" aria-labelledby="cp-tab-anatomy" tabindex="0" data-panel="anatomy" hidden><div class="cp-anatomy-grid">${[["01", "Container", "A surface defines the target and its emphasis."], ["02", "Label", "A verb explains what pressing will do."], ["03", "Icon", "A visual cue reinforces the label."], ["04", "Feedback", "A visible state confirms what happened."]].map(([n, label, text]) => `<div><span class="cp-anatomy-no t-mono">${n}</span><strong>${label}</strong><p>${text}</p></div>`).join("")}</div></div><div role="tabpanel" id="cp-panel-behavior" aria-labelledby="cp-tab-behavior" tabindex="0" data-panel="behavior" hidden><div class="cp-behavior-flow">${[["cursor", "Focus", "The next action is clear."], ["plus", "Press", "The target acknowledges input."], ["check-circle", "Resolve", "The outcome stays visible."]].map(([ic, label, text]) => `<div><span>${icon(ic, 22)}</span><strong>${label}</strong><p>${text}</p></div>`).join("")}</div><p class="cp-tab-access">Reduced motion keeps each state and its meaning, with no directional travel.</p></div></div></div>`, ["roving focus", "direction / out"])}
      <div class="grid cols-2">
        ${demo("Labels that make room", "Select a chip to include it. Remove one and its neighbors move into the space.", `<div class="cp-chip-list" data-chips role="group" aria-label="Collection labels">${tags.map((tag, i) => `<span class="cp-chip" data-chip="${tag}"><button type="button" data-chip-select aria-pressed="${i < 2}"><span class="cp-chip-dot" aria-hidden="true"></span>${tag}</button><button type="button" data-chip-remove aria-label="Remove ${tag}">${icon("x", 12)}</button></span>`).join("")}</div><div class="cp-chip-tools"><span class="t-caption" data-chip-count>2 labels selected</span><button type="button" class="btn btn-ghost btn-sm" data-reset-chips>${icon("refresh")} Restore labels</button></div>${note("chips", "Interface and Motion are selected.")}`, ["select / remove", "FLIP / snappy"])}
        ${demo("People, with presence", "Initials keep this demo entirely local. Choose a person to see their role and availability.", `<div class="cp-avatars" role="group" aria-label="Collection collaborators">${people.map((p, i) => `<button type="button" class="cp-avatar cp-avatar-${p.tone}" data-person="${i}" aria-label="${p.name}, ${p.status}" aria-pressed="${i === 0}" data-tip="${p.name} · ${p.status}"><span>${p.initials}</span><i class="cp-status-dot" aria-hidden="true"></i></button>`).join("")}</div><div class="cp-person-card" data-person-card><span class="cp-person-initials cp-avatar-success" data-person-initials>AL</span><div><strong data-person-name>Alex Lin</strong><span class="t-caption" data-person-role>Design systems</span></div><span class="cp-presence" data-person-status data-tone="success">Available</span></div><label class="cp-availability-label" for="cp-availability">Your availability</label><select id="cp-availability" class="cp-select" data-availability><option>Available</option><option>Focusing</option><option>Away</option></select>${note("people", "Alex Lin · Design systems · Available.")}`, ["status / text + colour"])}
      </div>
    </div>` })}
    <p class="cp-endnote">Built from the same tokens. Try another accent, appearance or motion setting in the tune panel.</p>
  </div>`;

  function mount(root) {
    const page = root.querySelector(".pg-components");
    const $ = (selector) => page.querySelector(selector);
    const $$ = (selector) => [...page.querySelectorAll(selector)];
    const state = { active: false, timers: new Set(), animations: new Set(), async: new Map(), archived: false, hold: null, holdFrame: 0, favourite: false, tab: 0, density: "Comfort", insert: "note", copyVersion: 0, copying: false, people: people.map((person) => ({ ...person })) };
    instances.set(root, state);
    const isVisible = () => state.active && !root.hidden && !document.hidden;
    const say = (key, message) => { $(`[data-note="${key}"]`).textContent = message; };
    const later = (fn, ms) => {
      const id = setTimeout(() => { state.timers.delete(id); if (isVisible()) fn(); }, ms);
      state.timers.add(id);
      return id;
    };
    const animate = (el, frames, duration = 320, easing = S.curve.out) => {
      if (!isVisible() || S.reduced() || !el.animate) return;
      const animation = el.animate(frames, { duration: S.dur(duration), easing });
      state.animations.add(animation);
      animation.finished.catch(() => {}).finally(() => { state.animations.delete(animation); });
      return animation;
    };
    const cancelAnimations = (el) => {
      el.getAnimations?.({ subtree: true }).forEach((a) => a.cancel());
    };
    const flip = (els, mutate) => {
      els.forEach(cancelAnimations);
      if (S.reduced() || !isVisible()) mutate();
      else S.flip(els, mutate, { duration: S.springDur.snappy });
    };

    $$('[data-variant]').forEach((button) => button.addEventListener("click", () => {
      say("variant", button.dataset.explanation);
      if (button.dataset.variant === "danger") {
        button.disabled = true;
        button.querySelector("span").textContent = "Sample removed";
        $("[data-undo-sample]").hidden = false;
        say("variant", "Sample removed from this demo. Undo restores it.");
      }
    }));
    $("[data-undo-sample]").addEventListener("click", () => {
      const button = $('[data-variant="danger"]');
      button.disabled = false;
      button.querySelector("span").textContent = "Remove sample";
      button.focus();
      $("[data-undo-sample]").hidden = true;
      say("variant", "Sample restored.");
    });

    const blockTotal = () => {
      const count = $("[data-composition]").children.length;
      $("[data-block-total]").textContent = `${count} of 8 blocks`;
      $$("[data-add-block]").forEach((b) => { b.disabled = count >= 8; });
    };
    $$("[data-add-block]").forEach((button) => button.addEventListener("click", () => {
      const composition = $("[data-composition]");
      if (composition.children.length >= 8) return;
      const block = document.createElement("span");
      block.className = "cp-block";
      block.dataset.size = button.dataset.addBlock;
      block.textContent = button.dataset.addBlock;
      flip([...composition.children], () => composition.append(block));
      animate(block, [{ opacity: 0, transform: "scale(.85)" }, { opacity: 1, transform: "none" }]);
      blockTotal();
      say("size", `${button.dataset.addBlock} block added. ${composition.children.length} of 8 blocks.`);
    }));
    $("[data-reset-blocks]").addEventListener("click", () => {
      flip([...$("[data-composition]").children], () => {
        $("[data-composition]").innerHTML = '<span class="cp-block" data-size="M">M</span><span class="cp-block" data-size="L">L</span><span class="cp-block" data-size="S">S</span>';
      });
      blockTotal();
      say("size", "Composition reset to three blocks.");
    });

    const asyncLabels = { save: ["document", "Save sample"], retry: ["refresh", "Test recovery"] };
    $$("[data-async]").forEach((button) => button.addEventListener("click", () => {
      const key = button.dataset.async;
      if (state.async.has(key) || !isVisible()) return;
      const recovering = button.dataset.result === "error";
      const shouldFail = key === "retry" && !recovering;
      state.async.set(key, button);
      button.setAttribute("aria-disabled", "true");
      button.setAttribute("aria-busy", "true");
      button.dataset.result = "pending";
      button.innerHTML = `${icon("loader")} ${recovering ? "Retrying…" : "Working…"}`;
      say("async", `${key === "save" ? "Save" : "Recovery"} simulation in progress.`);
      later(() => {
        state.async.delete(key);
        button.removeAttribute("aria-disabled");
        button.removeAttribute("aria-busy");
        button.dataset.result = shouldFail ? "error" : "success";
        button.innerHTML = shouldFail ? `${icon("alert")} Retry sample` : `${icon("check")} ${key === "save" ? "Saved · run again" : "Recovered · test again"}`;
        say("async", shouldFail ? "Simulated failure. Your sample is safe. Retry sample will succeed." : `${key === "save" ? "Sample saved in this demo." : "Retry succeeded in this demo."} You can run it again.`);
        S.playIcon(button.querySelector(".ic"));
      }, 1050); // Simulated operation time, not animation time.
    }));

    $("[data-copy]").addEventListener("click", async () => {
      if (state.copying || !isVisible()) return;
      const button = $("[data-copy]");
      const version = ++state.copyVersion;
      state.copying = true;
      button.setAttribute("aria-busy", "true");
      button.setAttribute("aria-disabled", "true");
      button.dataset.result = "pending";
      button.innerHTML = `${icon("loader")} Copying…`;
      let copied = false;
      try { copied = typeof S.copy === "function" && await S.copy(COPY_TEXT) === true; } catch { copied = false; }
      if (version !== state.copyVersion || !isVisible()) return;
      state.copying = false;
      button.removeAttribute("aria-busy");
      button.removeAttribute("aria-disabled");
      button.innerHTML = copied ? `${icon("check")} Copied · copy again` : `${icon("copy")} Try copy again`;
      button.dataset.result = copied ? "success" : "error";
      say("copy", copied ? "Copied to your clipboard." : "Clipboard unavailable or permission denied. Select and copy the sample text manually.");
    });

    const hold = $("[data-hold]");
    const progress = $("[data-hold-progress]");
    const setProgress = (value) => {
      hold.style.setProperty("--hold-progress", String(value / 100));
      progress.value = value;
    };
    const cancelHold = (message = "Hold cancelled. The folder is still active.") => {
      if (!state.hold) return;
      const pointer = state.hold.pointer;
      state.hold = null;
      cancelAnimationFrame(state.holdFrame);
      state.holdFrame = 0;
      hold.classList.remove("is-holding");
      setProgress(0);
      if (pointer !== undefined && hold.hasPointerCapture?.(pointer)) hold.releasePointerCapture(pointer);
      say("hold", message);
    };
    const completeHold = () => {
      if (!state.hold) return;
      const pointer = state.hold.pointer;
      state.hold = null;
      state.holdFrame = 0;
      state.archived = true;
      hold.classList.remove("is-holding");
      setProgress(100);
      hold.setAttribute("aria-disabled", "true");
      hold.classList.add("is-complete");
      $("[data-hold-label]").textContent = "Folder archived";
      $("[data-folder-state]").textContent = "Archived";
      $("[data-restore-folder]").hidden = false;
      if (pointer !== undefined && hold.hasPointerCapture?.(pointer)) hold.releasePointerCapture(pointer);
      say("hold", "Explorations archived in this demo. Restore folder reverses it.");
    };
    const startHold = (source) => {
      if (state.hold || state.archived || !isVisible()) return;
      state.hold = { started: performance.now(), ...source };
      hold.classList.add("is-holding");
      say("hold", "Keep holding. Release to cancel.");
      const tick = (now) => {
        if (!state.hold) return;
        if (!isVisible()) { cancelHold(); return; }
        const value = Math.min(100, (now - state.hold.started) / HOLD_MS * 100);
        setProgress(value);
        if (value >= 100) completeHold();
        else state.holdFrame = requestAnimationFrame(tick);
      };
      state.holdFrame = requestAnimationFrame(tick);
    };
    hold.addEventListener("pointerdown", (event) => {
      if (event.button !== 0 || event.isPrimary === false || state.archived) return;
      if (state.hold) return;
      hold.focus({ preventScroll: true });
      hold.setPointerCapture?.(event.pointerId);
      startHold({ pointer: event.pointerId });
    });
    hold.addEventListener("pointermove", (event) => {
      if (state.hold?.pointer !== event.pointerId) return;
      const rect = hold.getBoundingClientRect();
      if (event.clientX < rect.left || event.clientX > rect.right || event.clientY < rect.top || event.clientY > rect.bottom) cancelHold();
    });
    ["pointerup", "pointercancel", "lostpointercapture"].forEach((type) => hold.addEventListener(type, (event) => {
      if (state.hold?.pointer === event.pointerId) cancelHold();
    }));
    hold.addEventListener("keydown", (event) => {
      if (event.key === "Escape") { cancelHold(); return; }
      if (![" ", "Enter"].includes(event.key)) return;
      event.preventDefault();
      if (!event.repeat) startHold({ key: event.key });
    });
    hold.addEventListener("keyup", (event) => {
      if (![" ", "Enter"].includes(event.key)) return;
      event.preventDefault();
      if (state.hold?.key === event.key) cancelHold();
    });
    hold.addEventListener("click", (event) => {
      event.preventDefault();
      if (!state.archived && !state.hold) say("hold", "Hold for 1.5 seconds with a pointer, Space or Enter to archive. A quick press does not confirm.");
    });
    hold.addEventListener("blur", () => cancelHold());
    $("[data-restore-folder]").addEventListener("click", () => {
      state.archived = false;
      hold.removeAttribute("aria-disabled");
      hold.classList.remove("is-complete");
      setProgress(0);
      $("[data-hold-label]").textContent = "Hold to archive";
      $("[data-folder-state]").textContent = "Active";
      hold.focus();
      $("[data-restore-folder]").hidden = true;
      say("hold", "Folder restored. A new confirmation still requires the full 1.5-second hold.");
    });

    $("[data-favourite]").addEventListener("click", () => {
      const button = $("[data-favourite]");
      state.favourite = !state.favourite;
      button.setAttribute("aria-pressed", String(state.favourite));
      $("[data-favourite-word]").textContent = state.favourite ? "Favourited" : "Favourite";
      const count = $("[data-favourite-count]");
      S.roll(count, state.favourite ? "129" : "128");
      say("favourite", state.favourite ? "129 favourites. Yours is included." : "128 favourites. Your favourite was removed.");
      const particles = $("[data-particles]");
      particles.replaceChildren();
      if (!state.favourite || S.reduced()) return;
      for (let i = 0; i < 9; i++) {
        const particle = document.createElement("span");
        particle.className = "cp-particle";
        particles.append(particle);
        const angle = Math.PI * 2 * i / 9;
        const radius = parseFloat(getComputedStyle(page).getPropertyValue("--s-12")) || 48;
        const animation = animate(particle, [{ transform: "translate(0, 0) scale(1)", opacity: 1 }, { transform: `translate(${Math.cos(angle) * radius}px, ${Math.sin(angle) * radius}px) scale(.2)`, opacity: 0 }], 650);
        animation?.finished.catch(() => {}).finally(() => particle.remove());
      }
    });

    const trigger = $("[data-menu-trigger]");
    const menu = $("#cp-insert-menu");
    const menuItems = [...menu.querySelectorAll('[role="menuitemradio"]')];
    const menuAPI = S.popover(trigger, menu, { onOpen: () => {
      menu.inert = false;
      (menuItems.find((item) => item.getAttribute("aria-checked") === "true") || menuItems[0]).focus();
    }, onClose: () => { menu.inert = true; } });
    trigger.addEventListener("keydown", (event) => {
      if (!["ArrowDown", "ArrowUp"].includes(event.key)) return;
      event.preventDefault();
      menuAPI.open();
      (event.key === "ArrowUp" ? menuItems.at(-1) : menuItems[0]).focus();
    });
    menu.addEventListener("keydown", (event) => {
      const index = menuItems.indexOf(document.activeElement);
      let next;
      if (event.key === "ArrowDown") next = (index + 1) % menuItems.length;
      if (event.key === "ArrowUp") next = (index - 1 + menuItems.length) % menuItems.length;
      if (event.key === "Home") next = 0;
      if (event.key === "End") next = menuItems.length - 1;
      if (next !== undefined) { event.preventDefault(); menuItems[next].focus(); }
      if (event.key === "Tab") {
        // Return to the trigger before the browser performs its normal Tab step.
        trigger.focus();
        menuAPI.close();
      }
    });
    menuItems.forEach((button) => button.addEventListener("click", () => {
      state.insert = button.dataset.insertChoice;
      menuItems.forEach((item) => item.setAttribute("aria-checked", String(item === button)));
      $("[data-insert-label]").textContent = `Add ${state.insert}`;
      menuAPI.close();
      trigger.focus();
      say("split", `${state.insert[0].toUpperCase() + state.insert.slice(1)} selected. The main button now adds this block.`);
    }));
    $("[data-insert]").addEventListener("click", () => {
      const tray = $("[data-insert-tray]");
      if (tray.querySelectorAll(".cp-inserted-block").length >= 4) { say("split", "Four sample blocks added. Clear blocks to start again."); return; }
      tray.querySelector(".cp-empty-tray")?.remove();
      const block = document.createElement("span");
      block.className = "cp-inserted-block";
      block.innerHTML = `${icon({ note: "document", task: "check-circle", link: "link" }[state.insert])} ${S.esc(state.insert)}`;
      tray.append(block);
      animate(block, [{ transform: "translateY(8px)", opacity: 0 }, { transform: "none", opacity: 1 }]);
      say("split", `${state.insert[0].toUpperCase() + state.insert.slice(1)} added. ${tray.children.length} of 4 blocks.`);
    });
    $("[data-clear-tray]").addEventListener("click", () => {
      $("[data-insert-tray]").innerHTML = '<span class="cp-empty-tray">Your next idea goes here.</span>';
      say("split", "Sample blocks cleared.");
    });

    const toolbar = $("[data-toolbar]");
    const glide = S.glide(toolbar, { items: "[data-tool]" });
    const toolButtons = $$("[data-tool]");
    const toolDescriptions = { Select: "The shape’s selection boundary is visible.", Draw: "A drawn path crosses the shape.", Comment: "An annotation appears on the shape.", Measure: "The shape’s dimensions are shown." };
    toolButtons.forEach((button) => button.addEventListener("click", () => {
      toolButtons.forEach((item) => { item.setAttribute("aria-pressed", String(item === button)); item.tabIndex = item === button ? 0 : -1; });
      const mode = button.dataset.tool;
      $("[data-tool-preview]").dataset.mode = mode;
      $(".cp-tool-mark").innerHTML = icon({ Select: "cursor", Draw: "pencil", Comment: "chat", Measure: "ruler" }[mode], 20);
      say("tool", `${mode} tool. ${toolDescriptions[mode]}`);
      glide.moveTo(button);
    }));
    const rove = (event, buttons, activate) => {
      const index = buttons.indexOf(document.activeElement);
      if (index < 0) return;
      let next;
      if (event.key === "ArrowRight") next = (index + 1) % buttons.length;
      if (event.key === "ArrowLeft") next = (index - 1 + buttons.length) % buttons.length;
      if (event.key === "Home") next = 0;
      if (event.key === "End") next = buttons.length - 1;
      if (next === undefined) return;
      event.preventDefault();
      buttons[next].focus();
      if (activate) buttons[next].click();
      else buttons.forEach((button, i) => { button.tabIndex = i === next ? 0 : -1; });
    };
    toolbar.addEventListener("keydown", (event) => rove(event, toolButtons, false));

    const allLayers = $("[data-all-layers]");
    const layers = $$("[data-layer]");
    const updateSettings = () => {
      const count = layers.filter((input) => input.checked).length;
      allLayers.checked = count === layers.length;
      allLayers.indeterminate = count > 0 && count < layers.length;
      $("[data-layer-count]").textContent = `${count} of 3 visible`;
      layers.forEach((input) => { $(`[data-preview-layer="${input.dataset.layer}"]`).hidden = !input.checked; });
      const guides = $("#cp-guides").checked;
      const canvas = $('input[name="cp-canvas"]:checked').value;
      $("[data-workspace-preview]").dataset.guides = String(guides);
      $("[data-workspace-preview]").dataset.canvas = canvas;
      say("settings", `Guides ${guides ? "on" : "off"}. ${count} of 3 layers visible. ${canvas === "paper" ? "Paper" : "Tinted"} canvas.`);
    };
    allLayers.addEventListener("change", () => { layers.forEach((input) => { input.checked = allLayers.checked; }); updateSettings(); });
    [...layers, $("#cp-guides"), ...$$('input[name="cp-canvas"]')].forEach((input) => input.addEventListener("change", updateSettings));
    allLayers.indeterminate = true;
    $("[data-preview-layer=notes]").hidden = true;

    const densityButtons = $$("[data-density-value]");
    const densityThumb = $("[data-density] .seg-thumb");
    const tabButtons = $$("[data-tab]");
    const tabLine = $(".cp-tab-underline");
    const placeSelection = () => {
      if (!isVisible()) return;
      const tab = tabButtons[state.tab];
      tabLine.style.transform = `translateX(${tab.offsetLeft}px) scaleX(${tab.offsetWidth})`;
      const density = densityButtons.find((b) => b.dataset.densityValue === state.density);
      densityThumb.style.width = `${density.offsetWidth}px`;
      densityThumb.style.transform = `translateX(${density.offsetLeft}px)`;
    };
    densityButtons.forEach((button) => button.addEventListener("click", () => {
      state.density = button.dataset.densityValue;
      densityButtons.forEach((item) => item.setAttribute("aria-pressed", String(item === button)));
      flip($$(".cp-density-item"), () => { $("[data-density-list]").dataset.density = state.density; });
      placeSelection();
      say("density", `${state.density} density. ${{ Compact: "More information in less space.", Comfort: "Balanced spacing for everyday reading.", Roomy: "Extra space for focused reading." }[state.density]}`);
    }));
    $("[data-density]").addEventListener("keydown", (event) => rove(event, densityButtons, true));
    tabButtons.forEach((button, index) => button.addEventListener("click", () => {
      if (state.tab === index) return;
      const direction = index > state.tab ? 1 : -1;
      state.tab = index;
      tabButtons.forEach((item, i) => { item.setAttribute("aria-selected", String(i === index)); item.tabIndex = i === index ? 0 : -1; });
      $$("[data-panel]").forEach((panel) => {
        cancelAnimations(panel);
        panel.hidden = panel.dataset.panel !== button.dataset.tab;
        if (!panel.hidden) {
          const distance = parseFloat(getComputedStyle(page).getPropertyValue("--s-3")) || 12;
          animate(panel, [{ opacity: 0, transform: `translateX(${direction * distance}px)` }, { opacity: 1, transform: "none" }], 320, S.curve.out);
        }
      });
      placeSelection();
    }));
    $("[data-tabs]").addEventListener("keydown", (event) => rove(event, tabButtons, true));

    const updateChips = () => {
      const selected = $$("[data-chip-select]").filter((button) => button.getAttribute("aria-pressed") === "true");
      $("[data-chip-count]").textContent = `${selected.length} label${selected.length === 1 ? "" : "s"} selected`;
      return selected.map((button) => button.closest("[data-chip]").dataset.chip);
    };
    $("[data-chips]").addEventListener("click", (event) => {
      const select = event.target.closest("[data-chip-select]");
      const remove = event.target.closest("[data-chip-remove]");
      if (select) {
        select.setAttribute("aria-pressed", String(select.getAttribute("aria-pressed") !== "true"));
        const selected = updateChips();
        say("chips", selected.length ? `Selected: ${selected.join(", ")}.` : "No labels selected.");
      }
      if (remove) {
        const chip = remove.closest("[data-chip]");
        const next = chip.nextElementSibling?.querySelector("[data-chip-select]") || chip.previousElementSibling?.querySelector("[data-chip-select]") || $("[data-reset-chips]");
        const name = chip.dataset.chip;
        flip($$("[data-chip]"), () => chip.remove());
        next.focus();
        updateChips();
        say("chips", `${name} removed. Restore labels brings all five back.`);
      }
    });
    $("[data-reset-chips]").addEventListener("click", () => {
      const list = $("[data-chips]");
      flip($$("[data-chip]"), () => {
        // Reuse existing nodes so surviving labels have real FLIP positions.
        tags.forEach((tag, i) => {
          let chip = [...list.children].find((el) => el.dataset.chip === tag);
          if (!chip) {
            chip = document.createElement("span");
            chip.className = "cp-chip";
            chip.dataset.chip = tag;
            chip.innerHTML = `<button type="button" data-chip-select aria-pressed="false"><span class="cp-chip-dot" aria-hidden="true"></span>${S.esc(tag)}</button><button type="button" data-chip-remove aria-label="Remove ${S.esc(tag)}">${icon("x", 12)}</button>`;
          }
          chip.querySelector("[data-chip-select]").setAttribute("aria-pressed", String(i < 2));
          list.append(chip);
        });
      });
      updateChips();
      say("chips", "All five labels restored. Interface and Motion selected.");
    });

    const showPerson = (index) => {
      const person = state.people[index];
      $("[data-person-initials]").textContent = person.initials;
      $("[data-person-initials]").className = `cp-person-initials cp-avatar-${person.tone}`;
      $("[data-person-name]").textContent = person.name;
      $("[data-person-role]").textContent = person.role;
      $("[data-person-status]").textContent = person.status;
      $("[data-person-status]").dataset.tone = person.tone;
      $$("[data-person]").forEach((button) => button.setAttribute("aria-pressed", String(Number(button.dataset.person) === index)));
      say("people", `${person.name} · ${person.role} · ${person.status}.`);
      animate($("[data-person-card]"), [{ opacity: .45 }, { opacity: 1 }], 220);
    };
    $$("[data-person]").forEach((button) => button.addEventListener("click", () => showPerson(Number(button.dataset.person))));
    $("[data-availability]").addEventListener("change", (event) => {
      // Alex represents the current visitor; this state belongs to this mount.
      const self = state.people[0];
      self.status = event.target.value;
      self.tone = { Available: "success", Focusing: "warning", Away: "neutral" }[self.status];
      const button = $('[data-person="0"]');
      button.className = `cp-avatar cp-avatar-${self.tone}`;
      button.setAttribute("aria-label", `${self.name}, ${self.status}`);
      button.dataset.tip = `${self.name} · ${self.status}`;
      showPerson(0);
      say("people", `Your availability is now ${self.status.toLowerCase()}.`);
    });

    const observer = new ResizeObserver(placeSelection);
    state.resume = () => {
      if (!state.active || document.hidden || root.hidden) return;
      page.classList.remove("cp-paused");
      observer.observe($("[data-tabs]"));
      observer.observe($("[data-density]"));
      placeSelection();
    };
    state.stop = () => {
      page.classList.add("cp-paused");
      cancelHold("Hold cancelled because this page is no longer active.");
      observer.disconnect();
      state.timers.forEach(clearTimeout);
      state.timers.clear();
      state.animations.forEach((animation) => animation.cancel());
      state.animations.clear();
      cancelAnimations(page);
      $("[data-particles]").replaceChildren();
      state.async.forEach((button, key) => {
        button.removeAttribute("aria-busy");
        button.removeAttribute("aria-disabled");
        delete button.dataset.result;
        button.innerHTML = `${icon(asyncLabels[key][0])} ${asyncLabels[key][1]}`;
      });
      if (state.async.size) say("async", "Pending simulations cancelled when the page became inactive. Ready to run again.");
      state.async.clear();
      if (state.copying) {
        ++state.copyVersion;
        state.copying = false;
        $("[data-copy]").removeAttribute("aria-busy");
        $("[data-copy]").removeAttribute("aria-disabled");
        delete $("[data-copy]").dataset.result;
        $("[data-copy]").innerHTML = `${icon("copy")} Copy text`;
        say("copy", "Clipboard confirmation interrupted. Copy again to verify the result.");
      }
      menuAPI.close();
      glide.hide();
    };
    document.addEventListener("visibilitychange", () => { if (document.hidden) state.stop(); else if (state.active) state.resume(); });
    window.addEventListener("blur", () => cancelHold("Hold cancelled because the window lost focus."));
  }

  S.page({
    id: "components", title: "Components", group: "Build", icon: "cube", order: 40,
    description: "Buttons, settings, tabs and small interactions with a complete response.",
    render, mount,
    enter(root) { const state = instances.get(root); if (state) { state.active = true; state.resume(); } },
    leave(root) { const state = instances.get(root); if (state) { state.active = false; state.stop(); } },
  });
})();

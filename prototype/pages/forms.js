(() => {
  "use strict";
  const S = window.Schematic;
  const controllers = new WeakMap();
  const icon = (name, size = 16) => S.icon(name, { size });
  const field = (id, label, control, hint = "", extra = "") => `
    <div class="forms-field" ${extra}><label for="forms-${id}">${label}</label>${control}
    ${hint ? `<p class="forms-hint" id="forms-${id}-hint">${hint}</p>` : ""}</div>`;
  const input = (id, attrs = "") => `<input class="forms-input" id="forms-${id}" ${attrs}>`;
  const demo = (title, desc, body, specs = []) => S.demo({ title, desc, body, specs, stage: "is-plain", cls: "forms-demo" });

  function render() {
    return `<div class="pg-forms">
      ${S.pageHead({ icon: "input", eyebrow: "Build / Interaction primitives", title: "Forms & Inputs", lead: "Inputs, validation, uploads and account settings.", meta: ['<span class="badge badge-accent">Native at the core</span>', '<span class="badge">Keyboard ready</span>', '<span class="badge">Local demos only</span>'] })}
      <aside class="forms-intro" aria-label="Demo information"><span class="forms-intro-icon">${icon("info", 20)}</span><div><strong>Try every state. Nothing leaves this page.</strong><p>Validation runs locally. Profile saves last for this visit; file progress is a simulation. Use sample details.</p></div><span class="forms-wire" aria-hidden="true">input → check → feedback</span></aside>

      ${S.section({ id: "forms-text", num: "01", title: "Text inputs", body: `<div class="grid cols-2">
        ${demo("Input sizes", "Three sizes, a leading icon, and a floating label that makes room for your answer.", `<div class="forms-stack forms-wide">
          ${field("compact", '<span>Compact</span><span class="forms-measure">28</span>', `<div class="forms-control forms-control-sm"><span class="forms-leading">${icon("search")}</span>${input("compact", 'type="search" placeholder="Find a component" autocomplete="off"')}</div>`)}
          ${field("standard", '<span>Comfortable</span><span class="forms-measure">40</span>', `<div class="forms-control"><span class="forms-leading">${icon("user")}</span>${input("standard", 'type="text" value="Alex Morgan" autocomplete="off"')}</div>`)}
          ${field("large", '<span>Spacious</span><span class="forms-measure">48</span>', `<div class="forms-control forms-control-lg"><span class="forms-leading">${icon("mail")}</span>${input("large", 'type="email" placeholder="you@example.com" autocomplete="off"')}</div>`)}
          <div class="forms-floating"><input class="forms-input" id="forms-floating" type="text" placeholder=" " autocomplete="off"><label for="forms-floating">Project name</label></div>
        </div>`, ["focus / settle", "token sizes"])}
        ${demo("States with something to say", "Success and errors use words as well as colour. Read-only content remains selectable.", `<div class="forms-stack forms-wide">
          ${field("ready", "Ready to edit", input("ready", 'type="text" value="Untitled project"'))}
          ${field("valid", `Format checked <span class="forms-state-label">${icon("check-circle", 14)} Valid example</span>`, input("valid", 'type="email" value="alex@example.com" aria-describedby="forms-valid-hint"'), 'Example state: valid email syntax only.', 'data-state="valid"')}
          ${field("invalid", `Needs attention <span class="forms-state-label">${icon("alert", 14)} Error example</span>`, input("invalid", 'type="email" value="alex@" aria-invalid="true" aria-describedby="forms-invalid-hint"'), 'Enter a complete address, such as alex@example.com.', 'data-state="error" data-static-invalid')}
          <div class="grid cols-2 forms-state-pair">${field("readonly", "Read only", input("readonly", 'type="text" value="SCHEMATIC-001" readonly'))}${field("disabled", "Unavailable", input("disabled", 'type="text" value="Managed by team" disabled'))}</div>
        </div>`, ["explicit states", "no colour-only cues"])}
        ${demo("Validation, in context", "Try an empty submission or an incomplete address. Passing checks the format; it does not verify a mailbox.", `<form class="forms-stack forms-wide" id="forms-email-demo" novalidate>
          <div class="forms-field" id="forms-email-field"><label for="forms-email">Email address <span class="forms-required">Required</span></label><div class="forms-control"><span class="forms-leading">${icon("mail")}</span>${input("email", 'type="email" required autocomplete="off" placeholder="you@example.com" aria-describedby="forms-email-message"')}</div><p class="forms-message" id="forms-email-message" aria-live="polite" aria-atomic="true">Use a sample address. Checked locally as you type.</p></div>
          <button class="btn btn-primary" type="submit">Try demo submit ${icon("arrow-right")}</button><p class="forms-status" id="forms-email-status" role="status" aria-atomic="true"></p>
        </form>`, ["live validation", "submit errors"])}
        ${demo("Auto-growing textarea", "The textarea grows with your words. A live counter keeps the limit in view.", `<div class="forms-stack forms-wide"><div class="forms-field"><label for="forms-note">A note for your team <span class="forms-optional">Optional</span></label><textarea class="forms-input forms-textarea" id="forms-note" rows="3" maxlength="240" aria-describedby="forms-note-hint forms-note-count" placeholder="What should the next person know?">Review the updated navigation before release.</textarea><div class="row-between"><p class="forms-hint" id="forms-note-hint">Grows as you type · 240 character limit</p><output class="forms-counter t-num" id="forms-note-count" for="forms-note">62 / 240</output></div><p class="forms-sr" id="forms-note-announcement" role="status"></p></div><div class="forms-note-rule">${icon("pencil")} 240 character limit.</div></div>`, ["measured growth", "live counter"])}
      </div>` })}

      ${S.section({ id: "forms-security", num: "02", title: "Password and verification", body: `<div class="grid cols-2">
        ${demo("Show the requirements", "A local rules meter, not a guarantee of password security. Use a sample password here.", `<div class="forms-stack forms-wide"><div class="forms-field"><label for="forms-password">Sample password</label><div class="forms-control"><span class="forms-leading">${icon("lock")}</span>${input("password", 'type="password" autocomplete="new-password" aria-describedby="forms-password-hint forms-strength-text forms-requirements" placeholder="Build a sample password"')}<button class="forms-reveal btn btn-ghost btn-sm" type="button" id="forms-reveal" aria-controls="forms-password" aria-pressed="false" aria-label="Show sample password">${icon("eye")}<span>Show</span></button></div><p class="forms-hint" id="forms-password-hint">Checked in memory only. Never included in a demo save.</p></div><div class="forms-strength" id="forms-strength" role="meter" aria-label="Password demo rules met" aria-valuemin="0" aria-valuemax="4" aria-valuenow="0" aria-valuetext="0 of 4 demo rules met">${[0, 1, 2, 3].map(i => `<span data-strength-segment="${i}"></span>`).join("")}</div><p class="forms-hint" id="forms-strength-text" aria-live="polite" aria-atomic="true">Not started · 0 of 4 demo rules met</p><ul class="forms-requirements" id="forms-requirements">${["At least 12 characters", "Uppercase and lowercase letters", "At least one number", "At least one symbol"].map((text, i) => `<li data-requirement="${i}"><span class="forms-req-icon">${icon("check", 12)}</span><span>${text}</span><span class="forms-req-status">Needed</span></li>`).join("")}</ul></div>`, ["reveal / conceal", "4 local rules"])}
        ${demo("One code, six small steps", "Paste the whole code, use arrow keys to review, or Backspace to move back through empty slots.", `<form class="forms-stack forms-wide" id="forms-otp-demo" novalidate><fieldset class="forms-fieldset"><legend>Six-digit demo code</legend><p class="forms-hint" id="forms-otp-hint">Use <code>123456</code>. This is a local comparison, not account verification.</p><div class="forms-otp" id="forms-otp-slots">${Array.from({ length: 6 }, (_, i) => `<label class="forms-sr" for="forms-otp-${i + 1}">Digit ${i + 1} of 6</label><input class="forms-input forms-otp-digit" id="forms-otp-${i + 1}" type="text" inputmode="numeric" pattern="[0-9]" maxlength="1" autocomplete="${i === 0 ? "one-time-code" : "off"}" aria-describedby="forms-otp-hint forms-otp-status" data-otp="${i}">`).join("")}</div></fieldset><button class="btn btn-primary" type="submit">Verify demo code ${icon("check")}</button><p class="forms-status" id="forms-otp-status" role="status" aria-atomic="true">Ready for a six-digit sample.</p><div class="forms-key-note"><kbd class="kbd">←</kbd><kbd class="kbd">→</kbd> Review digits <span>·</span> Paste all six at once</div></form>`, ["auto advance", "paste aware"])}
      </div>` })}

      ${S.section({ id: "forms-choice", num: "03", title: "Selection controls", body: `<div class="grid cols-2">
        ${demo("Let the platform help", "A real select: use arrow keys, type an option’s first letters, and confirm with Enter. Shortcuts follow your platform.", `<div class="forms-stack forms-wide">${field("timezone", "Workspace time zone", `<select class="forms-input forms-select" id="forms-timezone" aria-describedby="forms-timezone-hint"><option value="Berlin">Berlin · Europe/Berlin</option><option value="London">London · Europe/London</option><option value="New York">New York · America/New_York</option><option value="San Francisco">San Francisco · America/Los_Angeles</option><option value="Singapore">Singapore · Asia/Singapore</option><option value="Tokyo">Tokyo · Asia/Tokyo</option></select>`, 'Native typeahead: try “Tok” to jump to Tokyo.')}<div class="forms-selection"><span class="forms-selection-icon">${icon("globe", 20)}</span><div><span class="forms-eyebrow">Selected locally</span><strong id="forms-timezone-value">Berlin</strong></div><span class="badge">Time zone</span></div><p class="forms-status" id="forms-timezone-status" role="status"></p></div>`, ["native select", "type to jump"])}
        ${demo("Every step accounted for", "The native number input supports direct entry and arrow keys. The readout rolls to the new quantity.", `<div class="forms-stack forms-wide"><div class="forms-field"><label for="forms-seats">Team seats <span class="forms-optional">1–25</span></label><div class="forms-stepper"><button class="btn btn-secondary btn-icon" type="button" id="forms-seat-minus" aria-label="Remove one team seat">${icon("minus")}</button>${input("seats", 'type="number" min="1" max="25" step="1" value="4" inputmode="numeric" aria-describedby="forms-seats-hint forms-seats-error"')}<button class="btn btn-secondary btn-icon" type="button" id="forms-seat-plus" aria-label="Add one team seat">${icon("plus")}</button></div><p class="forms-hint" id="forms-seats-hint">Whole numbers from 1 to 25. Demo only; no purchase.</p><p class="forms-message" id="forms-seats-error" role="status"></p></div><div class="forms-seat-summary"><span class="forms-seat-icon">${icon("users", 22)}</span><span class="forms-seat-roll" id="forms-seat-count" aria-hidden="true">4</span><div><strong>seats in your demo team</strong><p class="forms-hint">A small team. Room to make things.</p></div></div><p class="forms-sr" id="forms-seat-status" role="status"></p></div>`, ["native spinbutton", "rolling count"])}
      </div>` })}

      ${S.section({ id: "forms-ranges", num: "04", title: "Sliders", body: `<div class="grid cols-2">
        ${demo("A single point of control", "A value bubble follows the thumb. Arrow keys adjust by one; Home and End reach the limits.", `<div class="forms-stack forms-wide">${field("volume", "Preview volume", `<div class="forms-slider"><div class="forms-bubble-rail"><output class="forms-bubble" id="forms-volume-bubble" for="forms-volume" style="--p:65%">65%</output></div><input class="range forms-range" id="forms-volume" type="range" min="0" max="100" value="65" aria-describedby="forms-volume-hint" aria-valuetext="65 percent" style="--p:65%"><div class="forms-range-ends"><span>Silent</span><span>Full</span></div></div>`, 'Changes the demo value only. No audio is played.')}<div class="forms-volume-bars" aria-hidden="true">${Array.from({ length: 20 }, (_, i) => `<span data-volume-bar="${i}" style="--bar-height:${25 + i * 3.5}%" ${i < 13 ? 'data-on=""' : ""}></span>`).join("")}</div></div>`, ["value bubble", "0–100%"])}
        ${demo("Two bounds, one interval", "The lower and upper handles cannot cross. Tab to each handle; use arrows for fine adjustments.", `<div class="forms-stack forms-wide"><fieldset class="forms-fieldset"><legend>Demo intensity window</legend><p class="forms-hint" id="forms-window-hint">0–100 units, with at least 10 units between bounds.</p><div class="forms-dual"><div class="forms-bubble-rail forms-dual-bubbles"><output class="forms-bubble" id="forms-low-bubble" for="forms-low" style="--p:20%">20</output><output class="forms-bubble forms-bubble-upper" id="forms-high-bubble" for="forms-high" style="--p:80%">80</output></div><div class="forms-dual-track" style="--low:20%;--high:80%"><div class="forms-dual-line" aria-hidden="true"></div><label class="forms-sr" for="forms-low">Lower intensity bound</label><input class="range forms-range" id="forms-low" type="range" min="0" max="100" step="1" value="20" aria-valuemax="70" aria-describedby="forms-window-hint"><label class="forms-sr" for="forms-high">Upper intensity bound</label><input class="range forms-range" id="forms-high" type="range" min="0" max="100" step="1" value="80" aria-valuemin="30" aria-describedby="forms-window-hint"></div><div class="forms-range-ends"><span>0 units</span><span>100 units</span></div></div></fieldset><div class="forms-window-summary"><span>Selected interval</span><output id="forms-window-output" for="forms-low forms-high">20–80 units</output></div></div>`, ["dual native thumbs", "non-crossing"])}
      </div>` })}

      ${S.section({ id: "forms-files", num: "05", title: "File upload", body: demo("Local files / simulated progress", "Only filenames and sizes are used. File contents are never read, and no network request is made.", `<div class="forms-upload-layout forms-wide"><div class="forms-dropzone" id="forms-dropzone"><span class="forms-drop-icon">${icon("upload", 26)}</span><strong>Drop a few files here</strong><p class="forms-hint" id="forms-files-hint">Up to 4 files · 10 MiB each · any file type</p><label class="forms-file-label" for="forms-files-input">Or choose local files</label><input id="forms-files-input" type="file" multiple aria-describedby="forms-files-hint forms-upload-status"><span class="badge badge-warning">Simulation · no network upload</span></div><div class="forms-upload-details"><div class="row-between"><h3 class="t-h3">Local selection</h3><span class="forms-file-count" id="forms-file-count">0 files</span></div><ul class="forms-file-list" id="forms-file-list"><li class="forms-file-empty">${icon("document", 24)}<span>Your selection will appear here.<small>No files are selected.</small></span></li></ul><div class="forms-progress-head"><label id="forms-progress-label">Simulated progress</label><output id="forms-progress-value">0%</output></div><div class="forms-progress" id="forms-progress" role="progressbar" aria-labelledby="forms-progress-label" aria-valuemin="0" aria-valuemax="100" aria-valuenow="0"><span></span></div><div class="row"><button class="btn btn-primary btn-sm" type="button" id="forms-simulate" disabled>${icon("play")}<span>Simulate progress</span></button><button class="btn btn-ghost btn-sm" type="button" id="forms-clear-files" disabled>Clear files</button></div><p class="forms-status" id="forms-upload-status" role="status" aria-atomic="true">Choose files to try the local progress simulation.</p></div></div>`, ["local metadata", "pause on leave"]) })}

      ${S.section({ id: "forms-profile", num: "06", title: "Profile form", body: `<div class="forms-profile-shell"><div class="forms-profile-top"><div><span class="forms-eyebrow">Composed example</span><h3 class="t-h3">Your workspace profile</h3><p class="forms-hint">Required fields are marked. Save is in memory for this visit only.</p></div><span class="badge" id="forms-dirty">Example profile</span></div><div class="forms-profile-layout"><form class="forms-profile-form" id="forms-profile-form" novalidate><div class="forms-error-summary" id="forms-profile-errors" role="alert" tabindex="-1" hidden></div><div class="grid cols-2">
          ${field("profile-name", 'Display name <span class="forms-required">Required</span>', input("profile-name", 'name="displayName" type="text" value="Alex Morgan" minlength="2" maxlength="64" required autocomplete="off" aria-describedby="forms-profile-name-error"') + '<p class="forms-message" id="forms-profile-name-error" aria-live="polite"></p>')}
          ${field("profile-email", 'Email address <span class="forms-required">Required</span>', input("profile-email", 'name="email" type="email" value="alex@example.com" required autocomplete="off" aria-describedby="forms-profile-email-error"') + '<p class="forms-message" id="forms-profile-email-error" aria-live="polite"></p>')}
          ${field("profile-handle", 'Username <span class="forms-required">Required</span>', `<div class="forms-control"><span class="forms-leading forms-at" aria-hidden="true">@</span>${input("profile-handle", 'name="handle" type="text" value="alexmorgan" required minlength="3" maxlength="24" pattern="[A-Za-z0-9_]{3,24}" spellcheck="false" autocomplete="off" aria-describedby="forms-profile-handle-hint forms-profile-handle-error"')}</div><p class="forms-message" id="forms-profile-handle-error" aria-live="polite"></p>`, '3–24 letters, numbers, or underscores.')}
          ${field("profile-role", 'Role <span class="forms-required">Required</span>', '<select class="forms-input forms-select" id="forms-profile-role" name="role" required aria-describedby="forms-profile-role-error"><option value="">Choose a role</option><option value="Designer" selected>Designer</option><option value="Engineer">Engineer</option><option value="Product manager">Product manager</option><option value="Researcher">Researcher</option><option value="Other">Other</option></select><p class="forms-message" id="forms-profile-role-error" aria-live="polite"></p>')}
        </div>${field("profile-site", 'Website <span class="forms-optional">Optional</span>', input("profile-site", 'name="website" type="url" placeholder="https://example.com" autocomplete="off" aria-describedby="forms-profile-site-hint forms-profile-site-error"') + '<p class="forms-message" id="forms-profile-site-error" aria-live="polite"></p>', 'Use a complete http:// or https:// address.')}
        <div class="forms-field"><label for="forms-profile-bio">About you <span class="forms-optional">Optional</span></label><textarea class="forms-input forms-textarea" id="forms-profile-bio" name="bio" maxlength="160" rows="3" aria-describedby="forms-profile-bio-count forms-profile-bio-error">Making thoughtful tools for people who care about the details.</textarea><output class="forms-counter" id="forms-profile-bio-count" for="forms-profile-bio">60 / 160</output><p class="forms-message" id="forms-profile-bio-error" aria-live="polite"></p></div>
        <fieldset class="forms-fieldset forms-visibility"><legend>Profile visibility</legend><div class="forms-radio-options"><label class="forms-radio-card"><input type="radio" name="visibility" value="team" checked><span><strong>Team only</strong><small>Preview for your workspace.</small></span>${icon("users")}</label><label class="forms-radio-card"><input type="radio" name="visibility" value="public"><span><strong>Public</strong><small>Preview a public profile.</small></span>${icon("globe")}</label></div><p class="forms-hint">Changes the local preview only. Nothing is published.</p></fieldset>
        <label class="forms-checkbox-row" for="forms-profile-updates"><input id="forms-profile-updates" name="updates" type="checkbox" checked><span><strong>Weekly product updates</strong><small>Demo preference only. This does not subscribe you.</small></span></label>
        <div class="forms-profile-actions"><button class="btn btn-primary" type="submit">${icon("check")} Save demo profile</button><button class="btn btn-secondary" type="reset">Reset changes</button></div><p class="forms-status" id="forms-profile-status" role="status" aria-atomic="true">Edit a field to try validation and the live preview.</p>
        </form><aside class="forms-preview" aria-label="Local profile preview"><span class="forms-eyebrow">Live / local preview</span><div class="forms-avatar" id="forms-preview-avatar" aria-hidden="true">AM</div><h3 id="forms-preview-name">Alex Morgan</h3><p class="forms-preview-handle" id="forms-preview-handle">@alexmorgan</p><span class="badge badge-accent" id="forms-preview-role">Designer</span><p class="forms-preview-bio" id="forms-preview-bio">Making thoughtful tools for people who care about the details.</p><div class="forms-preview-foot">${icon("users")}<span id="forms-preview-visibility">Team only · local preview</span></div><p class="forms-hint">A preview, not a published profile. Reloading clears demo saves.</p></aside></div></div>` })}
    </div>`;
  }

  function mount(root) {
    const page = root.querySelector(".pg-forms");
    const $ = (id) => page.querySelector(`#forms-${id}`);
    const $$ = (sel) => [...page.querySelectorAll(sel)];
    let active = false;
    let simulation = null;
    let progress = 0;
    let files = [];
    const animations = new Set();
    const flash = (el, error = false) => {
      if (!active || S.reduced() || !el.animate) return;
      const frames = error ? [{ transform: "translateX(0)" }, { transform: "translateX(var(--s-1))" }, { transform: "translateX(calc(var(--s-1) * -1))" }, { transform: "translateX(0)" }] : [{ opacity: 0.35 }, { opacity: 1 }];
      const a = el.animate(frames, { duration: S.dur(error ? 320 : 220), easing: S.curve.out });
      animations.add(a);
      a.finished.catch(() => {}).finally(() => animations.delete(a));
    };
    const status = (el, text, tone = "") => { el.textContent = text; el.dataset.tone = tone; };
    const setField = (el, message, state = "") => {
      el.setAttribute("aria-invalid", String(state === "error"));
      const host = el.closest(".forms-field");
      if (host) host.dataset.state = state;
      const msg = el.id === "forms-email" ? $("email-message") : page.querySelector(`#${el.id}-error`);
      if (msg) msg.textContent = message;
    };
    const emailError = (el) => !el.value.trim() ? "Enter an email address." : el.validity.typeMismatch || !/^[^\s@]+@[^\s@]+\.[^\s@]+$/.test(el.value.trim()) ? "Enter a complete address, such as alex@example.com." : "";
    const email = $("email");
    const validateEmail = (force = false) => {
      const error = emailError(email);
      if (!email.value && !force) setField(email, "Use a sample address. Checked locally as you type.");
      else setField(email, error || "Format looks valid. Mailbox delivery has not been checked.", error ? "error" : "valid");
      return error;
    };
    email.addEventListener("input", () => { validateEmail(); status($("email-status"), ""); });
    email.addEventListener("blur", () => validateEmail(true));
    $("email-demo").addEventListener("submit", (e) => {
      e.preventDefault();
      const error = validateEmail(true);
      status($("email-status"), error ? "Demo submit stopped. Correct the email address above." : "Demo submit accepted: the format is valid. No email was sent.", error ? "error" : "success");
      if (error) email.focus();
      flash($("email-status"), !!error);
    });
    ["valid", "invalid"].forEach(key => $(key).addEventListener("input", (e) => {
      const error = emailError(e.target), host = e.target.closest(".forms-field");
      e.target.setAttribute("aria-invalid", String(!!error));
      host.dataset.state = error ? "error" : "valid";
      host.querySelector(".forms-state-label").innerHTML = `${icon(error ? "alert" : "check-circle", 14)} ${error ? "Error example" : "Valid example"}`;
      $(key + "-hint").textContent = error || "Format looks valid. This is a local example.";
    }));

    function grow(el) {
      if (root.hidden || document.hidden) return;
      const before = el.getBoundingClientRect().height;
      el.style.height = "auto";
      const style = getComputedStyle(el);
      const after = el.scrollHeight + parseFloat(style.borderTopWidth) + parseFloat(style.borderBottomWidth);
      el.style.height = `${after}px`;
      if (active && before && Math.abs(after - before) > 3 && !S.reduced() && el.animate) {
        el.getAnimations().forEach(a => a.cancel());
        const a = el.animate([{ height: `${before}px` }, { height: `${after}px` }], { duration: S.dur(220), easing: S.curve.out });
        animations.add(a); a.finished.catch(() => {}).finally(() => animations.delete(a));
      }
    }
    let noteThreshold = 0;
    const updateNote = () => {
      const el = $("note"), remaining = 240 - el.value.length;
      $("note-count").textContent = `${el.value.length} / 240`;
      $("note-count").dataset.near = String(remaining <= 24);
      const threshold = remaining === 0 ? 2 : remaining <= 24 ? 1 : 0;
      if (threshold !== noteThreshold) $("note-announcement").textContent = threshold === 2 ? "Character limit reached." : threshold === 1 ? `${remaining} characters remaining.` : "More than 24 characters available.";
      noteThreshold = threshold;
      grow(el);
    };
    $("note").addEventListener("input", updateNote);
    updateNote();

    $("reveal").addEventListener("click", () => {
      const el = $("password"), show = el.type === "password";
      el.type = show ? "text" : "password";
      $("reveal").setAttribute("aria-pressed", String(show));
      $("reveal").setAttribute("aria-label", show ? "Hide sample password" : "Show sample password");
      $("reveal").querySelector("span").textContent = show ? "Hide" : "Show";
    });
    $("password").addEventListener("input", (e) => {
      const value = e.target.value;
      const checks = [value.length >= 12, /[A-Z]/.test(value) && /[a-z]/.test(value), /[0-9]/.test(value), /[^A-Za-z0-9\s]/.test(value)];
      const score = checks.filter(Boolean).length;
      $("strength").setAttribute("aria-valuenow", String(score));
      $("strength").setAttribute("aria-valuetext", `${score} of 4 demo rules met`);
      $("strength").dataset.score = String(score);
      $$('[data-strength-segment]').forEach((el, i) => el.dataset.on = String(i < score));
      $$('[data-requirement]').forEach((el, i) => { el.dataset.met = String(checks[i]); el.querySelector(".forms-req-status").textContent = checks[i] ? "Met" : "Needed"; });
      $("strength-text").textContent = `${value ? ["Needs work", "A start", "Getting there", "Almost there", "Meets the demo rules"][score] : "Not started"} · ${score} of 4 demo rules met`;
    });

    const digits = $$("[data-otp]");
    const otpEditing = () => {
      digits.forEach(el => el.removeAttribute("aria-invalid"));
      $("otp-slots").removeAttribute("data-result");
      status($("otp-status"), digits.every(el => el.value) ? "Six digits entered. Choose Verify demo code." : "Enter the six-digit demo code. Use 123456.");
    };
    const distribute = (text, start) => {
      const clean = text.replace(/[\s-]/g, "");
      if (!/^\d{1,6}$/.test(clean)) { status($("otp-status"), "Paste up to six digits. The demo code is 123456.", "error"); return; }
      const offset = clean.length === 6 ? 0 : start;
      [...clean].slice(0, 6 - offset).forEach((v, i) => digits[offset + i].value = v);
      otpEditing();
      digits[Math.min(5, offset + clean.length)].focus();
    };
    digits.forEach((el, i) => {
      el.addEventListener("focus", () => el.select());
      el.addEventListener("paste", (e) => { e.preventDefault(); distribute(e.clipboardData.getData("text"), i); });
      el.addEventListener("input", () => {
        const value = el.value;
        if (value.length > 1) { distribute(value, i); return; }
        el.value = value.replace(/\D/g, "");
        otpEditing();
        if (el.value && i < 5) digits[i + 1].focus();
      });
      el.addEventListener("keydown", (e) => {
        if (e.key === "Backspace" && !el.value && i > 0) { e.preventDefault(); digits[i - 1].value = ""; digits[i - 1].focus(); otpEditing(); }
        if (e.key === "ArrowLeft" || e.key === "ArrowRight") { e.preventDefault(); digits[S.clamp(i + (e.key === "ArrowRight" ? 1 : -1), 0, 5)].focus(); }
      });
    });
    $("otp-demo").addEventListener("submit", (e) => {
      e.preventDefault();
      const code = digits.map(el => el.value).join("");
      const success = code === "123456";
      digits.forEach(el => el.setAttribute("aria-invalid", String(!success)));
      $("otp-slots").dataset.result = success ? "success" : "error";
      status($("otp-status"), success ? "Demo code matches 123456. No account was verified." : code.length < 6 ? "Enter all six digits before checking the demo code." : "That code does not match. Try 123456 for this local demo.", success ? "success" : "error");
      if (!success) (digits.find(el => !el.value) || digits[0]).focus();
      flash($("otp-slots"), !success);
    });

    $("timezone").addEventListener("change", (e) => { $("timezone-value").textContent = e.target.value; status($("timezone-status"), `Local selection: ${e.target.selectedOptions[0].textContent}. No system setting changed.`); flash($("timezone-value")); });
    let seats = 4;
    const setSeats = (n, announce = true) => {
      seats = S.clamp(Math.round(n), 1, 25);
      $("seats").value = String(seats);
      $("seats").removeAttribute("aria-invalid");
      status($("seats-error"), "");
      $("seat-minus").disabled = seats === 1;
      $("seat-plus").disabled = seats === 25;
      if (active && !S.reduced()) S.roll($("seat-count"), String(seats));
      else {
        $("seat-count").textContent = String(seats);
        // S.roll rebuilds its strips after the plain reduced-motion readout.
        delete $("seat-count")._rollSig;
      }
      if (announce) $("seat-status").textContent = `${seats} demo team ${seats === 1 ? "seat" : "seats"}. No purchase made.`;
    };
    $("seat-minus").addEventListener("click", () => setSeats(seats - 1));
    $("seat-plus").addEventListener("click", () => setSeats(seats + 1));
    $("seats").addEventListener("input", (e) => {
      const n = e.target.valueAsNumber;
      if (Number.isInteger(n) && n >= 1 && n <= 25) setSeats(n);
      else { e.target.setAttribute("aria-invalid", "true"); status($("seats-error"), "Enter a whole number from 1 to 25.", "error"); }
    });
    $("seats").addEventListener("blur", (e) => { if (e.target.getAttribute("aria-invalid") === "true") { setSeats(seats, false); status($("seats-error"), `Restored the last valid quantity: ${seats}.`); } });
    setSeats(4, false);
    $("volume").addEventListener("input", (e) => {
      const n = Number(e.target.value);
      e.target.style.setProperty("--p", `${n}%`);
      e.target.setAttribute("aria-valuetext", `${n} percent`);
      $("volume-bubble").style.setProperty("--p", `${n}%`);
      $("volume-bubble").textContent = `${n}%`;
      $$("[data-volume-bar]").forEach((el, i) => el.toggleAttribute("data-on", i < n / 5));
    });
    const updateBounds = (changed) => {
      let low = +$("low").value, high = +$("high").value;
      if (high - low < 10) { if (changed === "low") low = high - 10; else high = low + 10; }
      $("low").value = String(low); $("high").value = String(high);
      $("low").setAttribute("aria-valuemax", String(high - 10));
      $("high").setAttribute("aria-valuemin", String(low + 10));
      ["low", "high"].forEach((key, i) => { const v = i ? high : low; $(key + "-bubble").style.setProperty("--p", `${v}%`); $(key + "-bubble").textContent = String(v); $(key).setAttribute("aria-valuetext", `${v} units`); });
      const track = page.querySelector(".forms-dual-track");
      track.style.setProperty("--low", `${low}%`); track.style.setProperty("--high", `${high}%`);
      $("window-output").textContent = `${low}–${high} units`;
    };
    ["low", "high"].forEach(key => $(key).addEventListener("input", () => updateBounds(key)));
    updateBounds();

    const syncProgress = () => {
      const value = Math.round(progress);
      $("progress").setAttribute("aria-valuenow", String(value));
      $("progress").setAttribute("aria-valuetext", `${value} percent, simulated only`);
      $("progress").firstElementChild.style.transform = `scaleX(${progress / 100})`;
      $("progress-value").textContent = `${value}%`;
    };
    const syncSimulationButton = () => {
      $("simulate").disabled = !files.length;
      $("simulate").querySelector("span").textContent = simulation ? "Pause simulation" : progress >= 100 ? "Replay simulation" : progress > 0 ? "Resume simulation" : "Simulate progress";
    };
    const pauseSimulation = (message = "Simulation paused. Resume when ready; nothing was uploaded.") => {
      if (!simulation) return;
      simulation.cancel(); simulation = null;
      status($("upload-status"), message);
      syncSimulationButton();
    };
    const size = (n) => n >= 1048576 ? `${(n / 1048576).toFixed(1)} MiB` : n >= 1024 ? `${(n / 1024).toFixed(1)} KiB` : `${n} B`;
    const selectFiles = (incoming) => {
      pauseSimulation();
      const all = [...incoming], accepted = all.filter(f => f.size <= 10 * 1048576).slice(0, 4);
      files = accepted.map(f => ({ name: f.name, size: f.size })); progress = 0;
      $("file-list").innerHTML = files.length ? files.map(f => `<li><span class="forms-file-icon">${icon("document", 20)}</span><span class="forms-file-name">${S.esc(f.name)}<small>Local file · ${size(f.size)}</small></span><span class="forms-file-local">Selected</span></li>`).join("") : `<li class="forms-file-empty">${icon("document", 24)}<span>Your selection will appear here.<small>No files are selected.</small></span></li>`;
      $("file-count").textContent = `${files.length} ${files.length === 1 ? "file" : "files"}`;
      $("clear-files").disabled = !all.length;
      syncProgress(); syncSimulationButton();
      const skipped = all.length - files.length;
      status($("upload-status"), `${files.length ? `${files.length} local ${files.length === 1 ? "file" : "files"} selected. Ready to simulate progress.` : "No eligible files selected."}${skipped ? ` ${skipped} skipped: maximum 4 files, 10 MiB per file.` : ""} Nothing uploaded.`, skipped ? "error" : "");
    };
    $("files-input").addEventListener("change", (e) => { if (e.target.files.length) selectFiles(e.target.files); e.target.value = ""; });
    const dropzone = $("dropzone");
    let dragDepth = 0;
    dropzone.addEventListener("dragenter", (e) => { e.preventDefault(); dragDepth++; dropzone.dataset.drag = "true"; });
    dropzone.addEventListener("dragover", (e) => { e.preventDefault(); if (e.dataTransfer) e.dataTransfer.dropEffect = "copy"; });
    dropzone.addEventListener("dragleave", () => { dragDepth = Math.max(0, dragDepth - 1); if (!dragDepth) delete dropzone.dataset.drag; });
    dropzone.addEventListener("drop", (e) => { e.preventDefault(); dragDepth = 0; delete dropzone.dataset.drag; if (e.dataTransfer?.files.length) selectFiles(e.dataTransfer.files); });
    $("clear-files").addEventListener("click", () => { selectFiles([]); $("files-input").value = ""; status($("upload-status"), "Local selection cleared. Nothing was uploaded."); });
    $("simulate").addEventListener("click", () => {
      if (simulation) { pauseSimulation(); return; }
      if (!files.length || !active) return;
      if (progress >= 100) progress = 0;
      const done = () => { simulation = null; progress = 100; syncProgress(); syncSimulationButton(); status($("upload-status"), "Simulation complete. Files remain on your device; nothing was uploaded.", "success"); };
      status($("upload-status"), "Simulating progress locally. No upload is taking place.");
      if (S.reduced()) { done(); return; }
      simulation = S.tween({ from: progress, to: 100, duration: 2600 * (1 - progress / 100), ease: S.ease.inOut, onUpdate: v => { progress = v; syncProgress(); }, onDone: done });
      syncSimulationButton();
    });

    const form = $("profile-form");
    const fields = ["name", "email", "handle", "role", "site", "bio"].map(key => $("profile-" + key));
    const touched = new Set();
    let submitted = false, savedOnce = false;
    const snapshot = () => Object.fromEntries(new FormData(form));
    let saved = snapshot();
    const serialize = (data) => JSON.stringify(Object.keys(data).sort().map(key => [key, data[key]]));
    const profileError = (el) => {
      const value = el.value.trim();
      switch (el.id) {
        case "forms-profile-name": return value.length < 2 || value.length > 64 ? "Enter a display name with 2–64 characters." : "";
        case "forms-profile-email": return emailError(el);
        case "forms-profile-handle": return /^[A-Za-z0-9_]{3,24}$/.test(value) && value === el.value ? "" : "Use 3–24 letters, numbers, or underscores; no spaces.";
        case "forms-profile-role": return value ? "" : "Choose a role for the demo profile.";
        case "forms-profile-site": {
          if (!value) return "";
          try { const url = new URL(value); return /^(https?:)$/.test(url.protocol) && url.hostname && !el.validity.typeMismatch ? "" : "Use a complete http:// or https:// address."; } catch { return "Use a complete http:// or https:// address."; }
        }
        case "forms-profile-bio": return el.value.length > 160 ? "Keep your bio to 160 characters or fewer." : "";
        default: return "";
      }
    };
    const validateProfileField = (el) => { const error = profileError(el); setField(el, error, error ? "error" : ""); return error; };
    const updateErrorSummary = () => {
      const errors = fields.map(el => [el, profileError(el)]).filter(([, message]) => message);
      $("profile-errors").hidden = !errors.length;
      $("profile-errors").innerHTML = errors.length ? `<strong>${errors.length === 1 ? "One field needs" : `${errors.length} fields need`} attention</strong><ul>${errors.map(([el, message]) => `<li><a href="#${el.id}">${S.esc(message)}</a></li>`).join("")}</ul>` : "";
      return errors;
    };
    $("profile-errors").addEventListener("click", (e) => { const a = e.target.closest("a"); if (a) { e.preventDefault(); page.querySelector(a.getAttribute("href"))?.focus(); } });
    const preview = () => {
      const data = snapshot(), dirty = serialize(data) !== serialize(saved);
      $("dirty").textContent = dirty ? "Unsaved changes" : savedOnce ? "Saved for this visit" : "Example profile";
      $("dirty").classList.toggle("badge-warning", dirty);
      $("dirty").classList.toggle("badge-success", !dirty && savedOnce);
      const name = data.displayName.trim();
      $("preview-name").textContent = name || "Your display name";
      $("preview-avatar").textContent = name.split(/\s+/).filter(Boolean).slice(0, 2).map(s => [...s][0]).join("").toLocaleUpperCase() || "?";
      $("preview-handle").textContent = data.handle.trim() ? `@${data.handle.trim()}` : "Your username";
      $("preview-role").textContent = data.role || "Choose a role";
      $("preview-bio").textContent = data.bio.trim() || "Add a short introduction to your local preview.";
      $("preview-visibility").textContent = `${data.visibility === "public" ? "Public" : "Team only"} · local preview`;
      $("profile-bio-count").textContent = `${data.bio.length} / 160`;
      grow($("profile-bio"));
      return dirty;
    };
    fields.forEach(el => el.addEventListener("blur", () => { touched.add(el); validateProfileField(el); }));
    form.addEventListener("input", (e) => {
      if (fields.includes(e.target) && (touched.has(e.target) || submitted)) validateProfileField(e.target);
      if (submitted) updateErrorSummary();
      const dirty = preview();
      status($("profile-status"), dirty ? "Unsaved local changes. Save the demo profile when ready." : "Matches the current demo profile. Nothing sent.");
    });
    form.addEventListener("change", preview);
    form.addEventListener("submit", (e) => {
      e.preventDefault(); submitted = true;
      fields.forEach(validateProfileField);
      const errors = updateErrorSummary();
      if (errors.length) {
        status($("profile-status"), "Demo save stopped. Correct the marked fields.", "error");
        errors[0][0].focus(); flash($("profile-errors"), true); return;
      }
      saved = snapshot(); savedOnce = true; submitted = false;
      preview();
      status($("profile-status"), "Demo profile saved in memory for this visit. Nothing sent, published, or written to disk.", "success");
      flash($("profile-status"));
    });
    form.addEventListener("reset", (e) => {
      e.preventDefault();
      [...form.elements].forEach(el => {
        if (!el.name) return;
        if (el.type === "radio") el.checked = saved[el.name] === el.value;
        else if (el.type === "checkbox") el.checked = Object.hasOwn(saved, el.name);
        else el.value = saved[el.name] ?? "";
      });
      touched.clear(); submitted = false;
      fields.forEach(el => setField(el, ""));
      $("profile-errors").hidden = true; $("profile-errors").textContent = "";
      preview();
      status($("profile-status"), savedOnce ? "Restored your last saved demo profile for this visit." : "Restored the example profile. Nothing sent or saved to disk.");
    });
    preview();

    let lastWidth = 0;
    const resize = new ResizeObserver(entries => {
      const width = entries[0].contentRect.width;
      if (!active || width === lastWidth) return;
      lastWidth = width;
      grow($("note")); grow($("profile-bio"));
    });
    const pause = () => {
      active = false; page.dataset.paused = "true";
      resize.disconnect();
      pauseSimulation("Simulation paused while the page is hidden. Resume when ready; nothing uploaded.");
      animations.forEach(a => a.cancel()); animations.clear();
      page.getAnimations?.({ subtree: true }).forEach(a => a.cancel());
      dragDepth = 0; delete dropzone.dataset.drag;
    };
    const enter = () => {
      if (root.hidden || document.hidden) return;
      active = true; delete page.dataset.paused;
      lastWidth = page.getBoundingClientRect().width;
      resize.observe(page);
      updateNote(); grow($("profile-bio"));
    };
    document.addEventListener("visibilitychange", () => { if (document.hidden) pause(); else if (!root.hidden) enter(); });
    controllers.set(root, { enter, leave: pause });
  }

  S.page({ id: "forms", title: "Forms & Inputs", group: "Build", icon: "input", order: 50, description: "Thoughtful text fields, validation, credentials, ranges, local file demos, and a complete profile form.", render, mount, enter: root => controllers.get(root)?.enter(), leave: root => controllers.get(root)?.leave() });
})();

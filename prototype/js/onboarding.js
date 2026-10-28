/* Optional, local-only quick guide. App and palette boot before this listener. */
(() => {
  const S = window.Schematic;
  const root = document.documentElement;
  const completedKey = "guide-completed";
  const labels = ["Explore", "Palette", "Motion", "Reuse"];
  const panels = [];
  let dialog, launch, title, content, progress, back, next, palette;
  let step = 0, returnTo = null, exit = null, entrance = null, closing = null;

  const steps = [
    {
      title: "A working reference. Yours to explore.",
      desc: "Schematic brings spacing, type, components and purposeful motion into one place. Every example runs locally.",
      body: () => `
        <div class="onboarding-map">
          <div><span class="onboarding-map-num">01</span><div><strong>Find your foundation</strong><p>Tokens, type and color in Foundations.</p></div></div>
          <div><span class="onboarding-map-num">02</span><div><strong>Try the real interactions</strong><p>Components, forms, charts and overlays.</p></div></div>
          <div><span class="onboarding-map-num">03</span><div><strong>Jump straight there</strong><p>Use the sidebar, or search with <kbd class="kbd">Cmd / Ctrl K</kbd>.</p></div></div>
        </div>`,
    },
    {
      title: "One system. Two palettes.",
      desc: "Color pairs warm paper with a forest accent. Mono keeps the interface grayscale. Try either: this changes the whole app.",
      body: () => `
        <div class="seg seg-block onboarding-palette" role="group" aria-label="Interface palette" aria-describedby="onboarding-palette-help">
          <button type="button" class="seg-btn" data-value="color">Color</button>
          <button type="button" class="seg-btn" data-value="mono">Mono</button>
        </div>
        <div class="onboarding-preview" aria-hidden="true">
          <div class="onboarding-preview-top"><span class="badge badge-accent">Your workspace</span><span class="onboarding-swatches"><i></i><i></i><i></i></span></div>
          <strong>Make room for good work.</strong>
          <span>Quiet surfaces. Clear actions.</span>
          <div class="onboarding-preview-line"><span></span><span></span></div>
        </div>
        <p class="t-caption" id="onboarding-palette-help">Your choice is saved. Charts keep their own colors. Change it anytime in Tune tokens; the theme button switches light / dark.</p>`,
    },
    {
      title: "Motion with a reason to move.",
      desc: "A selection should travel to its new home. Try the sample, then use Motion Lab to inspect curves and springs.",
      body: () => `
        <div class="onboarding-motion">
          <div class="onboarding-motion-track" aria-hidden="true"><span class="onboarding-motion-marker">${S.icon("check", { size: 18 })}</span></div>
          <div class="onboarding-motion-labels" aria-hidden="true"><span>Ready</span><span>Selected</span></div>
        </div>
        <div class="onboarding-try"><button type="button" class="btn btn-secondary" data-guide-motion aria-pressed="false">Try motion ${S.icon("arrow-right", { size: 16 })}</button><span class="t-caption" data-guide-motion-status role="status">Ready to move.</span></div>
        <p class="t-caption" data-guide-motion-help></p>`,
    },
    {
      title: "Take the language into your project.",
      desc: "Start with the CSS tokens and shared styles, then reuse the JavaScript helpers and SVG icons your interface needs.",
      body: () => `
        <pre class="onboarding-code"><code>/* prototype/css/tokens.css */
.your-surface {
  background: var(--surface);
  color: var(--text);
  padding: var(--s-4);
  border-radius: var(--r-lg);
}</code></pre>
        <p class="t-small muted">HTML, CSS and JavaScript are the working reference. React / Next.js, SwiftUI and Rust adapters are planned.</p>
        <p class="t-caption">In the source download: docs/QUICKSTART.md explains reuse; prototype/CONTRACT.md explains page lifecycles.</p>`,
    },
  ];

  const paletteValue = () => root.dataset.palette === "mono" ? "mono" : "color";
  const syncNudge = () => {
    const fresh = S.store.get(completedKey, null) !== "1";
    launch.toggleAttribute("data-guide-new", fresh);
    launch.setAttribute("aria-label", fresh ? "Guide: take a quick tour of Schematic" : "Open Schematic guide");
  };
  const reduced = () => S.reduced() || (matchMedia("(prefers-reduced-motion: reduce)").matches && root.dataset.motion !== "full");
  const syncMotion = () => {
    const help = content.querySelector("[data-guide-motion-help]");
    if (help) help.textContent = reduced()
      ? "Reduced motion is on. The sample changes position instantly. Tune tokens offers motion settings."
      : "Tune tokens offers Full, Slow-mo and Reduced. Your browser's reduced-motion preference is respected unless you select Full.";
  };

  const render = ({ focus = true } = {}) => {
    entrance?.cancel();
    const current = steps[step];
    title.textContent = current.title;
    dialog.querySelector("#onboarding-desc").textContent = current.desc;
    dialog.querySelector("[data-guide-count]").textContent = `${step + 1} / ${steps.length}`;
    progress.querySelectorAll("li").forEach((item, i) => {
      if (i === step) item.setAttribute("aria-current", "step"); else item.removeAttribute("aria-current");
      item.toggleAttribute("data-past", i < step);
    });
    // Reuse each panel so its control listeners and ResizeObserver stay bounded.
    panels[step] ||= S.html(`<div class="onboarding-step">${current.body()}</div>`);
    content.replaceChildren(panels[step]);
    back.disabled = step === 0;
    next.textContent = step === steps.length - 1 ? "Finish guide" : "Next";
    if (step === 1) {
      if (!palette) palette = S.segmented(content.querySelector(".onboarding-palette"), { value: paletteValue(), onChange: S.setPalette });
      else palette.set(paletteValue(), { silent: true, instant: true });
    }
    if (step === 2) {
      content.querySelector("[data-guide-motion]").setAttribute("aria-pressed", "false");
      content.querySelector(".onboarding-motion-track").removeAttribute("data-selected");
      content.querySelector("[data-guide-motion-status]").textContent = "Ready to move.";
    }
    syncMotion();
    dialog.querySelector(".onboarding-body").scrollTop = 0;
    if (focus) title.focus({ preventScroll: true });
    if (!reduced()) entrance = content.animate([{ opacity: 0, transform: "translateY(8px)" }, { opacity: 1, transform: "none" }], { duration: S.dur(320), easing: S.curve.outExpo });
  };

  const restoreFocus = () => {
    const target = returnTo?.isConnected && !returnTo.closest("[inert]") && returnTo.getClientRects().length ? returnTo : launch;
    target?.focus({ preventScroll: true });
    returnTo = null;
  };
  const cleanup = () => {
    entrance?.cancel();
    exit?.cancel();
    exit = entrance = null;
    closing = null;
    launch.setAttribute("aria-expanded", "false");
    restoreFocus();
  };

  const close = () => {
    if (!dialog?.open) return Promise.resolve();
    if (closing) return closing;
    entrance?.cancel();
    // A reduced-motion dismissal is immediate; slow-mo scales the exit too.
    if (reduced()) { dialog.close(); cleanup(); return Promise.resolve(); }
    const animation = exit = dialog.animate([{ opacity: 1, transform: "none" }, { opacity: 0, transform: "translateY(8px)" }], { duration: S.dur(220), easing: S.curve.in, fill: "forwards" });
    closing = animation.finished.catch(() => {}).then(() => {
      // Reopening interrupts the exit instead of closing a newly opened guide.
      if (exit !== animation) return;
      dialog.close(); cleanup();
    });
    return closing;
  };
  const open = () => {
    if (!dialog) return false;
    if (dialog.open && !closing) return true;
    if (!dialog.open) {
      // Keep existing modal workflows independent; never stack on top of one.
      if (document.querySelector("dialog[open]")) return false;
      returnTo = document.activeElement instanceof HTMLElement && document.activeElement !== document.body ? document.activeElement : launch;
      dialog.showModal();
    }
    exit?.cancel(); exit = null; closing = null;
    step = 0;
    render();
    launch.setAttribute("aria-expanded", "true");
    return true;
  };
  S.onboarding = { open, close };

  document.addEventListener("DOMContentLoaded", () => {
    launch = document.querySelector("#guide-btn");
    if (!launch) return;
    dialog = S.html(`
      <dialog class="onboarding-dialog" id="schematic-guide" aria-labelledby="onboarding-title" aria-describedby="onboarding-desc">
        <div class="onboarding-head"><span class="t-eyebrow">Schematic / Quick guide</span><button type="button" class="btn btn-ghost btn-sm" data-guide-close>Close guide ${S.icon("x", { size: 14 })}</button></div>
        <div class="onboarding-progress-head"><span class="t-caption">A minute to make it yours</span><span class="t-caption t-mono" data-guide-count>1 / 4</span></div>
        <ol class="onboarding-progress" aria-label="Guide steps">${labels.map((label, i) => `<li${i === 0 ? ' aria-current="step"' : ""}><span>${label}</span></li>`).join("")}</ol>
        <div class="onboarding-body"><h2 class="t-h2" id="onboarding-title" tabindex="-1"></h2><p class="onboarding-desc" id="onboarding-desc"></p><div class="onboarding-content"></div></div>
        <div class="onboarding-foot"><p class="t-caption">Close anytime. Reopen from Guide.</p><div class="onboarding-actions"><button type="button" class="btn btn-secondary" data-guide-back>Back</button><button type="button" class="btn btn-primary" data-guide-next>Next</button></div></div>
      </dialog>`);
    document.body.append(dialog);
    title = dialog.querySelector("#onboarding-title");
    content = dialog.querySelector(".onboarding-content");
    progress = dialog.querySelector(".onboarding-progress");
    back = dialog.querySelector("[data-guide-back]");
    next = dialog.querySelector("[data-guide-next]");
    const nudge = document.createElement("span");
    nudge.className = "onboarding-nudge";
    nudge.textContent = "Start here";
    launch.append(nudge);
    launch.setAttribute("aria-expanded", "false");
    syncNudge();
    launch.addEventListener("click", open);
    dialog.querySelector("[data-guide-close]").addEventListener("click", close);
    back.addEventListener("click", () => { if (!closing && step > 0) { step--; render(); } });
    next.addEventListener("click", () => {
      if (closing) return;
      if (step < steps.length - 1) { step++; render(); }
      else { S.store.set(completedKey, "1"); syncNudge(); close(); }
    });
    content.addEventListener("click", (e) => {
      const button = e.target.closest("[data-guide-motion]");
      if (!button || closing) return;
      const selected = button.getAttribute("aria-pressed") !== "true";
      button.setAttribute("aria-pressed", String(selected));
      content.querySelector(".onboarding-motion-track").toggleAttribute("data-selected", selected);
      content.querySelector("[data-guide-motion-status]").textContent = selected ? "Selected. Try again to return." : "Ready. Try again to select.";
    });
    dialog.addEventListener("cancel", (e) => { e.preventDefault(); close(); });
    dialog.addEventListener("close", () => { if (!dialog.open && returnTo) cleanup(); });
    // Keep app shortcuts local. Explicitly wrap Tab at the two boundaries;
    // some browsers otherwise visit their chrome before cycling a native dialog.
    dialog.addEventListener("keydown", (e) => {
      e.stopPropagation();
      if ((e.metaKey || e.ctrlKey) && e.key.toLowerCase() === "k") e.preventDefault();
      if (e.key === "Tab") {
        const controls = [...dialog.querySelectorAll("button:not(:disabled), [href], input:not(:disabled), [tabindex='0']")].filter((el) => el.getClientRects().length && !el.closest("[inert]"));
        const first = controls[0], last = controls[controls.length - 1];
        if (e.shiftKey && document.activeElement === first) { e.preventDefault(); last?.focus(); }
        else if (!e.shiftKey && document.activeElement === last) { e.preventDefault(); first?.focus(); }
      }
    });
    const preferences = new MutationObserver(() => {
      if (!dialog.open) return;
      palette?.set(paletteValue(), { silent: true });
      syncMotion();
    });
    preferences.observe(root, { attributes: true, attributeFilter: ["data-palette", "data-motion"] });
    matchMedia("(prefers-reduced-motion: reduce)").addEventListener("change", syncMotion);
    window.addEventListener("storage", (e) => { if (e.key === null || e.key === `schematic:${completedKey}`) syncNudge(); });
  });
})();

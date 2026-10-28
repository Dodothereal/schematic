/* Optional, local-only quick guide. App and palette boot before this listener. */
(() => {
  const S = window.Schematic;
  const root = document.documentElement;
  const completedKey = "guide-completed";
  const labels = ["Explore", "Palette", "Motion", "Reuse"];
  const panels = [];
  const transitions = new Map(), presence = new Map();
  let dialog, launch, title, content, body, progress, back, next, palette;
  let step = 0, returnTo = null, closing = null;

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
    dialog?.querySelectorAll("[data-guide-motion-help]").forEach((help) => { help.textContent = reduced()
      ? "Reduced motion is on. The sample changes position instantly. Tune tokens offers motion settings."
      : "Tune tokens offers Full, Slow-mo and Reduced. Your browser's reduced-motion preference is respected unless you select Full."; });
  };

  const cancel = (animations) => {
    animations.forEach((animation) => animation.cancel());
    animations.clear();
  };
  const play = (animations, key, element, frames, options) => {
    const animation = element.animate(frames, { fill: "both", ...options });
    animations.set(key, animation);
    animation.finished.then(() => {
      if (animations.get(key) !== animation) return;
      animations.delete(key);
      animation.cancel();
    }, () => {});
    return animation;
  };

  const render = ({ focus = true, animate = true, direction = 1 } = {}) => {
    // Read the displayed frame before cancelling. A quick reversal continues
    // from that frame, including a panel that was already on its way out.
    const frames = panels.map((panel) => {
      const style = getComputedStyle(panel);
      return { opacity: style.opacity, transform: style.transform, visible: style.visibility === "visible" };
    });
    cancel(transitions);
    panels.forEach((panel, i) => {
      const active = i === step;
      panel.toggleAttribute("data-active", active);
      panel.inert = !active;
      panel.setAttribute("aria-hidden", String(!active));
      panel.querySelector("h2").id = active ? "onboarding-title" : `onboarding-title-${i}`;
      panel.querySelector(".onboarding-desc").id = active ? "onboarding-desc" : `onboarding-desc-${i}`;
    });
    title = panels[step].querySelector("h2");
    content = panels[step].querySelector(".onboarding-content");
    dialog.querySelector("[data-guide-count]").textContent = `${step + 1} / ${steps.length}`;
    progress.querySelectorAll("li").forEach((item, i) => {
      if (i === step) item.setAttribute("aria-current", "step"); else item.removeAttribute("aria-current");
      item.toggleAttribute("data-past", i < step);
    });
    back.disabled = step === 0;
    const nextLabel = next.querySelector("span"), label = step === steps.length - 1 ? "Finish guide" : "Next";
    const labelChanged = nextLabel.textContent !== label;
    nextLabel.textContent = label;
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
    body.scrollTop = 0;
    if (focus) title.focus({ preventScroll: true });
    if (!animate || reduced()) return;
    panels.forEach((panel, i) => {
      const frame = frames[i], active = i === step;
      if (!active && (!frame.visible || Number(frame.opacity) < 0.001)) return;
      play(transitions, panel, panel, [
        { opacity: frame.visible ? frame.opacity : 0, transform: frame.visible ? frame.transform : `translateX(${direction * 24}px)`, visibility: "visible" },
        { opacity: active ? 1 : 0, transform: active ? "none" : `translateX(${-direction * 18}px)`, visibility: "visible" },
      // Let the old text clear before the new reading surface arrives. Mixing
      // two readable paragraphs in a crossfade creates a distracting ghost.
      ], { duration: S.dur(active ? 460 : 140), delay: active && !frame.visible ? S.dur(140) : 0, easing: active ? S.curve.out : S.curve.in });
    });
    if (labelChanged) play(transitions, nextLabel, nextLabel, [{ opacity: 0, transform: "translateY(4px)" }, { opacity: 1, transform: "none" }], { duration: S.dur(220), easing: S.curve.out });
  };

  const restoreFocus = () => {
    const target = returnTo?.isConnected && !returnTo.closest("[inert]") && returnTo.getClientRects().length ? returnTo : launch;
    target?.focus({ preventScroll: true });
    returnTo = null;
  };
  const cleanup = () => {
    cancel(transitions);
    cancel(presence);
    closing = null;
    launch.setAttribute("aria-expanded", "false");
    restoreFocus();
  };

  const close = () => {
    if (!dialog?.open) return Promise.resolve();
    if (closing) return closing;
    // A reduced-motion dismissal is immediate; slow-mo scales the exit too.
    if (reduced()) { dialog.close(); cleanup(); return Promise.resolve(); }
    const frame = getComputedStyle(dialog), shade = getComputedStyle(dialog, "::backdrop").opacity;
    const start = { opacity: frame.opacity, transform: frame.transform };
    cancel(presence);
    // Hold the final frame until the native dialog leaves the top layer.
    const animation = dialog.animate([start, { opacity: 0, transform: "translateY(8px) scale(.99)" }], { duration: S.dur(220), easing: S.curve.in, fill: "both" });
    presence.set("surface", animation);
    const backdrop = dialog.animate([{ opacity: shade }, { opacity: 0 }], { pseudoElement: "::backdrop", duration: S.dur(220), easing: S.curve.in, fill: "both" });
    presence.set("backdrop", backdrop);
    closing = animation.finished.catch(() => {}).then(() => {
      // Reopening owns a different animation and cancels this dismissal.
      if (presence.get("surface") !== animation) return;
      dialog.close(); cleanup();
    });
    return closing;
  };
  const open = () => {
    if (!dialog) return false;
    if (dialog.open && !closing) return true;
    const fresh = !dialog.open;
    if (fresh) {
      // Keep existing modal workflows independent; never stack on top of one.
      if (document.querySelector("dialog[open]")) return false;
      returnTo = document.activeElement instanceof HTMLElement && document.activeElement !== document.body ? document.activeElement : launch;
      dialog.showModal();
    }
    const style = getComputedStyle(dialog);
    const start = fresh ? { opacity: 0, transform: "translateY(16px) scale(.985)" } : { opacity: style.opacity, transform: style.transform };
    const shade = fresh ? 0 : getComputedStyle(dialog, "::backdrop").opacity;
    cancel(presence); closing = null;
    step = 0;
    render({ animate: !fresh, direction: -1 });
    if (!reduced()) {
      play(presence, "surface", dialog, [start, { opacity: 1, transform: "none" }], { duration: S.dur(460), easing: S.curve.outExpo });
      play(presence, "backdrop", dialog, [{ opacity: shade }, { opacity: 1 }], { pseudoElement: "::backdrop", duration: S.dur(320), easing: S.curve.out });
    }
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
        <div class="onboarding-body"><div class="onboarding-stage"></div></div>
        <div class="onboarding-foot"><p class="t-caption">Close anytime. Reopen from Guide.</p><div class="onboarding-actions"><button type="button" class="btn btn-secondary" data-guide-back>${S.icon("arrow-left", { size: 16 })} Back</button><button type="button" class="btn btn-primary" data-guide-next><span>Next</span>${S.icon("arrow-right", { size: 16 })}</button></div></div>
      </dialog>`);
    document.body.append(dialog);
    body = dialog.querySelector(".onboarding-body");
    const stage = dialog.querySelector(".onboarding-stage");
    steps.forEach((current) => {
      const panel = S.html(`<section class="onboarding-panel" inert aria-hidden="true"><h2 class="t-h2" tabindex="-1">${current.title}</h2><p class="onboarding-desc">${current.desc}</p><div class="onboarding-content"><div class="onboarding-step">${current.body()}</div></div></section>`);
      panels.push(panel);
      stage.append(panel);
    });
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
    back.addEventListener("click", () => { if (!closing && step > 0) { step--; render({ direction: -1 }); } });
    next.addEventListener("click", () => {
      if (closing) return;
      if (step < steps.length - 1) { step++; render(); }
      else { S.store.set(completedKey, "1"); syncNudge(); close(); }
    });
    body.addEventListener("click", (e) => {
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
    const settleReduced = () => {
      syncMotion();
      if (!dialog.open || !reduced()) return;
      cancel(transitions);
      cancel(presence);
      if (closing) { dialog.close(); cleanup(); }
    };
    const preferences = new MutationObserver((records) => {
      if (!dialog.open) return;
      palette?.set(paletteValue(), { silent: true });
      syncMotion();
      if (records.some((record) => record.attributeName === "data-motion")) settleReduced();
    });
    preferences.observe(root, { attributes: true, attributeFilter: ["data-palette", "data-motion"] });
    matchMedia("(prefers-reduced-motion: reduce)").addEventListener("change", settleReduced);
    window.addEventListener("storage", (e) => { if (e.key === null || e.key === `schematic:${completedKey}`) syncNudge(); });
  });
})();

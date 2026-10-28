/* =============================================================================
   SCHEMATIC — CORE RUNTIME
   Motion utilities, interaction primitives, router with view transitions and
   intent-based prerendering, tooltips, toasts, command palette, theming.
   Pages register with Schematic.page({...}) — see CONTRACT.md.
   ============================================================================= */
(() => {
  const S = (window.Schematic = window.Schematic || {});
  const root = document.documentElement;

  /* ===================================================================== utils */
  S.$ = (sel, ctx = document) => ctx.querySelector(sel);
  S.$$ = (sel, ctx = document) => [...ctx.querySelectorAll(sel)];
  S.html = (str) => { const t = document.createElement("template"); t.innerHTML = str.trim(); return t.content.firstElementChild; };
  S.esc = (s) => String(s).replace(/[&<>"']/g, (c) => ({ "&": "&amp;", "<": "&lt;", ">": "&gt;", '"': "&quot;", "'": "&#39;" }[c]));
  S.clamp = (v, a, b) => Math.min(b, Math.max(a, v));
  S.lerp = (a, b, t) => a + (b - a) * t;
  S.token = (name) => getComputedStyle(root).getPropertyValue(name).trim();
  S.motionScale = () => { const n = parseFloat(S.token("--motion-scale")); return Number.isFinite(n) ? n : 1; };
  S.reduced = () => S.motionScale() < 0.05;
  /** Scale a duration in ms by the global motion setting */
  S.dur = (ms) => ms * S.motionScale();
  S.wait = (ms) => new Promise((r) => setTimeout(r, S.dur(ms)));
  S.nextFrame = () => new Promise((r) => requestAnimationFrame(() => requestAnimationFrame(r)));
  S.idle = (fn) => (window.requestIdleCallback ? requestIdleCallback(fn, { timeout: 1500 }) : setTimeout(fn, 120));
  /** Report clipboard success only after a confirmed write, including file://. */
  S.copy = async (text) => {
    try { await navigator.clipboard.writeText(text); return true; } catch {}
    const active = document.activeElement;
    const field = document.createElement("textarea");
    field.value = text; field.setAttribute("aria-label", "Copy text");
    field.style.cssText = "position:fixed;opacity:0;pointer-events:none";
    document.body.append(field); field.select();
    let ok = false;
    try { ok = document.execCommand("copy"); } catch {}
    field.remove(); active?.focus({ preventScroll: true });
    return ok;
  };

  /* ===================================================================== easing
     JS mirrors of the CSS tokens, for rAF tweens (charts, scroll, counters). */
  const bezier = (x1, y1, x2, y2) => {
    const cx = 3 * x1, bx = 3 * (x2 - x1) - cx, ax = 1 - cx - bx;
    const cy = 3 * y1, by = 3 * (y2 - y1) - cy, ay = 1 - cy - by;
    const sx = (t) => ((ax * t + bx) * t + cx) * t;
    const sy = (t) => ((ay * t + by) * t + cy) * t;
    const dx = (t) => (3 * ax * t + 2 * bx) * t + cx;
    return (x) => {
      if (x <= 0) return 0; if (x >= 1) return 1;
      let t = x;
      for (let i = 0; i < 8; i++) { const e = sx(t) - x; if (Math.abs(e) < 1e-6) return sy(t); const d = dx(t); if (Math.abs(d) < 1e-6) break; t -= e / d; }
      let lo = 0, hi = 1; t = x;
      for (let i = 0; i < 30; i++) { if (sx(t) < x) lo = t; else hi = t; t = (lo + hi) / 2; }
      return sy(t);
    };
  };
  const fromLinear = (str) => {
    const pts = (str.match(/linear\((.*)\)/)?.[1] || "0, 1").split(",").map((v) => parseFloat(v));
    const n = pts.length - 1;
    return (x) => { if (x <= 0) return 0; if (x >= 1) return 1; const p = x * n, i = Math.floor(p); return pts[i] + (pts[i + 1] - pts[i]) * (p - i); };
  };
  S.curve = {
    out: "cubic-bezier(0.22, 1, 0.36, 1)",
    outExpo: "cubic-bezier(0.16, 1, 0.3, 1)",
    in: "cubic-bezier(0.64, 0, 0.78, 0)",
    inOut: "cubic-bezier(0.65, 0, 0.35, 1)",
    overshoot: "cubic-bezier(0.34, 1.56, 0.64, 1)",
  };
  S.ease = {
    linear: (t) => t,
    out: bezier(0.22, 1, 0.36, 1),
    outExpo: bezier(0.16, 1, 0.3, 1),
    in: bezier(0.64, 0, 0.78, 0),
    inOut: bezier(0.65, 0, 0.35, 1),
    overshoot: bezier(0.34, 1.56, 0.64, 1),
  };
  S.springDur = { smooth: 600, snappy: 470, bouncy: 930 };
  const loadSprings = () => {
    ["smooth", "snappy", "bouncy"].forEach((k) => {
      const v = S.token(`--spring-${k}`);
      S.curve[`spring${k[0].toUpperCase()}${k.slice(1)}`] = v;
      S.ease[`spring${k[0].toUpperCase()}${k.slice(1)}`] = fromLinear(v);
    });
  };

  /* ===================================================================== tween
     S.tween({from, to, duration, ease, delay, onUpdate}) → { cancel, finished }
     from/to may be numbers or equal-length arrays (for path morphing).       */
  S.tween = ({ from = 0, to = 1, duration = 400, ease = S.ease.out, delay = 0, onUpdate, onDone } = {}) => {
    const d = S.dur(duration), dl = S.dur(delay), arr = Array.isArray(from);
    let raf, start = null, dead = false, resolve;
    const finished = new Promise((r) => (resolve = r));
    const step = (now) => {
      if (dead) return;
      if (start === null) start = now + dl;
      if (now < start) { raf = requestAnimationFrame(step); return; }
      const t = d <= 0 ? 1 : S.clamp((now - start) / d, 0, 1), k = ease(t);
      onUpdate?.(arr ? from.map((f, i) => f + (to[i] - f) * k) : from + (to - from) * k, t);
      if (t < 1) raf = requestAnimationFrame(step); else { onDone?.(); resolve(); }
    };
    raf = requestAnimationFrame(step);
    return { cancel() { dead = true; cancelAnimationFrame(raf); resolve(); }, finished };
  };

  /** Animate a number in an element's text. */
  S.countTo = (el, to, { from, duration = 900, format = (v) => Math.round(v).toLocaleString(), ease = S.ease.outExpo } = {}) => {
    el._count?.cancel();
    const start = from ?? el._countVal ?? 0;
    el._countVal = to;
    el._count = S.tween({ from: start, to, duration, ease, onUpdate: (v) => (el.textContent = format(v)) });
    return el._count.finished;
  };

  /* ===================================================================== FLIP
     S.flip(elements, mutate, opts) — record, mutate the DOM, animate the delta.
     New elements (not present before) scale/fade in.                          */
  S.flip = (els, mutate, { duration = S.springDur.snappy, easing, stagger = 0, enter = true } = {}) => {
    const list = [...(typeof els === "function" ? els() : els)];
    // Capture the displayed positions before cancelling an in-flight reorder.
    // Repeated actions then continue from the visible frame instead of jumping
    // back to the preceding layout before starting their next move.
    const first = new Map(list.map((el) => [el, el.getBoundingClientRect()]));
    list.forEach((el) => el.getAnimations().forEach((a) => a.cancel()));
    mutate();
    if (S.reduced()) return Promise.resolve([]);
    const ease = easing || S.curve.springSnappy || S.curve.out;
    const anims = [];
    const after = typeof els === "function" ? els() : list;
    [...after].forEach((el, i) => {
      if (!el.isConnected || el.hidden) return;
      const f = first.get(el), l = el.getBoundingClientRect();
      if (!f) {
        if (enter) anims.push(el.animate([{ opacity: 0, transform: "scale(0.94)" }, { opacity: 1, transform: "none" }], { duration: S.dur(duration), easing: ease, delay: S.dur(i * stagger), fill: "backwards" }));
        return;
      }
      const dx = f.left - l.left, dy = f.top - l.top;
      const sx = l.width ? f.width / l.width : 1, sy = l.height ? f.height / l.height : 1;
      if (Math.abs(dx) < 0.5 && Math.abs(dy) < 0.5 && Math.abs(sx - 1) < 0.01 && Math.abs(sy - 1) < 0.01) return;
      anims.push(el.animate(
        [{ transformOrigin: "top left", transform: `translate(${dx}px, ${dy}px) scale(${sx}, ${sy})` }, { transformOrigin: "top left", transform: "none" }],
        { duration: S.dur(duration), easing: ease, delay: S.dur(i * stagger) }
      ));
    });
    return Promise.all(anims.map((a) => a.finished.catch(() => {})));
  };

  /* ===================================================================== number roll
     Digits roll on vertical strips (odometer). S.roll(el, "1,284")            */
  S.roll = (el, value, { stagger = 28 } = {}) => {
    const text = String(value), chars = [...text];
    el.classList.add("roll");
    el.setAttribute("aria-label", text);
    const sig = chars.map((c) => (/\d/.test(c) ? "d" : c)).join("");
    if (el._rollSig !== sig) {
      el._rollSig = sig;
      el.innerHTML = chars.map((c) => /\d/.test(c)
        ? `<span class="roll-col" aria-hidden="true"><span class="roll-strip" style="--d:0">${"0123456789".split("").map((n) => `<span>${n}</span>`).join("")}</span></span>`
        : `<span class="roll-sym" aria-hidden="true">${S.esc(c)}</span>`).join("");
      el.offsetWidth; // commit the 0-state so the first roll animates
    }
    const strips = S.$$(".roll-strip", el);
    const digits = chars.filter((c) => /\d/.test(c));
    strips.forEach((strip, i) => {
      strip.style.transitionDelay = `${S.dur((strips.length - 1 - i) * stagger)}ms`; // ones place leads
      strip.style.setProperty("--d", digits[i]);
    });
  };

  /* ===================================================================== spotlight
     Any .btn, .spot or [data-spot] gets --mx/--my = pointer position inside it */
  document.addEventListener("pointermove", (e) => {
    const el = e.target.closest?.(".btn, .spot, [data-spot]");
    if (!el) return;
    const r = el.getBoundingClientRect();
    el.style.setProperty("--mx", `${e.clientX - r.left}px`);
    el.style.setProperty("--my", `${e.clientY - r.top}px`);
  }, { passive: true });

  /* ===================================================================== glide
     S.glide(container, { items, persist }) — a single hover highlight that
     travels between items. Returns { moveTo(el), hide() }.                    */
  S.glide = (container, { items = "[data-glide-item]", persist = false, className = "" } = {}) => {
    container.setAttribute("data-glide", "");
    const pill = document.createElement("span");
    pill.className = `glide-pill ${className}`.trim();
    pill.setAttribute("aria-hidden", "true");
    container.prepend(pill);
    let on = false, current = null;
    const moveTo = (el, { instant = false } = {}) => {
      if (!el) return;
      current = el;
      const c = container.getBoundingClientRect(), r = el.getBoundingClientRect();
      const x = r.left - c.left + container.scrollLeft, y = r.top - c.top + container.scrollTop;
      const place = () => { pill.style.transform = `translate(${x}px, ${y}px)`; pill.style.width = `${r.width}px`; pill.style.height = `${r.height}px`; };
      if (!on || instant) { pill.classList.add("no-move"); place(); pill.offsetWidth; pill.classList.remove("no-move"); }
      else place();
      pill.classList.add("is-on"); on = true;
    };
    const hide = () => { pill.classList.remove("is-on"); on = false; current = null; };
    container.addEventListener("pointerover", (e) => {
      if (e.pointerType === "touch") return;
      const it = e.target.closest(items);
      if (it && container.contains(it) && it !== current && !it.matches(":disabled,[aria-disabled=true]")) moveTo(it);
    });
    if (!persist) {
      container.addEventListener("pointerleave", hide);
      container.addEventListener("focusin", (e) => { const it = e.target.closest(items); if (it && e.target.matches(":focus-visible")) moveTo(it); });
      container.addEventListener("focusout", (e) => { if (!container.contains(e.relatedTarget)) hide(); });
    }
    return { moveTo, hide, pill, get current() { return current; } };
  };

  /* ===================================================================== segmented
     <div class="seg"><button class="seg-btn" data-value="a">…</button>…</div>
     The thumb springs to the chosen option. S.segmented(el, { value, onChange }) */
  S.segmented = (el, { value, onChange } = {}) => {
    if (!el) return;
    const thumb = document.createElement("span");
    thumb.className = "seg-thumb no-move"; thumb.setAttribute("aria-hidden", "true");
    el.prepend(thumb);
    const btns = S.$$(".seg-btn", el);
    if (!el.hasAttribute("role")) el.setAttribute("role", "group");
    const place = (btn) => {
      thumb.style.transform = `translateX(${btn.offsetLeft}px)`;
      thumb.style.width = `${btn.offsetWidth}px`;
    };
    const set = (v, { silent = false, instant = false } = {}) => {
      const btn = btns.find((b) => b.dataset.value === String(v)) || btns[0];
      btns.forEach((b) => b.setAttribute("aria-pressed", String(b === btn)));
      if (instant) thumb.classList.add("no-move");
      place(btn);
      if (instant) { thumb.offsetWidth; thumb.classList.remove("no-move"); }
      if (!silent) onChange?.(btn.dataset.value, btn);
    };
    btns.forEach((b) => b.addEventListener("click", () => set(b.dataset.value)));
    el.addEventListener("keydown", (e) => {
      const i = btns.indexOf(e.target);
      if (i < 0 || !["ArrowLeft", "ArrowRight", "Home", "End"].includes(e.key)) return;
      e.preventDefault();
      const next = e.key === "Home" ? 0 : e.key === "End" ? btns.length - 1 : (i + (e.key === "ArrowRight" ? 1 : -1) + btns.length) % btns.length;
      btns[next].focus(); set(btns[next].dataset.value);
    });
    const init = () => { set(value ?? (btns.find((b) => b.getAttribute("aria-pressed") === "true") || btns[0]).dataset.value, { silent: true, instant: true }); };
    if (el.isConnected && el.offsetWidth) init(); else requestAnimationFrame(init);
    new ResizeObserver(() => { const a = btns.find((b) => b.getAttribute("aria-pressed") === "true"); if (a) { thumb.classList.add("no-move"); place(a); thumb.offsetWidth; thumb.classList.remove("no-move"); } }).observe(el);
    return { set, get value() { return btns.find((b) => b.getAttribute("aria-pressed") === "true")?.dataset.value; } };
  };

  /* ===================================================================== popover
     S.popover(trigger, panel) — toggles [data-open], closes on outside click /
     Escape, and sets transform-origin so the panel grows out of its trigger.  */
  S.popover = (trigger, panel, { onOpen, onClose } = {}) => {
    panel.inert = !panel.hasAttribute("data-open");
    const open = () => {
      panel.inert = false;
      panel.setAttribute("data-open", ""); trigger.setAttribute("aria-expanded", "true");
      const t = trigger.getBoundingClientRect(), p = panel.getBoundingClientRect();
      panel.style.setProperty("--pop-origin", `${S.clamp(t.left + t.width / 2 - p.left, 0, p.width)}px ${t.top < p.top ? "0" : "100%"}`);
      onOpen?.();
    };
    const close = () => { panel.removeAttribute("data-open"); panel.inert = true; trigger.setAttribute("aria-expanded", "false"); onClose?.(); };
    const isOpen = () => panel.hasAttribute("data-open");
    trigger.addEventListener("click", (e) => { e.stopPropagation(); isOpen() ? close() : open(); });
    document.addEventListener("pointerdown", (e) => { if (isOpen() && !panel.contains(e.target) && !trigger.contains(e.target)) close(); });
    document.addEventListener("keydown", (e) => { if (e.key === "Escape" && isOpen()) { close(); trigger.focus(); } });
    document.addEventListener("schematic:navigate", close);
    return { open, close, isOpen };
  };

  S._loadSprings = loadSprings;
})();

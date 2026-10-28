/* =============================================================================
   SCHEMATIC — FEEDBACK: tooltips & toasts
   ============================================================================= */
(() => {
  const S = window.Schematic;

  /* ===================================================================== tooltip
     <button data-tip="Copy" data-tip-kbd="⌘C" data-tip-side="bottom">
     · first tooltip waits 450ms (intent), then the group is "warm":
       moving to a neighbour makes the SAME tooltip glide over — no re-fade.
     · hides on press, scroll and Escape.                                     */
  const initTooltip = () => {
    const tip = S.html(`<div class="tooltip" role="tooltip" id="schematic-tip"><div class="tooltip-inner"><span class="tip-text"></span></div></div>`);
    document.body.append(tip);
    const inner = tip.firstElementChild, text = S.$(".tip-text", tip);
    let current = null, describedTarget = null, showT, warmT, warm = false, visible = false;
    const detachDescription = () => {
      if (!describedTarget) return;
      const rest = (describedTarget.getAttribute("aria-describedby") || "").split(/\s+/).filter((id) => id && id !== tip.id);
      rest.length ? describedTarget.setAttribute("aria-describedby", rest.join(" ")) : describedTarget.removeAttribute("aria-describedby");
      describedTarget = null;
    };

    const position = (el) => {
      const r = el.getBoundingClientRect(), tr = inner.getBoundingClientRect();
      let side = el.dataset.tipSide || "top";
      const gap = 8;
      let x, y;
      if (side === "top" && r.top - tr.height - gap < 8) side = "bottom";
      if (side === "bottom" && r.bottom + tr.height + gap > innerHeight - 8) side = "top";
      if (side === "top" || side === "bottom") {
        x = r.left + r.width / 2 - tr.width / 2;
        y = side === "top" ? r.top - tr.height - gap : r.bottom + gap;
      } else {
        y = r.top + r.height / 2 - tr.height / 2;
        x = side === "right" ? r.right + gap : r.left - tr.width - gap;
      }
      x = S.clamp(x, 8, innerWidth - tr.width - 8);
      tip.style.transform = `translate(${Math.round(x)}px, ${Math.round(y)}px)`;
      inner.style.setProperty("--tip-dy", side === "top" ? "3px" : side === "bottom" ? "-3px" : "0px");
      inner.style.setProperty("--tip-origin", { top: "bottom center", bottom: "top center", left: "center right", right: "center left" }[side]);
    };
    const fill = (el) => {
      const kbd = el.dataset.tipKbd;
      text.innerHTML = S.esc(el.dataset.tip) + (kbd ? ` <span class="kbd-group" style="margin-left:6px">${kbd.split(" ").map((k) => `<kbd class="kbd">${S.esc(k)}</kbd>`).join("")}</span>` : "");
    };
    const show = (el) => {
      clearTimeout(warmT);
      detachDescription();
      const glide = visible && warm;
      fill(el);
      if (glide) {
        tip.classList.add("is-gliding");
        text.classList.remove("is-swap"); text.offsetWidth; text.classList.add("is-swap");
      } else tip.classList.remove("is-gliding");
      position(el);
      tip.classList.add("is-on");
      visible = true; warm = true;
      const described = new Set((el.getAttribute("aria-describedby") || "").split(/\s+/).filter(Boolean));
      described.add(tip.id); el.setAttribute("aria-describedby", [...described].join(" "));
      describedTarget = el;
    };
    const hide = () => {
      clearTimeout(showT);
      detachDescription();
      tip.classList.remove("is-on");
      visible = false; current = null;
      clearTimeout(warmT);
      warmT = setTimeout(() => (warm = false), 400);
    };

    document.addEventListener("pointerover", (e) => {
      if (e.pointerType === "touch") return;
      const t = e.target.closest?.("[data-tip]");
      if (t === current) return;
      if (!t) { if (current) hide(); return; }
      current = t;
      clearTimeout(showT);
      if (warm) show(t); else showT = setTimeout(() => current === t && show(t), 450);
    });
    document.addEventListener("focusin", (e) => { const t = e.target.closest?.("[data-tip]"); if (t && t.matches(":focus-visible")) { current = t; show(t); } });
    document.addEventListener("focusout", () => current && hide());
    document.addEventListener("pointerdown", () => current && hide(), true);
    document.addEventListener("scroll", () => current && hide(), true);
    document.addEventListener("keydown", (e) => e.key === "Escape" && current && hide());
    S.tooltipRefresh = () => current && visible && (fill(current), position(current));
  };

  /* ===================================================================== toasts
     S.toast({ title, desc, tone, icon, action:{label,onClick}, duration })
     Sonner-style stack: newest in front, older ones tuck behind. Hover to fan
     out, swipe right to dismiss, timers pause while you're reading.          */
  const initToaster = () => {
    const box = S.html(`<section class="toaster" aria-live="polite" aria-label="Notifications"></section>`);
    document.body.append(box);
    let toasts = [], expanded = false;
    const GAP = 10, PEEK = 10, MAX_VISIBLE = 3;
    const toneIcon = { success: "check-circle", danger: "x-circle", warning: "alert", info: "info", neutral: "bell" };

    const layout = () => {
      const live = toasts.filter((t) => !t.leaving);
      const frontH = live[0]?.h || 0;
      let offset = 0;
      live.forEach((t, i) => {
        const el = t.el;
        el.style.zIndex = String(100 - i);
        if (expanded) {
          el.style.transform = `translateY(${-offset}px)`;
          el.style.height = `${t.h}px`;
          el.style.opacity = i < 6 ? "1" : "0";
          el.classList.remove("is-behind");
          offset += t.h + GAP;
        } else {
          const s = 1 - i * 0.05;
          el.style.transform = `translateY(${-i * PEEK}px) scale(${s})`;
          el.style.height = `${i === 0 ? t.h : frontH}px`;
          el.style.opacity = i < MAX_VISIBLE ? "1" : "0";
          el.classList.toggle("is-behind", i > 0);
        }
        el.style.pointerEvents = i < (expanded ? 6 : MAX_VISIBLE) ? "auto" : "none";
        el.inert = expanded ? i >= 6 : i > 0;
      });
      box.style.height = expanded ? `${offset}px` : `${frontH}px`;
    };

    const dismiss = (t, dir = "down") => {
      if (t.leaving) return;
      t.leaving = true; clearTimeout(t.timer);
      const el = t.el;
      el.classList.add("is-leaving");
      const cur = el.style.transform || "";
      el.style.transform = dir === "right" ? `translateX(110%)` : `${cur} translateY(14px) scale(0.96)`;
      el.style.opacity = "0";
      layout();
      setTimeout(() => { el.remove(); toasts = toasts.filter((x) => x !== t); layout(); }, S.dur(240));
    };

    const arm = (t) => {
      clearTimeout(t.timer);
      if (t.duration === Infinity) return;
      t.started = Date.now();
      t.timer = setTimeout(() => dismiss(t), t.remaining);
    };
    const pause = (t) => { if (t.timer) { clearTimeout(t.timer); t.timer = null; t.remaining = Math.max(0, t.remaining - (Date.now() - t.started)); } };

    box.addEventListener("pointerenter", () => { expanded = true; toasts.forEach(pause); layout(); });
    box.addEventListener("pointerleave", () => { expanded = false; toasts.forEach((t) => !t.leaving && arm(t)); layout(); });
    box.addEventListener("focusin", () => { expanded = true; toasts.forEach(pause); layout(); });
    box.addEventListener("focusout", (e) => { if (!box.contains(e.relatedTarget)) { expanded = false; toasts.forEach((t) => !t.leaving && arm(t)); layout(); } });

    S.toast = (opts = {}) => {
      const o = typeof opts === "string" ? { title: opts } : opts;
      const tone = o.tone || "neutral";
      const el = S.html(`
        <div class="toast" role="status" data-tone="${tone}">
          <span class="toast-ic">${S.icon(o.icon || toneIcon[tone] || "bell")}</span>
          <div class="toast-body">
            <div class="toast-title">${S.esc(o.title || "")}</div>
            ${o.desc ? `<div class="toast-desc">${S.esc(o.desc)}</div>` : ""}
          </div>
          <div class="toast-actions">
            ${o.action ? `<button class="btn btn-secondary btn-sm" data-act>${S.esc(o.action.label)}</button>` : ""}
            <button class="btn btn-ghost btn-sm btn-icon" data-close aria-label="Dismiss">${S.icon("x")}</button>
          </div>
        </div>`);
      const t = { el, duration: o.duration ?? 4200, remaining: o.duration ?? 4200, h: 0 };
      box.append(el);
      t.h = el.offsetHeight;
      el.style.height = `${t.h}px`;
      // start below the fold, then spring into the stack
      el.style.transition = "none";
      el.style.transform = `translateY(calc(100% + 32px)) scale(0.98)`;
      el.style.opacity = "0";
      el.offsetWidth;
      el.style.transition = "";
      toasts.unshift(t);
      layout();
      requestAnimationFrame(() => S.playIcon(S.$(".toast-ic .ic", el)));
      if (!expanded) arm(t);

      el.querySelector("[data-close]").addEventListener("click", () => dismiss(t));
      el.querySelector("[data-act]")?.addEventListener("click", () => { o.action.onClick?.(); dismiss(t); });

      // swipe to dismiss
      let x0 = null, dx = 0, t0 = 0;
      el.addEventListener("pointerdown", (e) => {
        if (e.target.closest("button")) return;
        x0 = e.clientX; dx = 0; t0 = performance.now();
        el.setPointerCapture(e.pointerId); el.classList.add("is-dragging");
      });
      el.addEventListener("pointermove", (e) => {
        if (x0 === null) return;
        dx = e.clientX - x0;
        const d = dx < 0 ? dx * 0.2 : dx; // resist leftwards
        el.style.transform = `translateX(${d}px)`;
        el.style.opacity = String(1 - S.clamp(dx / 260, 0, 0.6));
      });
      const end = () => {
        if (x0 === null) return;
        el.classList.remove("is-dragging");
        const v = dx / Math.max(1, performance.now() - t0);
        x0 = null;
        if (dx > 90 || v > 0.6) dismiss(t, "right"); else { el.style.opacity = "1"; layout(); }
      };
      el.addEventListener("pointerup", end);
      el.addEventListener("pointercancel", end);
      return { dismiss: () => dismiss(t) };
    };
  };

  S._initFeedback = () => { initTooltip(); initToaster(); };
})();

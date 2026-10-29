/* =============================================================================
   SCHEMATIC — APP: command palette, theme, tuning panel, boot
   ============================================================================= */
(() => {
  const S = window.Schematic;
  const root = document.documentElement;
  const store = {
    get: (k, d) => { try { return localStorage.getItem(`schematic:${k}`) ?? d; } catch { return d; } },
    set: (k, v) => { try { localStorage.setItem(`schematic:${k}`, v); } catch {} },
  };
  S.store = store;

  /* ===================================================================== theme
     The new theme grows as a circle out of whatever triggered it.           */
  let themeTransition = null, themeTarget = null, themeVersion = 0;
  S.setTheme = async (theme, origin) => {
    const version = ++themeVersion;
    themeTarget = theme;
    // A skipped transition still runs its update callback. Drain that update
    // before taking another snapshot; stale callbacks must not apply a theme.
    const previous = themeTransition;
    if (previous) {
      previous.skipTransition();
      await previous.updateCallbackDone.catch(() => {});
    }
    if (version !== themeVersion) return;
    const finish = () => {
      if (version !== themeVersion) return;
      themeTransition = null;
      themeTarget = null;
      root.classList.remove("vt-theme");
    };
    if (theme === root.dataset.theme) { finish(); return; }
    const apply = () => { root.dataset.theme = theme; store.set("theme", theme); };
    if (!document.startViewTransition || S.reduced()) { apply(); finish(); return; }
    const r = origin?.getBoundingClientRect();
    const x = r ? r.left + r.width / 2 : innerWidth / 2, y = r ? r.top + r.height / 2 : 0;
    root.style.setProperty("--vt-x", `${x}px`);
    root.style.setProperty("--vt-y", `${y}px`);
    root.style.setProperty("--vt-r", `${Math.hypot(Math.max(x, innerWidth - x), Math.max(y, innerHeight - y))}px`);
    root.classList.add("vt-theme");
    let vt;
    try {
      vt = document.startViewTransition(() => { if (version === themeVersion) apply(); });
    } catch {
      apply(); finish(); return;
    }
    themeTransition = vt;
    // Cancellation can reject ready/finished; callers may safely await this API.
    vt.ready.catch(() => {});
    vt.updateCallbackDone.catch(() => {});
    await vt.finished.catch(() => {});
    finish();
  };
  S.toggleTheme = (origin) => S.setTheme((themeTarget ?? root.dataset.theme) === "dark" ? "light" : "dark", origin);

  /* ===================================================================== tuning */
  const HUES = [168, 195, 55, 18, 235, 290, 330, 0];
  S.setHue = (h) => { root.style.setProperty("--accent-h", h); store.set("hue", h); syncTune(); };
  S.setRadius = (v) => { v === "default" ? delete root.dataset.radius : (root.dataset.radius = v); store.set("radius", v); };
  S.setMotion = (v) => { v === "full" ? (root.dataset.motion = "full") : (root.dataset.motion = v); store.set("motion", v); requestAnimationFrame(S._loadSprings); };

  let syncTune = () => {};
  const initTune = () => {
    const btn = S.$("#tune-btn"), panel = S.$("#tune-pop");
    const hue = () => { const n = parseFloat(getComputedStyle(root).getPropertyValue("--accent-h")); return Number.isFinite(n) ? n : 168; };
    panel.innerHTML = `
      <div class="pop-section">
        <div class="pop-label"><span>Accent</span><span class="faint t-mono" id="hue-val"></span></div>
        <div class="swatches">${HUES.map((h) => `<button class="swatch" style="--h:${h}" data-hue="${h}" aria-label="Hue ${h}" data-tip="Hue ${h}°"></button>`).join("")}</div>
        <input class="range is-hue" id="hue-range" type="range" min="0" max="359" step="1" aria-label="Accent hue">
      </div>
      <div class="pop-section">
        <div class="pop-label">Corner radius</div>
        <div class="seg seg-block" id="radius-seg">
          <button class="seg-btn" data-value="sharp">Sharp</button>
          <button class="seg-btn" data-value="default">Default</button>
          <button class="seg-btn" data-value="round">Round</button>
        </div>
      </div>
      <div class="pop-section">
        <div class="pop-label">Motion</div>
        <div class="seg seg-block" id="motion-seg">
          <button class="seg-btn" data-value="full">Full</button>
          <button class="seg-btn" data-value="slow">Slow-mo ×4</button>
          <button class="seg-btn" data-value="reduced">Reduced</button>
        </div>
      </div>`;
    const range = S.$("#hue-range", panel), val = S.$("#hue-val", panel);
    syncTune = () => {
      const n = parseFloat(root.style.getPropertyValue("--accent-h"));
      const h = Math.round(Number.isFinite(n) ? n : hue());
      range.value = h; range.style.setProperty("--p", `${(h / 359) * 100}%`); val.textContent = `${h}°`;
      S.$$(".swatch", panel).forEach((s) => s.setAttribute("aria-pressed", String(+s.dataset.hue === h)));
    };
    S.$$(".swatch", panel).forEach((s) => s.addEventListener("click", () => S.setHue(+s.dataset.hue)));
    range.addEventListener("input", () => S.setHue(+range.value));
    let segs;
    S.popover(btn, panel, {
      onOpen: () => {
        syncTune();
        if (!segs) segs = [
          S.segmented(S.$("#radius-seg", panel), { value: store.get("radius", "default"), onChange: S.setRadius }),
          S.segmented(S.$("#motion-seg", panel), { value: store.get("motion", "full"), onChange: S.setMotion }),
        ];
      },
    });
  };

  /* ===================================================================== command palette */
  const fuzzy = (q, s) => {
    if (!q) return { score: 1, html: S.esc(s) };
    const lq = q.toLowerCase(), ls = s.toLowerCase();
    let qi = 0, score = 0, last = -2, out = "";
    for (let i = 0; i < s.length; i++) {
      if (qi < lq.length && ls[i] === lq[qi]) { score += i === last + 1 ? 3 : 1; if (i === 0 || s[i - 1] === " ") score += 2; last = i; qi++; out += `<mark>${S.esc(s[i])}</mark>`; }
      else out += S.esc(s[i]);
    }
    return qi === lq.length ? { score, html: out } : null;
  };

  const initCmdk = () => {
    const dlg = S.html(`
      <dialog class="cmdk" aria-label="Command palette">
        <div class="cmdk-panel">
          <div class="cmdk-input-row">${S.icon("search", { size: 20 })}<input class="cmdk-input" role="combobox" aria-controls="command-results" aria-autocomplete="list" aria-expanded="true" placeholder="Search pages, actions, settings…" aria-label="Search" autocomplete="off" spellcheck="false"><kbd class="kbd">esc</kbd></div>
          <div class="cmdk-list-wrap"><div class="cmdk-list" id="command-results" role="listbox" aria-label="Commands"></div></div>
          <div class="cmdk-foot"><span><kbd class="kbd">↑</kbd><kbd class="kbd">↓</kbd> navigate</span><span><kbd class="kbd">↵</kbd> open</span><span style="margin-left:auto">Schematic</span></div>
        </div>
      </dialog>`);
    document.body.append(dlg);
    const input = S.$(".cmdk-input", dlg), list = S.$(".cmdk-list", dlg), wrap = S.$(".cmdk-list-wrap", dlg);
    const glide = S.glide(list, { items: ".cmdk-item", persist: true, className: "cmdk-sel" });
    let items = [], sel = 0, closing = false;

    const commands = () => [
      ...S.pages.map((p) => ({ group: "Pages", label: p.title, icon: p.icon, hint: p.group, run: () => S.navigate(p.id) })),
      { group: "Actions", label: "Toggle theme", icon: "moon", hint: "⇧⌘L", run: () => S.toggleTheme(S.$("#theme-btn")) },
      { group: "Actions", label: "Show a toast", icon: "bell", run: () => S.toast({ title: "Hello from the palette", desc: "Toasts stack, fan out on hover and swipe away.", tone: "success" }) },
      { group: "Actions", label: "Slow-motion mode", icon: "clock", run: () => S.setMotion("slow") },
      { group: "Actions", label: "Full motion", icon: "bolt", run: () => S.setMotion("full") },
      ...[["Forest", 168], ["Teal", 195], ["Ochre", 55], ["Clay", 18], ["Blue", 235], ["Violet", 290], ["Rose", 330], ["Red", 0]]
        .map(([n, h]) => ({ group: "Accent", label: `Accent: ${n}`, icon: "droplet", run: () => S.setHue(h) })),
    ];

    const render = (fresh = false) => {
      const q = input.value.trim();
      items = commands().map((c) => ({ ...c, m: fuzzy(q, c.label) })).filter((c) => c.m).sort((a, b) => (q ? b.m.score - a.m.score : 0));
      sel = 0;
      const groups = q ? [["Results", items]] : [...new Set(items.map((i) => i.group))].map((g) => [g, items.filter((i) => i.group === g)]);
      const flat = groups.flatMap(([, arr]) => arr);
      items = flat;
      const before = wrap.offsetHeight;
      list.querySelectorAll(".cmdk-group-label, .cmdk-item, .cmdk-empty").forEach((n) => n.remove());
      let i = 0;
      list.insertAdjacentHTML("beforeend", flat.length ? groups.map(([g, arr]) => `<div class="cmdk-group-label t-eyebrow">${g}</div>` +
        arr.map((c) => `<div class="cmdk-item ${fresh || q ? "is-new" : ""}" id="command-option-${i}" role="option" data-idx="${i}" style="--i:${Math.min(i++, 12)}">${S.icon(c.icon)}<span>${c.m.html}</span>${c.hint ? `<span class="cmdk-item-hint">${c.hint}</span>` : ""}</div>`).join("")).join("")
        : `<div class="cmdk-empty">No results for “${S.esc(q)}”</div>`);
      // animate the list height between result sets
      wrap.style.height = "auto";
      const after = wrap.offsetHeight;
      if (before && before !== after) { wrap.style.height = `${before}px`; wrap.offsetHeight; wrap.style.height = `${after}px`; }
      select(0, true);
    };
    const select = (i, instant = false) => {
      const els = S.$$(".cmdk-item", list);
      if (!els.length) { glide.hide(); input.removeAttribute("aria-activedescendant"); return; }
      sel = (i + els.length) % els.length;
      els.forEach((el, k) => el.setAttribute("aria-selected", String(k === sel)));
      input.setAttribute("aria-activedescendant", els[sel].id);
      glide.moveTo(els[sel], { instant });
      els[sel].scrollIntoView({ block: "nearest" });
    };
    const run = async (i) => {
      const c = items[i]; if (!c) return;
      await close();
      c.run();
    };
    const open = () => { if (dlg.open) return; input.value = ""; dlg.showModal(); render(true); input.focus(); };
    const close = () => new Promise((res) => {
      if (!dlg.open || closing) return res();
      closing = true; dlg.classList.add("is-closing");
      setTimeout(() => { dlg.classList.remove("is-closing"); dlg.close(); closing = false; res(); }, S.dur(140));
    });
    S.cmdk = { open, close };

    input.addEventListener("input", () => render());
    input.addEventListener("keydown", (e) => {
      if (e.key === "ArrowDown") { e.preventDefault(); select(sel + 1); }
      else if (e.key === "ArrowUp") { e.preventDefault(); select(sel - 1); }
      else if (e.key === "Enter") { e.preventDefault(); run(sel); }
    });
    dlg.addEventListener("cancel", (e) => { e.preventDefault(); close(); });
    dlg.addEventListener("click", (e) => { if (e.target === dlg) close(); });
    list.addEventListener("pointermove", (e) => { const it = e.target.closest(".cmdk-item"); if (it && +it.dataset.idx !== sel) select(+it.dataset.idx); });
    list.addEventListener("click", (e) => { const it = e.target.closest(".cmdk-item"); if (it) run(+it.dataset.idx); });

    document.addEventListener("keydown", (e) => {
      const mod = e.metaKey || e.ctrlKey;
      if (mod && e.key.toLowerCase() === "k") { e.preventDefault(); dlg.open ? close() : open(); }
      if (mod && e.shiftKey && e.key.toLowerCase() === "l") { e.preventDefault(); S.toggleTheme(S.$("#theme-btn")); }
    });
  };

  /* ===================================================================== boot */
  const boot = () => {
    S._loadSprings();
    const shell = S._shell;
    shell.buildNav();
    S._initFeedback();
    initCmdk();
    initTune();

    S.$("#theme-btn").addEventListener("click", (e) => S.toggleTheme(e.currentTarget));
    S.$$("[data-open-cmdk]").forEach((b) => b.addEventListener("click", () => S.cmdk.open()));
    const menu = S.$("#menu-btn"), sidebar = S.$(".sidebar"), mobile = matchMedia("(max-width: 900px)");
    const overlayNav = () => mobile.matches || root.dataset.currentPage === "workspace";
    const syncSidebar = () => { sidebar.inert = overlayNav() && !root.hasAttribute("data-nav-open"); };
    menu?.addEventListener("click", () => {
      const open = root.toggleAttribute("data-nav-open"); menu.setAttribute("aria-expanded", String(open)); syncSidebar();
      if (open) { sidebar.setAttribute("data-keyboard-open", ""); sidebar.querySelector("a")?.focus(); }
    });
    const navObserver = new MutationObserver(syncSidebar); navObserver.observe(root, { attributes: true, attributeFilter: ["data-nav-open", "data-current-page"] });
    mobile.addEventListener("change", syncSidebar); syncSidebar();
    document.addEventListener("keydown", (e) => {
      if (e.key === "Escape" && root.hasAttribute("data-nav-open")) { shell.closeMobileNav(); menu.focus(); }
      if (e.key === "Tab" && overlayNav() && root.hasAttribute("data-nav-open")) {
        const nodes = S.$$("a[href],button:not(:disabled),input:not(:disabled)", sidebar).filter((el) => el.offsetParent !== null);
        const first = nodes[0], last = nodes[nodes.length - 1];
        if (e.shiftKey && document.activeElement === first) { e.preventDefault(); last.focus(); }
        else if (!e.shiftKey && document.activeElement === last) { e.preventDefault(); first.focus(); }
      }
    });
    document.addEventListener("pointerdown", (e) => { if (overlayNav() && !sidebar.contains(e.target) && !menu.contains(e.target)) shell.closeMobileNav(); });
    S.$("#viewport").addEventListener("pointerdown", () => shell.closeMobileNav());

    const vp = S.$("#viewport"), bar = S.$("#topbar");
    vp.addEventListener("scroll", () => bar.classList.toggle("is-scrolled", vp.scrollTop > 4), { passive: true });

    // Replay buttons inside demos re-run the page's enter() choreography for that demo
    document.addEventListener("click", (e) => {
      const b = e.target.closest("[data-replay]"); if (!b) return;
      b.closest(".demo")?.dispatchEvent(new CustomEvent("replay", { bubbles: true }));
    });

    S.navigate(shell.routeFromHash());
    // Prerender every other page in idle time so navigation is always instant.
    const queue = S.pages.map((p) => p.id);
    const pump = () => { const id = queue.shift(); if (!id) return; S.prepare(id); S.idle(pump); };
    S.idle(pump);
    requestAnimationFrame(() => root.classList.add("is-ready"));
  };
  // All scripts are `defer`red, so every page has registered before this fires.
  document.addEventListener("DOMContentLoaded", boot);
})();

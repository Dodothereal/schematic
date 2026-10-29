/* =============================================================================
   SCHEMATIC — SHELL: page registry, router, view transitions, prerendering,
   on-this-page rail, command palette, theme + tuning, boot.
   ============================================================================= */
(() => {
  const S = window.Schematic;
  const root = document.documentElement;
  const pages = [];
  const state = { current: null, scroll: {}, built: new Map(), mounted: new Set() };
  S.pages = pages;

  /* ===================================================================== registry */
  /**
   * Register a page.
   * @param {{id:string,title:string,group:string,icon:string,order:number,
   *          description?:string,count?:string|number,
   *          render:()=>string, mount?:(root:HTMLElement)=>void,
   *          enter?:(root:HTMLElement)=>void, leave?:(root:HTMLElement)=>void}} def
   */
  S.page = (def) => { pages.push(def); pages.sort((a, b) => a.order - b.order); };

  /* ===================================================================== markup helpers
     Every page uses these so headers, sections and demos are identical.      */
  S.pageHead = ({ icon, eyebrow, title, lead, meta = [] }) => `
    <header class="page-head reveal">
      <div class="page-head-ic" style="--i:0">${S.icon(icon, { size: 22 })}</div>
      <div class="stack" style="--gap:var(--s-2);--i:1">
        ${eyebrow ? `<span class="t-eyebrow">${eyebrow}</span>` : ""}
        <h1 class="t-h1">${title}</h1>
      </div>
      ${lead ? `<p class="t-lead" style="--i:2">${lead}</p>` : ""}
      ${meta.length ? `<div class="page-meta" style="--i:3">${meta.join("")}</div>` : ""}
    </header>`;

  S.section = ({ id, num, title, desc, body, toc }) => `
    <section class="sec" id="${id}" data-toc="${S.esc(toc || title)}">
      <header class="sec-head">
        ${num ? `<span class="sec-num">${num}</span>` : ""}
        <h2 class="t-h2">${title}</h2>
        ${desc ? `<p>${desc}</p>` : ""}
      </header>
      ${body}
    </section>`;

  S.demo = ({ title, desc, body, specs = [], stage = "", cls = "", attrs = "", replay = false }) => `
    <article class="demo ${cls}" ${attrs}>
      ${replay ? `<button class="btn btn-ghost btn-sm btn-icon demo-replay" data-replay data-tip="Replay" aria-label="Replay ${S.esc(title)}">${S.icon("refresh")}</button>` : ""}
      <div class="demo-stage ${stage}">${body}</div>
      <footer class="demo-foot">
        <div><h3 class="demo-title">${title}</h3>${desc ? `<div class="demo-desc">${desc}</div>` : ""}</div>
        ${specs.length ? `<div class="demo-specs">${specs.map((s) => `<span class="spec">${S.esc(s)}</span>`).join("")}</div>` : ""}
      </footer>
    </article>`;

  /* ===================================================================== router */
  const host = () => S.$("#page-host");
  const viewport = () => S.$("#viewport");
  const byId = (id) => pages.find((p) => p.id === id);
  const routeFromHash = () => { const id = location.hash.match(/^#\/([\w-]+)/)?.[1]; return byId(id) ? id : pages[0]?.id; };

  /** Build (but don't show) a page's DOM. Called on intent and during idle. */
  S.prepare = (id) => {
    const p = byId(id);
    if (!p || state.built.has(id)) return state.built.get(id);
    const el = document.createElement("div");
    el.className = "page"; el.dataset.page = id; el.hidden = true;
    el.innerHTML = p.render();
    host().append(el);
    state.built.set(id, el);
    return el;
  };

  const swap = (id) => {
    document.dispatchEvent(new CustomEvent("schematic:navigate", { detail: { id } }));
    const prev = state.current && byId(state.current);
    const prevEl = state.current && state.built.get(state.current);
    if (prevEl) { state.scroll[state.current] = viewport().scrollTop; prev.leave?.(prevEl); prevEl.hidden = true; }
    const p = byId(id), el = S.prepare(id);
    el.hidden = false;
    document.documentElement.dataset.currentPage = id;
    if (!state.mounted.has(id)) { state.mounted.add(id); p.mount?.(el); }
    state.current = id;
    viewport().scrollTop = state.scroll[id] || 0;
    document.title = `${p.title} — Schematic`;
    S.$("#crumb-current").textContent = p.title;
    S.$("#crumb-group").textContent = p.group;
    updateNav(id);
    buildRail(el);
    return { p, el };
  };

  let navigating = null, navigationVersion = 0;
  S.navigate = async (id, { shared, focus = true } = {}) => {
    if (!byId(id)) return;
    const version = ++navigationVersion;
    if (navigating) {
      navigating.skipTransition();
      await navigating.updateCallbackDone.catch(() => {});
    }
    if (version !== navigationVersion || id === state.current) return;
    const focusContent = focus && !!document.activeElement?.closest("#page-host");
    const from = pages.findIndex((p) => p.id === state.current), to = pages.findIndex((p) => p.id === id);
    root.style.setProperty("--dir", to >= from ? "1" : "-1");
    if (location.hash !== `#/${id}`) history[state.current ? "pushState" : "replaceState"](null, "", `#/${id}`);
    closeMobileNav();

    const finish = ({ p, el }) => requestAnimationFrame(() => {
      if (state.current !== id) return;
      p.enter?.(el);
      if (focusContent) { const h = S.$("h1", el); if (h) { h.tabIndex = -1; h.focus({ preventScroll: true }); } }
    });
    if (!document.startViewTransition || S.reduced() || !state.current) {
      const r = swap(id);
      if (state.current && !document.startViewTransition && !S.reduced()) { r.el.classList.remove("page-enter"); r.el.offsetWidth; r.el.classList.add("page-enter"); }
      finish(r);
      return;
    }
    navigating?.skipTransition?.();
    if (shared) shared.style.viewTransitionName = "hero-ic";
    let result;
    const transition = document.startViewTransition(() => {
      if (shared) shared.style.viewTransitionName = "";
      result = swap(id);
      if (shared) { const target = S.$(".page-head-ic", result.el); if (target) target.style.viewTransitionName = "hero-ic"; }
    });
    navigating = transition;
    // ready rejects when a newer intent skips this animation; the DOM update
    // still completes. Treat that expected cancellation as a settled outcome.
    transition.ready.catch(() => {});
    transition.updateCallbackDone.then(() => finish(result)).catch((e) => console.error("Navigation failed", e));
    await transition.finished.catch(() => {});
      if (result) { const t = S.$(".page-head-ic", result.el); if (t) t.style.viewTransitionName = ""; }
      if (navigating === transition) navigating = null;
  };

  // Links: <a href="#/icons"> — intercepted so we control the transition.
  document.addEventListener("click", async (e) => {
    const a = e.target.closest?.('a[href^="#/"]');
    if (!a || e.metaKey || e.ctrlKey || e.shiftKey || e.button !== 0) return;
    e.preventDefault();
    const id = a.getAttribute("href").slice(2);
    await S.navigate(id, { shared: a.hasAttribute("data-shared") ? S.$(".shared-ic", a) : null, focus: !a.hasAttribute("data-specimen") });
    if (a.hasAttribute("data-specimen") && state.current === id) {
      const index = Number(a.dataset.specimen);
      const specimen = state.built.get(id)?.querySelectorAll(".demo")[index];
      if (specimen) {
        const vp = viewport();
        S.scrollTo(vp.scrollTop + specimen.getBoundingClientRect().top - vp.getBoundingClientRect().top - 32);
        const heading = specimen.querySelector(".demo-title");
        if (heading) { heading.tabIndex = -1; heading.focus({ preventScroll: true }); }
      }
    }
  });
  window.addEventListener("popstate", () => S.navigate(routeFromHash()));
  window.addEventListener("hashchange", () => { if (location.hash.startsWith("#/") && routeFromHash() !== state.current) S.navigate(routeFromHash()); });

  // Intent prerender: hovering, focusing or pressing a route link builds it.
  let intentT;
  const intent = (e) => {
    const a = e.target.closest?.('a[href^="#/"]');
    if (!a) return;
    clearTimeout(intentT);
    const id = a.getAttribute("href").slice(2);
    if (e.type === "pointerover") intentT = setTimeout(() => S.prepare(id), 60); else S.prepare(id);
  };
  document.addEventListener("pointerover", intent, { passive: true });
  document.addEventListener("focusin", intent);
  document.addEventListener("pointerdown", intent, { passive: true });

  /* ===================================================================== sidebar nav */
  const buildNav = () => {
    const nav = S.$("#nav");
    const groups = [...new Set(pages.map((p) => p.group))];
    nav.innerHTML = groups.map((g) => `
      <div class="nav-group">
        <div class="nav-label t-eyebrow">${g}</div>
        ${pages.filter((p) => p.group === g).map((p) => `
          <a class="nav-item" href="#/${p.id}" data-route="${p.id}" data-glide-item>
            ${S.icon(p.icon)}<span>${p.title}</span>${p.count != null ? `<span class="nav-count">${p.count}</span>` : ""}
          </a>`).join("")}
      </div>`).join("") + `<span class="nav-active" id="nav-active" aria-hidden="true"></span>`;
    S.glide(nav, { items: ".nav-item" });
  };
  let navPlaced = false;
  const updateNav = (id) => {
    S.$$(".nav-item").forEach((a) => (a.dataset.route === id ? a.setAttribute("aria-current", "page") : a.removeAttribute("aria-current")));
    const a = S.$(`.nav-item[data-route="${id}"]`), m = S.$("#nav-active");
    if (!a || !m) return;
    if (!navPlaced) { m.style.transition = "none"; }
    m.style.transform = `translateY(${a.offsetTop}px)`;
    if (!navPlaced) { m.offsetWidth; m.style.transition = ""; navPlaced = true; }
  };

  const closeMobileNav = () => { root.removeAttribute("data-nav-open"); S.$("#menu-btn")?.setAttribute("aria-expanded", "false"); S.$(".sidebar")?.removeAttribute("data-keyboard-open"); };

  /* ===================================================================== on-this-page rail */
  let railObserver;
  const buildRail = (pageEl) => {
    const rail = S.$("#rail");
    railObserver?.disconnect();
    const secs = S.$$("section[data-toc]", pageEl);
    if (!secs.length) { rail.innerHTML = ""; return; }
    rail.innerHTML = `<div class="t-eyebrow">On this page</div><nav class="toc">${secs.map((s) => `<a href="#${s.id}" data-toc-link="${s.id}">${s.dataset.toc}</a>`).join("")}<span class="toc-marker"></span></nav>`;
    const links = S.$$("[data-toc-link]", rail), marker = S.$(".toc-marker", rail);
    const setActive = (id) => {
      const a = links.find((l) => l.dataset.tocLink === id); if (!a) return;
      links.forEach((l) => l.classList.toggle("is-active", l === a));
      marker.style.transform = `translateY(${a.offsetTop}px)`; marker.style.height = `${a.offsetHeight}px`;
    };
    links.forEach((l) => l.addEventListener("click", (e) => {
      e.preventDefault();
      const target = S.$(`#${CSS.escape(l.dataset.tocLink)}`, pageEl);
      const vp = viewport();
      S.scrollTo(vp.scrollTop + target.getBoundingClientRect().top - vp.getBoundingClientRect().top - 24);
      setActive(l.dataset.tocLink);
    }));
    const visible = new Map();
    railObserver = new IntersectionObserver((entries) => {
      entries.forEach((en) => visible.set(en.target.id, en.isIntersecting ? en.intersectionRatio : 0));
      const best = secs.find((s) => (visible.get(s.id) || 0) > 0);
      if (best) setActive(best.id);
    }, { root: viewport(), rootMargin: "-10% 0px -55% 0px", threshold: [0, 0.01, 0.5, 1] });
    secs.forEach((s) => railObserver.observe(s));
    requestAnimationFrame(() => setActive(secs[0].id));
  };

  /** Scroll the viewport with our own curve; distance-aware duration. */
  let scrollTween;
  S.scrollTo = (y) => {
    const vp = viewport(), from = vp.scrollTop, to = S.clamp(y, 0, vp.scrollHeight - vp.clientHeight);
    scrollTween?.cancel();
    const dist = Math.abs(to - from);
    scrollTween = S.tween({ from, to, duration: S.clamp(300 + dist * 0.25, 360, 900), ease: S.ease.inOut, onUpdate: (v) => (vp.scrollTop = v) });
  };

  S._shell = { buildNav, routeFromHash, swap, state, byId, closeMobileNav };
})();

(() => {
  const S = window.Schematic;
  const icon = (name, size = 16) => S.icon(name, { size });
  const parts = [
    {
      id: "navigation",
      title: "Navigation",
      source: "components/sidebar",
      icon: "sidebar",
      value: "6 destinations",
      path: "navigation",
    },
    {
      id: "metric",
      title: "Metric",
      source: "data/metric-card",
      icon: "trending",
      value: "$24,680",
      path: "data",
    },
    {
      id: "chart",
      title: "Chart",
      source: "data/line-chart",
      icon: "trending",
      value: "7 data points",
      path: "data",
    },
    {
      id: "list",
      title: "Task list",
      source: "primitives/sortable",
      icon: "grip",
      value: "3 items",
      path: "primitives",
    },
    {
      id: "actions",
      title: "Actions",
      source: "components/button",
      icon: "cursor",
      value: "4 states",
      path: "components",
    },
  ];
  const contents = {
    navigation: `<span class="as-nav-brand">${icon("cube", 19)} <b>Atlas</b></span><span class="as-nav-item is-current">${icon("home", 14)} Overview</span><span class="as-nav-item">${icon("layers", 14)} Projects <i>08</i></span><span class="as-nav-item">${icon("users", 14)} Team</span><span class="as-nav-item">${icon("chart-bar", 14)} Reports</span><span class="as-nav-foot"><span class="as-avatar">JD</span> Jamie Davis ${icon("chevron-down", 12)}</span>`,
    metric: `<span class="as-small">Monthly revenue ${icon("arrow-up-right", 14)}</span><span class="as-number">$24,680<span class="as-change">+12.8%</span></span><span class="as-mini-bars" aria-hidden="true">${[24, 38, 29, 50, 43, 65, 56, 74, 68, 86, 77, 100].map((h, i) => `<i style="height:${h}%;--bar:${i}"></i>`).join("")}</span>`,
    chart: `<span class="as-small"><b>Revenue</b><span>7 days</span></span><span class="as-chart-value">$6,240 <small>Today</small></span><svg class="as-chart" viewBox="0 0 260 110" fill="none" aria-hidden="true"><path class="as-chart-grid" d="M0 20H260M0 55H260M0 90H260"/><path class="as-chart-area" d="M0 90C16 90 18 54 38 58S62 83 79 59 95 62 115 39 138 57 157 29 178 46 195 24 217 37 235 13 248 21 260 7V110H0Z"/><path class="as-chart-line" pathLength="1" d="M0 90C16 90 18 54 38 58S62 83 79 59 95 62 115 39 138 57 157 29 178 46 195 24 217 37 235 13 248 21 260 7"/><circle class="as-chart-dot" cx="260" cy="7" r="3"/></svg><span class="as-axis"><span>MON</span><span>WED</span><span>FRI</span><span>SUN</span></span>`,
    list: `<span class="as-small"><b>Release checklist</b><span>1 / 3</span></span>${[
      ["Define tokens", true],
      ["Build components", false],
      ["Review interactions", false],
    ]
      .map(
        ([t, checked]) =>
          `<span class="as-list-row">${icon("grip", 12)}<i class="as-checkbox ${checked ? "is-done" : ""}">${checked ? icon("check", 10) : ""}</i><span>${t}</span>${icon("chevron-right", 12)}</span>`,
      )
      .join("")}`,
    actions: `<span class="as-action-primary">${icon("plus", 14)} New project</span><span class="as-action-icon">${icon("search", 16)}</span><span class="as-action-icon">${icon("bell", 16)}</span>`,
  };
  S.assembly = {
    render() {
      return `<section class="ov-opening" aria-label="Interactive component assembly">
        <header class="ov-opening-head"><div><span class="ov-edition">INTERFACE LIBRARY <span>/</span> 0.4</span><h1 class="ov-wordmark" tabindex="-1">Schematic<span>.</span></h1></div><div class="ov-opening-links"><a class="btn btn-ghost btn-sm" href="https://github.com/Dodothereal/schematic" target="_blank" rel="noreferrer">Source ${icon("arrow-up-right", 14)}</a><button class="btn btn-primary" data-as-browse>Browse components ${icon("arrow-down", 16)}</button></div></header>
        <div class="as-surface" data-assembly data-view="exploded" data-selected="chart" style="--as-radius:14px">
          <div class="as-surface-bar"><span class="as-caption">01 <span>/</span> COMPONENT ASSEMBLY</span><div class="as-view-switch" role="group" aria-label="Assembly view">${[
            ["assembled", "Assemble"],
            ["exploded", "Explode"],
            ["wireframe", "Wireframe"],
          ]
            .map(
              ([id, label]) =>
                `<button type="button" data-as-view="${id}" aria-pressed="${id === "exploded"}">${label}</button>`,
            )
            .join("")}</div></div>
          <div class="as-stage" data-as-stage><div class="as-plane" data-as-plane>
            <svg class="as-ground" viewBox="0 0 1000 450" aria-hidden="true"><defs><pattern id="as-grid" width="24" height="24" patternUnits="userSpaceOnUse"><path d="M12 9V15M9 12H15"/></pattern></defs><path fill="url(#as-grid)" d="M0 0H1000V450H0Z"/><path class="as-axis-line" d="M500 0V450M0 225H1000"/></svg>
            <div class="as-window" aria-hidden="true"><span class="as-window-top">${icon("cube", 13)} Atlas <i>workspace / overview</i><span>•••</span></span></div>
            <svg class="as-connections" viewBox="0 0 1000 450" fill="none" aria-hidden="true"><path pathLength="1" d="M204 150H267V84H326"/><path pathLength="1" d="M564 84H617V140H666"/><path pathLength="1" d="M204 230H234V286H286"/><path pathLength="1" d="M560 286H610V356H666"/><path class="as-dimension" d="M35 403H957M35 398V408M957 398V408"/><text x="498" y="421" text-anchor="middle">SPACING / 24</text></svg>
            ${parts.map((part, i) => `<button type="button" class="as-piece as-${part.id}" data-as-piece="${part.id}" aria-label="Inspect ${part.title}" aria-pressed="${part.id === "chart"}" style="--part:${i}"><span class="as-part-label" aria-hidden="true"><span>0${i + 1}</span>${part.title}${icon("arrow-up-right", 11)}</span><span class="as-piece-content">${contents[part.id]}</span><span class="as-corner as-corner-tl"></span><span class="as-corner as-corner-br"></span></button>`).join("")}
          </div></div>
          <div class="as-part-switch" role="group" aria-label="Select component">${parts.map((part) => `<button type="button" data-as-select="${part.id}" aria-pressed="${part.id === "chart"}">${icon(part.icon, 14)}<span>${part.title}</span></button>`).join("")}</div>
          <div class="as-inspector"><div class="as-selection"><span class="as-selection-icon" data-as-icon>${icon("chart-bar", 18)}</span><div><strong data-as-title>Chart</strong><span data-as-source>data/line-chart</span></div></div><label class="as-radius">Radius <input type="range" min="0" max="24" value="14" data-as-radius aria-label="Component corner radius"><output data-as-radius-value>14</output></label><button class="btn btn-ghost btn-sm as-replay" data-as-replay aria-label="Replay component assembly">${icon("refresh", 15)}</button><a class="btn btn-secondary btn-sm" href="#/data" data-specimen="0" data-as-open>Open example ${icon("arrow-up-right", 14)}</a></div>
          <span class="as-status" role="status" data-as-status></span>
        </div><div class="ov-opening-foot"><span><b>${S.catalog().length}</b> examples <span>/</span> HTML · CSS · JavaScript</span><span>Select a component. Change its shape.</span><a href="#/workspace">Coding workspace ${icon("arrow-up-right", 14)}</a></div>
      </section>`;
    },
    mount(root, browse) {
      const host = root.querySelector("[data-assembly]");
      const stage = host.querySelector("[data-as-stage]"),
        plane = host.querySelector("[data-as-plane]");
      let active = false,
        visible = true,
        replayed = false,
        motion = new Set(),
        pointerFrame = 0,
        pending = null;
      const $ = (s) => host.querySelector(s),
        $$ = (s) => [...host.querySelectorAll(s)];
      const settle = () => {
        motion.forEach((a) => a.cancel());
        motion.clear();
        cancelAnimationFrame(pointerFrame);
        pointerFrame = 0;
        plane.style.removeProperty("--as-px");
        plane.style.removeProperty("--as-py");
      };
      const animate = (el, frames, options = {}) => {
        if (!active || !visible || document.hidden || S.reduced()) return;
        const a = el.animate(frames, {
          duration: S.dur(850),
          easing: S.curve.outExpo,
          ...options,
        });
        motion.add(a);
        a.finished.catch(() => {}).finally(() => motion.delete(a));
      };
      const replay = () => {
        settle();
        if (S.reduced()) return;
        $$("[data-as-piece]").forEach((el, i) =>
          animate(
            el,
            [
              {
                opacity: 0,
                translate: `${500 - el.offsetLeft - el.offsetWidth / 2}px ${225 - el.offsetTop - el.offsetHeight / 2}px`,
                filter: "blur(4px)",
              },
              { opacity: 1, translate: "0 0", filter: "blur(0)" },
            ],
            { delay: S.dur(i * 58) },
          ),
        );
        $$(".as-connections path:not(.as-dimension),.as-chart-line").forEach(
          (el) =>
            animate(el, [{ strokeDashoffset: 1 }, { strokeDashoffset: 0 }], {
              duration: S.dur(1100),
              delay: S.dur(160),
            }),
        );
      };
      const select = (id) => {
        const part = parts.find((p) => p.id === id);
        if (!part) return;
        host.dataset.selected = id;
        $$("[data-as-piece],[data-as-select]").forEach((el) =>
          el.setAttribute(
            "aria-pressed",
            String((el.dataset.asPiece || el.dataset.asSelect) === id),
          ),
        );
        $("[data-as-title]").textContent = part.title;
        $("[data-as-source]").textContent = part.source;
        $("[data-as-icon]").innerHTML = icon(part.icon, 18);
        $("[data-as-open]").href = `#/${part.path}`;
        $("[data-as-open]").dataset.specimen = String(id === "list" ? 3 : 0);
        $("[data-as-status]").textContent =
          `${part.title} selected. ${part.value}.`;
      };
      $$("[data-as-piece],[data-as-select]").forEach((el) =>
        el.addEventListener("click", () =>
          select(el.dataset.asPiece || el.dataset.asSelect),
        ),
      );
      $$("[data-as-view]").forEach((button) =>
        button.addEventListener("click", () => {
          settle();
          host.dataset.view = button.dataset.asView;
          $$("[data-as-view]").forEach((el) =>
            el.setAttribute("aria-pressed", String(el === button)),
          );
          $("[data-as-status]").textContent = `${button.textContent} view.`;
        }),
      );
      $("[data-as-radius]").addEventListener("input", (e) => {
        host.style.setProperty("--as-radius", `${e.target.value}px`);
        $("[data-as-radius-value]").value = e.target.value;
        $("[data-as-radius-value]").textContent = e.target.value;
      });
      $("[data-as-replay]").addEventListener("click", replay);
      root.querySelector("[data-as-browse]").addEventListener("click", browse);
      const resize = new ResizeObserver(() => {
        plane.style.setProperty(
          "--as-scale",
          Math.min(1, stage.clientWidth / 1000),
        );
      });
      resize.observe(stage);
      stage.addEventListener("pointermove", (e) => {
        if (!active || !visible || S.reduced() || e.pointerType === "touch")
          return;
        const r = stage.getBoundingClientRect();
        pending = {
          x: ((e.clientX - r.left - r.width / 2) / r.width) * 8,
          y: ((e.clientY - r.top - r.height / 2) / r.height) * 6,
        };
        if (pointerFrame) return;
        pointerFrame = requestAnimationFrame(() => {
          pointerFrame = 0;
          plane.style.setProperty("--as-px", `${pending.x}px`);
          plane.style.setProperty("--as-py", `${pending.y}px`);
        });
      });
      stage.addEventListener("pointerleave", () => {
        plane.style.setProperty("--as-px", "0px");
        plane.style.setProperty("--as-py", "0px");
      });
      const intersection = new IntersectionObserver(
        (entries) => {
          visible = entries[0].isIntersecting;
          if (!visible) settle();
        },
        { threshold: 0.05 },
      );
      intersection.observe(host);
      const observer = new MutationObserver(() => {
        if (S.reduced()) settle();
      });
      observer.observe(document.documentElement, {
        attributes: true,
        attributeFilter: ["data-motion"],
      });
      document.addEventListener("visibilitychange", () => {
        if (document.hidden) settle();
      });
      window
        .matchMedia("(prefers-reduced-motion: reduce)")
        .addEventListener("change", () => {
          if (S.reduced()) settle();
        });
      return {
        enter() {
          active = true;
          if (!replayed) {
            replayed = true;
            replay();
          }
        },
        leave() {
          active = false;
          settle();
        },
      };
    },
  };
})();

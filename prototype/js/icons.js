/* =============================================================================
   SCHEMATIC — ANIMATED ICON SYSTEM
   One 24×24 grid, 1.6 stroke (token: --ic-stroke), round caps & joins.
   Every icon names its moving parts with short classes (a, b, c, d1…) and
   icons.css gives each icon a *semantic* animation: a bell rings, a trash lid
   lifts, a paper plane flies off and comes back. Animations are triggered by
   the nearest interactive ancestor (button, link, [data-ic-trigger]) and always
   play to completion — they never snap back when the pointer leaves.
   ============================================================================= */
(() => {
  const S = (window.Schematic = window.Schematic || {});

  const I = {
    /* ------------------------------------------------------------ arrows */
    "arrow-right": `<g class="a"><path d="M5 12h14"/><path d="M13 6l6 6-6 6"/></g>`,
    "arrow-left": `<g class="a"><path d="M19 12H5"/><path d="M11 6l-6 6 6 6"/></g>`,
    "arrow-up-right": `<g class="a"><path d="M7 17L17 7"/><path d="M8 7h9v9"/></g>`,
    "chevron-right": `<path class="a" d="M9.5 6l6 6-6 6"/>`,
    "chevron-down": `<path class="a" d="M6 9.5l6 6 6-6"/>`,
    "chevrons-up-down": `<path class="a" d="M8 9l4-4 4 4"/><path class="b" d="M8 15l4 4 4-4"/>`,
    "download": `<g class="a"><path d="M12 4v11"/><path d="M7.5 10.5L12 15l4.5-4.5"/></g><path class="b" d="M4 16.5V19a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2v-2.5"/>`,
    "upload": `<g class="a"><path d="M12 15V4"/><path d="M7.5 8.5L12 4l4.5 4.5"/></g><path class="b" d="M4 16.5V19a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2v-2.5"/>`,
    "refresh": `<g class="a"><path d="M20 12a8 8 0 1 1-8-8c2.24 0 4.38.9 5.99 2.43L20 8.5"/><path d="M20 4v4.5h-4.5"/></g>`,
    "logout": `<path d="M10 4.5H6.5a2 2 0 0 0-2 2v11a2 2 0 0 0 2 2H10"/><g class="a"><path d="M10 12h10"/><path d="M16 8l4 4-4 4"/></g>`,
    "external": `<path d="M13 5.5H6.5a2 2 0 0 0-2 2v10a2 2 0 0 0 2 2h10a2 2 0 0 0 2-2V11"/><g class="a"><path d="M11 13l9-9"/><path d="M14.5 4H20v5.5"/></g>`,

    /* ------------------------------------------------------------ actions */
    "check": `<path class="draw" pathLength="1" d="M5 12.5l4.5 4.5L19 7.5"/>`,
    "x": `<g class="a"><path d="M6.5 6.5l11 11"/><path d="M17.5 6.5l-11 11"/></g>`,
    "plus": `<g class="a"><path d="M12 5v14"/><path d="M5 12h14"/></g>`,
    "minus": `<path class="a" d="M5 12h14"/>`,
    "search": `<g class="a"><circle cx="11" cy="11" r="6.5"/><path d="M20 20l-4.2-4.2"/></g>`,
    "trash": `<g class="a"><path d="M4 7h16"/><path d="M9 7V5a1 1 0 0 1 1-1h4a1 1 0 0 1 1 1v2"/></g><path d="M6 7l1 12a2 2 0 0 0 2 2h6a2 2 0 0 0 2-2l1-12"/><path d="M10 11v6M14 11v6"/>`,
    "copy": `<path class="b" d="M16 8V6a2 2 0 0 0-2-2H6a2 2 0 0 0-2 2v8a2 2 0 0 0 2 2h2"/><rect class="a" x="8" y="8" width="12" height="12" rx="2"/>`,
    "pencil": `<g class="a"><path d="M15.5 4.5l4 4L8 20H4v-4z"/><path d="M13.5 6.5l4 4"/></g>`,
    "send": `<g class="a"><path d="M21 3L10 14"/><path d="M21 3l-6.5 18-4.5-7-7-4.5z"/></g>`,
    "link": `<path class="a" d="M10 13.5a4 4 0 0 0 5.66 0l3-3a4 4 0 0 0-5.66-5.66l-1 1"/><path class="b" d="M14 10.5a4 4 0 0 0-5.66 0l-3 3a4 4 0 0 0 5.66 5.66l1-1"/>`,
    "filter": `<path class="a" d="M4 5h16l-6 7.5V19l-4 1.5v-8z"/>`,
    "sliders": `<path d="M4 7h16M4 17h16"/><circle class="k1" cx="8" cy="7" r="2.2" fill="currentColor" stroke="none"/><circle class="k2" cx="16" cy="17" r="2.2" fill="currentColor" stroke="none"/>`,
    "play": `<path class="a" d="M7.5 4.8v14.4a.8.8 0 0 0 1.2.7l11.3-7.2a.8.8 0 0 0 0-1.4L8.7 4.1a.8.8 0 0 0-1.2.7z"/>`,
    "pause": `<path class="a" d="M8 5v14"/><path class="b" d="M16 5v14"/>`,
    "more": `<circle class="d1" cx="5.5" cy="12" r="1.3" fill="currentColor" stroke="none"/><circle class="d2" cx="12" cy="12" r="1.3" fill="currentColor" stroke="none"/><circle class="d3" cx="18.5" cy="12" r="1.3" fill="currentColor" stroke="none"/>`,
    "menu": `<path class="l1" d="M4 7h16"/><path class="l2" d="M4 12h16"/><path class="l3" d="M4 17h10"/>`,
    "command": `<path class="a" d="M15 6v12a3 3 0 1 0 3-3H6a3 3 0 1 0 3 3V6a3 3 0 1 0-3 3h12a3 3 0 1 0-3-3"/>`,
    "cursor": `<path class="a" d="M6 5l5 14 2-6 6-2z"/><path class="b" d="M6 1.5V3M2.5 5H4M3.4 2.4l1 1"/>`,

    /* ------------------------------------------------------------ objects */
    "bell": `<path class="a" d="M6 10a6 6 0 0 1 12 0c0 4 1.5 6 2 7H4c.5-1 2-3 2-7z"/><path class="b" d="M10 20a2 2 0 0 0 4 0"/>`,
    "home": `<path class="a" d="M3.5 11L12 4l8.5 7"/><path d="M5.5 9.5V19a1 1 0 0 0 1 1H10v-5h4v5h3.5a1 1 0 0 0 1-1V9.5"/>`,
    "mail": `<rect x="3" y="5.5" width="18" height="13" rx="2"/><path class="a" d="M3.5 7l8.5 6 8.5-6"/>`,
    "lock": `<rect x="5" y="10.5" width="14" height="10" rx="2"/><path class="a" d="M8 10.5V7.5a4 4 0 0 1 8 0v3"/><path d="M12 14.5v2"/>`,
    "eye": `<path class="a" d="M2.5 12S6 5.5 12 5.5 21.5 12 21.5 12 18 18.5 12 18.5 2.5 12 2.5 12z"/><circle class="b" cx="12" cy="12" r="3"/>`,
    "heart": `<path class="a" d="M12 20s-7-4.35-7-10a4 4 0 0 1 7-2.65A4 4 0 0 1 19 10c0 5.65-7 10-7 10z"/>`,
    "star": `<path class="a" d="M12 3.5l2.6 5.3 5.9.9-4.25 4.1 1 5.8L12 16.9l-5.25 2.7 1-5.8L3.5 9.7l5.9-.9z"/>`,
    "bookmark": `<path class="a" d="M6.5 4.5a1 1 0 0 1 1-1h9a1 1 0 0 1 1 1v16l-5.5-4-5.5 4z"/>`,
    "calendar": `<rect x="4" y="5.5" width="16" height="15" rx="2"/><path d="M4 10h16"/><path class="a" d="M8.5 3.5v4M15.5 3.5v4"/><path class="b" d="M8.5 14h.01M12 14h.01M15.5 14h.01M8.5 17h.01M12 17h.01"/>`,
    "clock": `<circle cx="12" cy="12" r="8.5"/><path class="a" d="M12 12V7"/><path class="b" d="M12 12l3 2"/>`,
    "document": `<path d="M14 3.5H7a2 2 0 0 0-2 2v13a2 2 0 0 0 2 2h10a2 2 0 0 0 2-2V8.5z"/><path d="M14 3.5v5h5"/><path class="l1 draw" pathLength="1" d="M8.5 12.5h7"/><path class="l2 draw" pathLength="1" d="M8.5 16h5"/>`,
    "folder": `<path class="a" d="M3.5 7.5a2 2 0 0 1 2-2h3.6a2 2 0 0 1 1.4.6l1.4 1.4h6.6a2 2 0 0 1 2 2v8a2 2 0 0 1-2 2h-13a2 2 0 0 1-2-2z"/><path class="b" d="M3.5 11h17"/>`,
    "chat": `<path d="M20.5 12a8 8 0 0 1-11.6 7.1L4 20.5l1.4-4.6A8 8 0 1 1 20.5 12z"/><circle class="d1" cx="8.5" cy="12" r="1.1" fill="currentColor" stroke="none"/><circle class="d2" cx="12" cy="12" r="1.1" fill="currentColor" stroke="none"/><circle class="d3" cx="15.5" cy="12" r="1.1" fill="currentColor" stroke="none"/>`,
    "user": `<circle class="a" cx="12" cy="8" r="3.5"/><path d="M5 20a7 7 0 0 1 14 0"/>`,
    "users": `<circle cx="9" cy="8" r="3.2"/><path d="M3 19.5a6 6 0 0 1 12 0"/><g class="a"><path d="M15.5 4.9a3.2 3.2 0 0 1 0 6.2"/><path d="M17.5 14.2a6 6 0 0 1 3.5 5.3"/></g>`,
    "globe": `<circle cx="12" cy="12" r="8.5"/><path d="M3.5 12h17"/><ellipse class="a" cx="12" cy="12" rx="3.6" ry="8.5"/>`,
    "gauge": `<path d="M4.6 17.5a8.5 8.5 0 1 1 14.8 0"/><path class="a" d="M12 14l3.5-4.5"/><circle cx="12" cy="14" r="1.2" fill="currentColor" stroke="none"/>`,
    "droplet": `<path class="a" d="M12 3.5s-6 6.5-6 10.5a6 6 0 0 0 12 0c0-4-6-10.5-6-10.5z"/>`,
    "palette": `<path d="M12 3.5a8.5 8.5 0 1 0 0 17c1.1 0 1.8-.8 1.8-1.7 0-.5-.2-.9-.5-1.2-.3-.3-.5-.7-.5-1.2 0-.9.8-1.7 1.7-1.7h2a4 4 0 0 0 4-4C20.5 6.9 16.7 3.5 12 3.5z"/><circle class="d1" cx="7.5" cy="11.5" r="1.15" fill="currentColor" stroke="none"/><circle class="d2" cx="9" cy="7.6" r="1.15" fill="currentColor" stroke="none"/><circle class="d3" cx="13.2" cy="6.8" r="1.15" fill="currentColor" stroke="none"/><circle class="d4" cx="16.6" cy="9.2" r="1.15" fill="currentColor" stroke="none"/>`,
    "ruler": `<rect x="2.5" y="8" width="19" height="8" rx="1.5"/><path class="t1" d="M6.5 8v3"/><path class="t2" d="M10 8v2"/><path class="t3" d="M13.5 8v3"/><path class="t4" d="M17 8v2"/>`,
    "type": `<g class="a"><path d="M5 7V5h14v2"/><path d="M12 5v14"/><path d="M9.5 19h5"/></g>`,
    "cube": `<g class="a"><path d="M12 3l8 4.5v9L12 21l-8-4.5v-9z"/><path d="M4 7.5l8 4.5 8-4.5M12 12v9"/></g>`,
    "layers": `<path class="a" d="M12 3.5l8.5 4.5-8.5 4.5L3.5 8z"/><path class="b" d="M3.5 12l8.5 4.5 8.5-4.5"/><path class="c" d="M3.5 16l8.5 4.5 8.5-4.5"/>`,
    "window": `<rect x="3" y="4" width="18" height="16" rx="2.5"/><rect class="a" x="7" y="8.5" width="10" height="7" rx="1.5"/>`,
    "input": `<rect x="2.5" y="6.5" width="19" height="11" rx="2.5"/><path class="a" d="M7.5 9.5v5"/>`,
    "sidebar": `<rect x="3" y="4.5" width="18" height="15" rx="2.5"/><path class="a" d="M9 4.5v15"/>`,
    "code": `<path class="a" d="M8.5 7.5L4 12l4.5 4.5"/><path class="b" d="M15.5 7.5L20 12l-4.5 4.5"/><path class="c" d="M13.5 5.5l-3 13"/>`,
    "grid": `<rect class="s1" x="4" y="4" width="6.5" height="6.5" rx="1.6"/><rect class="s2" x="13.5" y="4" width="6.5" height="6.5" rx="1.6"/><rect class="s3" x="13.5" y="13.5" width="6.5" height="6.5" rx="1.6"/><rect class="s4" x="4" y="13.5" width="6.5" height="6.5" rx="1.6"/>`,

    /* ------------------------------------------------------------ data */
    "chart-bar": `<path d="M3.5 20.5h17"/><rect class="b1" x="5.5" y="12" width="3" height="6" rx="1"/><rect class="b2" x="10.5" y="6" width="3" height="12" rx="1"/><rect class="b3" x="15.5" y="9" width="3" height="9" rx="1"/>`,
    "trending": `<path class="draw" pathLength="1" d="M3 17l6-6 4 4 8-8"/><path class="b" d="M15 7h6v6"/>`,
    "wave": `<path class="draw" pathLength="1" d="M2.5 12c1.6-4 3.2-6 4.75-6S10.4 18 12 18s3.2-12 4.75-12S19.9 10 21.5 12"/>`,
    "pie": `<path d="M20.5 13.5A8.5 8.5 0 1 1 10.5 3.6"/><path class="a" d="M14 3.5a7 7 0 0 1 6.5 6.5H14z"/>`,

    /* ------------------------------------------------------------ status */
    "info": `<circle cx="12" cy="12" r="8.5"/><path d="M12 11v5"/><path class="a" d="M12 7.8v.01" stroke-width="2.2"/>`,
    "alert": `<g class="a"><path d="M10.3 4.3L2.9 17a2 2 0 0 0 1.7 3h14.8a2 2 0 0 0 1.7-3L13.7 4.3a2 2 0 0 0-3.4 0z"/><path d="M12 9.5v4"/><path d="M12 16.8v.01" stroke-width="2.2"/></g>`,
    "check-circle": `<circle class="b" cx="12" cy="12" r="8.5"/><path class="draw" pathLength="1" d="M8.3 12.3l2.6 2.6 4.9-5.2"/>`,
    "x-circle": `<circle class="b" cx="12" cy="12" r="8.5"/><g class="a"><path d="M9.2 9.2l5.6 5.6"/><path d="M14.8 9.2l-5.6 5.6"/></g>`,
    "bolt": `<path class="a" d="M13 2.5L4.5 13.5H12l-1 8 8.5-11H12l1-8z"/>`,
    "sparkles": `<path class="s1" d="M10 3c.5 3.5 2.5 5.5 6 6-3.5.5-5.5 2.5-6 6-.5-3.5-2.5-5.5-6-6 3.5-.5 5.5-2.5 6-6z"/><path class="s2" d="M18 13.5c.2 1.4 1 2.3 2.5 2.5-1.5.2-2.3 1-2.5 2.5-.2-1.5-1-2.3-2.5-2.5 1.5-.2 2.3-1.1 2.5-2.5z"/><path class="s3" d="M5.5 17.5v3M4 19h3"/>`,
    "sun": `<circle cx="12" cy="12" r="4"/><path class="a" d="M12 2.5v2M12 19.5v2M4.6 4.6L6 6M18 18l1.4 1.4M2.5 12h2M19.5 12h2M4.6 19.4L6 18M18 6l1.4-1.4"/>`,
    "moon": `<path class="a" d="M20 14.5A8 8 0 0 1 9.5 4a8 8 0 1 0 10.5 10.5z"/>`,
    "loader": `<path d="M12 3.5a8.5 8.5 0 1 0 8.5 8.5" />`,
  };

  /* Icons that animate continuously instead of on hover */
  const LOOPING = new Set(["loader"]);

  S.iconNames = Object.keys(I);

  /**
   * Render an icon as an SVG string.
   * @param {string} name
   * @param {{size?:number, cls?:string, label?:string, anim?:boolean}} [o]
   */
  S.icon = (name, o = {}) => {
    const body = I[name];
    if (!body) { console.warn(`[schematic] unknown icon "${name}"`); return ""; }
    const size = o.size ?? 18;
    const anim = o.anim === false || LOOPING.has(name) ? "" : " data-anim";
    const a11y = o.label ? `role="img" aria-label="${o.label}"` : `aria-hidden="true"`;
    return `<svg class="ic${o.cls ? " " + o.cls : ""}" data-i="${name}"${anim} viewBox="0 0 24 24" width="${size}" height="${size}" fill="none" stroke="currentColor" stroke-linecap="round" stroke-linejoin="round" ${a11y}>${body}</svg>`;
  };

  /** Play an icon's animation once, to completion. Re-entrant safe. */
  S.playIcon = (svg) => {
    if (!svg || svg.classList.contains("is-anim")) return;
    svg.classList.add("is-anim");
    const anims = svg.getAnimations ? svg.getAnimations({ subtree: true }) : [];
    if (!anims.length) { svg.classList.remove("is-anim"); return; }
    Promise.all(anims.map((a) => a.finished.catch(() => {}))).then(() => svg.classList.remove("is-anim"));
  };

  const TRIGGER = 'button, a, summary, label, [role="button"], [role="tab"], [role="option"], [role="menuitem"], [data-ic-trigger]';
  S.ICON_TRIGGER = TRIGGER;

  const playWithin = (host) => {
    host.querySelectorAll(".ic[data-anim]").forEach((svg) => {
      if (svg.closest(TRIGGER) === host || host.matches("[data-ic-deep]")) S.playIcon(svg);
    });
  };

  document.addEventListener("pointerover", (e) => {
    if (e.pointerType === "touch") return;
    const host = e.target.closest?.(TRIGGER);
    if (!host || (e.relatedTarget && host.contains(e.relatedTarget))) return;
    playWithin(host);
  }, { passive: true });

  document.addEventListener("focusin", (e) => {
    const t = e.target;
    if (t.matches?.(":focus-visible") && t.matches(TRIGGER)) playWithin(t);
  });

  /* Touch: play on press so mobile users still get the feedback */
  document.addEventListener("pointerdown", (e) => {
    if (e.pointerType !== "touch") return;
    const host = e.target.closest?.(TRIGGER);
    if (host) playWithin(host);
  }, { passive: true });
})();

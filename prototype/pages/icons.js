(() => {
  const S = window.Schematic;

  const CATS = {
    Arrows: ["arrow-right", "arrow-left", "arrow-up-right", "chevron-right", "chevron-down", "chevrons-up-down", "download", "upload", "refresh", "logout", "external"],
    Actions: ["check", "x", "plus", "minus", "search", "trash", "copy", "pencil", "send", "link", "filter", "sliders", "play", "pause", "more", "menu", "command", "cursor"],
    Objects: ["bell", "home", "mail", "lock", "eye", "heart", "star", "bookmark", "calendar", "clock", "document", "folder", "chat", "user", "users", "globe", "gauge", "droplet", "palette", "ruler", "type", "cube", "layers", "window", "input", "sidebar", "code", "grid"],
    Data: ["chart-bar", "trending", "wave", "pie"],
    Status: ["info", "alert", "check-circle", "x-circle", "bolt", "sparkles", "sun", "moon", "loader"],
  };
  const catOf = (n) => Object.keys(CATS).find((c) => CATS[c].includes(n)) || "Other";

  const VERBS = [
    ["bell", "Rings", "The body swings from its hook while the clapper lags behind it — secondary motion makes it feel weighted."],
    ["trash", "Lid lifts", "Hinged on the right edge, the lid tilts open and settles with a tiny counter-swing."],
    ["send", "Flies off", "Exits up-right on an in-curve, re-enters from the opposite corner on an out-curve: motion that loops through."],
    ["download", "Drops in", "The arrow falls into the tray, the tray squashes to absorb it. Cause, then effect."],
    ["copy", "Duplicates", "Front sheet slides off the back one — literally showing a copy being made."],
    ["eye", "Blinks", "A blink, then the pupil glances around. Used on 'reveal password' toggles."],
  ];

  S.page({
    id: "icons",
    title: "Icons",
    group: "Foundations",
    icon: "sparkles",
    order: 30,
    count: S.iconNames.length,
    description: `${S.iconNames.length} hand-built icons on one grid, each with its own meaningful animation.`,
    render: () => `
      <div class="pg-icons">
        ${S.pageHead({
          icon: "sparkles", eyebrow: "Foundations", title: "Icons that act",
          lead: "One 24px grid, one 1.6 stroke, round caps. Every icon animates the verb it stands for — and always plays to completion, never snapping back when the pointer leaves.",
          meta: [`<span class="badge">${S.iconNames.length} icons</span>`, `<span class="badge">24 × 24 grid</span>`, `<span class="badge">stroke var(--ic-stroke)</span>`],
        })}

        ${S.section({
          id: "ic-gallery", num: "01", title: "Gallery", desc: "Hover to play, click to copy the snippet. Search filters with a FLIP layout animation, so tiles slide to their new places instead of jumping.",
          body: `
            <div class="ic-toolbar">
              <label class="ic-search">${S.icon("search", { size: 16 })}<input id="icons-q" type="search" placeholder="Search icons…" autocomplete="off" aria-label="Search icons"><span class="ic-count t-mono" id="icons-count">${S.iconNames.length}</span></label>
              <div class="seg" id="icons-size" aria-label="Icon size">
                ${[16, 20, 24, 32].map((s) => `<button class="seg-btn" data-value="${s}" ${s === 24 ? 'aria-pressed="true"' : ""}>${s}</button>`).join("")}
              </div>
              <label class="ic-stroke"><span class="t-caption">Stroke</span><input class="range" id="icons-stroke" type="range" min="1" max="2.4" step="0.1" value="1.6" aria-label="Stroke width"><span class="t-mono t-caption" id="icons-stroke-val">1.6</span></label>
              <button class="btn btn-secondary btn-sm" id="icons-wave">${S.icon("play")} Play all</button>
            </div>
            <div class="ic-grid" id="icons-grid" style="--ic-size:24px">
              ${S.iconNames.map((n) => `
                <button class="ic-tile" data-name="${n}" data-cat="${catOf(n)}" aria-label="Copy ${n}">
                  ${S.icon(n, { size: 24 })}
                  <span class="ic-name">${n}</span>
                </button>`).join("")}
            </div>
            <div class="ic-empty" id="icons-empty" hidden>${S.icon("search", { size: 20 })}<span>No icon matches “<b id="icons-empty-q"></b>”</span></div>`,
        })}

        ${S.section({
          id: "ic-verbs", num: "02", title: "Motion is the meaning", desc: "Each animation is designed around what the icon does, not a generic wiggle. A few of the stories:",
          body: `<div class="grid cols-3">${VERBS.map(([n, t, d]) => `
            <div class="card ic-verb" data-ic-trigger tabindex="0">
              <div class="ic-verb-stage">${S.icon(n, { size: 40 })}</div>
              <div class="t-h3">${t}</div>
              <p class="t-small muted">${d}</p>
              <span class="spec">${n}</span>
            </div>`).join("")}</div>`,
        })}

        ${S.section({
          id: "ic-usage", num: "03", title: "Usage", desc: "Icons are strings — framework-agnostic and trivially portable.",
          body: `<div class="grid cols-2">
            ${S.demo({
              title: "Inside controls", desc: "Any button, link, label or [data-ic-trigger] plays its icons on hover, focus and touch.",
              specs: ["auto-trigger", "plays to end"],
              body: `<button class="btn btn-primary">${S.icon("send")} Send</button><button class="btn btn-secondary">${S.icon("download")} Export</button><button class="btn btn-ghost btn-icon" data-tip="Notifications">${S.icon("bell")}</button><button class="btn btn-ghost btn-icon" data-tip="Delete">${S.icon("trash")}</button>`,
            })}
            ${S.demo({
              title: "Snippet", desc: "Ported 1:1 to React (<Icon name/>), SwiftUI (custom Shape + phaseAnimator) and Rust (svg! macro).",
              specs: ["S.icon(name, {size})"], stage: "is-plain",
              body: `<pre class="ic-code"><code><span class="faint">// vanilla</span>
el.innerHTML = S.icon(<span class="accent-text">"bell"</span>, { size: 18 });

<span class="faint">// programmatic play</span>
S.playIcon(el.querySelector(<span class="accent-text">".ic"</span>));</code></pre>`,
            })}
          </div>`,
        })}
      </div>`,

    mount: (root) => {
      const grid = S.$("#icons-grid", root), tiles = S.$$(".ic-tile", grid);
      const q = S.$("#icons-q", root), count = S.$("#icons-count", root), empty = S.$("#icons-empty", root);

      // Search with FLIP: tiles slide into place, leavers fade out first.
      let last = "";
      q.addEventListener("input", () => {
        const term = q.value.trim().toLowerCase();
        if (term === last) return; last = term;
        const match = (t) => !term || t.dataset.name.includes(term) || t.dataset.cat.toLowerCase().includes(term);
        const visibleBefore = tiles.filter((t) => !t.hidden);
        S.flip(visibleBefore, () => tiles.forEach((t) => (t.hidden = !match(t))), { stagger: 0 });
        const n = tiles.filter((t) => !t.hidden).length;
        S.roll(count, String(n));
        empty.hidden = n > 0;
        S.$("#icons-empty-q", root).textContent = q.value;
      });

      S.segmented(S.$("#icons-size", root), {
        onChange: (v) => S.flip(tiles.filter((t) => !t.hidden), () => {
          grid.style.setProperty("--ic-size", `${v}px`);
          S.$$(".ic", grid).forEach((s) => { s.setAttribute("width", v); s.setAttribute("height", v); });
        }),
      });

      const stroke = S.$("#icons-stroke", root), sv = S.$("#icons-stroke-val", root);
      const setStroke = () => { grid.style.setProperty("--ic-stroke", stroke.value); sv.textContent = (+stroke.value).toFixed(1); stroke.style.setProperty("--p", `${((stroke.value - 1) / 1.4) * 100}%`); };
      stroke.addEventListener("input", setStroke); setStroke();

      // Play all: a diagonal wave across the grid
      S.$("#icons-wave", root).addEventListener("click", () => {
        const vis = tiles.filter((t) => !t.hidden), g = grid.getBoundingClientRect();
        vis.forEach((t) => {
          const r = t.getBoundingClientRect();
          const d = (r.left - g.left) * 0.6 + (r.top - g.top) * 0.9;
          setTimeout(() => S.playIcon(S.$(".ic", t)), S.dur(d));
        });
      });

      grid.addEventListener("click", async (e) => {
        const t = e.target.closest(".ic-tile"); if (!t) return;
        const snippet = `S.icon("${t.dataset.name}")`;
        try { await navigator.clipboard.writeText(snippet); } catch {}
        t.classList.remove("is-copied"); t.offsetWidth; t.classList.add("is-copied");
        S.toast({ title: "Copied to clipboard", desc: snippet, tone: "success", duration: 2200 });
      });
    },
  });
})();

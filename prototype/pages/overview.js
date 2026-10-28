(() => {
  const S = window.Schematic;

  const principles = [
    ["wave", "Motion with meaning", "Every animation explains something: where a thing came from, what changed, what will happen. Decoration-only motion is a bug."],
    ["droplet", "One source of truth", "Colour, space, type, radius and motion live in tokens. Change one hue and the whole product re-tints — try the tune panel."],
    ["bolt", "Instant by default", "Pages are prerendered on hover intent and during idle time. A click never waits on the network or the parser."],
    ["eye", "Motion for everyone", "Reduced-motion users get the same information through opacity and colour. Slow-mo mode lets designers inspect every curve."],
    ["ruler", "On the grid", "A 4px spacing scale, a 24px icon grid, and one stroke weight. Nothing is nudged by eye."],
    ["layers", "Ports, not rewrites", "The same tokens and choreography map to React/Next, SwiftUI and Rust UIs. The HTML prototype is the reference."],
  ];

  const ports = [
    ["HTML / CSS / JS", "Reference prototype", "success", "Live"],
    ["React · Next.js", "Tokens → CSS vars, Motion for springs", "warning", "Next"],
    ["SwiftUI", "Tokens → Color/Font extensions, .spring() mapping", "neutral", "Planned"],
    ["Rust · Leptos / Dioxus / Tauri", "Tokens → consts, CSS shared with web", "neutral", "Planned"],
  ];

  S.page({
    id: "overview",
    title: "Overview",
    group: "Start",
    icon: "home",
    order: 0,
    description: "What Schematic is and how to use it.",
    render: () => `
      <div class="pg-overview">
        <section class="ov-hero spot" data-spot>
          <div class="ov-glow" aria-hidden="true"></div>
          <div class="ov-hero-inner reveal">
            <span class="badge badge-accent" style="--i:0">${S.icon("sparkles", { size: 12 })} Design system · prototype 0.1</span>
            <h1 class="t-display ov-title" style="--i:1">One feel for<br><span class="ov-title-accent">every project.</span></h1>
            <p class="t-lead ov-lead" style="--i:2">Schematic is a motion-first design language: tokens, icons, components and choreography that port to the web, Swift and Rust — so a new project starts finished, not from zero.</p>
            <div class="row ov-cta" style="--i:3">
              <a class="btn btn-primary btn-lg" href="#/components">Explore components ${S.icon("arrow-right")}</a>
              <button class="btn btn-secondary btn-lg" data-open-cmdk-ov>${S.icon("command")} Command palette <span class="kbd-group"><kbd class="kbd">⌘</kbd><kbd class="kbd">K</kbd></span></button>
            </div>
          </div>
        </section>

        <div class="ov-stats grid cols-4">
          ${[["icons", S.iconNames.length, "Animated icons", "sparkles"], ["tokens", 64, "Design tokens", "droplet"], ["curves", 10, "Motion curves", "wave"], ["deps", 0, "Dependencies", "cube"]]
            .map(([k, v, l, ic]) => `<div class="card ov-stat" data-ic-trigger><div class="row-between"><span class="t-caption">${l}</span><span class="ov-stat-ic">${S.icon(ic, { size: 16 })}</span></div><div class="ov-stat-val t-num" data-stat="${v}">0</div></div>`).join("")}
        </div>

        ${S.section({
          id: "ov-map", num: "01", title: "The system", desc: "Each area is a living page. Hover a card — the page is already being built before you click.",
          body: `<div class="grid cols-3 ov-cards">${S.pages.filter((p) => p.id !== "overview").map((p) => `
            <a class="card spot ov-card" href="#/${p.id}" data-shared>
              <span class="ov-card-ic shared-ic">${S.icon(p.icon, { size: 20 })}</span>
              <span class="ov-card-title">${p.title}</span>
              <span class="ov-card-desc">${p.description || ""}</span>
              <span class="ov-card-go">${S.icon("arrow-right", { size: 16 })}</span>
            </a>`).join("")}</div>`,
        })}

        ${S.section({
          id: "ov-principles", num: "02", title: "Principles", desc: "Six rules every component, page and port is reviewed against.",
          body: `<div class="grid cols-3 ov-principles">${principles.map(([ic, t, d]) => `
            <div class="ov-principle" data-ic-trigger>
              <span class="ov-principle-ic">${S.icon(ic, { size: 18 })}</span>
              <h3 class="t-h3">${t}</h3>
              <p class="t-small muted">${d}</p>
            </div>`).join("")}</div>`,
        })}

        ${S.section({
          id: "ov-ports", num: "03", title: "Platforms", desc: "The HTML prototype is the reference implementation. Every port consumes the same tokens.",
          body: `<div class="card ov-ports">${ports.map(([n, d, tone, st]) => `
            <div class="ov-port"><div><div class="ov-port-name">${n}</div><div class="t-caption">${d}</div></div><span class="badge ${tone === "neutral" ? "" : "badge-" + tone} ${st === "Live" ? "badge-dot" : ""}">${st}</span></div>`).join("")}</div>`,
        })}
      </div>`,
    mount: (root) => {
      S.$("[data-open-cmdk-ov]", root).addEventListener("click", () => S.cmdk.open());
    },
    enter: (root) => {
      S.$$("[data-stat]", root).forEach((el, i) => setTimeout(() => S.roll(el, el.dataset.stat), S.dur(200 + i * 80)));
    },
  });
})();

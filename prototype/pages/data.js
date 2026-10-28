(() => {
  "use strict";
  const S = window.Schematic;
  const instances = new WeakMap();
  const money = (n) => new Intl.NumberFormat("en-US", { style: "currency", currency: "USD", maximumFractionDigits: 0 }).format(n);
  const number = (n) => Math.round(n).toLocaleString("en-US");
  const percent = (n) => `${n.toFixed(1)}%`;
  const sum = (rows, key) => rows.reduce((total, row) => total + row[key], 0);
  const clamp = (n, a, b) => Math.max(a, Math.min(b, n));
  const seeded = (seed) => () => {
    seed = (Math.imul(1664525, seed) + 1013904223) >>> 0;
    return seed / 4294967296;
  };
  const random = seeded(6028);
  const days = Array.from({ length: 180 }, (_, i) => {
    const date = new Date(Date.UTC(2026, 9, 28 - 179 + i));
    const orders = Math.round(27 + i * .16 + Math.sin(i * .65) * 9 + random() * 15);
    const visitors = Math.round(orders / (.029 + random() * .011));
    return { date: date.toISOString().slice(0, 10), orders, visitors, revenue: Math.round(orders * (63 + random() * 24)) };
  });
  const names = ["Forma Studio", "Linear House", "Orbit Supply", "Northstar Lab", "Mono Works", "Field Notes", "Common Ground", "Index Studio", "Parallel Co.", "Good Measure", "Open Edition", "Atlas Objects", "Studio Tuesday", "Quiet Practice", "Outer Office", "Paper Plane", "New Standard", "Basecamp Goods", "Another Country", "Small Hours", "Made of Matter", "Third Space", "Daybreak Co.", "Pattern House", "Local Assembly", "Frame & Form", "Workshop One", "Future Ordinary"];
  const channels = ["Organic search", "Direct", "Referral", "Social", "Email", "Paid search"];
  const accounts = names.map((name, i) => ({
    id: `SC-${String(1024 + i).padStart(4, "0")}`, name, channel: channels[i % channels.length],
    status: i % 7 === 3 ? "Trial" : i % 9 === 4 ? "Paused" : "Active",
    orders: 12 + Math.floor(random() * 140), revenue: 850 + Math.round(random() * 14500),
    growth: +(random() * 38 - 9).toFixed(1), initials: name.split(/\s+/).slice(0, 2).map((v) => v[0]).join("")
  }));
  const devices = [{ name: "Desktop", count: 12400, color: 1 }, { name: "Mobile", count: 5400, color: 2 }, { name: "Tablet", count: 2200, color: 3 }];
  const goals = [{ name: "Revenue", value: 76, detail: "$76k / $100k", color: 1 }, { name: "Activation", value: 88, detail: "880 / 1,000", color: 2 }, { name: "Retention", value: 64, detail: "640 / 1,000", color: 4 }];
  const dateLabel = (date) => new Date(`${date}T12:00:00Z`).toLocaleDateString("en-US", { month: "short", day: "numeric", timeZone: "UTC" });
  const sample = (values, length = 90) => Array.from({ length }, (_, i) => {
    const p = i / (length - 1) * (values.length - 1), j = Math.floor(p);
    return values[j] + ((values[j + 1] ?? values[j]) - values[j]) * (p - j);
  });
  const line = (values, width, height, max, left = 48, top = 22, right = 18, bottom = 36) => {
    const x = (i) => left + i / (values.length - 1) * (width - left - right);
    const y = (v) => top + (1 - v / max) * (height - top - bottom);
    const points = values.map((v, i) => [x(i), y(v)]), h = (width - left - right) / (values.length - 1);
    const slopes = points.slice(1).map((point, i) => (point[1] - points[i][1]) / h);
    const tangents = points.map((_, i) => i === 0 ? slopes[0] : i === points.length - 1 ? slopes.at(-1) : slopes[i - 1] * slopes[i] <= 0 ? 0 : 2 * slopes[i - 1] * slopes[i] / (slopes[i - 1] + slopes[i]));
    slopes.forEach((slope, i) => { if (!slope) { tangents[i] = tangents[i + 1] = 0; return; } const a = tangents[i] / slope, b = tangents[i + 1] / slope, norm = Math.hypot(a, b); if (norm > 3) { tangents[i] = 3 / norm * a * slope; tangents[i + 1] = 3 / norm * b * slope; } });
    return points.map(([px, py], i) => i === 0 ? `M${px.toFixed(2)},${py.toFixed(2)}` : `C${(points[i - 1][0] + h / 3).toFixed(2)},${(points[i - 1][1] + tangents[i - 1] * h / 3).toFixed(2)} ${(px - h / 3).toFixed(2)},${(py - tangents[i] * h / 3).toFixed(2)} ${px.toFixed(2)},${py.toFixed(2)}`).join(" ");
  };
  const smallPath = (values) => line(sample(values, 24), 160, 42, Math.max(...values) * 1.12, 0, 3, 0, 3);
  const segment = (id, values, chosen, label) => `<div class="seg dt-seg" data-control="${id}" role="group" aria-label="${label}"><span class="seg-thumb" aria-hidden="true"></span>${values.map(([value, text]) => `<button type="button" class="seg-btn" data-value="${value}" aria-pressed="${chosen === value}">${text}</button>`).join("")}</div>`;
  const kpis = [
    ["Revenue", "trending", "revenue", money], ["Orders", "layers", "orders", number],
    ["Avg. order value", "gauge", "average", money], ["Conversion", "cursor", "conversion", percent]
  ];
  const getRows = (period) => days.slice(-period);
  const metrics = (rows) => ({ revenue: sum(rows, "revenue"), orders: sum(rows, "orders"), average: sum(rows, "revenue") / sum(rows, "orders"), conversion: sum(rows, "orders") / sum(rows, "visitors") * 100 });
  const initial = metrics(getRows(30));
  const cardMarkup = () => `<div class="grid cols-4 dt-kpis">${kpis.map(([title, icon, key, format], i) => `<article class="card spot dt-kpi" style="--dt-series:var(--chart-${i + 1})"><div class="row-between"><h3 class="dt-kpi-label">${title}</h3>${S.icon(icon, { size: 16 })}</div><div class="dt-kpi-value t-num" data-kpi="${key}">${format(initial[key])}</div><div class="dt-kpi-bottom"><span class="dt-delta t-num" data-delta="${key}"></span><svg viewBox="0 0 160 42" aria-hidden="true"><path data-spark="${key}" d="${smallPath(getRows(30).map((d) => key === "average" ? d.revenue / d.orders : key === "conversion" ? d.orders / d.visitors * 100 : d[key]))}"/></svg></div><span class="t-caption">vs. previous <span data-period-label>30</span> days</span></article>`).join("")}</div>`;
  const selectBox = (attrs, label) => `<label class="dt-box"><input type="checkbox" ${attrs} aria-label="${S.esc(label)}"><span class="dt-box-face" aria-hidden="true">${S.icon("check", {size:12, anim:false})}<span class="dt-box-mixed"></span></span></label>`;
  const chartMarkup = () => S.demo({
    title: "A continuous story", desc: "A fresh trace for each period. The line leads; its area follows the drawing head. Hover or use the day selector to inspect a value.", specs: ["path tracing", "1.1s", "keyboard"], stage: "is-plain dt-stage", replay: true,
    body: `<div class="dt-chart-panel"><div class="dt-panel-head"><div><span class="t-eyebrow">Revenue over time</span><div class="dt-chart-total t-num" data-revenue-total>${money(initial.revenue)}</div><p class="t-caption" data-date-range>Sep 29 - Oct 28, 2026</p></div><div class="dt-chart-controls"><label class="dt-check"><input type="checkbox" data-compare checked> Compare previous period</label><span class="dt-legend-key"><i class="dt-dot"></i> Current <i class="dt-dot dt-dot-previous"></i> Previous</span></div></div><div class="dt-chart-wrap" data-chart-wrap><svg class="dt-main-chart" data-main-chart viewBox="0 0 800 300" role="img" aria-labelledby="dt-line-title dt-line-desc"><title id="dt-line-title">Generated daily revenue</title><desc id="dt-line-desc">Use the day selector below to inspect the daily values.</desc><defs><linearGradient id="dt-revenue-fill" x1="0" y1="0" x2="0" y2="1"><stop class="dt-gradient-start" offset="0"/><stop class="dt-gradient-end" offset="1"/></linearGradient><mask id="dt-comparison-mask" maskUnits="userSpaceOnUse" x="0" y="0" width="2000" height="500"><path data-comparison-mask fill="none" stroke="white" stroke-width="8" pathLength="1"/></mask></defs><g data-chart-grid></g><path class="dt-area" data-area/><path class="dt-line dt-previous-line" data-previous-line mask="url(#dt-comparison-mask)"/><path class="dt-line" data-current-line pathLength="1"/><g class="dt-draw-head" data-draw-head aria-hidden="true"><circle class="dt-draw-halo" r="8"/><circle class="dt-draw-dot" r="3.5"/></g><g class="dt-crosshair" data-crosshair aria-hidden="true"><line data-guide-y class="dt-guide-horizontal"/><line data-guide-x/><circle data-point-halo r="8"/><circle data-point r="4"/></g><rect class="dt-chart-hit" data-chart-hit x="48" y="22" width="734" height="242"/></svg><div class="dt-chart-tooltip" data-chart-tooltip aria-hidden="true"><span data-tip-date></span><strong class="t-num" data-tip-value></strong><span data-tip-previous></span></div></div><div class="dt-inspector"><label for="dt-day">Inspect a day <span class="t-caption">Arrow keys / Home / End</span></label><input class="range" id="dt-day" type="range" min="0" max="29" value="29" step="1" aria-describedby="dt-point-readout"><output id="dt-point-readout" for="dt-day" class="t-small t-num"></output></div><p class="dt-chart-summary" data-chart-summary></p><div class="dt-live"><div><label class="dt-check"><input type="checkbox" data-live> Simulate live updates</label><p class="t-caption" data-live-status>Off. Seeded daily values; no external connection.</p></div><button type="button" class="btn btn-secondary btn-sm" data-tick>${S.icon("plus", { size: 14 })} Add sample</button></div></div>`
  });
  const barMarkup = () => S.demo({
    title: "Change the relationship", desc: "Group to compare series; stack to read the combined volume. Labels retain exact values.", specs: ["staggered draw", "linked hover"], stage: "is-plain dt-stage", replay: true,
    body: `<div class="dt-panel"><div class="dt-panel-head"><div><h3 class="t-h3">Order mix</h3><p class="t-caption">Seven buckets across the chosen period</p></div>${segment("bars", [["grouped", "Grouped"], ["stacked", "Stacked"]], "grouped", "Bar arrangement")}</div><div class="dt-legend-key"><i class="dt-dot"></i> New customers <i class="dt-dot dt-dot-previous dt-solid"></i> Returning</div><svg class="dt-bars" data-bars viewBox="0 0 600 280" role="img" aria-labelledby="dt-bars-title dt-bars-desc"><title id="dt-bars-title">Orders by period bucket</title><desc id="dt-bars-desc"></desc><g data-bar-grid></g>${Array.from({ length: 7 }, (_, i) => `<g data-bar-group="${i}" tabindex="0" role="button" aria-pressed="false"><rect class="dt-bucket-halo" data-bucket-halo="${i}" rx="8"/><rect class="dt-bar dt-bar-new" data-bar="${i * 2}" rx="3"/><rect class="dt-bar dt-bar-returning" data-bar="${i * 2 + 1}" rx="3"/><text class="dt-bar-value" data-bar-value="${i * 2}"></text><text class="dt-bar-value" data-bar-value="${i * 2 + 1}"></text><text class="dt-axis-label" data-bar-label="${i}"></text></g>`).join("")}</svg><p class="dt-bar-insight" data-bar-insight role="status">Hover a bucket to compare its exact values. Click to keep it in view.</p><details class="dt-details"><summary>Read order values</summary><div data-bar-text></div></details></div>`
  });
  const rankMarkup = () => S.demo({
    title: "Keep your place", desc: "Stable rows move to their new rank with FLIP. Their meters continue from the current width.", specs: ["FLIP", "snappy"], stage: "is-plain dt-stage", replay: true,
    body: `<div class="dt-panel"><div class="dt-panel-head"><div><h3 class="t-h3">Acquisition channels</h3><p class="t-caption">Generated attribution snapshot</p></div><label class="dt-sort-label">Sort by<select class="dt-select" data-rank-sort><option value="revenue">Revenue</option><option value="growth">Growth</option><option value="name">Name</option></select></label></div><ol class="dt-ranks" data-ranks>${channels.map((name, i) => `<li data-rank="${i}"><button type="button" class="dt-rank-button" data-rank-inspect="${i}" aria-pressed="false" style="--dt-series:var(--chart-${i % 5 + 1})"><span class="dt-rank-top"><span class="dt-rank-number t-mono" data-rank-number>${i + 1}</span><span class="dt-rank-name">${name}</span><strong class="t-num" data-rank-value></strong><span class="dt-rank-cue" aria-hidden="true">${S.icon("arrow-up-right", { size: 12 })}</span></span><span class="dt-rank-track"><span style="--dt-series:var(--chart-${i % 5 + 1})" data-rank-fill></span></span><span class="dt-rank-note t-caption" data-rank-note></span></button></li>`).join("")}</ol><p class="dt-chart-summary" data-rank-summary role="status"></p></div>`
  });
  const donutMarkup = () => S.demo({
    title: "A part of the whole", desc: "Toggle a device to recalculate the visible mix. Segment length and total respond together.", specs: ["sequential sweep", "1.15s"], stage: "is-plain dt-stage", replay: true,
    body: `<div class="dt-panel"><div class="dt-panel-head"><div><h3 class="t-h3">Audience by device</h3><p class="t-caption">Generated snapshot · 20,000 sessions</p></div>${S.icon("pie", { size: 20 })}</div><div class="dt-donut-layout"><div class="dt-donut-wrap"><svg viewBox="0 0 220 220" role="group" aria-labelledby="dt-donut-title dt-donut-desc"><title id="dt-donut-title">Device distribution</title><desc id="dt-donut-desc">Desktop 62%, Mobile 27%, Tablet 11%. All devices included.</desc><circle class="dt-donut-track" cx="110" cy="110" r="82"/>${devices.map((d, i) => `<circle class="dt-donut-segment" data-donut="${i}" role="button" tabindex="0" aria-pressed="true" aria-label="Toggle ${d.name} sessions" cx="110" cy="110" r="82" pathLength="100" stroke-dasharray="0 100" style="--dt-series:var(--chart-${d.color})"/>`).join("")}</svg><div class="dt-donut-center" aria-hidden="true"><strong class="t-num" data-donut-total>20,000</strong><span class="t-caption" data-donut-label>included sessions</span></div></div><div class="dt-device-legend">${devices.map((d, i) => `<button type="button" class="dt-device" data-device="${i}" aria-pressed="true" aria-label="Include ${d.name} sessions"><span class="dt-device-dot" style="--dt-series:var(--chart-${d.color})"></span><span>${d.name}<small>${number(d.count)} sessions</small></span><strong class="t-num" data-device-share="${i}"></strong><span class="dt-device-check" aria-hidden="true">${S.icon("check", { size: 14 })}</span></button>`).join("")}</div></div><p class="dt-chart-summary" data-device-summary role="status">All three devices included.</p></div>`
  });
  const goalMarkup = () => S.demo({
    title: "Progress that resolves", desc: "Each ring traces a finite, labeled target. Replay the entrance without changing the data.", specs: ["draw + endpoint", "1.1s"], stage: "is-plain dt-stage", replay: true,
    body: `<div class="dt-panel"><div class="dt-panel-head"><div><h3 class="t-h3">Monthly targets</h3><p class="t-caption">Independent illustrative goals · October</p></div><span class="badge badge-accent">3 targets</span></div><div class="dt-goals">${goals.map((g, i) => `<button type="button" class="dt-goal" data-goal="${i}" aria-pressed="false" aria-label="Inspect ${g.name}, ${g.value}% of target" style="--dt-series:var(--chart-${g.color})"><span class="dt-ring"><svg viewBox="0 0 100 100" aria-hidden="true"><circle class="dt-ring-track" cx="50" cy="50" r="40"/><circle class="dt-ring-fill" data-ring="${i}" cx="50" cy="50" r="40" pathLength="100" style="--dt-series:var(--chart-${g.color})"/><circle class="dt-ring-end" data-ring-end="${i}" r="3.5"/></svg><strong class="t-num">${g.value}%</strong><span class="dt-goal-remainder">${100 - g.value}% to go</span></span><span class="dt-goal-name">${g.name}</span><span class="t-caption">${g.detail}</span></button>`).join("")}</div><div class="dt-target-note">${S.icon("info", { size: 16 })}<p class="t-small muted" data-goal-insight role="status">Hover a target to see what remains. Click to keep its detail visible.</p></div></div>`
  });
  const tableMarkup = () => `<div class="card dt-table-card"><div class="dt-table-toolbar"><div><h3 class="t-h3">Customer activity</h3><p class="t-caption">28 generated accounts · independent snapshot</p></div><button type="button" class="btn btn-secondary btn-sm" data-export>${S.icon("download", { size: 14 })}<span data-export-label>Export filtered CSV</span></button></div><div class="dt-table-filters"><label class="dt-search">${S.icon("search", { size: 16 })}<span class="dt-sr">Filter customers, account IDs or channels</span><input type="search" data-search placeholder="Filter customers or channels…" aria-controls="dt-customers"></label><label class="dt-filter-label">Status<select class="dt-select" data-status><option value="all">All statuses</option><option>Active</option><option>Trial</option><option>Paused</option></select></label><button type="button" class="btn btn-ghost btn-sm" data-reset>Reset filters</button></div><div class="dt-selection"><span data-selection-count>0 selected</span><button type="button" class="btn btn-ghost btn-sm" data-clear-selection hidden>Clear selection</button><span class="t-caption">Selection persists across pages.</span></div><div class="dt-table-scroll" tabindex="0" role="region" aria-label="Customer table, scroll horizontally for all columns"><table id="dt-customers"><caption class="dt-sr">Generated customer activity. Activate a column heading to sort. Select accounts to export only the selection.</caption><thead><tr><th scope="col" class="dt-checkbox-cell">${selectBox("data-select-page", "Select all customers on this page")}</th>${[["name", "Customer"], ["status", "Status"], ["orders", "Orders"], ["revenue", "Revenue"], ["growth", "Growth"]].map(([key, title]) => `<th scope="col" data-column="${key}" aria-sort="${key === "revenue" ? "descending" : "none"}"><button type="button" data-table-sort="${key}">${title}<span class="dt-sort-icon" aria-hidden="true">${S.icon("chevrons-up-down", { size: 12 })}</span></button></th>`).join("")}</tr></thead><tbody>${accounts.map((a) => `<tr data-account="${a.id}" tabindex="0" aria-selected="false" aria-label="${S.esc(a.name)}, press Space to select" hidden><td class="dt-checkbox-cell">${selectBox(`data-select="${a.id}"`, `Select ${a.name}`)}</td><th scope="row"><div class="dt-customer"><span class="dt-avatar" aria-hidden="true">${a.initials}</span><span>${S.esc(a.name)}<small>${a.channel} · ${a.id}</small></span></div></th><td><span class="badge ${a.status === "Active" ? "badge-success" : a.status === "Trial" ? "badge-info" : ""}">${a.status}</span></td><td class="t-num">${a.orders}</td><td class="t-num dt-money">${money(a.revenue)}</td><td class="t-num ${a.growth >= 0 ? "dt-positive" : "dt-negative"}">${a.growth > 0 ? "+" : ""}${a.growth}%</td></tr>`).join("")}</tbody></table><div class="dt-empty" data-table-empty hidden>${S.icon("search", { size: 24 })}<strong>No matching customers</strong><p>Try a different name, channel, or status.</p><button type="button" class="btn btn-secondary btn-sm" data-reset>Clear filters</button></div></div><div class="dt-table-footer"><p class="t-caption" data-table-summary role="status"></p><div class="dt-pagination"><label>Rows<select class="dt-select" data-page-size><option value="8">8</option><option value="16">16</option><option value="28">28</option></select></label><button type="button" class="btn btn-secondary btn-sm btn-icon" data-prev-page aria-label="Previous page">${S.icon("arrow-left", { size: 14 })}</button><span class="t-caption t-num" data-page-count></span><button type="button" class="btn btn-secondary btn-sm btn-icon" data-next-page aria-label="Next page">${S.icon("arrow-right", { size: 14 })}</button></div></div><p class="dt-export-status t-caption" data-export-status role="status">CSV is generated locally. No data leaves this page.</p></div>`;

  function mount(root) {
    const page = root.querySelector(".pg-data");
    const $ = (selector) => page.querySelector(selector);
    const $$ = (selector) => [...page.querySelectorAll(selector)];
    const st = { active: false, period: 30, compare: true, mode: "grouped", index: 29, width: 800, height: 300, barWidth: 600, live: false, ticks: 0, increments: new Map(), timer: null, tweens: new Map(), animations: new Set(), visual: null, barVisual: null, donutVisual: [0, 0, 0], enabled: [true, true, true], rankSort: "revenue", query: "", status: "all", sort: "revenue", direction: -1, page: 0, size: 8, selected: new Set(), reduced: S.reduced(), url: null, revokeTimer: null };
    const current = () => days.slice(-st.period).map((d) => ({ ...d, revenue: d.revenue + (st.increments.get(d.date) || 0) }));
    const previous = () => days.slice(-st.period * 2, -st.period);
    const running = () => st.active && !document.hidden;
    st.pending = new Map(); st.barHover = null; st.barPinned = null; st.deviceHover = null; st.goalPinned = null;
    const animate = (key, from, to, update, duration = 650, ease = S.ease.inOut, delay = 0) => {
      st.tweens.get(key)?.cancel();
      st.tweens.delete(key);
      if (!running() || S.reduced()) { update(to); return; }
      const tween = S.tween({ from, to, duration, ease, delay, onUpdate: update });
      st.tweens.set(key, tween);
      tween.finished.then(() => { if (st.tweens.get(key) === tween) st.tweens.delete(key); });
    };
    const track = (animation) => {
      st.animations.add(animation);
      animation.finished.catch(() => {}).then(() => st.animations.delete(animation));
    };
    const revealObserver = new IntersectionObserver((entries) => {
      entries.forEach((entry) => {
        if (!entry.isIntersecting || !running()) return;
        [...st.pending].forEach(([key, task]) => { if (task.stage === entry.target) { st.pending.delete(key); task.play(); } });
        revealObserver.unobserve(entry.target);
      });
    }, { root: S.$("#viewport"), threshold: .12 });
    // Prepare the finished geometry once. Only the trace/reveal moves each frame.
    // Below-fold charts wait for their own stage to enter the viewport.
    const present = (key, el, reset, play, finish) => {
      st.pending.delete(key);
      if (!running() || S.reduced()) { finish(); return; }
      reset();
      const stage = el.closest(".dt-chart-wrap,[data-bars],.dt-donut-wrap,.dt-goals,.dt-ranks,.dt-kpi") || el.closest(".demo") || el;
      const rect = stage.getBoundingClientRect(), vp = S.$("#viewport").getBoundingClientRect();
      const visibleHeight = Math.min(rect.bottom, vp.bottom) - Math.max(rect.top, vp.top);
      if (visibleHeight >= Math.min(rect.height * .12, 80)) play();
      else { st.pending.set(key, { stage, play }); revealObserver.observe(stage); }
    };
    const flip = (nodes, mutate) => {
      nodes.forEach((el) => el.getAnimations().forEach((a) => a.cancel()));
      const before = new Set(nodes.filter((el) => !el.hidden));
      if (!running() || S.reduced()) { mutate(); return; }
      S.flip([...before], mutate, { duration: 470, stagger: 16 });
      nodes.forEach((el) => {
        if (!el.hidden && !before.has(el)) track(el.animate([{ opacity: 0, clipPath: "inset(0 0 100% 0)" }, { opacity: 1, clipPath: "inset(0)" }], { duration: S.dur(320), easing: S.curve.out }));
        el.getAnimations().forEach((a) => { if (!st.animations.has(a)) track(a); });
      });
    };
    const placeSegments = () => $$('[data-control]').forEach((control) => {
      const button = control.querySelector('[aria-pressed="true"]');
      const thumb = control.querySelector(".seg-thumb");
      if (button) { thumb.style.width = `${button.offsetWidth}px`; thumb.style.transform = `translateX(${button.offsetLeft}px)`; }
    });
    const setSegment = (name, value) => {
      $$(`[data-control="${name}"] .seg-btn`).forEach((button) => button.setAttribute("aria-pressed", String(button.dataset.value === String(value))));
      placeSegments();
    };
    function inspect(announce = false) {
      if (!st.visual) return;
      const rows = current(), prior = previous(), i = clamp(st.index, 0, rows.length - 1);
      st.index = i;
      const left = 48, right = st.width - 18, bottom = st.height - 36;
      const p = i / (rows.length - 1) * 89, j = Math.floor(p);
      const value = st.visual[j] + ((st.visual[j + 1] ?? st.visual[j]) - st.visual[j]) * (p - j);
      const x = left + i / (rows.length - 1) * (right - left), y = 22 + (1 - value / st.visual[180]) * (bottom - 22);
      $("[data-guide-x]").setAttribute("x1", x); $("[data-guide-x]").setAttribute("x2", x);
      $("[data-guide-x]").setAttribute("y1", 22); $("[data-guide-x]").setAttribute("y2", bottom);
      $("[data-guide-y]").setAttribute("x1", left); $("[data-guide-y]").setAttribute("x2", right);
      $("[data-guide-y]").setAttribute("y1", y); $("[data-guide-y]").setAttribute("y2", y);
      ["[data-point]", "[data-point-halo]"].forEach((selector) => { $(selector).setAttribute("cx", x); $(selector).setAttribute("cy", y); });
      $("[data-tip-date]").textContent = `${dateLabel(rows[i].date)}, 2026`;
      $("[data-tip-value]").textContent = money(rows[i].revenue);
      $("[data-tip-previous]").textContent = st.compare ? `${money(prior[i].revenue)} previous period` : "Generated daily revenue";
      const wrap = $("[data-chart-wrap]"), tooltip = $("[data-chart-tooltip]");
      const scale = wrap.clientWidth / st.width;
      const tipX = clamp(x * scale - tooltip.offsetWidth / 2, 4, Math.max(4, wrap.clientWidth - tooltip.offsetWidth - 4));
      const tipY = Math.max(0, y * scale - tooltip.offsetHeight - 16);
      tooltip.style.transform = `translate(${tipX}px,${tipY}px)`;
      $("#dt-day").value = i;
      $("#dt-day").style.setProperty("--p", `${i / (rows.length - 1) * 100}%`);
      const text = `${dateLabel(rows[i].date)}: ${money(rows[i].revenue)}${st.compare ? `; previous ${money(prior[i].revenue)}` : ""}`;
      if ($("#dt-day").getAttribute("aria-valuetext") !== text) $("#dt-day").setAttribute("aria-valuetext", text);
      if ($("#dt-point-readout").textContent !== text) $("#dt-point-readout").textContent = text;
      if (announce) $("[data-chart-wrap]").classList.add("is-inspecting");
    }
    function paintChart(values) {
      st.visual = values;
      const max = values[180], w = st.width, h = st.height;
      const d = line(values.slice(0, 90), w, h, max);
      $("[data-current-line]").setAttribute("d", d);
      $("[data-previous-line]").setAttribute("d", line(values.slice(90, 180), w, h, max));
      $("[data-comparison-mask]").setAttribute("d", $("[data-previous-line]").getAttribute("d"));
      $("[data-area]").setAttribute("d", `${d} L${w - 18},${h - 36} L48,${h - 36}Z`);
      $("[data-chart-grid]").innerHTML = Array.from({ length: 5 }, (_, i) => {
        const y = 22 + i / 4 * (h - 58);
        return `<line class="dt-grid-line" x1="48" x2="${w - 18}" y1="${y}" y2="${y}"/><text class="dt-axis-label" x="38" y="${y + 4}" text-anchor="end">${i === 4 ? "$0" : `$${(max * (1 - i / 4) / 1000).toFixed(1)}k`}</text>`;
      }).join("") + [0, .25, .5, .75, 1].map((t, i) => `<text class="dt-axis-label" x="${48 + t * (w - 66)}" y="${h - 10}" text-anchor="${i === 0 ? "start" : i === 4 ? "end" : "middle"}">${dateLabel(current()[Math.round(t * (st.period - 1))].date)}</text>`).join("");
      inspect();
    }
    function traceChart() {
      st.tweens.get("chart-draw")?.cancel();
      const path = $("[data-current-line]"), mask = $("[data-comparison-mask]"), area = $("[data-area]"), head = $("[data-draw-head]");
      const length = path.getTotalLength();
      path.style.strokeDasharray = "1"; mask.style.strokeDasharray = "1";
      const frame = (p) => {
        st.chartProgress = p;
        path.style.strokeDashoffset = String(1 - p);
        mask.style.strokeDashoffset = String(1 - clamp((p - .06) / .94, 0, 1));
        const point = path.getPointAtLength(length * p);
        area.style.clipPath = `inset(0 ${Math.max(0, (st.width - point.x) / st.width * 100)}% 0 0)`;
        area.style.opacity = String(clamp(p * 2.5, 0, 1));
        head.style.transform = `translate(${point.x}px,${point.y}px)`;
        head.style.opacity = String(p >= 1 ? 0 : clamp(p * 10, 0, 1) * clamp((1 - p) * 10, 0, 1));
        $("[data-chart-wrap]").dataset.drawing = p < 1 ? "true" : "false";
        if (p >= 1) area.style.clipPath = "none";
      };
      present("chart-draw", path, () => frame(0), () => animate("chart-draw", 0, 1, frame, 1100, S.ease.inOut), () => frame(1));
    }
    function updateChart(replay = false) {
      const rows = current(), prior = previous();
      const max = Math.ceil(Math.max(...rows.map((d) => d.revenue), ...(st.compare ? prior.map((d) => d.revenue) : [0])) / 1000) * 1000;
      const target = [...sample(rows.map((d) => d.revenue)), ...sample(prior.map((d) => d.revenue)), max];
      $("[data-main-chart]").classList.toggle("dt-no-comparison", !st.compare);
      $("#dt-day").max = rows.length - 1;
      $("[data-revenue-total]").textContent = money(sum(rows, "revenue"));
      $("[data-date-range]").textContent = `${dateLabel(rows[0].date)} - ${dateLabel(rows.at(-1).date)}, 2026`;
      const peak = rows.reduce((a, b) => a.revenue > b.revenue ? a : b);
      const change = (sum(rows, "revenue") / sum(prior, "revenue") - 1) * 100;
      const summary = `Generated ${st.period}-day revenue is ${money(sum(rows, "revenue"))}, ${Math.abs(change).toFixed(1)}% ${change >= 0 ? "above" : "below"} the preceding ${st.period} days. Peak: ${money(peak.revenue)} on ${dateLabel(peak.date)}.`;
      $("[data-chart-summary]").textContent = summary;
      $("#dt-line-desc").textContent = `${summary} Use the day slider below; Left and Right select a day, Home and End select the first and last day.`;
      paintChart(target);
      traceChart();
    }
    function updateKpis(replay = false) {
      const rows = current(), m = metrics(rows), prev = metrics(previous());
      kpis.forEach(([, , key, format]) => {
        const el = $(`[data-kpi="${key}"]`), start = replay ? 0 : Number(el.dataset.value ?? m[key]);
        animate(`kpi-${key}`, start, m[key], (v) => { el.textContent = format(v); el.dataset.value = v; }, 650, S.ease.outExpo);
        const change = (m[key] / prev[key] - 1) * 100, delta = $(`[data-delta="${key}"]`);
        delta.textContent = `${change >= 0 ? "+" : ""}${change.toFixed(1)}%`;
        delta.classList.toggle("dt-negative", change < 0);
        const values = rows.map((d) => key === "average" ? d.revenue / d.orders : key === "conversion" ? d.orders / d.visitors * 100 : d[key]);
        const path = $(`[data-spark="${key}"]`), target = sample(values, 24);
        path.setAttribute("d", line(target, 160, 42, Math.max(...target) * 1.12, 0, 3, 0, 3));
        path.setAttribute("pathLength", "1"); path.style.strokeDasharray = "1";
        const draw = (p) => { path.style.strokeDashoffset = String(1 - p); };
        present(`spark-${key}`, path, () => draw(0), () => animate(`spark-${key}`, 0, 1, draw, 950, S.ease.inOut, kpis.findIndex((k) => k[2] === key) * 70), () => draw(1));
      });
      $$("[data-period-label]").forEach((el) => { el.textContent = st.period; });
    }
    function updateBars(replay = false) {
      const rows = current();
      const buckets = Array.from({ length: 7 }, (_, i) => {
        const chunk = rows.slice(Math.floor(i * rows.length / 7), Math.floor((i + 1) * rows.length / 7));
        const total = sum(chunk, "orders"), first = Math.round(total * (.57 + Math.sin(i + st.period) * .09));
        return { a: first, b: total - first, label: dateLabel(chunk[0].date) };
      });
      const w = st.barWidth, h = st.barHeight || 380, base = h - 40, top = 28, plot = w - 64, step = plot / 7;
      $("[data-bars]").setAttribute("viewBox", `0 0 ${w} ${h}`);
      const max = Math.ceil(Math.max(...buckets.map((b) => st.mode === "stacked" ? b.a + b.b : Math.max(b.a, b.b))) / 100) * 100;
      const target = buckets.flatMap((b, i) => {
        const x = 48 + i * step + step * .18, bw = step * (st.mode === "grouped" ? .27 : .56);
        const ah = b.a / max * (base - top), bh = b.b / max * (base - top);
        return [x, base - ah, bw, ah, x + (st.mode === "grouped" ? bw + step * .04 : 0), base - bh - (st.mode === "stacked" ? ah : 0), bw, bh];
      });
      $("[data-bar-grid]").innerHTML = Array.from({ length: 4 }, (_, i) => `<line class="dt-grid-line" x1="44" x2="${w - 10}" y1="${top + (base - top) * i / 3}" y2="${top + (base - top) * i / 3}"/><text class="dt-axis-label" x="35" y="${top + (base - top) * i / 3 + 4}" text-anchor="end">${number(max * (1 - i / 3))}</text>`).join("");
      st.buckets = buckets;
      buckets.forEach((b, i) => {
        const label = $(`[data-bar-label="${i}"]`); label.textContent = b.label; label.setAttribute("x", 48 + i * step + step * .46); label.setAttribute("y", h - 12); label.setAttribute("text-anchor", "middle");
        const group = $(`[data-bar-group="${i}"]`), halo = $(`[data-bucket-halo="${i}"]`);
        group.setAttribute("aria-label", `${b.label}: ${b.a} new, ${b.b} returning. Press to pin this bucket.`);
        halo.setAttribute("x", 44 + i * step); halo.setAttribute("y", 16); halo.setAttribute("width", step); halo.setAttribute("height", base - 8);
      });
      const text = buckets.map((b) => `${b.label}: ${b.a} new, ${b.b} returning, ${b.a + b.b} total.`).join(" ");
      $("#dt-bars-desc").textContent = `${st.mode} orders. ${text}`;
      $("[data-bar-text]").textContent = text;
      const draw = (v) => {
        st.barVisual = v;
        $$("[data-bar]").forEach((bar, i) => {
          const p = i * 4; ["x", "y", "width", "height"].forEach((attr, j) => bar.setAttribute(attr, v[p + j]));
          const label = $(`[data-bar-value="${i}"]`), b = buckets[Math.floor(i / 2)];
          label.textContent = number(i % 2 ? b.b : b.a);
          label.setAttribute("x", v[p] + v[p + 2] / 2);
          label.setAttribute("y", v[p + 1] + (st.mode === "stacked" ? Math.min(18, v[p + 3] / 2 + 4) : -7));
          label.classList.toggle("dt-value-inside", st.mode === "stacked");
        });
        if (st.mode === "grouped") buckets.forEach((_, i) => {
          const a = $(`[data-bar-value="${i * 2}"]`), b = $(`[data-bar-value="${i * 2 + 1}"]`);
          const ay = Number(a.getAttribute("y")), by = Number(b.getAttribute("y"));
          if (Math.abs(ay - by) < 14) (ay <= by ? a : b).setAttribute("y", Math.min(ay, by) - (14 - Math.abs(ay - by)));
        });
      };
      draw(target);
      const bars = $$("[data-bar]"), labels = $$("[data-bar-value]");
      bars.forEach((bar, i) => {
        st.tweens.get(`bar-${i}`)?.cancel();
        bar.style.transformOrigin = `0px ${base}px`;
        const frame = (p) => { bar.style.transform = `scaleY(${p})`; labels[i].style.opacity = String(clamp((p - .25) / .75, 0, 1)); };
        present(`bar-${i}`, bar, () => frame(0), () => animate(`bar-${i}`, 0, 1, frame, 850, S.ease.outExpo, Math.floor(i / 2) * 65 + i % 2 * 35), () => frame(1));
      });
      highlightBar(st.barPinned);
    }
    function highlightBar(index) {
      st.barHover = index;
      const chosen = index !== null && index !== undefined;
      $("[data-bars]").classList.toggle("is-highlighting", chosen);
      $$("[data-bar-group]").forEach((group, i) => { group.classList.toggle("is-highlight", i === index); group.setAttribute("aria-pressed", String(i === st.barPinned)); });
      $("[data-bar-insight]").textContent = chosen && st.buckets ? `${st.buckets[index].label} · ${number(st.buckets[index].a)} new + ${number(st.buckets[index].b)} returning = ${number(st.buckets[index].a + st.buckets[index].b)} orders${st.barPinned === index ? " · pinned" : ""}` : "Hover a bucket to compare exact values. Click to keep it in view.";
    }
    const rankData = () => channels.map((name, i) => ({ name, i, revenue: Math.round(sum(current(), "revenue") * [.32, .24, .17, .12, .09, .06][i]), growth: [14.2, 8.7, 23.4, -3.1, 18.9, 5.6][i] }));
    function updateRanks(replay = false) {
      const data = rankData().sort((a, b) => st.rankSort === "name" ? a.name.localeCompare(b.name) : b[st.rankSort] - a[st.rankSort]);
      const nodes = $$("[data-rank]");
      flip(nodes, () => data.forEach((d, i) => {
        const li = $(`[data-rank="${d.i}"]`); $("[data-ranks]").append(li);
        li.querySelector("[data-rank-number]").textContent = String(i + 1).padStart(2, "0");
        li.querySelector("[data-rank-value]").textContent = st.rankSort === "growth" ? `${d.growth >= 0 ? "+" : ""}${d.growth}%` : money(d.revenue);
        li.querySelector("[data-rank-note]").textContent = `${d.growth >= 0 ? "+" : ""}${d.growth}% growth · ${money(d.revenue)} attributed revenue`;
        const fill = li.querySelector("[data-rank-fill]");
        const target = st.rankSort === "growth" ? Math.abs(d.growth) / 25 : d.revenue / Math.max(...data.map((r) => r.revenue));
        st.tweens.get(`rank-${d.i}`)?.cancel();
        const draw = (v) => { fill.dataset.value = v; fill.style.transform = `scaleX(${v})`; };
        present(`rank-${d.i}`, fill, () => draw(0), () => animate(`rank-${d.i}`, 0, target, draw, 950, S.ease.outExpo, i * 60), () => draw(target));
      }));
      st.rankSummary = `Sorted by ${st.rankSort}. First: ${data[0].name}, ${st.rankSort === "growth" ? `${data[0].growth}% growth` : money(data[0].revenue)}.`;
      $("[data-rank-summary]").textContent = st.rankSummary;
    }
    function highlightRank(index) {
      $("[data-ranks]").classList.toggle("is-highlighting", index !== null);
      $$("[data-rank-inspect]").forEach((button) => { button.classList.toggle("is-highlight", +button.dataset.rankInspect === index); button.setAttribute("aria-pressed", String(+button.dataset.rankInspect === st.rankPinned)); });
      if (index !== null) { const item = rankData().find((d) => d.i === index); $("[data-rank-summary]").textContent = `${item.name} · ${money(item.revenue)} attributed revenue · ${item.growth > 0 ? "+" : ""}${item.growth}% growth${st.rankPinned === index ? " · pinned" : ""}`; }
      else if (st.rankSummary) $("[data-rank-summary]").textContent = st.rankSummary;
    }
    function updateDonut(replay = false) {
      const total = devices.reduce((n, d, i) => n + (st.enabled[i] ? d.count : 0), 0);
      const target = devices.map((d, i) => st.enabled[i] && total ? d.count / total * 100 : 0);
      $$("[data-device]").forEach((button, i) => { button.setAttribute("aria-pressed", st.enabled[i]); $(`[data-device-share="${i}"]`).textContent = st.enabled[i] ? `${Math.round(target[i])}%` : "Off"; });
      const summary = total ? `${number(total)} included sessions. ${devices.map((d, i) => `${d.name}: ${st.enabled[i] ? `${target[i].toFixed(1)}%` : "excluded"}`).join("; ")}.` : "No devices included. Select a device to show its sessions.";
      $("[data-device-summary]").textContent = summary; $("#dt-donut-desc").textContent = summary;
      st.donutVisual = target; st.donutTotal = total;
      $$("[data-donut]").forEach((circle, i) => circle.setAttribute("aria-pressed", String(st.enabled[i])));
      const draw = (p) => { let offset = 0; target.forEach((share, i) => { const circle = $(`[data-donut="${i}"]`); circle.setAttribute("stroke-dasharray", `${Math.max(0, Math.min(share, p * 100 - offset) - 1.3)} 100`); circle.setAttribute("stroke-dashoffset", -offset); offset += share; }); };
      st.finishDonut = () => draw(1);
      st.tweens.get("donut")?.cancel();
      present("donut", $("[data-donut]"), () => draw(0), () => animate("donut", 0, 1, draw, 1150, S.ease.inOut), () => draw(1));
      highlightDevice(st.deviceHover, false);
    }
    function highlightDevice(index, reveal = true) {
      st.deviceHover = index;
      const active = index !== null && index !== undefined;
      if (active && reveal && st.finishDonut) { st.pending.delete("donut"); st.tweens.get("donut")?.cancel(); st.finishDonut(); }
      $(".dt-donut-layout").classList.toggle("is-highlighting", active);
      let offset = 0;
      $$("[data-donut]").forEach((arc, i) => {
        const share = st.donutVisual[i], angle = (offset + share / 2) / 100 * Math.PI * 2 - Math.PI / 2;
        arc.classList.toggle("is-highlight", i === index);
        arc.style.setProperty("--arc-x", `${i === index ? Math.cos(angle) * 4 : 0}px`); arc.style.setProperty("--arc-y", `${i === index ? Math.sin(angle) * 4 : 0}px`);
        $(`[data-device="${i}"]`).classList.toggle("is-highlight", i === index); offset += share;
      });
      const count = active ? st.enabled[index] ? devices[index].count : 0 : st.donutTotal;
      const label = active ? `${devices[index].name.toLowerCase()} ${st.enabled[index] ? "sessions" : "excluded"}` : "included sessions";
      $("[data-donut-label]").textContent = label;
      const el = $("[data-donut-total]"), from = Number(el.dataset.value ?? count);
      animate("donut-total", from, count, (v) => { el.textContent = number(v); el.dataset.value = v; }, 420, S.ease.outExpo);
    }
    function updateGoals(replay = false) {
      goals.forEach((goal, i) => {
        const ring = $(`[data-ring="${i}"]`);
        st.tweens.get(`goal-${i}`)?.cancel();
        const draw = (p) => { const v = goal.value * p, angle = v / 100 * Math.PI * 2 - Math.PI / 2, end = $(`[data-ring-end="${i}"]`); ring.dataset.value = v; ring.setAttribute("stroke-dasharray", "100"); ring.setAttribute("stroke-dashoffset", 100 - v); end.setAttribute("cx", 50 + 40 * Math.cos(angle)); end.setAttribute("cy", 50 + 40 * Math.sin(angle)); end.style.opacity = String(clamp((p - .7) / .3, 0, 1)); };
        st.goalFinishes ||= []; st.goalFinishes[i] = () => draw(1);
        present(`goal-${i}`, ring, () => draw(0), () => animate(`goal-${i}`, 0, 1, draw, 1100, S.ease.inOut, i * 90), () => draw(1));
      });
    }
    function highlightGoal(index) {
      if (index !== null && st.goalFinishes?.[index]) { st.pending.delete(`goal-${index}`); st.tweens.get(`goal-${index}`)?.cancel(); st.goalFinishes[index](); }
      $$("[data-goal]").forEach((el, i) => { el.classList.toggle("is-highlight", i === index); el.setAttribute("aria-pressed", String(i === st.goalPinned)); });
      $("[data-goal-insight]").textContent = index === null ? "Hover a target to see what remains. Click to keep its detail visible." : `${goals[index].name} · ${goals[index].detail} · ${100 - goals[index].value}% remaining${index === st.goalPinned ? " · pinned" : ""}`;
    }
    const filtered = () => accounts.filter((a) => (st.status === "all" || st.status === a.status) && `${a.name} ${a.id} ${a.channel}`.toLowerCase().includes(st.query.toLowerCase().trim())).sort((a, b) => st.direction * (typeof a[st.sort] === "string" ? a[st.sort].localeCompare(b[st.sort]) : a[st.sort] - b[st.sort]) || a.id.localeCompare(b.id));
    function selection() {
      const visible = $$("[data-account]").filter((row) => !row.hidden);
      const count = visible.filter((row) => st.selected.has(row.dataset.account)).length;
      const all = $("[data-select-page]"); all.checked = !!visible.length && count === visible.length; all.indeterminate = count > 0 && count < visible.length; all.disabled = !visible.length;
      $$("[data-select]").forEach((input) => { input.checked = st.selected.has(input.dataset.select); const row = input.closest("tr"); row.classList.toggle("is-selected", input.checked); row.setAttribute("aria-selected", String(input.checked)); });
      $("[data-selection-count]").textContent = `${st.selected.size} selected`;
      $("[data-clear-selection]").hidden = st.selected.size === 0;
      $("[data-export-label]").textContent = st.selected.size ? `Export ${st.selected.size} selected` : "Export filtered CSV";
      $("[data-export]").disabled = st.selected.size === 0 && filtered().length === 0;
    }
    function updateTable() {
      const result = filtered(), pages = Math.max(1, Math.ceil(result.length / st.size));
      st.page = clamp(st.page, 0, pages - 1);
      const visible = result.slice(st.page * st.size, (st.page + 1) * st.size), ids = new Set(visible.map((a) => a.id));
      flip($$("[data-account]"), () => {
        $$("[data-account]").forEach((row) => { row.hidden = !ids.has(row.dataset.account); });
        result.forEach((a) => $("tbody").append($(`[data-account="${a.id}"]`)));
      });
      $$("[data-column]").forEach((th) => { const active = th.dataset.column === st.sort; th.setAttribute("aria-sort", active ? st.direction === 1 ? "ascending" : "descending" : "none"); th.querySelector("button").setAttribute("aria-label", `${th.querySelector("button").textContent.trim()}, sort ${active && st.direction === 1 ? "descending" : "ascending"}`); });
      $("[data-table-empty]").hidden = !!result.length;
      $("[data-table-summary]").textContent = result.length ? `${st.page * st.size + 1}-${Math.min((st.page + 1) * st.size, result.length)} of ${result.length} customers` : "0 of 28 customers match";
      $("[data-page-count]").textContent = `${st.page + 1} / ${pages}`;
      $("[data-prev-page]").disabled = st.page === 0; $("[data-next-page]").disabled = st.page >= pages - 1;
      selection();
    }
    function exportCsv() {
      const rows = st.selected.size ? accounts.filter((a) => st.selected.has(a.id)) : filtered();
      if (!rows.length) return;
      const csv = (v) => `"${String(v).replace(/"/g, '""')}"`;
      const text = [["Generated demo data", "Account ID", "Customer", "Channel", "Status", "Orders", "Revenue USD", "Growth percent"], ...rows.map((a) => ["Simulated", a.id, a.name, a.channel, a.status, a.orders, a.revenue, a.growth])].map((r) => r.map(csv).join(",")).join("\r\n");
      if (st.url) URL.revokeObjectURL(st.url); clearTimeout(st.revokeTimer);
      st.url = URL.createObjectURL(new Blob(["\uFEFF", text], { type: "text/csv;charset=utf-8" }));
      const link = document.createElement("a"); link.href = st.url; link.download = "schematic-generated-customers.csv"; page.append(link); link.click(); link.remove();
      $("[data-export-status]").textContent = `Generated CSV for ${rows.length} ${st.selected.size ? "selected" : "filtered"} accounts. Download requested locally.`;
      st.revokeTimer = setTimeout(() => { URL.revokeObjectURL(st.url); st.url = null; }, 1000);
    }
    function tick() {
      const date = days.at(-1).date, next = seeded(6028 + ++st.ticks)();
      st.increments.set(date, (st.increments.get(date) || 0) + 15 + Math.round(next * 80));
      updateChart(); updateKpis(); updateRanks();
      $("[data-live-status]").textContent = `Simulated sample ${st.ticks} added to Oct 28. ${st.live && !S.reduced() ? "Next sample in 4 seconds." : "Manual updates available."}`;
    }
    function schedule() {
      clearTimeout(st.timer); st.timer = null;
      if (running() && st.live && !S.reduced()) st.timer = setTimeout(() => { if (running() && st.live && !S.reduced()) tick(); schedule(); }, 4000);
    }
    const liveLabel = () => { $("[data-live-status]").textContent = !st.live ? `Off. ${st.ticks ? `${st.ticks} simulated samples retained.` : "Seeded daily values; no external connection."}` : S.reduced() ? "Reduced motion: automatic updates paused. Use Add sample." : !running() ? "Simulation paused while this page is hidden." : "Simulation enabled. One local sample every 4 seconds."; };
    const stop = () => {
      clearTimeout(st.timer); st.timer = null;
      st.tweens.forEach((tween) => tween.cancel()); st.tweens.clear();
      st.animations.forEach((animation) => animation.cancel()); st.animations.clear();
      st.pending.clear(); revealObserver.disconnect();
      if (st.url) { URL.revokeObjectURL(st.url); st.url = null; } clearTimeout(st.revokeTimer);
      $("[data-chart-wrap]").classList.remove("is-inspecting");
    };
    function resize() {
      if (!running()) return;
      const width = Math.max(320, Math.round($("[data-chart-wrap]").clientWidth));
      st.width = width; st.height = width < 500 ? 280 : 300;
      $("[data-main-chart]").setAttribute("viewBox", `0 0 ${width} ${st.height}`);
      $("[data-chart-hit]").setAttribute("width", width - 66); $("[data-chart-hit]").setAttribute("height", st.height - 58);
      placeSegments(); if (st.visual) paintChart(st.visual);
      const barWidth = Math.max(320, Math.round($("[data-bars]").clientWidth));
      const barHeight = Math.max(320, Math.round($("[data-bars]").clientHeight));
      if (barWidth !== st.barWidth || barHeight !== st.barHeight) { st.barWidth = barWidth; st.barHeight = barHeight; updateBars(); }
    }
    const ro = new ResizeObserver(resize);
    const media = matchMedia("(prefers-reduced-motion: reduce)");
    function motionChanged() {
      const reduced = S.reduced(); if (st.reduced === reduced) return; st.reduced = reduced;
      page.classList.toggle("dt-reduced", reduced);
      if (reduced) { stop(); updateChart(); updateKpis(); updateBars(); updateDonut(); updateGoals(); updateRanks(); }
      liveLabel(); schedule();
    }
    const mo = new MutationObserver(motionChanged);
    function resume(replay) {
      if (!running()) return;
      st.reduced = S.reduced(); page.classList.toggle("dt-reduced", st.reduced);
      ro.observe($("[data-chart-wrap]")); ro.observe($("[data-control='period']")); ro.observe($("[data-bars]"));
      mo.observe(document.documentElement, { attributes: true, attributeFilter: ["data-motion", "style"] });
      resize(); updateChart(replay); updateKpis(replay); updateBars(replay); updateRanks(replay); updateDonut(replay); updateGoals(replay);
      liveLabel(); schedule();
    }
    document.addEventListener("visibilitychange", () => { if (!st.active) return; if (document.hidden) { stop(); ro.disconnect(); mo.disconnect(); liveLabel(); } else resume(false); });
    media.addEventListener("change", () => { if (running()) motionChanged(); });
    page.addEventListener("click", (event) => {
      const row = event.target.closest("[data-account]");
      if (row && !event.target.closest("button,input,label,a,select")) {
        const textSelection = window.getSelection();
        if (!textSelection?.isCollapsed && textSelection?.containsNode(row, true)) return;
        toggleRow(row, event.shiftKey); row.focus({ preventScroll: true }); return;
      }
      const bucket = event.target.closest("[data-bar-group]");
      if (bucket) { st.barPinned = st.barPinned === +bucket.dataset.barGroup ? null : +bucket.dataset.barGroup; highlightBar(st.barPinned); return; }
      const arc = event.target.closest("[data-donut]");
      if (arc) { const i = +arc.dataset.donut; st.enabled[i] = !st.enabled[i]; updateDonut(); return; }
      const button = event.target.closest("button"); if (!button) return;
      const control = button.closest("[data-control]");
      if (control && button.dataset.value) {
        const name = control.dataset.control; setSegment(name, button.dataset.value);
        if (name === "period" && st.period !== Number(button.dataset.value)) { st.period = Number(button.dataset.value); st.index = st.period - 1; updateChart(); updateKpis(); updateBars(); updateRanks(); }
        if (name === "bars" && st.mode !== button.dataset.value) { st.mode = button.dataset.value; updateBars(); }
      }
      if (button.hasAttribute("data-device")) { const i = Number(button.dataset.device); st.enabled[i] = !st.enabled[i]; updateDonut(); }
      if (button.hasAttribute("data-rank-inspect")) { const i = +button.dataset.rankInspect; st.rankPinned = st.rankPinned === i ? null : i; highlightRank(st.rankPinned ?? null); }
      if (button.hasAttribute("data-goal")) { const i = +button.dataset.goal; st.goalPinned = st.goalPinned === i ? null : i; highlightGoal(st.goalPinned); }
      if (button.hasAttribute("data-table-sort")) { const key = button.dataset.tableSort; st.direction = st.sort === key ? -st.direction : 1; st.sort = key; updateTable(); }
      if (button.hasAttribute("data-prev-page")) { st.page--; updateTable(); }
      if (button.hasAttribute("data-next-page")) { st.page++; updateTable(); }
      if (button.hasAttribute("data-clear-selection")) { st.selected.clear(); selection(); }
      if (button.hasAttribute("data-export")) exportCsv();
      if (button.hasAttribute("data-reset")) { st.query = ""; st.status = "all"; st.page = 0; $("[data-search]").value = ""; $("[data-status]").value = "all"; updateTable(); }
      if (button.hasAttribute("data-tick")) { tick(); schedule(); }
    });
    page.addEventListener("change", (event) => {
      const el = event.target;
      if (el.matches("[data-compare]")) { st.compare = el.checked; updateChart(); }
      if (el.matches("[data-live]")) { st.live = el.checked; liveLabel(); schedule(); }
      if (el.matches("[data-rank-sort]")) { st.rankSort = el.value; updateRanks(); }
      if (el.matches("[data-status]")) { st.status = el.value; st.page = 0; updateTable(); }
      if (el.matches("[data-page-size]")) { st.size = Number(el.value); st.page = 0; updateTable(); }
      if (el.matches("[data-select]")) { el.checked ? st.selected.add(el.dataset.select) : st.selected.delete(el.dataset.select); st.selectionAnchor = el.dataset.select; selection(); }
      if (el.matches("[data-select-page]")) { $$("[data-account]").filter((row) => !row.hidden).forEach((row) => { el.checked ? st.selected.add(row.dataset.account) : st.selected.delete(row.dataset.account); }); selection(); }
    });
    function toggleRow(row, extend = false) {
      const id = row.dataset.account;
      const rows = $$("[data-account]").filter((el) => !el.hidden);
      const anchor = rows.findIndex((el) => el.dataset.account === st.selectionAnchor), end = rows.indexOf(row);
      if (extend && anchor >= 0) rows.slice(Math.min(anchor, end), Math.max(anchor, end) + 1).forEach((el) => st.selected.add(el.dataset.account));
      else { st.selected.has(id) ? st.selected.delete(id) : st.selected.add(id); st.selectionAnchor = id; }
      selection();
    }
    page.addEventListener("keydown", (event) => {
      const row = event.target.closest("[data-account]");
      if (row && event.target === row) {
        if ([" ", "Enter"].includes(event.key)) { event.preventDefault(); toggleRow(row, event.shiftKey); }
        if (["ArrowUp", "ArrowDown"].includes(event.key)) { event.preventDefault(); const rows = $$("[data-account]").filter((el) => !el.hidden); rows[clamp(rows.indexOf(row) + (event.key === "ArrowDown" ? 1 : -1), 0, rows.length - 1)].focus(); }
      }
      const target = event.target.closest("[data-bar-group],[data-donut]");
      if (target && [" ", "Enter"].includes(event.key)) { event.preventDefault(); target.dispatchEvent(new MouseEvent("click", { bubbles: true })); }
      if (event.key === "Escape" && target?.hasAttribute("data-bar-group")) { st.barPinned = null; highlightBar(null); }
    });
    const highlightTarget = (target) => {
      const bar = target.closest("[data-bar-group]"); if (bar) highlightBar(+bar.dataset.barGroup);
      const rank = target.closest("[data-rank-inspect]"); if (rank) highlightRank(+rank.dataset.rankInspect);
      const device = target.closest("[data-device],[data-donut]"); if (device) { const i = Number(device.dataset.device ?? device.dataset.donut); if (st.deviceHover !== i) highlightDevice(i); }
      const goal = target.closest("[data-goal]"); if (goal) highlightGoal(+goal.dataset.goal);
    };
    page.addEventListener("pointerover", (event) => { if (event.pointerType !== "touch") highlightTarget(event.target); });
    page.addEventListener("focusin", (event) => highlightTarget(event.target));
    $("[data-bars]").addEventListener("pointerleave", () => { if (!$("[data-bars]").contains(document.activeElement)) highlightBar(st.barPinned); });
    $("[data-ranks]").addEventListener("pointerleave", () => { if (!$("[data-ranks]").contains(document.activeElement)) highlightRank(st.rankPinned ?? null); });
    $(".dt-donut-layout").addEventListener("pointerleave", () => { if (!$(".dt-donut-layout").contains(document.activeElement)) highlightDevice(null); });
    $(".dt-goals").addEventListener("pointerleave", () => { if (!$(".dt-goals").contains(document.activeElement)) highlightGoal(st.goalPinned); });
    page.addEventListener("focusout", (event) => {
      if (!$("[data-bars]").contains(event.relatedTarget)) highlightBar(st.barPinned);
      if (!$("[data-ranks]").contains(event.relatedTarget)) highlightRank(st.rankPinned ?? null);
      if (!$(".dt-donut-layout").contains(event.relatedTarget)) highlightDevice(null);
      if (!$(".dt-goals").contains(event.relatedTarget)) highlightGoal(st.goalPinned);
    });
    $("[data-search]").addEventListener("input", (event) => { st.query = event.target.value; st.page = 0; updateTable(); });
    $("#dt-day").addEventListener("input", (event) => { st.index = Number(event.target.value); inspect(true); });
    $("#dt-day").addEventListener("focus", () => inspect(true));
    $("#dt-day").addEventListener("keydown", (event) => { if (event.key === "Escape") $("[data-chart-wrap]").classList.remove("is-inspecting"); });
    $("#dt-day").addEventListener("blur", () => $("[data-chart-wrap]").classList.remove("is-inspecting"));
    const pointAt = (event) => {
      const rect = $("[data-main-chart]").getBoundingClientRect();
      const x = (event.clientX - rect.left) / rect.width * st.width;
      st.index = Math.round(clamp((x - 48) / (st.width - 66), 0, 1) * (st.period - 1)); inspect(true);
    };
    $("[data-chart-hit]").addEventListener("pointermove", pointAt);
    $("[data-chart-hit]").addEventListener("pointerdown", pointAt);
    $("[data-chart-wrap]").addEventListener("pointerleave", () => { if (document.activeElement !== $("#dt-day")) $("[data-chart-wrap]").classList.remove("is-inspecting"); });
    page.addEventListener("replay", (event) => {
      if (!running()) return;
      const demo = event.target.closest(".demo"); if (!demo) return;
      if (demo.querySelector("[data-main-chart]")) { updateChart(true); updateKpis(true); }
      if (demo.querySelector("[data-bars]")) updateBars(true);
      if (demo.querySelector("[data-ranks]")) updateRanks(true);
      if (demo.querySelector("[data-donut]")) updateDonut(true);
      if (demo.querySelector("[data-ring]")) updateGoals(true);
    });
    updateTable();
    instances.set(root, {
      enter() { st.active = true; resume(true); },
      leave() { st.active = false; stop(); ro.disconnect(); mo.disconnect(); liveLabel(); }
    });
  }

  S.page({
    id: "data", title: "Data & Charts", group: "Build", icon: "chart-bar", order: 60,
    description: "Continuous charts, meaningful comparisons, and data you can explore.",
    render: () => `<div class="pg-data">${S.pageHead({ icon: "chart-bar", eyebrow: "Build / 06", title: "Data & Charts", lead: "Make the change visible. Numbers resolve, paths find their next shape, and every point has a story you can read.", meta: ['<span class="badge badge-accent">Generated demo data</span>', '<span class="badge">Local & deterministic</span>', '<span class="badge">Keyboard ready</span>'] })}${S.section({ id: "dt-overview", num: "01", title: "Signal, at a glance", desc: "One period, four perspectives. The same seeded daily records power the cards, revenue curve, and order buckets.", body: `<div class="dt-period-bar"><div><span class="dt-workspace-name">Schematic Commerce</span><span class="t-caption">Analytics playground · USD</span></div>${segment("period", [["7", "7D"], ["30", "30D"], ["90", "90D"]], "30", "Analytics period")}</div>${cardMarkup()}${chartMarkup()}` })}${S.section({ id: "dt-comparisons", num: "02", title: "See the relationship", desc: "A different arrangement answers a different question. Compare volumes, then follow a channel as it changes rank.", body: `<div class="grid cols-2 dt-comparison-grid">${barMarkup()}${rankMarkup()}</div>` })}${S.section({ id: "dt-composition", num: "03", title: "Parts & progress", desc: "Give every segment a name and every target a denominator. These illustrative snapshots are independent of the period above.", body: `<div class="grid cols-2">${donutMarkup()}${goalMarkup()}</div>` })}${S.section({ id: "dt-records", num: "04", title: "Details you can work with", desc: "Click anywhere in a customer row to select it, or press Space when focused. Filter, sort and take the selected records with you.", body: tableMarkup() })}<footer class="dt-page-note"><span class="t-mono">[ DATA / DEMO ]</span><p>All figures, accounts, and live samples are generated for this prototype. No production analytics or network requests.</p></footer></div>`,
    mount,
    enter: (root) => instances.get(root)?.enter(),
    leave: (root) => instances.get(root)?.leave()
  });
})();

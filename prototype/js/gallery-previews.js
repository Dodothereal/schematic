(() => {
  const S = window.Schematic,
    icon = (name, size = 16) => S.icon(name, { size });
  const kinds = {
    "components-2": "action",
    "data-0": "chart",
    "primitives-3": "sort",
    "components-10": "tabs",
    "icons-0": "icons",
    "primitives-5": "counter",
  };
  const priority = Object.keys(kinds);
  S.galleryEntries = () =>
    [...S.catalog()].sort((a, b) => {
      const ai = priority.indexOf(a.id),
        bi = priority.indexOf(b.id);
      return (ai < 0 ? 99 : ai) - (bi < 0 ? 99 : bi);
    });
  const scenes = {
    action: () =>
      `<div class="ov-action-state"><span class="ov-preview-eyebrow"><i></i><span data-gallery-action-status role="status">Ready</span></span><button class="ov-run-action" data-gallery-action><span data-gallery-action-icon>${icon("play", 17)}</span><span data-gallery-action-label>Run action</span><span class="ov-action-progress" aria-hidden="true" data-gallery-progress></span></button><span class="ov-action-caption" data-gallery-action-caption></span></div>`,
    chart: () =>
      `<div class="ov-chart-head"><div><span class="ov-chart-amount" data-gallery-chart-value>$4,190</span><span class="ov-chart-caption" data-gallery-chart-caption>7 days</span></div><div class="ov-chart-ranges" role="group" aria-label="Revenue preview period"><button data-gallery-range="7" aria-pressed="true">7D</button><button data-gallery-range="30" aria-pressed="false">30D</button></div></div><svg class="ov-gallery-chart" data-gallery-chart viewBox="0 0 300 116" preserveAspectRatio="none" fill="none" tabindex="0" role="img" aria-label="Revenue preview. Use Left and Right to inspect a day."><path class="ov-mini-grid" d="M8 25H292M8 65H292M8 105H292"/><path data-gallery-area class="ov-mini-area"/><path data-gallery-line class="ov-mini-line" pathLength="1"/><g class="ov-chart-inspect" data-gallery-inspect hidden><path data-gallery-crosshair/><circle data-gallery-dot r="4"/></g></svg>`,
    sort: () =>
      `<div class="ov-sort-head"><span>Release checklist</span><output data-gallery-task-count>1 / 3</output></div><div class="ov-gallery-sort" data-gallery-sort aria-label="Release tasks">${["Design tokens", "Component API", "Keyboard review"].map((title, i) => `<div class="ov-gallery-task" data-gallery-task="${i}"><button class="ov-task-grip" data-sortable-handle aria-label="Move ${title}">${icon("grip", 15)}</button><label><input type="checkbox" ${i === 0 ? "checked" : ""} data-gallery-task-check><span>${title}</span></label>${icon("chevron-right", 12)}</div>`).join("")}</div>`,
    tabs: () =>
      `<div class="seg seg-block ov-gallery-tabs" role="group" aria-label="Preview tabs">${["Overview", "Activity", "Settings"].map((title, i) => `<button class="seg-btn" data-value="${i}">${title}</button>`).join("")}</div><div class="ov-gallery-panels"><div data-gallery-panel="0"><span class="ov-preview-eyebrow">Weekly activity</span><div class="ov-tab-chart" aria-hidden="true">${[32, 48, 37, 63, 52, 78, 91, 72, 84, 61, 94, 100].map((h) => `<i style="height:${h}%"></i>`).join("")}</div></div><div data-gallery-panel="1" hidden><span class="ov-activity-row">${icon("check-circle", 16)}<span>Build completed</span><small>2m</small></span><span class="ov-activity-row">${icon("git-pull-request", 16)}<span>Review requested</span><small>8m</small></span></div><div data-gallery-panel="2" hidden><label class="ov-preview-setting">Email updates<input type="checkbox" checked></label><label class="ov-preview-setting">Weekly digest<input type="checkbox"></label></div></div>`,
    icons: () =>
      `<div class="ov-icon-preview-grid">${["send", "check", "bell", "trash", "lock", "folder", "heart", "download"].map((name) => `<button data-gallery-icon="${name}" aria-label="Play ${name} icon" data-tip="${name}">${icon(name, 25)}</button>`).join("")}</div><span class="ov-icon-caption" data-gallery-icon-name>Hover or focus</span>`,
    counter: () =>
      `<div class="ov-counter-controls"><button data-gallery-step="-1" aria-label="Decrease preview counter">${icon("minus", 18)}</button><output class="ov-counter-number" data-gallery-counter aria-live="polite">024</output><button data-gallery-step="1" aria-label="Increase preview counter">${icon("plus", 18)}</button></div><button class="ov-counter-reset" data-gallery-counter-reset>Reset ${icon("refresh", 12)}</button>`,
  };
  S.galleryPreview = {
    render(id) {
      const kind = kinds[id];
      return kind
        ? `<div class="ov-live-preview ov-live-${kind}" data-gallery-kind="${kind}">${scenes[kind]()}</div>`
        : "";
    },
    mount(root) {
      let active = false,
        job = 0,
        chartAnimation = null,
        chartStarted = false,
        areaAnimation = null,
        tabAnimations = new Set(),
        range = 7,
        points = [],
        selected = -1,
        counter = 24,
        started = 0;
      const $ = (s) => root.querySelector(s),
        $$ = (s) => [...root.querySelectorAll(s)];
      const cards = $$(".ov-specimen-card[data-has-preview]"),
        visibility = new Map(cards.map((el) => [el, false]));
      const allowed = (kind) =>
        active &&
        !document.hidden &&
        !!$(`[data-gallery-kind="${kind}"]`)?.closest(
          ".ov-specimen-card:not([hidden])",
        ) &&
        visibility.get(
          $(`[data-gallery-kind="${kind}"]`).closest(".ov-specimen-card"),
        ) !== false;
      const money = (v) => "$" + Math.round(v).toLocaleString("en-US");
      const resetAction = (label = "") => {
        cancelAnimationFrame(job);
        job = 0;
        $("[data-gallery-action]").disabled = false;
        $("[data-gallery-action]").removeAttribute("data-state");
        $("[data-gallery-action-icon]").innerHTML = icon("play", 17);
        $("[data-gallery-action-label]").textContent = "Run action";
        $("[data-gallery-action-status]").textContent = "Ready";
        $("[data-gallery-action-caption]").textContent = label;
        $("[data-gallery-progress]").style.transform = "scaleX(0)";
      };
      $("[data-gallery-action]").addEventListener("click", () => {
        if (!allowed("action") || job) return;
        const button = $("[data-gallery-action]");
        $("[data-gallery-progress]").style.transform = "scaleX(0)";
        button.disabled = true;
        button.dataset.state = "running";
        $("[data-gallery-action-icon]").innerHTML = icon("loader", 17);
        $("[data-gallery-action-status]").textContent = "Running";
        $("[data-gallery-action-caption]").textContent = "Local preview";
        started = performance.now();
        const tick = (t) => {
          if (!allowed("action")) {
            resetAction("Cancelled");
            return;
          }
          const p = Math.min(1, (t - started) / 1300);
          $("[data-gallery-action-label]").textContent =
            `Working ${Math.floor(p * 100)}%`;
          $("[data-gallery-progress]").style.transform = `scaleX(${p})`;
          if (p < 1) {
            job = requestAnimationFrame(tick);
            return;
          }
          job = 0;
          button.disabled = false;
          button.dataset.state = "complete";
          $("[data-gallery-action-icon]").innerHTML = icon("check", 17);
          $("[data-gallery-action-label]").textContent = "Complete";
          $("[data-gallery-action-status]").textContent = "Done";
          $("[data-gallery-action-caption]").textContent = "Click to replay";
          S.playIcon($("[data-gallery-action-icon] svg"));
        };
        job = requestAnimationFrame(tick);
      });
      const base = [38, 53, 48, 68, 52, 76, 84];
      function drawChart(trace = true) {
        if (trace && allowed("chart")) chartStarted = true;
        points = Array.from({ length: range }, (_, i) => ({
          x: 8 + (i * 284) / (range - 1),
          value:
            range === 7
              ? base[i]
              : Math.round(
                  48 +
                    Math.sin(i * 0.68) * 13 +
                    i * 0.65 +
                    Math.cos(i * 1.7) * 9,
                ),
        }));
        const min = Math.min(...points.map((p) => p.value)) - 12,
          max = Math.max(...points.map((p) => p.value)) + 10;
        points.forEach(
          (p) => (p.y = 105 - ((p.value - min) / (max - min)) * 91),
        );
        let path = `M${points[0].x},${points[0].y}`;
        for (let i = 1; i < points.length; i++) {
          const a = points[Math.max(0, i - 2)],
            b = points[i - 1],
            c = points[i],
            d = points[Math.min(points.length - 1, i + 1)];
          path += `C${b.x + (c.x - a.x) / 6},${b.y + (c.y - a.y) / 6} ${c.x - (d.x - b.x) / 6},${c.y - (d.y - b.y) / 6} ${c.x},${c.y}`;
        }
        $("[data-gallery-line]").setAttribute("d", path);
        $("[data-gallery-area]").setAttribute("d", path + "L292 116H8Z");
        selected = -1;
        showPoint();
        chartAnimation?.cancel();
        chartAnimation = null;
        areaAnimation?.cancel();
        areaAnimation = null;
        if (trace && allowed("chart") && !S.reduced()) {
          const a = $("[data-gallery-line]").animate(
            [{ strokeDashoffset: 1 }, { strokeDashoffset: 0 }],
            { duration: S.dur(950), easing: S.curve.out },
          );
          const area = $("[data-gallery-area]").animate(
            [
              { clipPath: "inset(0 100% 0 0)" },
              { clipPath: "inset(0 0% 0 0)" },
            ],
            { duration: S.dur(950), easing: S.curve.out },
          );
          areaAnimation = area;
          area.finished.then(
            () => {
              if (areaAnimation === area) {
                areaAnimation = null;
                area.cancel();
              }
            },
            () => {},
          );
          chartAnimation = a;
          a.finished.then(
            () => {
              if (chartAnimation === a) {
                chartAnimation = null;
                a.cancel();
              }
            },
            () => {},
          );
        }
      }
      function showPoint() {
        const p = points[selected];
        $("[data-gallery-inspect]").toggleAttribute("hidden", !p);
        $("[data-gallery-chart-value]").textContent = money(
          p ? p.value * 10 : points.reduce((n, p) => n + p.value * 10, 0),
        );
        $("[data-gallery-chart-caption]").textContent = p
          ? `Day ${selected + 1} of ${range}`
          : `${range} days`;
        if (p) {
          $("[data-gallery-crosshair]").setAttribute("d", `M${p.x} 6V112`);
          $("[data-gallery-dot]").setAttribute("cx", p.x);
          $("[data-gallery-dot]").setAttribute("cy", p.y);
        }
      }
      $$("[data-gallery-range]").forEach((button) =>
        button.addEventListener("click", () => {
          range = +button.dataset.galleryRange;
          $$("[data-gallery-range]").forEach((el) =>
            el.setAttribute("aria-pressed", String(el === button)),
          );
          drawChart();
        }),
      );
      $("[data-gallery-chart]").addEventListener("pointermove", (e) => {
        const r = e.currentTarget.getBoundingClientRect();
        selected = Math.max(
          0,
          Math.min(
            points.length - 1,
            Math.round(
              ((((e.clientX - r.left) / r.width) * 300 - 8) / 284) *
                (points.length - 1),
            ),
          ),
        );
        showPoint();
      });
      $("[data-gallery-chart]").addEventListener("pointerleave", () => {
        selected = -1;
        showPoint();
      });
      $("[data-gallery-chart]").addEventListener("keydown", (e) => {
        if (!["ArrowLeft", "ArrowRight", "Home", "End"].includes(e.key)) return;
        e.preventDefault();
        selected =
          e.key === "Home"
            ? 0
            : e.key === "End"
              ? points.length - 1
              : Math.max(
                  0,
                  Math.min(
                    points.length - 1,
                    (selected < 0 ? 0 : selected) +
                      (e.key === "ArrowRight" ? 1 : -1),
                  ),
                );
        showPoint();
      });
      drawChart(false);
      const sortable = S.sortable($("[data-gallery-sort]"), {
        items: "[data-gallery-task]",
        label: (el) => el.querySelector("label span").textContent,
      });
      $$("[data-gallery-task-check]").forEach((input) =>
        input.addEventListener("change", () => {
          $("[data-gallery-task-count]").textContent =
            `${$$("[data-gallery-task-check]:checked").length} / 3`;
        }),
      );
      S.segmented($(".ov-gallery-tabs"), {
        value: "0",
        onChange: (value) => {
          tabAnimations.forEach((a) => a.cancel());
          tabAnimations.clear();
          $$("[data-gallery-panel]").forEach((panel) => {
            const current = panel.dataset.galleryPanel === value;
            panel.hidden = !current;
            if (current && allowed("tabs") && !S.reduced()) {
              const a = panel.animate(
                [
                  { opacity: 0, transform: "translateX(12px)" },
                  { opacity: 1, transform: "none" },
                ],
                { duration: S.dur(320), easing: S.curve.out },
              );
              tabAnimations.add(a);
              a.finished.catch(() => {}).finally(() => tabAnimations.delete(a));
            }
          });
        },
      });
      $$("[data-gallery-icon]").forEach((button) => {
        const update = () => {
          $("[data-gallery-icon-name]").textContent =
            button.dataset.galleryIcon;
        };
        button.addEventListener("pointerenter", update);
        button.addEventListener("focus", update);
        button.addEventListener("click", () =>
          S.playIcon(button.querySelector("svg")),
        );
      });
      const roll = () =>
        S.roll($("[data-gallery-counter]"), String(counter).padStart(3, "0"));
      roll();
      $$("[data-gallery-step]").forEach((button) =>
        button.addEventListener("click", () => {
          counter = Math.min(
            999,
            Math.max(0, counter + +button.dataset.galleryStep),
          );
          roll();
        }),
      );
      $("[data-gallery-counter-reset]").addEventListener("click", () => {
        counter = 24;
        roll();
      });
      const stop = () => {
        if (job) resetAction("Cancelled");
        chartAnimation?.cancel();
        chartAnimation = null;
        areaAnimation?.cancel();
        areaAnimation = null;
        tabAnimations.forEach((a) => a.cancel());
        tabAnimations.clear();
        sortable.setEnabled(false);
        $$("[data-gallery-counter] .roll-strip").forEach((el) =>
          el.getAnimations().forEach((a) => a.cancel()),
        );
      };
      const sync = () => {
        if (!allowed("action") && job) resetAction("Cancelled");
        if (!allowed("chart")) {
          chartStarted = false;
          chartAnimation?.cancel();
          chartAnimation = null;
          areaAnimation?.cancel();
          areaAnimation = null;
        }
        sortable.setEnabled(allowed("sort"));
        if (!allowed("tabs")) {
          tabAnimations.forEach((a) => a.cancel());
          tabAnimations.clear();
        }
        if (!allowed("counter"))
          $$("[data-gallery-counter] .roll-strip").forEach((el) =>
            el.getAnimations().forEach((a) => a.cancel()),
          );
      };
      const observer = new IntersectionObserver(
        (entries) => {
          for (const e of entries) visibility.set(e.target, e.isIntersecting);
          sync();
          if (allowed("chart") && !chartStarted) drawChart();
        },
        { threshold: 0.08 },
      );
      cards.forEach((el) => observer.observe(el));
      document.addEventListener("visibilitychange", () => {
        if (document.hidden) stop();
        else {
          sync();
          if (allowed("chart") && !chartStarted) drawChart();
        }
      });
      const motion = () => {
        if (S.reduced()) {
          chartAnimation?.cancel();
          chartAnimation = null;
          areaAnimation?.cancel();
          areaAnimation = null;
          tabAnimations.forEach((a) => a.cancel());
          tabAnimations.clear();
        }
      };
      new MutationObserver(motion).observe(document.documentElement, {
        attributes: true,
        attributeFilter: ["data-motion"],
      });
      window
        .matchMedia("(prefers-reduced-motion: reduce)")
        .addEventListener("change", motion);
      return {
        enter() {
          chartStarted = false;
          active = true;
          sync();
          drawChart();
        },
        leave() {
          active = false;
          stop();
        },
        sync,
        stop,
      };
    },
  };
})();

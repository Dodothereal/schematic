(() => {
  const S = window.Schematic,
    instances = new WeakMap();
  const icon = (name, size = 16) => S.icon(name, { size });
  const studies = ["Checklist", "Pinned list", "Accordion"];
  const categories = [
    ["all", "Everything"],
    ["examples", "Apps"],
    ["controls", "Controls"],
    ["motion", "Motion"],
    ["data", "Data"],
    ["feedback", "Feedback"],
    ["foundations", "Foundations"],
  ];

  const render = () => {
    const catalog = S.galleryEntries();
    return `<div class="pg-overview">
      <header class="ov-header"><h1 class="ov-title" tabindex="-1"><span class="ov-title-mask"><span class="ov-title-word">Component</span></span> <span class="ov-title-mask ov-title-accent"><span class="ov-title-word">library</span></span></h1><a class="btn btn-secondary btn-sm" href="https://github.com/Dodothereal/schematic" target="_blank" rel="noreferrer">Source ${icon("arrow-up-right", 14)}</a></header>
      <section class="sec ov-browser" id="ov-library" data-toc="Components"><div class="ov-library-tools"><div class="ov-search">${S.icon("search", { size: 18, anim: false })}<input type="search" data-catalog-search placeholder="Search accordion, spring, button…" aria-label="Search the component library" autocomplete="off"><span class="ov-search-end"><kbd class="kbd" aria-hidden="true">/</kbd><button type="button" class="ov-search-reset" data-catalog-reset-query aria-label="Clear search" hidden>${icon("x", 14)}</button></span><span class="ov-search-activity" aria-hidden="true"></span></div><div class="ov-filters" role="group" aria-label="Filter specimens"><span class="ov-filter-indicator" aria-hidden="true"></span>${categories.map(([value, label], i) => `<button type="button" data-catalog-filter="${value}" aria-pressed="${i === 0}">${label}</button>`).join("")}</div></div><div class="ov-result-head"><span class="t-caption" role="status" aria-live="polite" aria-atomic="true" data-catalog-count>${catalog.length} examples</span><button type="button" class="btn btn-ghost btn-sm" data-catalog-clear hidden>Clear filters ${icon("x", 12)}</button></div><div class="ov-results"><div class="ov-catalog">${catalog
        .map((entry) => {
          const preview = S.galleryPreview.render(entry.id);
          return `<article class="ov-specimen-card${preview ? " is-preview" : ""}" ${preview ? "data-has-preview" : ""} data-gallery-card="${entry.id}">${preview}<a class="ov-specimen" href="#/${entry.page}" data-specimen="${entry.index}" data-catalog-id="${entry.id}"><span class="ov-specimen-icon">${icon(entry.icon, 20)}</span><span class="ov-specimen-copy"><span class="t-eyebrow">${entry.collection}</span><strong>${S.esc(entry.title)}</strong><span class="ov-specimen-desc">${S.esc(entry.desc || "")}</span></span><span class="ov-specimen-go">${icon("arrow-up-right", 16)}</span></a></article>`;
        })
        .join(
          "",
        )}</div><div class="ov-empty" data-catalog-empty hidden>${S.icon("search", { size: 24, anim: false })}<h3>No matches.</h3><p>Try another search or clear the filters.</p></div><div class="ov-result-exits" aria-hidden="true" inert></div></div><div class="ov-load-row"><button type="button" class="btn btn-secondary" data-catalog-more>Show more ${icon("plus", 16)}</button></div></section>
      <section class="sec ov-previews" id="ov-preview" data-toc="Preview"><h2 class="t-h2">Preview</h2><div class="ov-workbench" aria-label="Component previews">
          <div class="seg seg-block ov-study-tabs" role="group" aria-label="Choose an interaction study">${studies.map((label, i) => `<button class="seg-btn" data-value="${i}">${label}</button>`).join("")}</div>
          <div class="ov-study-stage">
            <div class="ov-study" data-study="0" data-active aria-hidden="false">
              <div class="ov-study-kicker"><span>Release checklist</span><span data-study-total class="t-mono">1 / 3</span></div>
              <div class="ov-tasks" role="group" aria-label="Release checklist">${["Define tokens", "Build components", "Test keyboard access"].map((label, i) => `<button type="button" class="ov-task" data-study-task aria-pressed="${i === 0}"><span class="ov-task-check" aria-hidden="true">${S.icon("check", { size: 16, anim: false })}</span><span>${label}</span><span class="ov-task-order t-mono">0${i + 1}</span></button>`).join("")}</div>
              <div class="ov-study-meter" aria-hidden="true"><span data-study-meter style="transform:scaleX(.333333)"></span></div>
            </div>
            <div class="ov-study" data-study="1" inert aria-hidden="true"><div class="ov-study-kicker"><span>Project files</span></div><div class="ov-arrange-list">${[
              ["tokens.css", "layers"],
              ["components.js", "document"],
              ["README.md", "document"],
            ]
              .map(
                ([name, ic]) =>
                  `<button type="button" class="ov-arrange-row" data-study-pin aria-pressed="false">${icon(ic)}<span>${name}</span><span class="ov-pin-label">Pin</span>${icon("plus", 14)}</button>`,
              )
              .join("")}</div></div>
            <div class="ov-study" data-study="2" inert aria-hidden="true"><div class="ov-study-kicker"><span>FAQ</span></div>${[
              [
                "Does it work offline?",
                "Yes. Download the HTML file and open it locally.",
              ],
              [
                "How do I reduce animations?",
                "Open Tune tokens and select Reduced under Motion. Your system preference also applies.",
              ],
            ]
              .map(
                ([label, desc], i) =>
                  `<div class="ov-disclosure"><button type="button" data-study-reveal aria-expanded="false" aria-controls="ov-reveal-${i}"><span>${label}</span>${icon("plus", 16)}</button><div class="ov-disclosure-grid" id="ov-reveal-${i}" inert><div><p>${desc}</p></div></div></div>`,
              )
              .join("")}</div>
          </div>
          <div class="ov-workbench-foot"><span class="ov-announcement" role="status" data-study-status></span><button type="button" class="btn btn-ghost btn-icon btn-sm" data-study-reset aria-label="Reset previews">${icon("refresh", 14)}</button></div>
        </div></section>
      <section class="sec" id="ov-map" data-toc="Collections"><h2 class="t-h2">Collections</h2><div class="ov-collection-links">${S.pages
        .filter((p) => p.id !== "overview")
        .map(
          (p) =>
            `<a href="#/${p.id}">${icon(p.icon, 18)}<span>${p.title}</span>${icon("arrow-right", 14)}</a>`,
        )
        .join("")}</div></section>
      <footer class="ov-footer"><button type="button" class="btn btn-ghost btn-sm" data-open-cmdk-ov>${icon("command", 14)} Commands</button><button type="button" class="btn btn-ghost btn-sm" data-browse-library>Back to components ${icon("chevron-down", 14)}</button></footer>
    </div>`;
  };

  // Keep results live while their presentation changes. Read displayed geometry
  // before cancelling a previous transition so rapid input continues in place.
  function catalogMotion(page, state) {
    const $ = (s) => page.querySelector(s),
      cards = [...page.querySelectorAll("[data-gallery-card]")],
      results = $(".ov-results"),
      exits = $(".ov-result-exits"),
      filters = $(".ov-filters"),
      indicator = $(".ov-filter-indicator"),
      motions = new Map(),
      ghosts = new Map();
    let ghostId = 0;
    const enabled = () => state.active && !document.hidden && !S.reduced();
    const cancel = (el) => {
      motions.get(el)?.cancel();
      motions.delete(el);
    };
    const play = (el, frames, duration = 420, delay = 0, easing = S.curve.outExpo) => {
      cancel(el);
      if (!enabled()) return;
      const animation = el.animate(frames, {
        duration: S.dur(duration), delay: S.dur(delay), easing, fill: "both",
      });
      motions.set(el, animation);
      animation.finished.then(() => {
        if (motions.get(el) !== animation) return;
        motions.delete(el);
        animation.cancel();
      }).catch(() => {});
      return animation;
    };
    const dropGhost = (id) => {
      const ghost = ghosts.get(id);
      if (!ghost) return;
      cancel(ghost);
      ghost.remove();
      ghosts.delete(id);
    };
    const snapshot = (el) => ({ rect: el.getBoundingClientRect(), opacity: getComputedStyle(el).opacity });
    function placeIndicator(animate = true) {
      const selected = filters.querySelector('[aria-pressed="true"]');
      if (!selected || !state.active || document.hidden) return;
      const previous = indicator.hasAttribute("data-ready") ? snapshot(indicator) : null;
      cancel(indicator);
      const box = selected.getBoundingClientRect(), parent = filters.getBoundingClientRect();
      const x = box.left - parent.left, y = box.top - parent.top;
      Object.assign(indicator.style, { width: `${box.width}px`, height: `${box.height}px`, transform: `translate(${x}px, ${y}px)` });
      indicator.setAttribute("data-ready", "");
      if (animate && previous) play(indicator, [
        { transform: `translate(${previous.rect.left - parent.left}px, ${previous.rect.top - parent.top}px)`, width: `${previous.rect.width}px`, height: `${previous.rect.height}px` },
        { transform: `translate(${x}px, ${y}px)`, width: `${box.width}px`, height: `${box.height}px` },
      ], 440);
    }
    function exitCard(card, before, origin) {
      const id = card.dataset.galleryCard;
      dropGhost(id);
      const ghost = card.cloneNode(true), prefix = `ov-exit-${++ghostId}-`;
      ghost.hidden = false;
      ghost.querySelector(".ov-specimen").hidden = false;
      // Presentation only: no duplicate control hooks, IDs, focus or live regions.
      [ghost, ...ghost.querySelectorAll("*")].forEach((node) => {
        [...node.attributes].forEach(({ name, value }) => {
          if (name.startsWith("data-") || ["aria-live", "aria-controls", "aria-labelledby", "aria-describedby", "name", "for", "href", "tabindex", "role"].includes(name)) node.removeAttribute(name);
          if (name === "id") node.id = prefix + value;
          if (value.includes("url(#")) node.setAttribute(name, value.replaceAll("url(#", `url(#${prefix}`));
        });
      });
      ghost.inert = true;
      ghost.setAttribute("aria-hidden", "true");
      ghost.classList.add("ov-result-ghost");
      Object.assign(ghost.style, {
        position: "absolute", left: `${before.rect.left - origin.left}px`, top: `${before.rect.top - origin.top}px`,
        width: `${before.rect.width}px`, height: `${before.rect.height}px`, margin: "0",
      });
      exits.append(ghost);
      ghosts.set(id, ghost);
      const animation = play(ghost, [
        { opacity: before.opacity, transform: "none" },
        { opacity: 0, transform: "translateY(-6px)" },
      ], 110, 0, S.curve.in);
      animation?.finished.then(() => { if (ghosts.get(id) === ghost) dropGhost(id); }).catch(() => {});
    }
    function update(mutate, { typing = false } = {}) {
      const animate = enabled(), previous = new Map();
      const origin = results.getBoundingClientRect(), oldCount = $("[data-catalog-count]").textContent;
      const queryResetWasHidden = $("[data-catalog-reset-query]").hidden;
      const clearWasHidden = $("[data-catalog-clear]").hidden;
      if (animate) {
        cards.filter((card) => !card.hidden).forEach((card) => previous.set(card.dataset.galleryCard, snapshot(card)));
        ghosts.forEach((ghost, id) => { if (!previous.has(id)) previous.set(id, snapshot(ghost)); });
      }
      cards.forEach(cancel);
      cancel(results);
      mutate();
      placeIndicator();
      if (!animate) return;
      const nextOrigin = results.getBoundingClientRect();
      const visible = cards.filter((card) => !card.hidden);
      const visibleIds = new Set(visible.map((card) => card.dataset.galleryCard));
      // Old cards remain as inert paint for the short exit, outside grid layout.
      cards.forEach((card) => {
        const id = card.dataset.galleryCard, before = previous.get(id);
        if (card.hidden && before && !ghosts.has(id)) exitCard(card, before, nextOrigin);
      });
      let entering = 0;
      visible.forEach((card) => {
        const id = card.dataset.galleryCard, before = previous.get(id), after = card.getBoundingClientRect();
        dropGhost(id);
        if (before) {
          const dx = before.rect.left - after.left, dy = before.rect.top - after.top;
          if (Math.abs(dx) > 0.5 || Math.abs(dy) > 0.5 || Number(before.opacity) < 0.999) play(card, [
            { transform: `translate(${dx}px, ${dy}px)`, opacity: before.opacity },
            { transform: "none", opacity: 1 },
          ], 420, 65);
        } else {
          // Bound the stagger; a long result list must never queue seconds of motion.
          play(card, [{ opacity: 0, transform: "translateY(10px)" }, { opacity: 1, transform: "none" }], 380, 85 + Math.min(entering++, 5) * 22);
        }
      });
      // Nothing below the results jumps when filters change the number of rows.
      if (Math.abs(origin.height - nextOrigin.height) > 0.5) play(results, [
        { height: `${origin.height}px` }, { height: `${nextOrigin.height}px` },
      ], 440);
      if (oldCount !== $("[data-catalog-count]").textContent) play($("[data-catalog-count]"), [
        { opacity: 0, transform: "translateY(5px)" }, { opacity: 1, transform: "none" },
      ], 280);
      if (queryResetWasHidden && !$("[data-catalog-reset-query]").hidden) play($("[data-catalog-reset-query]"), [
        { opacity: 0, transform: "rotate(-20deg)" }, { opacity: 1, transform: "none" },
      ], 260);
      if (clearWasHidden && !$("[data-catalog-clear]").hidden) play($("[data-catalog-clear]"), [
        { opacity: 0, transform: "translateX(4px)" }, { opacity: 1, transform: "none" },
      ], 280);
      if (!$("[data-catalog-empty]").hidden && !visibleIds.size) play($("[data-catalog-empty]"), [
        { opacity: 0, transform: "translateY(8px)" }, { opacity: 1, transform: "none" },
      ], 360, 60);
      if (typing) play($(".ov-search-activity"), [
        { opacity: 0.75, transform: "scaleX(0)" },
        { opacity: 0.75, transform: "scaleX(1)", offset: 0.7 },
        { opacity: 0, transform: "scaleX(1)" },
      ], 520, 0, S.curve.inOut);
    }
    const stop = () => {
      motions.forEach((animation) => animation.cancel());
      motions.clear();
      [...ghosts.keys()].forEach(dropGhost);
    };
    const enter = () => {
      placeIndicator(false);
      if (!enabled()) return;
      const viewport = document.querySelector("#viewport").getBoundingClientRect();
      if ($(".ov-header").getBoundingClientRect().bottom < viewport.top) return;
      [...page.querySelectorAll(".ov-title-word")].forEach((word, i) => play(word, [
        { transform: "translateY(105%)", opacity: 0 }, { transform: "none", opacity: 1 },
      ], 620, i * 65));
      [$(".ov-header > a"), $(".ov-search"), filters].forEach((el, i) => play(el, [
        { opacity: 0, transform: "translateY(8px)" }, { opacity: 1, transform: "none" },
      ], 460, 80 + i * 45));
    };
    let filterSize;
    new ResizeObserver(([entry]) => {
      if (!state.active || !entry.contentRect.width || !entry.contentRect.height) return;
      const next = `${entry.contentRect.width}:${entry.contentRect.height}`;
      if (filterSize && next !== filterSize) stop();
      filterSize = next;
      placeIndicator(false);
    }).observe(filters);
    new MutationObserver(() => { if (S.reduced()) { stop(); placeIndicator(false); } }).observe(document.documentElement, { attributes: true, attributeFilter: ["data-motion", "style"] });
    matchMedia("(prefers-reduced-motion: reduce)").addEventListener("change", () => { if (S.reduced()) { stop(); placeIndicator(false); } });
    return { update, enter, stop };
  }

  function mount(root) {
    const page = root.querySelector(".pg-overview"),
      $ = (s) => page.querySelector(s),
      $$ = (s) => [...page.querySelectorAll(s)];
    const state = {
      active: false,
      animations: new Set(),
      category: "all",
      limit: 12,
      study: 0,
    };
    instances.set(root, state);
    const animate = (el, frames, duration = 320) => {
      if (S.reduced() || !state.active) return;
      const a = el.animate(frames, {
        duration: S.dur(duration),
        easing: S.curve.out,
      });
      state.animations.add(a);
      a.finished.catch(() => {}).finally(() => state.animations.delete(a));
    };
    const stop = () => {
      state.animations.forEach((a) => a.cancel());
      state.animations.clear();
      $$(".ov-arrange-row").forEach((el) =>
        el.getAnimations().forEach((a) => a.cancel()),
      );
    };
    state.gallery = S.galleryPreview.mount(page);
    state.catalogMotion = catalogMotion(page, state);
    state.stop = () => {
      stop();
      state.gallery.stop();
      state.catalogMotion.stop();
    };
    $("[data-open-cmdk-ov]").addEventListener("click", () => S.cmdk.open());
    $("[data-browse-library]").addEventListener("click", () => {
      const v = document.querySelector("#viewport"),
        t = $("#ov-library");
      S.scrollTo(
        v.scrollTop +
          t.getBoundingClientRect().top -
          v.getBoundingClientRect().top -
          32,
      );
      $("[data-catalog-search]").focus({ preventScroll: true });
    });
    S.segmented($(".ov-study-tabs"), {
      value: "0",
      onChange: (value) => {
        state.study = Number(value);
        stop();
        $$(".ov-study").forEach((panel, i) => {
          const active = i === state.study;
          panel.toggleAttribute("data-active", active);
          panel.inert = !active;
          panel.setAttribute("aria-hidden", String(!active));
          if (active)
            animate(panel, [
              { opacity: 0, transform: "translateY(8px)" },
              { opacity: 1, transform: "none" },
            ]);
        });
        $("[data-study-status]").textContent = [
          "Checklist selected.",
          "Pinned list selected.",
          "Accordion selected.",
        ][state.study];
      },
    });
    const updateTasks = () => {
      const count = $$("[data-study-task][aria-pressed=true]").length;
      $("[data-study-total]").textContent = `${count} / 3`;
      $("[data-study-meter]").style.transform = `scaleX(${count / 3})`;
      $("[data-study-status]").textContent = `${count} of 3 tasks completed.`;
    };
    $$("[data-study-task]").forEach((button) =>
      button.addEventListener("click", () => {
        button.setAttribute(
          "aria-pressed",
          String(button.getAttribute("aria-pressed") !== "true"),
        );
        updateTasks();
      }),
    );
    const originalRows = $$("[data-study-pin]");
    originalRows.forEach((button) =>
      button.addEventListener("click", () => {
        const selected = button.getAttribute("aria-pressed") !== "true";
        button.setAttribute("aria-pressed", String(selected));
        button.querySelector(".ov-pin-label").textContent = selected
          ? "Pinned"
          : "Pin";
        S.flip(
          () => $$(".ov-arrange-row"),
          () => {
            originalRows
              .filter((r) => r.getAttribute("aria-pressed") === "true")
              .concat(
                originalRows.filter(
                  (r) => r.getAttribute("aria-pressed") !== "true",
                ),
              )
              .forEach((row) => $(".ov-arrange-list").append(row));
          },
        );
        $("[data-study-status]").textContent = selected
          ? "File pinned."
          : "File unpinned.";
      }),
    );
    $$("[data-study-reveal]").forEach((button) =>
      button.addEventListener("click", () => {
        const expanded = button.getAttribute("aria-expanded") !== "true";
        button.setAttribute("aria-expanded", String(expanded));
        button.parentElement.toggleAttribute("data-open", expanded);
        button.nextElementSibling.inert = !expanded;
        $("[data-study-status]").textContent = expanded
          ? "Expanded."
          : "Collapsed.";
      }),
    );
    $("[data-study-reset]").addEventListener("click", () => {
      stop();
      $$("[data-study-task]").forEach((el, i) =>
        el.setAttribute("aria-pressed", String(i === 0)),
      );
      updateTasks();
      originalRows.forEach((el) => {
        el.setAttribute("aria-pressed", "false");
        el.querySelector(".ov-pin-label").textContent = "Pin";
      });
      S.flip(
        () => $$(".ov-arrange-row"),
        () => originalRows.forEach((row) => $(".ov-arrange-list").append(row)),
      );
      $$("[data-study-reveal]").forEach((el) => {
        el.setAttribute("aria-expanded", "false");
        el.parentElement.removeAttribute("data-open");
        el.nextElementSibling.inert = true;
      });
      $("[data-study-status]").textContent = "Previews reset.";
    });
    const entries = S.galleryEntries(),
      elements = new Map(
        $$("[data-catalog-id]").map((el) => [el.dataset.catalogId, el]),
      );
    function filter(options) {
      const query = $("[data-catalog-search]").value.trim().toLocaleLowerCase();
      const matches = entries.filter(
        (e) =>
          (state.category === "all" || e.family === state.category) &&
          `${e.title} ${e.desc} ${e.collection}`
            .toLocaleLowerCase()
            .includes(query),
      );
      const visible = new Set(matches.slice(0, state.limit).map((e) => e.id));
      state.catalogMotion.update(() => {
        elements.forEach((el, id) => {
          el.hidden = !visible.has(id);
          el.closest(".ov-specimen-card").hidden = el.hidden;
        });
        $("[data-catalog-count]").textContent =
          `${matches.length} ${matches.length === 1 ? "example" : "examples"}`;
        $("[data-catalog-empty]").hidden = matches.length !== 0;
        $("[data-catalog-more]").hidden = matches.length <= state.limit;
        $("[data-catalog-clear]").hidden = !query && state.category === "all";
        const hasQuery = $("[data-catalog-search]").value.length > 0;
        $(".ov-search").toggleAttribute("data-has-query", hasQuery);
        $("[data-catalog-reset-query]").hidden = !hasQuery;
        $$("[data-catalog-filter]").forEach((el) =>
          el.setAttribute("aria-pressed", String(el.dataset.catalogFilter === state.category)),
        );
      }, options);
      state.gallery.sync();
    }
    $("[data-catalog-search]").addEventListener("input", () => {
      state.limit = 12;
      filter({ typing: true });
    });
    $$("[data-catalog-filter]").forEach((button) =>
      button.addEventListener("click", () => {
        state.category = button.dataset.catalogFilter;
        state.limit = 12;
        filter();
      }),
    );
    $(".ov-search").addEventListener("click", (e) => {
      if (!e.target.closest("button,input")) $("[data-catalog-search]").focus();
    });
    $("[data-catalog-reset-query]").addEventListener("click", () => {
      $("[data-catalog-search]").value = "";
      state.limit = 12;
      filter({ typing: true });
      $("[data-catalog-search]").focus();
    });
    $("[data-catalog-clear]").addEventListener("click", () => {
      state.category = "all";
      state.limit = 12;
      $("[data-catalog-search]").value = "";
      filter();
      $("[data-catalog-search]").focus();
    });
    $("[data-catalog-more]").addEventListener("click", () => {
      state.limit += 24;
      filter();
    });
    filter();
    document.addEventListener("keydown", (e) => {
      if (
        !state.active ||
        e.key !== "/" ||
        e.ctrlKey ||
        e.metaKey ||
        e.altKey ||
        e.target.closest("input,textarea,select,[contenteditable=true],dialog")
      )
        return;
      e.preventDefault();
      $("[data-browse-library]").click();
    });
    document.addEventListener("visibilitychange", () => {
      if (document.hidden) state.stop();
    });
  }
  S.page({
    id: "overview",
    title: "Overview",
    group: "Start",
    icon: "home",
    order: 0,
    description: "A working collection of interfaces, interactions and motion.",
    render,
    mount,
    enter(root) {
      const state = instances.get(root);
      if (state) {
        state.active = true;
        state.catalogMotion.enter();
        state.gallery.enter();
      }
    },
    leave(root) {
      const state = instances.get(root);
      if (state) {
        state.active = false;
        state.gallery.leave();
        state.stop();
      }
    },
  });
})();

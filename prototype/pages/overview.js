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
    const catalog = S.catalog();
    return `<div class="pg-overview">
      ${S.assembly.render()}
      <section class="sec ov-browser" id="ov-library" data-toc="Components"><div class="ov-library-tools"><label class="ov-search">${icon("search", 18)}<input type="search" data-catalog-search placeholder="Search accordion, spring, button…" aria-label="Search the component library" autocomplete="off"><kbd class="kbd" aria-hidden="true">/</kbd></label><div class="ov-filters" role="group" aria-label="Filter specimens">${categories.map(([value, label], i) => `<button type="button" data-catalog-filter="${value}" aria-pressed="${i === 0}">${label}</button>`).join("")}</div></div><div class="ov-result-head"><span class="t-caption" role="status" data-catalog-count>${catalog.length} examples</span><button type="button" class="btn btn-ghost btn-sm" data-catalog-clear hidden>Clear filters ${icon("x", 12)}</button></div><div class="ov-catalog">${catalog.map((entry) => `<a class="ov-specimen" href="#/${entry.page}" data-specimen="${entry.index}" data-catalog-id="${entry.id}"><span class="ov-specimen-icon">${icon(entry.icon, 20)}</span><span class="ov-specimen-copy"><span class="t-eyebrow">${entry.collection}</span><strong>${S.esc(entry.title)}</strong><span class="ov-specimen-desc">${S.esc(entry.desc || "")}</span></span><span class="ov-specimen-go">${icon("arrow-up-right", 16)}</span></a>`).join("")}</div><div class="ov-empty" data-catalog-empty hidden>${icon("search", 24)}<h3>No matches.</h3><p>Try another search or clear the filters.</p></div><div class="ov-load-row"><button type="button" class="btn btn-secondary" data-catalog-more>Show more ${icon("plus", 16)}</button></div></section>
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
    state.stop = stop;
    state.assembly = S.assembly.mount(root, () =>
      $("[data-browse-library]").click(),
    );
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
    const entries = S.catalog(),
      elements = new Map(
        $$("[data-catalog-id]").map((el) => [el.dataset.catalogId, el]),
      );
    function filter() {
      const query = $("[data-catalog-search]").value.trim().toLocaleLowerCase();
      const matches = entries.filter(
        (e) =>
          (state.category === "all" || e.family === state.category) &&
          `${e.title} ${e.desc} ${e.collection}`
            .toLocaleLowerCase()
            .includes(query),
      );
      const visible = new Set(matches.slice(0, state.limit).map((e) => e.id));
      elements.forEach((el, id) => (el.hidden = !visible.has(id)));
      $("[data-catalog-count]").textContent =
        `${matches.length} ${matches.length === 1 ? "example" : "examples"}`;
      $("[data-catalog-empty]").hidden = matches.length !== 0;
      $("[data-catalog-more]").hidden = matches.length <= state.limit;
      $("[data-catalog-clear]").hidden = !query && state.category === "all";
      $$("[data-catalog-filter]").forEach((el) =>
        el.setAttribute(
          "aria-pressed",
          String(el.dataset.catalogFilter === state.category),
        ),
      );
    }
    $("[data-catalog-search]").addEventListener("input", () => {
      state.limit = 12;
      filter();
    });
    $$("[data-catalog-filter]").forEach((button) =>
      button.addEventListener("click", () => {
        state.category = button.dataset.catalogFilter;
        state.limit = 12;
        filter();
      }),
    );
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
      if (document.hidden) stop();
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
        state.assembly.enter();
      }
    },
    leave(root) {
      const state = instances.get(root);
      if (state) {
        state.active = false;
        state.assembly.leave();
        state.stop();
      }
    },
  });
})();

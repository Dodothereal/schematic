(() => {
  const S = window.Schematic,
    instances = new WeakMap();
  const icon = (name, size = 16) => S.icon(name, { size });
  const studies = ["Select", "Arrange", "Reveal"];
  const collections = [
    [
      "primitives",
      "01",
      "Little things.\nA different feeling.",
      "Disclosure, reordering, type and surfaces that respond.",
      "layers",
    ],
    [
      "navigation",
      "02",
      "A clear way\nthrough the work.",
      "Menus, trees, pagination, flows and moving selections.",
      "layers",
    ],
    [
      "components",
      "03",
      "Every action,\nfollowed through.",
      "From the first press to pending, success and retry.",
      "cube",
    ],
  ];
  const categories = [
    ["all", "Everything"],
    ["controls", "Controls"],
    ["motion", "Motion"],
    ["data", "Data"],
    ["feedback", "Feedback"],
    ["foundations", "Foundations"],
  ];

  const render = () => {
    const catalog = S.catalog();
    return `<div class="pg-overview">
      <section class="ov-hero">
        <div class="ov-hero-inner reveal">
          <div class="ov-edition t-eyebrow" style="--i:0"><span class="ov-edition-line"></span> Schematic / Interface library <span class="t-mono">002</span></div>
          <h1 class="t-display ov-title" style="--i:1">Interfaces,<br><span class="ov-title-accent">with feeling.</span></h1>
          <p class="t-lead ov-lead" style="--i:2">The little details make the whole experience. Explore a growing collection of controls, patterns and motion. Touch everything.</p>
          <div class="row ov-cta" style="--i:3"><a class="btn btn-primary btn-lg" href="#/primitives">Explore the new collection ${icon("arrow-right")}</a><button class="btn btn-ghost btn-lg" data-browse-library>Browse the library ${icon("arrow-right")}</button></div>
          <div class="ov-hero-notes" style="--i:4"><span>${icon("check-circle", 14)} Keyboard considered</span><span>${icon("wave", 14)} Motion you can tune</span><span>${icon("download", 14)} Entirely offline</span></div>
        </div>
        <div class="ov-workbench" aria-label="Interactive motion studies">
          <div class="ov-workbench-head"><span class="t-eyebrow">The feeling is in the response</span><span class="ov-live t-mono"><i></i> LIVE</span></div>
          <div class="seg seg-block ov-study-tabs" role="group" aria-label="Choose an interaction study">${studies.map((label, i) => `<button class="seg-btn" data-value="${i}">${label}</button>`).join("")}</div>
          <div class="ov-study-stage">
            <div class="ov-study" data-study="0" data-active aria-hidden="false">
              <div class="ov-study-kicker"><span class="t-mono">01 / SELECTION</span><span data-study-total class="t-mono">1 / 3</span></div>
              <h2>A little more considered.</h2><div class="ov-tasks" role="group" aria-label="Considered interface details">${["A quiet surface", "A precise response", "Room to explore"].map((label, i) => `<button type="button" class="ov-task" data-study-task aria-pressed="${i === 0}"><span class="ov-task-check" aria-hidden="true"><svg viewBox="0 0 20 20"><path d="m5 10 3.2 3.2L15 6.5"/></svg></span><span>${label}</span><span class="ov-task-order t-mono">0${i + 1}</span></button>`).join("")}</div>
              <div class="ov-study-meter" aria-hidden="true"><span data-study-meter style="transform:scaleX(.333333)"></span></div>
            </div>
            <div class="ov-study" data-study="1" inert aria-hidden="true"><div class="ov-study-kicker t-mono"><span>02 / SHARED SPACE</span><span>PIN TO PRIORITISE</span></div><h2>Make room for what matters.</h2><div class="ov-arrange-list">${[
              ["Design language", "layers"],
              ["Interaction notes", "document"],
              ["Next release", "bolt"],
            ]
              .map(
                ([name, ic]) =>
                  `<button type="button" class="ov-arrange-row" data-study-pin aria-pressed="false">${icon(ic)}<span>${name}</span><span class="ov-pin-label">Pin</span>${icon("plus", 14)}</button>`,
              )
              .join("")}</div></div>
            <div class="ov-study" data-study="2" inert aria-hidden="true"><div class="ov-study-kicker t-mono"><span>03 / DISCLOSURE</span><span>SPACE, REVEALED</span></div><h2>Only what you need. For now.</h2>${[
              [
                "One language, every detail",
                "Shared tokens connect the smallest control to the whole interface. A quiet surface, a clear label, a precise response.",
              ],
              [
                "Motion with a purpose",
                "See what changed, where something went and what happens next. Interrupt any transition. The next action is always yours.",
              ],
            ]
              .map(
                ([label, desc], i) =>
                  `<div class="ov-disclosure"><button type="button" data-study-reveal aria-expanded="false" aria-controls="ov-reveal-${i}"><span>${label}</span>${icon("plus", 16)}</button><div class="ov-disclosure-grid" id="ov-reveal-${i}" inert><div><p>${desc}</p></div></div></div>`,
              )
              .join("")}</div>
          </div>
          <div class="ov-workbench-foot"><span role="status" data-study-status>Try selecting a detail.</span><button type="button" class="btn btn-ghost btn-icon btn-sm" data-study-reset aria-label="Reset the motion studies">${icon("refresh", 14)}</button></div>
        </div>
      </section>
      <div class="ov-ledger" aria-label="Library at a glance"><span><strong class="t-num">${catalog.length}</strong> working specimens</span><span><strong class="t-num">${S.pages.length}</strong> collections</span><span><strong class="t-num">${S.iconNames.length}</strong> animated icons</span><span><strong class="t-num">0</strong> dependencies</span><span class="ov-ledger-note t-mono">BUILT TO BE TAKEN APART ${icon("arrow-right", 14)}</span></div>
      ${S.section({ id: "ov-new", num: "01 / NEW IN THE COLLECTION", title: "More to work with.", desc: "New ways to reveal, navigate and make a decision. One consistent language underneath.", body: `<div class="ov-collections">${collections.map(([id, num, title, desc, ic]) => `<a class="ov-collection" href="#/${id}"><div class="ov-collection-top"><span class="t-mono">${num} / ${id.toUpperCase()}</span>${icon(ic, 22)}</div><h3>${title.replace("\n", "<br>")}</h3><p>${desc}</p><span class="ov-collection-go">Explore collection ${icon("arrow-right", 16)}</span></a>`).join("")}</div>` })}
      ${S.section({ id: "ov-library", num: "02 / THE WORKING LIBRARY", title: "Find the right detail.", desc: "Search the actual specimens. Every result opens the working example, right where it lives.", body: `<div class="ov-library-tools"><label class="ov-search">${icon("search", 18)}<input type="search" data-catalog-search placeholder="Search accordion, spring, button…" aria-label="Search the component library" autocomplete="off"><kbd class="kbd" aria-hidden="true">/</kbd></label><div class="ov-filters" role="group" aria-label="Filter specimens">${categories.map(([value, label], i) => `<button type="button" data-catalog-filter="${value}" aria-pressed="${i === 0}">${label}</button>`).join("")}</div></div><div class="ov-result-head"><span class="t-caption" role="status" data-catalog-count>${catalog.length} specimens to explore</span><button type="button" class="btn btn-ghost btn-sm" data-catalog-clear hidden>Clear filters ${icon("x", 12)}</button></div><div class="ov-catalog">${catalog.map((entry) => `<a class="ov-specimen" href="#/${entry.page}" data-specimen="${entry.index}" data-catalog-id="${entry.id}"><span class="ov-specimen-icon">${icon(entry.icon, 20)}</span><span class="ov-specimen-copy"><span class="t-eyebrow">${entry.collection}</span><strong>${S.esc(entry.title)}</strong><span class="ov-specimen-desc">${S.esc(entry.desc || "")}</span></span><span class="ov-specimen-go">${icon("arrow-up-right", 16)}</span></a>`).join("")}</div><div class="ov-empty" data-catalog-empty hidden>${icon("search", 24)}<h3>No detail matches that search.</h3><p>Try a component name or clear the filters to explore everything.</p></div><div class="ov-load-row"><button type="button" class="btn btn-secondary" data-catalog-more>Show more specimens ${icon("plus", 16)}</button></div>` })}
      ${S.section({
        id: "ov-map",
        num: "03 / EXPLORE THE SYSTEM",
        title: "The whole picture.",
        desc: "From the decisions underneath to the finished interaction.",
        body: `<div class="ov-system">${S.pages
          .filter((p) => p.id !== "overview")
          .map(
            (p, i) =>
              `<a class="ov-system-row" href="#/${p.id}" data-shared><span class="t-mono ov-system-number">${String(i + 1).padStart(2, "0")}</span><span class="shared-ic">${icon(p.icon, 20)}</span><strong>${p.title}</strong><span class="ov-system-desc">${p.description || ""}</span>${icon("arrow-right", 16)}</a>`,
          )
          .join("")}</div>`,
      })}
      <footer class="ov-end"><div><span class="t-eyebrow">A reference. A starting point.</span><h2>Make something<br>feel considered.</h2></div><div><p>Take the tokens, the interaction and the details you need. The working HTML reference is ready to explore. Framework adapters are on the roadmap.</p><div class="row"><a class="btn btn-secondary" href="https://github.com/Dodothereal/schematic" target="_blank" rel="noreferrer">Get the source ${icon("arrow-up-right", 16)}</a><button type="button" class="btn btn-ghost" data-open-cmdk-ov>${icon("command", 16)} Find a page</button></div></div></footer>
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
          "Try selecting a detail.",
          "Pin a row and watch its neighbours make room.",
          "Reveal a little more context.",
        ][state.study];
      },
    });
    const updateTasks = () => {
      const count = $$("[data-study-task][aria-pressed=true]").length;
      $("[data-study-total]").textContent = `${count} / 3`;
      $("[data-study-meter]").style.transform = `scaleX(${count / 3})`;
      $("[data-study-status]").textContent =
        `${count} of 3 details considered.`;
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
          ? "Priority pinned. The list makes room."
          : "Priority released.";
      }),
    );
    $$("[data-study-reveal]").forEach((button) =>
      button.addEventListener("click", () => {
        const expanded = button.getAttribute("aria-expanded") !== "true";
        button.setAttribute("aria-expanded", String(expanded));
        button.parentElement.toggleAttribute("data-open", expanded);
        button.nextElementSibling.inert = !expanded;
        $("[data-study-status]").textContent = expanded
          ? "A little more context, in place."
          : "Back to the essentials.";
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
      $("[data-study-status]").textContent = "Ready to explore again.";
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
        `${matches.length} ${matches.length === 1 ? "specimen" : "specimens"}${query ? " found" : " to explore"}`;
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
      if (state) state.active = true;
    },
    leave(root) {
      const state = instances.get(root);
      if (state) {
        state.active = false;
        state.stop();
      }
    },
  });
})();

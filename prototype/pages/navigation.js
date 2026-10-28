(() => {
  "use strict";
  const S = window.Schematic;
  const instances = new WeakMap();
  const ic = (name, size = 16) => S.icon(name, { size });
  const pageIcon = S.iconNames.includes("compass") ? "compass" : "layers";
  const demo = (title, desc, body, specs) =>
    S.demo({ title, desc, body, specs, stage: "is-col is-plain" });
  const status = (name, text) =>
    `<p class="nv-status" data-nv-${name}-status role="status" aria-live="polite" aria-atomic="true">${text}</p>`;
  const pane = (key, body, active = false, attrs = "") =>
    `<div class="nv-pane" data-nv-pane="${key}" data-nv-active="${active}" aria-hidden="${!active}" ${active ? "" : "inert"} ${attrs}>${body}</div>`;
  const collections = [
    ["Foundations", "Colour, type & spacing", "layers", "12 references"],
    ["Motion studies", "Curves with a purpose", "wave", "8 references"],
    ["Navigation", "Menus, trees and flows", "grid", "8 references"],
    ["Field notes", "Thoughts from the process", "document", "6 references"],
    ["Interface patterns", "Buttons, forms and dialogs", "cube", "16 references"],
    ["Team rituals", "Team processes", "users", "4 references"],
  ];
  const menuData = [
    {
      label: "Library",
      title: "Library",
      kicker: "YOUR COLLECTION",
      icon: "layers",
      items: [
        ["Foundations", "Colour, type and space", "layers"],
        ["Motion studies", "Timing with intention", "wave"],
      ],
    },
    {
      label: "Workspace",
      title: "Workspace",
      kicker: "SHARED SPACE",
      icon: "users",
      items: [
        ["Project board", "The work in progress", "grid"],
        ["Team directory", "People behind the work", "users"],
      ],
    },
    {
      label: "Resources",
      title: "Resources",
      kicker: "GOOD TO HAVE",
      icon: "document",
      items: [
        ["Field guide", "Principles for the everyday", "document"],
        ["Release notes", "What changed, and why", "clock"],
      ],
    },
  ];
  const records = [
    ["Navigation study", "Exploration", "Today"],
    ["Colour in context", "Reference", "Today"],
    ["Toolbar keyboard shortcuts", "Prototype", "Yesterday"],
    ["Spacing that breathes", "Reference", "Oct 24"],
    ["The first five minutes", "Research", "Oct 23"],
    ["Keyboard paths", "Checklist", "Oct 22"],
    ["A measured spring", "Exploration", "Oct 21"],
    ["Naming the layers", "Reference", "Oct 20"],
    ["The empty canvas", "Prototype", "Oct 19"],
    ["Small confirmations", "Exploration", "Oct 18"],
    ["An honest loading state", "Prototype", "Oct 17"],
    ["A better way back", "Research", "Oct 16"],
  ];
  const folders = {
    workspace: {
      label: "Workspace",
      parent: null,
      children: ["projects", "reference"],
    },
    projects: {
      label: "Projects",
      parent: "workspace",
      children: ["fieldwork", "atlas"],
    },
    fieldwork: {
      label: "Fieldwork",
      parent: "projects",
      children: ["research", "prototype"],
    },
    atlas: { label: "Atlas", parent: "projects", children: [] },
    research: { label: "Research", parent: "fieldwork", children: [] },
    prototype: { label: "Prototype", parent: "fieldwork", children: [] },
    reference: {
      label: "Reference",
      parent: "workspace",
      children: ["principles"],
    },
    principles: { label: "Principles", parent: "reference", children: [] },
  };
  const tree = [
    {
      id: "source",
      name: "source",
      children: [
        {
          id: "components",
          name: "components",
          children: [
            {
              id: "navigation-file",
              name: "navigation.js",
              detail: "8 working specimens · JavaScript",
            },
            {
              id: "button-file",
              name: "button.js",
              detail: "Actions & feedback · JavaScript",
            },
          ],
        },
        {
          id: "tokens-file",
          name: "tokens.css",
          detail: "Semantic design tokens · CSS",
        },
      ],
    },
    {
      id: "notes",
      name: "notes",
      children: [
        {
          id: "readme-file",
          name: "README.md",
          detail: "A place to begin · Markdown",
        },
        {
          id: "decisions-file",
          name: "decisions.md",
          detail: "The why behind the work · Markdown",
        },
      ],
    },
  ];
  const treeMarkup = (nodes, level = 1, parent = "") =>
    nodes
      .map(
        (n, i) =>
          `<li role="treeitem" tabindex="${n.id === "source" ? 0 : -1}" data-nv-node="${n.id}" data-nv-parent="${parent}" aria-level="${level}" aria-posinset="${i + 1}" aria-setsize="${nodes.length}" aria-selected="false" aria-labelledby="nv-tree-${n.id}" ${n.children ? `aria-expanded="${n.id === "source"}"` : ""}><span class="nv-tree-row"><span class="nv-tree-chevron" aria-hidden="true">${n.children ? ic("chevron-right", 12) : ""}</span>${ic(n.children ? "folder" : "document")}<span id="nv-tree-${n.id}">${n.name}</span><span class="nv-tree-end" aria-hidden="true">${n.children ? n.children.length : ""}</span></span>${n.children ? `<div class="nv-tree-children" data-nv-tree-children ${n.id === "source" ? "" : 'inert aria-hidden="true"'}><ul role="group">${treeMarkup(n.children, level + 1, n.id)}</ul></div>` : ""}</li>`,
      )
      .join("");
  const directions = [
    {
      name: "Editorial",
      subtitle: "Component library",
      desc: "Tokens, controls and documentation.",
      type: "editorial",
      tags: "TYPE / RHYTHM / SPACE",
    },
    {
      name: "Structural",
      subtitle: "Project dashboard",
      desc: "Track tasks, reviews and releases.",
      type: "structural",
      tags: "GRID / CONNECTION / ORDER",
    },
    {
      name: "Essential",
      subtitle: "Admin dashboard",
      desc: "Tables, filters and account settings.",
      type: "essential",
      tags: "FOCUS / CLARITY / INTENT",
    },
  ];
  const render = () => `<div class="pg-navigation">
    ${S.pageHead({ icon: pageIcon, eyebrow: "Build / Paths & places", title: "Navigation", lead: "Menus, search, trees and multi-step flows.", meta: [] })}
    ${S.section({
      id: "nv-menus",
      num: "01",
      title: "Menus and search",
      body: `<div class="grid cols-2">
      ${demo("Travelling navigation menu", "Switch destinations inside one continuous viewport. Escape returns to the trigger; a click outside closes it.", `<div class="nv-menu" data-nv-menu data-nv-open="false"><nav class="nv-menu-bar" aria-label="Library destinations" data-nv-menu-bar><span class="nv-menu-marker" data-nv-menu-marker aria-hidden="true"></span>${menuData.map((m, i) => `<button type="button" data-nv-menu-trigger="${i}" aria-expanded="false" aria-controls="nv-menu-panel-${i}">${m.label}${ic("chevron-down", 12)}</button>`).join("")}</nav><div class="nv-menu-space"><div class="nv-menu-hint"><span class="nv-orbit" aria-hidden="true">${ic("layers", 24)}</span><strong>Where shall we go?</strong><span>Choose a destination above.</span></div><div class="nv-menu-viewport" data-nv-menu-viewport inert aria-hidden="true"><span class="nv-menu-pointer" aria-hidden="true"></span><div class="nv-stack" data-nv-menu-stack>${menuData.map((m, i) => pane(String(i), `<div class="nv-menu-heading"><span class="t-eyebrow">${m.kicker}</span><strong>${m.title}</strong></div><div class="nv-menu-links">${m.items.map(([name, desc, icon]) => `<button type="button" data-nv-destination="${name}">${ic(icon, 18)}<span><strong>${name}</strong><small>${desc}</small></span><span class="nv-arrow">${ic("arrow-right", 14)}</span></button>`).join("")}</div>`, i === 0, `id="nv-menu-panel-${i}" aria-label="${m.label}"`)).join("")}</div></div></div></div>${status("menu", "Choose Library, Workspace or Resources.")}`, ["shared viewport", "arrows / escape"])}
      ${demo("Searchable collection picker", "Filter real collections. Arrow keys explore matches; Enter chooses. Your input keeps keyboard focus.", `<div class="nv-combo" data-nv-combo><label class="nv-label" for="nv-collection">Find a collection <span>6 available</span></label><div class="nv-combo-input">${ic("search")}<input id="nv-collection" data-nv-combo-input type="text" role="combobox" autocomplete="off" spellcheck="false" aria-autocomplete="list" aria-expanded="false" aria-controls="nv-collection-list" aria-describedby="nv-combo-help" placeholder="Search collections…"><span aria-hidden="true">${ic("chevrons-up-down", 14)}</span></div><div class="nv-combo-slot"><div class="nv-combo-rest" data-nv-combo-rest><span class="nv-orbit" aria-hidden="true">${ic("layers", 24)}</span><span class="t-eyebrow">CURRENT COLLECTION</span><strong data-nv-combo-selected>Foundations</strong><span data-nv-combo-detail>12 references · Colour, type & spacing</span></div><div class="nv-listbox" id="nv-collection-list" role="listbox" aria-label="Collections" data-nv-combo-list hidden>${collections.map(([name, desc, icon], i) => `<div role="option" id="nv-option-${i}" aria-selected="${i === 0}" data-nv-option="${i}">${ic(icon)}<span><strong>${name}</strong><small>${desc}</small></span><span class="nv-option-check">${ic("check", 14)}</span></div>`).join("")}<p class="nv-no-results" data-nv-no-results hidden>No matching collections. Try “motion” or “field”.</p></div></div><p id="nv-combo-help" class="nv-help">Type to filter · ↑ ↓ to explore · Enter to choose</p></div>${status("combo", "Foundations selected. Search six collections.")}`, ["actual filtering", "listbox / active descendant"])}
    </div>`,
    })}
    ${S.section({
      id: "nv-wayfinding",
      num: "02",
      title: "Breadcrumbs and pagination",
      body: `<div class="grid cols-2">
      ${demo("Living breadcrumb trail", "Drill into a real folder hierarchy. Every ancestor remains a working way back, and the path draws as you go.", `<div class="nv-browser" data-nv-breadcrumb><nav aria-label="Sample folder breadcrumb" class="nv-breadcrumb" data-nv-crumbs></nav><div class="nv-path-line" aria-hidden="true"><span data-nv-path-line></span></div><div class="nv-folder-view" data-nv-folder-view></div><div class="nv-browser-foot"><span>${ic("folder", 14)} Local workspace</span><button type="button" class="nv-text-button" data-nv-folder-up>${ic("arrow-left", 14)} Up one level</button></div></div>${status("breadcrumb", "Workspace. Two folders to explore.")}`, ["working ancestors", "path / trace"])}
      ${demo(
        "Directional pagination",
        "Twelve project notes, three at a time. The frame stays steady while each page travels in the direction you chose.",
        `<div class="nv-pagination" data-nv-pagination><div class="nv-mini-head"><span class="t-eyebrow">PROJECT NOTES</span><span class="nv-mini-count" data-nv-page-range>01—03 / 12</span></div><div class="nv-stack nv-page-stack" data-nv-page-stack>${Array.from(
          { length: 4 },
          (_, p) =>
            pane(
              String(p),
              `<ul class="nv-note-list">${records
                .slice(p * 3, p * 3 + 3)
                .map(
                  ([title, type, date], j) =>
                    `<li><span class="nv-note-number t-mono">${String(p * 3 + j + 1).padStart(2, "0")}</span><span><strong>${title}</strong><small>${type}</small></span><time>${date}</time></li>`,
                )
                .join("")}</ul>`,
              p === 0,
            ),
        ).join(
          "",
        )}</div><nav class="nv-page-controls" aria-label="Project note pages"><button type="button" class="nv-page-arrow" data-nv-page-prev aria-label="Previous page" disabled>${ic("arrow-left", 16)}</button><div>${[0, 1, 2, 3].map((i) => `<button type="button" data-nv-page="${i}" aria-label="Page ${i + 1}" ${i === 0 ? 'aria-current="page"' : ""}>${i + 1}</button>`).join("")}</div><button type="button" class="nv-page-arrow" data-nv-page-next aria-label="Next page">${ic("arrow-right", 16)}</button></nav></div>${status("page", "Page 1 of 4. Notes 1 to 3 of 12.")}`,
        ["stable frame", "direction / in-out"],
      )}
    </div>`,
    })}
    ${S.section({
      id: "nv-structure",
      num: "03",
      title: "Tree and dock",
      body: `<div class="grid cols-2">
      ${demo("Keyboard file tree", "Open nested folders and select a file. Use arrows to traverse, Right to expand and Left to return to its parent.", `<div class="nv-file-browser" data-nv-file-browser><div class="nv-mini-head"><span class="t-eyebrow">FIELDWORK / FILES</span><span class="nv-key-hint t-mono">↑ ↓ ← →</span></div><ul class="nv-tree" role="tree" aria-label="Fieldwork files" data-nv-tree>${treeMarkup(tree)}</ul><div class="nv-file-detail"><span data-nv-file-icon aria-hidden="true">${ic("folder", 18)}</span><span><strong data-nv-file-name>source</strong><small data-nv-file-desc>Choose a file to inspect its purpose.</small></span></div></div>${status("tree", "source is expanded. Two items inside.")}`, ["roving focus", "nested disclosure"])}
      ${demo(
        "Workspace action dock",
        "Board, list and activity show the same project from different angles. The selection travels; the controls stay grounded.",
        `<div class="nv-dock-workspace" data-nv-dock><div class="nv-mini-head"><span class="t-eyebrow">FIELDWORK</span><span class="badge badge-dot">3 open tasks</span></div><div class="nv-stack nv-dock-stack" data-nv-dock-stack>${pane("0", `<div class="nv-board-view"><div><span>TO DO <b>2</b></span><p>Add search tests<small>Research · In progress</small></p><p>Update navigation<small>Interface · Next</small></p></div><div><span>DONE <b>1</b></span><p class="nv-task-done">Review design<small>Planning · Complete</small></p></div></div>`, true, 'id="nv-dock-panel-0" role="tabpanel" aria-labelledby="nv-dock-tab-0"')}${pane("1", `<ul class="nv-task-list"><li>${ic("check-circle")}<span>Review design<small>Planning · Complete</small></span><span class="nv-task-tag">Done</span></li><li>${ic("clock")}<span>Add search tests<small>Research · In progress</small></span><span class="nv-task-tag">Now</span></li><li>${ic("document")}<span>Update navigation<small>Interface · Next</small></span><span class="nv-task-tag">Next</span></li></ul>`, false, 'id="nv-dock-panel-1" role="tabpanel" aria-labelledby="nv-dock-tab-1"')}${pane("2", `<ol class="nv-activity"><li><span>09:42</span><p><strong>Design approved.</strong><small>Planning moved to Done.</small></p></li><li><span>10:08</span><p><strong>Tests started.</strong><small>Search tests are running.</small></p></li><li><span>10:24</span><p><strong>Navigation task added.</strong><small>Navigation update moved to Next.</small></p></li></ol>`, false, 'id="nv-dock-panel-2" role="tabpanel" aria-labelledby="nv-dock-tab-2"')}</div><div class="nv-dock-bar" role="tablist" aria-label="Workspace view" data-nv-dock-bar><span class="nv-dock-marker" data-nv-dock-marker aria-hidden="true"></span>${[
          ["grid", "Board"],
          ["menu", "List"],
          ["clock", "Activity"],
        ]
          .map(
            ([icon, label], i) =>
              `<button type="button" role="tab" id="nv-dock-tab-${i}" aria-controls="nv-dock-panel-${i}" aria-selected="${i === 0}" tabindex="${i === 0 ? 0 : -1}" data-nv-dock-view="${i}">${ic(icon)}<span>${label}</span></button>`,
          )
          .join(
            "",
          )}</div></div>${status("dock", "Board view. Two open tasks and one completed task.")}`,
        ["travelling selection", "three real views"],
      )}
    </div>`,
    })}
    ${S.section({
      id: "nv-flows",
      num: "04",
      title: "Wizard and carousel",
      body: `<div class="grid cols-2">
      ${demo("Three-step workspace wizard", "Name a workspace, choose its owner and review the details. Native fields validate each step; Back preserves your work.", `<form class="nv-wizard" data-nv-wizard novalidate><ol class="nv-wizard-progress" aria-label="Workspace setup progress">${["Name", "Owner", "Review"].map((label, i) => `<li data-nv-wizard-step="${i}" ${i === 0 ? 'aria-current="step"' : ""}><span>${i + 1}</span>${label}</li>`).join("")}</ol><div class="nv-stack nv-wizard-stack" data-nv-wizard-stack>${pane("0", `<h3 tabindex="-1">Workspace name</h3><p class="nv-help">Enter a name to continue.</p><label class="nv-label" for="nv-workspace-name">Workspace name</label><input class="nv-input" id="nv-workspace-name" name="workspace" type="text" required minlength="2" maxlength="40" autocomplete="organization" placeholder="e.g. Fieldwork" aria-describedby="nv-name-error"><p class="nv-field-error" id="nv-name-error" data-nv-name-error></p>`, true)}${pane("1", `<h3 tabindex="-1">Workspace owner</h3><p class="nv-help">A local sample. No invitations are sent.</p><label class="nv-label" for="nv-workspace-email">Owner’s email</label><input class="nv-input" id="nv-workspace-email" name="email" type="email" required autocomplete="email" placeholder="you@example.com" aria-describedby="nv-email-error"><p class="nv-field-error" id="nv-email-error" data-nv-email-error></p><label class="nv-label" for="nv-workspace-access">Visibility</label><select class="nv-input" id="nv-workspace-access" name="access"><option value="Private">Private · just the owner</option><option value="Shared">Shared · ready for a team</option></select>`)}${pane("2", `<div data-nv-wizard-review><h3 tabindex="-1">Review workspace</h3><p class="nv-help">Review your choices before creating the sample.</p><dl class="nv-review"><div><dt>Workspace</dt><dd data-nv-review-name></dd></div><div><dt>Owner</dt><dd data-nv-review-email></dd></div><div><dt>Visibility</dt><dd data-nv-review-access></dd></div></dl></div><div class="nv-wizard-success" data-nv-wizard-success hidden><span class="nv-orbit" aria-hidden="true">${ic("check", 24)}</span><h3 tabindex="-1">Workspace created</h3><p data-nv-wizard-created></p><span class="badge badge-accent">Local sample created</span></div>`)}</div><div class="nv-wizard-actions"><button type="button" class="btn btn-secondary btn-sm" data-nv-wizard-back disabled>${ic("arrow-left", 14)} Back</button><span class="nv-wizard-count t-mono" data-nv-wizard-count>01 / 03</span><button type="submit" class="btn btn-primary btn-sm" data-nv-wizard-next><span data-nv-wizard-next-label>Next</span>${ic("arrow-right", 14)}</button></div></form>${status("wizard", "Step 1 of 3. Give your workspace a name.")}`, ["native validation", "state preserved"])}
      ${demo("Carousel", "Use arrows or dots to change slides.", `<div class="nv-carousel" data-nv-carousel tabindex="0" role="region" aria-roledescription="carousel" aria-label="Project directions"><div class="nv-mini-head"><span class="t-eyebrow">LAYOUTS</span><span class="nv-mini-count" data-nv-slide-count>01 / 03</span></div><div class="nv-stack nv-carousel-stack" data-nv-carousel-stack>${directions.map((d, i) => pane(String(i), `<div class="nv-direction-art nv-art-${d.type}" aria-hidden="true"><span class="nv-art-index t-mono">0${i + 1}</span>${d.type === "editorial" ? "<strong>Design<br>system</strong><i></i><i></i>" : d.type === "structural" ? '<div class="nv-art-nodes"><i></i><i></i><i></i><i></i></div>' : '<div class="nv-art-focus"><i></i><i></i><i></i></div>'}<span class="nv-art-tags t-mono">${d.tags}</span></div><div class="nv-direction-copy"><span>${d.name}</span><h3>${d.subtitle}</h3><p>${d.desc}</p></div>`, i === 0, `role="group" aria-roledescription="slide" aria-label="${i + 1} of 3: ${d.name}"`)).join("")}</div><div class="nv-carousel-controls"><button type="button" class="nv-page-arrow" data-nv-slide-prev aria-label="Previous direction">${ic("arrow-left")}</button><div class="nv-slide-dots" aria-label="Review design">${directions.map((d, i) => `<button type="button" data-nv-slide="${i}" aria-pressed="${i === 0}" aria-label="Show ${d.name} direction"><span></span></button>`).join("")}</div><button type="button" class="nv-page-arrow" data-nv-slide-next aria-label="Next direction">${ic("arrow-right")}</button></div><button type="button" class="nv-text-button nv-use-direction" data-nv-use-direction>Use this direction ${ic("arrow-right", 14)}</button></div>${status("carousel", "Slide 1 of 3. Component library.")}`, ["directional travel", "one accessible slide"])}
    </div>`,
    })}
  </div>`;

  function mount(root) {
    const $ = (s, context = root) => context.querySelector(s);
    const $$ = (s, context = root) => [...context.querySelectorAll(s)];
    const animations = new Map();
    const state = {
      visible: false,
      menu: -1,
      comboOpen: false,
      comboSelected: 0,
      comboActive: 0,
      folder: "workspace",
      page: 0,
      dock: 0,
      wizard: 0,
      completed: false,
      slide: 0,
    };
    const say = (name, text) => {
      $(`[data-nv-${name}-status]`).textContent = text;
    };
    const run = (
      el,
      frames,
      duration = 360,
      easing = S.curve.outExpo,
      delay = 0,
    ) => {
      animations.get(el)?.cancel();
      animations.delete(el);
      if (!state.visible || document.hidden || S.reduced()) return;
      const animation = el.animate(frames, {
        duration: S.dur(duration),
        delay: S.dur(delay),
        easing,
        fill: "backwards",
      });
      animations.set(el, animation);
      animation.finished.then(
        () => {
          if (animations.get(el) === animation) animations.delete(el);
        },
        () => {},
      );
    };
    // Capture displayed geometry before cancellation, including on rapid reversals.
    // All panes stay in one grid cell, so each specimen keeps a stable frame.
    const showPane = (stack, key, direction = 1, animate = true) => {
      const nodes = $$(":scope > [data-nv-pane]", stack);
      const before = nodes.map((el) => {
        const cs = getComputedStyle(el);
        return {
          el,
          visible: cs.visibility !== "hidden",
          opacity: cs.opacity,
          transform: cs.transform,
        };
      });
      before.forEach(({ el }) => {
        animations.get(el)?.cancel();
        animations.delete(el);
      });
      before.forEach(({ el, visible, opacity, transform }) => {
        const active = el.dataset.nvPane === String(key);
        el.dataset.nvActive = String(active);
        el.inert = !active;
        el.setAttribute("aria-hidden", String(!active));
        el.style.visibility = "";
        if (
          !animate ||
          !state.visible ||
          document.hidden ||
          S.reduced() ||
          (!active && !visible)
        )
          return;
        el.style.visibility = "visible";
        const delay =
          active &&
          !visible &&
          before.some((p) => p.visible && Number(p.opacity) > 0)
            ? 140
            : 0;
        run(
          el,
          [
            {
              opacity: visible ? opacity : 0,
              transform: visible
                ? transform
                : `translateX(${direction * 20}px)`,
            },
            {
              opacity: active ? 1 : 0,
              transform: active
                ? "translateX(0)"
                : `translateX(${-direction * 16}px)`,
            },
          ],
          active ? 420 : 140,
          active ? S.curve.outExpo : S.curve.in,
          delay,
        );
        const animation = animations.get(el);
        animation?.finished.then(
          () => {
            if (!animations.has(el)) el.style.visibility = "";
          },
          () => {},
        );
      });
    };
    const moveMarker = (bar, marker, button, animate = true) => {
      if (!button || !bar.getClientRects().length) return;
      const old = marker.getBoundingClientRect(),
        base = bar.getBoundingClientRect(),
        target = button.getBoundingClientRect();
      const x = target.left - base.left - bar.clientLeft;
      const wasReady = marker.dataset.nvReady === "true";
      animations.get(marker)?.cancel();
      animations.delete(marker);
      marker.style.width = `${target.width}px`;
      marker.style.transform = `translateX(${x}px)`;
      marker.dataset.nvReady = "true";
      if (animate && wasReady && target.width > 0)
        run(
          marker,
          [
            {
              transform: `translateX(${old.left - base.left - bar.clientLeft}px) scaleX(${old.width / target.width})`,
            },
            { transform: `translateX(${x}px) scaleX(1)` },
          ],
          420,
          S.curve.springSmooth || S.curve.outExpo,
        );
    };
    const menu = $("[data-nv-menu]"),
      menuViewport = $("[data-nv-menu-viewport]"),
      menuButtons = $$("[data-nv-menu-trigger]");
    const menuClose = (focus = false, animate = true) => {
      if (state.menu < 0) return;
      const old = state.menu;
      const cs = getComputedStyle(menuViewport);
      const frames = [
        { opacity: cs.opacity, transform: cs.transform },
        { opacity: 0, transform: "translateY(-6px)" },
      ];
      state.menu = -1;
      menu.dataset.nvOpen = "false";
      menuButtons.forEach((b) => b.setAttribute("aria-expanded", "false"));
      menuViewport.inert = true;
      menuViewport.setAttribute("aria-hidden", "true");
      if (animate) run(menuViewport, frames, 220, S.curve.in);
      else {
        animations.get(menuViewport)?.cancel();
        animations.delete(menuViewport);
      }
      if (focus) menuButtons[old].focus({ preventScroll: true });
    };
    const menuOpen = (index, focusFirst = false) => {
      const old = state.menu,
        cs = getComputedStyle(menuViewport);
      const from = {
        opacity: old < 0 ? cs.opacity : 1,
        transform: old < 0 ? cs.transform : "translateY(0)",
      };
      state.menu = index;
      menu.dataset.nvOpen = "true";
      menuViewport.inert = false;
      menuViewport.setAttribute("aria-hidden", "false");
      menuButtons.forEach((b, i) =>
        b.setAttribute("aria-expanded", String(index === i)),
      );
      showPane(
        $("[data-nv-menu-stack]"),
        index,
        index >= old ? 1 : -1,
        old >= 0,
      );
      moveMarker(
        $("[data-nv-menu-bar]"),
        $("[data-nv-menu-marker]"),
        menuButtons[index],
      );
      const barRect = menu.getBoundingClientRect(),
        buttonRect = menuButtons[index].getBoundingClientRect();
      menuViewport.style.setProperty(
        "--nv-pointer",
        `${buttonRect.left + buttonRect.width / 2 - barRect.left}px`,
      );
      if (old < 0)
        run(
          menuViewport,
          [from, { opacity: 1, transform: "translateY(0)" }],
          320,
        );
      if (focusFirst)
        $(
          `[data-nv-pane="${index}"] [data-nv-destination]`,
          menuViewport,
        ).focus();
      say("menu", `${menuData[index].label} open. Two destinations available.`);
    };
    menuButtons.forEach((button, i) => {
      button.addEventListener("click", () =>
        state.menu === i ? menuClose(true) : menuOpen(i),
      );
      button.addEventListener("keydown", (e) => {
        if (["ArrowLeft", "ArrowRight", "Home", "End"].includes(e.key)) {
          e.preventDefault();
          const next =
            e.key === "Home"
              ? 0
              : e.key === "End"
                ? 2
                : (i + (e.key === "ArrowRight" ? 1 : 2)) % 3;
          menuButtons[next].focus();
          if (state.menu >= 0) menuOpen(next);
        } else if (e.key === "ArrowDown") {
          e.preventDefault();
          menuOpen(i, true);
        }
      });
    });
    menu.addEventListener("click", (e) => {
      const b = e.target.closest("[data-nv-destination]");
      if (!b) return;
      say(
        "menu",
        `${b.dataset.nvDestination} selected. Your destination is ready.`,
      );
      menuClose(true);
    });
    menu.addEventListener("keydown", (e) => {
      if (e.key === "Escape" && state.menu >= 0) {
        e.preventDefault();
        e.stopPropagation();
        menuClose(true);
      }
    });
    menu.addEventListener("focusout", () =>
      queueMicrotask(() => {
        if (!menu.contains(document.activeElement)) menuClose();
      }),
    );
    document.addEventListener("pointerdown", (e) => {
      if (state.visible && !menu.contains(e.target)) menuClose();
    });

    const combo = $("[data-nv-combo]"),
      input = $("[data-nv-combo-input]"),
      options = $$("[data-nv-option]"),
      listbox = $("[data-nv-combo-list]");
    const matches = () => options.filter((o) => !o.hidden);
    const activeOption = (index) => {
      const filtered = matches();
      state.comboActive = filtered.length
        ? Math.max(0, Math.min(index, filtered.length - 1))
        : -1;
      options.forEach((o) => (o.dataset.nvHighlighted = "false"));
      const active = filtered[state.comboActive];
      if (state.comboOpen && active) {
        active.dataset.nvHighlighted = "true";
        input.setAttribute("aria-activedescendant", active.id);
        active.scrollIntoView({ block: "nearest" });
      } else input.removeAttribute("aria-activedescendant");
    };
    const comboToggle = (open) => {
      state.comboOpen = open;
      input.setAttribute("aria-expanded", String(open));
      listbox.hidden = !open;
      $("[data-nv-combo-rest]").hidden = open;
      if (open) {
        activeOption(state.comboActive);
        run(
          listbox,
          [
            { opacity: 0, transform: "translateY(-6px)" },
            { opacity: 1, transform: "translateY(0)" },
          ],
          220,
        );
      } else {
        input.removeAttribute("aria-activedescendant");
        animations.get(listbox)?.cancel();
        animations.delete(listbox);
      }
    };
    const filterOptions = () => {
      const query = input.value.trim().toLowerCase();
      options.forEach((o, i) => {
        o.hidden = !`${collections[i][0]} ${collections[i][1]}`
          .toLowerCase()
          .includes(query);
      });
      const count = matches().length;
      $("[data-nv-no-results]").hidden = count > 0;
      state.comboActive = 0;
      if (!state.comboOpen) comboToggle(true);
      else activeOption(0);
      say(
        "combo",
        count
          ? `${count} ${count === 1 ? "collection" : "collections"} found${query ? ` for “${input.value.trim()}”` : ""}.`
          : "No collections found. Try another search.",
      );
    };
    const chooseOption = (option) => {
      const i = Number(option.dataset.nvOption);
      state.comboSelected = i;
      options.forEach((o, j) =>
        o.setAttribute("aria-selected", String(i === j)),
      );
      $("[data-nv-combo-selected]").textContent = collections[i][0];
      $("[data-nv-combo-detail]").textContent =
        `${collections[i][3]} · ${collections[i][1]}`;
      input.value = "";
      options.forEach((o) => (o.hidden = false));
      $("[data-nv-no-results]").hidden = true;
      state.comboActive = i;
      comboToggle(false);
      input.focus({ preventScroll: true });
      say("combo", `${collections[i][0]} selected. ${collections[i][3]}.`);
    };
    input.addEventListener("focus", () => comboToggle(true));
    input.addEventListener("click", () => {
      if (!state.comboOpen) comboToggle(true);
    });
    input.addEventListener("input", filterOptions);
    input.addEventListener("keydown", (e) => {
      if (["ArrowDown", "ArrowUp", "Home", "End"].includes(e.key)) {
        if (!state.comboOpen && ["Home", "End"].includes(e.key)) return;
        e.preventDefault();
        const wasOpen = state.comboOpen;
        if (!wasOpen) comboToggle(true);
        activeOption(
          e.key === "Home"
            ? 0
            : e.key === "End"
              ? matches().length - 1
              : !wasOpen
                ? e.key === "ArrowDown"
                  ? 0
                  : matches().length - 1
                : state.comboActive + (e.key === "ArrowDown" ? 1 : -1),
        );
      } else if (e.key === "Enter" && state.comboOpen) {
        e.preventDefault();
        const option = matches()[state.comboActive];
        if (option) chooseOption(option);
      } else if (e.key === "Escape" && state.comboOpen) {
        e.preventDefault();
        e.stopPropagation();
        comboToggle(false);
        say(
          "combo",
          `Search closed. ${collections[state.comboSelected][0]} remains selected.`,
        );
      } else if (e.key === "Tab") comboToggle(false);
    });
    listbox.addEventListener("pointerdown", (e) => e.preventDefault());
    listbox.addEventListener("click", (e) => {
      const option = e.target.closest("[data-nv-option]");
      if (option) chooseOption(option);
    });
    combo.addEventListener("focusout", () =>
      queueMicrotask(() => {
        if (!combo.contains(document.activeElement)) comboToggle(false);
      }),
    );
    document.addEventListener("pointerdown", (e) => {
      if (state.visible && state.comboOpen && !combo.contains(e.target))
        comboToggle(false);
    });

    const folderView = $("[data-nv-folder-view]");
    const showFolder = (key, animate = true) => {
      const previous = state.folder,
        current = folders[key];
      state.folder = key;
      const path = [];
      for (let node = key; node; node = folders[node].parent)
        path.unshift(node);
      $("[data-nv-crumbs]").innerHTML = path
        .map(
          (id, i) =>
            `${i ? `<span class="nv-crumb-separator" aria-hidden="true">${ic("chevron-right", 12)}</span>` : ""}<button type="button" data-nv-folder="${id}" ${id === key ? 'aria-current="page"' : ""}>${id === "workspace" ? ic("home", 14) : ""}${folders[id].label}</button>`,
        )
        .join("");
      folderView.innerHTML = `<div class="nv-folder-heading"><span class="t-eyebrow">${current.children.length ? "FOLDERS" : "INSIDE THE FOLDER"}</span><span class="t-mono">${String(current.children.length).padStart(2, "0")}</span></div>${current.children.length ? current.children.map((id) => `<button type="button" class="nv-folder-item" data-nv-folder="${id}">${ic("folder", 20)}<span><strong>${folders[id].label}</strong><small>${folders[id].children.length ? `${folders[id].children.length} folders` : "Ready for your work"}</small></span><span class="nv-arrow">${ic("arrow-right", 14)}</span></button>`).join("") : `<div class="nv-folder-empty">${ic("document", 24)}<strong>Empty folder</strong><span>${current.label} is ready for your next idea.</span></div>`}`;
      $("[data-nv-folder-up]").disabled = !current.parent;
      const pathLine = $("[data-nv-path-line]");
      pathLine.style.transform = `scaleX(${path.length / 4})`;
      if (animate) {
        const direction = folders[previous]?.parent === key ? -1 : 1;
        run(
          folderView,
          [
            { opacity: 0, transform: `translateX(${direction * 12}px)` },
            { opacity: 1, transform: "translateX(0)" },
          ],
          320,
        );
        run(
          $("[data-nv-crumbs]"),
          [
            { opacity: 0.5, transform: "translateY(4px)" },
            { opacity: 1, transform: "translateY(0)" },
          ],
          260,
        );
      }
      say(
        "breadcrumb",
        `${path.map((id) => folders[id].label).join(" / ")}. ${current.children.length ? `${current.children.length} folders to explore.` : "This folder is ready for your work."}`,
      );
    };
    $("[data-nv-breadcrumb]").addEventListener("click", (e) => {
      const b = e.target.closest("[data-nv-folder]");
      if (b) {
        const focusCrumb = b.closest("[data-nv-crumbs]");
        const key = b.dataset.nvFolder;
        showFolder(key);
        (focusCrumb
          ? $(`[data-nv-crumbs] [data-nv-folder="${key}"]`)
          : $("[data-nv-folder-view] [data-nv-folder]") ||
            $("[data-nv-crumbs] [aria-current]")
        ).focus({ preventScroll: true });
      } else if (
        e.target.closest("[data-nv-folder-up]") &&
        folders[state.folder].parent
      )
        showFolder(folders[state.folder].parent);
    });
    showFolder("workspace", false);
    const showPage = (next) => {
      if (next < 0 || next > 3 || next === state.page) return;
      const direction = next > state.page ? 1 : -1;
      state.page = next;
      showPane($("[data-nv-page-stack]"), next, direction);
      $$("[data-nv-page]").forEach((b) => {
        if (Number(b.dataset.nvPage) === next)
          b.setAttribute("aria-current", "page");
        else b.removeAttribute("aria-current");
      });
      $("[data-nv-page-prev]").disabled = next === 0;
      $("[data-nv-page-next]").disabled = next === 3;
      $("[data-nv-page-range]").textContent =
        `${String(next * 3 + 1).padStart(2, "0")}—${String(next * 3 + 3).padStart(2, "0")} / 12`;
      say(
        "page",
        `Page ${next + 1} of 4. Notes ${next * 3 + 1} to ${next * 3 + 3} of 12.`,
      );
    };
    $("[data-nv-pagination]").addEventListener("click", (e) => {
      const b = e.target.closest("button");
      if (!b || b.disabled) return;
      if (b.hasAttribute("data-nv-page")) showPage(Number(b.dataset.nvPage));
      else if (b.hasAttribute("data-nv-page-prev")) showPage(state.page - 1);
      else if (b.hasAttribute("data-nv-page-next")) showPage(state.page + 1);
    });

    const treeRoot = $("[data-nv-tree]"),
      treeNodes = $$("[data-nv-node]");
    const nodeInfo = new Map();
    const visit = (nodes, path = []) =>
      nodes.forEach((n) => {
        nodeInfo.set(n.id, { ...n, path: [...path, n.name] });
        if (n.children) visit(n.children, [...path, n.name]);
      });
    visit(tree);
    const visibleTreeNodes = () =>
      treeNodes.filter((n) => {
        for (
          let p = n.parentElement.closest("[data-nv-node]");
          p;
          p = p.parentElement.closest("[data-nv-node]")
        )
          if (p.getAttribute("aria-expanded") === "false") return false;
        return true;
      });
    const treeFocus = (node) => {
      treeNodes.forEach((n) => (n.tabIndex = n === node ? 0 : -1));
      node.focus({ preventScroll: true });
    };
    const expandNode = (node, expanded) => {
      if (!node.hasAttribute("aria-expanded")) return;
      node.setAttribute("aria-expanded", String(expanded));
      const children = $(":scope > [data-nv-tree-children]", node);
      children.inert = !expanded;
      children.setAttribute("aria-hidden", String(!expanded));
      if (!expanded) {
        const roving = treeNodes.find((n) => n.tabIndex === 0);
        if (roving && roving !== node && node.contains(roving)) treeFocus(node);
      }
      say(
        "tree",
        `${nodeInfo.get(node.dataset.nvNode).name} ${expanded ? "expanded" : "collapsed"}.`,
      );
    };
    const selectNode = (node) => {
      const info = nodeInfo.get(node.dataset.nvNode);
      treeNodes.forEach((n) =>
        n.setAttribute("aria-selected", String(n === node)),
      );
      $("[data-nv-file-name]").textContent = info.name;
      $("[data-nv-file-desc]").textContent =
        info.detail || `${info.children.length} items · Folder`;
      $("[data-nv-file-icon]").innerHTML = ic(
        info.children ? "folder" : "document",
        18,
      );
      say(
        "tree",
        `${info.path.join(" / ")} selected. ${info.detail || `${info.children.length} items in this folder.`}`,
      );
    };
    treeRoot.addEventListener("click", (e) => {
      const n = e.target.closest("[data-nv-node]");
      if (
        !n ||
        e.target.closest("[data-nv-tree-children]") ===
          $(":scope > [data-nv-tree-children]", n)
      )
        return;
      treeFocus(n);
      selectNode(n);
      if (n.hasAttribute("aria-expanded"))
        expandNode(n, n.getAttribute("aria-expanded") !== "true");
    });
    treeRoot.addEventListener("focusin", (e) => {
      const n = e.target.closest("[data-nv-node]");
      if (n) treeNodes.forEach((el) => (el.tabIndex = el === n ? 0 : -1));
    });
    treeRoot.addEventListener("keydown", (e) => {
      const n = e.target.closest("[data-nv-node]");
      if (!n) return;
      const list = visibleTreeNodes(),
        i = list.indexOf(n);
      let focus;
      if (e.key === "ArrowDown") focus = list[Math.min(i + 1, list.length - 1)];
      else if (e.key === "ArrowUp") focus = list[Math.max(i - 1, 0)];
      else if (e.key === "Home") focus = list[0];
      else if (e.key === "End") focus = list.at(-1);
      else if (e.key === "ArrowRight") {
        if (n.getAttribute("aria-expanded") === "false") expandNode(n, true);
        else if (n.hasAttribute("aria-expanded")) focus = list[i + 1];
      } else if (e.key === "ArrowLeft") {
        if (n.getAttribute("aria-expanded") === "true") expandNode(n, false);
        else if (n.dataset.nvParent)
          focus = $(`[data-nv-node="${n.dataset.nvParent}"]`);
      } else if (e.key === "Enter" || e.key === " ") {
        selectNode(n);
        if (n.hasAttribute("aria-expanded"))
          expandNode(n, n.getAttribute("aria-expanded") !== "true");
      } else return;
      e.preventDefault();
      e.stopPropagation();
      if (focus) treeFocus(focus);
    });
    const dockButtons = $$("[data-nv-dock-view]");
    const showDock = (next) => {
      const old = state.dock;
      state.dock = next;
      showPane($("[data-nv-dock-stack]"), next, next >= old ? 1 : -1);
      dockButtons.forEach((b, i) => {
        b.setAttribute("aria-selected", String(i === next));
        b.tabIndex = i === next ? 0 : -1;
      });
      moveMarker(
        $("[data-nv-dock-bar]"),
        $("[data-nv-dock-marker]"),
        dockButtons[next],
      );
      say(
        "dock",
        [
          "Board view. Two open tasks and one completed task.",
          "List view. All three tasks, with their current status.",
          "Activity view. Three updates from this morning.",
        ][next],
      );
    };
    dockButtons.forEach((b, i) => {
      b.addEventListener("click", () => showDock(i));
      b.addEventListener("keydown", (e) => {
        if (!["ArrowLeft", "ArrowRight", "Home", "End"].includes(e.key)) return;
        e.preventDefault();
        const next =
          e.key === "Home"
            ? 0
            : e.key === "End"
              ? 2
              : (i + (e.key === "ArrowRight" ? 1 : 2)) % 3;
        showDock(next);
        dockButtons[next].focus();
      });
    });

    const wizard = $("[data-nv-wizard]"),
      nameInput = $("#nv-workspace-name"),
      emailInput = $("#nv-workspace-email");
    const updateReview = () => {
      $("[data-nv-review-name]").textContent = nameInput.value.trim();
      $("[data-nv-review-email]").textContent = emailInput.value.trim();
      $("[data-nv-review-access]").textContent = $(
        "#nv-workspace-access",
      ).value;
    };
    const showWizard = (next, focus = true) => {
      const old = state.wizard;
      state.wizard = next;
      if (next === 2) updateReview();
      showPane($("[data-nv-wizard-stack]"), next, next >= old ? 1 : -1);
      $$("[data-nv-wizard-step]").forEach((li, i) => {
        if (i === next) li.setAttribute("aria-current", "step");
        else li.removeAttribute("aria-current");
        li.dataset.nvPast = String(i < next);
      });
      $("[data-nv-wizard-back]").disabled = next === 0 || state.completed;
      $("[data-nv-wizard-count]").textContent = `0${next + 1} / 03`;
      $("[data-nv-wizard-next-label]").textContent = state.completed
        ? "Start again"
        : next === 2
          ? "Create"
          : "Next";
      say(
        "wizard",
        [
          `Step 1 of 3. Give your workspace a name.`,
          `Step 2 of 3. Choose an owner and visibility.`,
          `Step 3 of 3. Review ${nameInput.value.trim() || "your workspace"} before creating the local sample.`,
        ][next],
      );
      if (focus)
        $(`[data-nv-pane="${next}"] h3`, wizard).focus({ preventScroll: true });
    };
    const validate = () => {
      const field = state.wizard === 0 ? nameInput : emailInput;
      const error =
        state.wizard === 0
          ? $("[data-nv-name-error]")
          : $("[data-nv-email-error]");
      const valid =
        field.checkValidity() &&
        (state.wizard !== 0 || field.value.trim().length >= 2);
      field.setAttribute("aria-invalid", String(!valid));
      error.textContent = valid
        ? ""
        : state.wizard === 0
          ? "Use at least two characters for the workspace name."
          : "Enter a complete email, such as you@example.com.";
      if (!valid) {
        field.focus();
        say("wizard", error.textContent);
      }
      return valid;
    };
    wizard.addEventListener("submit", (e) => {
      e.preventDefault();
      if (state.completed) {
        state.completed = false;
        wizard.reset();
        $("[data-nv-wizard-success]").hidden = true;
        $("[data-nv-wizard-review]").hidden = false;
        [nameInput, emailInput].forEach((f) =>
          f.removeAttribute("aria-invalid"),
        );
        $("[data-nv-name-error]").textContent = "";
        $("[data-nv-email-error]").textContent = "";
        showWizard(0);
        return;
      }
      if (state.wizard < 2) {
        if (validate()) showWizard(state.wizard + 1);
        return;
      }
      state.completed = true;
      $("[data-nv-wizard-review]").hidden = true;
      $("[data-nv-wizard-success]").hidden = false;
      $("[data-nv-wizard-created]").textContent =
        `${nameInput.value.trim()} is ready for ${emailInput.value.trim()}.`;
      $("[data-nv-wizard-back]").disabled = true;
      $("[data-nv-wizard-next-label]").textContent = "Start again";
      $("[data-nv-wizard-success] h3").focus({ preventScroll: true });
      run(
        $("[data-nv-wizard-success]"),
        [
          { opacity: 0, transform: "translateY(8px)" },
          { opacity: 1, transform: "translateY(0)" },
        ],
        360,
      );
      say(
        "wizard",
        `${nameInput.value.trim()} created as a local sample. No data was sent.`,
      );
    });
    $("[data-nv-wizard-back]").addEventListener("click", () => {
      if (state.wizard > 0 && !state.completed) showWizard(state.wizard - 1);
    });
    [nameInput, emailInput].forEach((f) =>
      f.addEventListener("input", () => {
        if (f.getAttribute("aria-invalid") === "true") {
          f.removeAttribute("aria-invalid");
          $(
            f === nameInput ? "[data-nv-name-error]" : "[data-nv-email-error]",
          ).textContent = "";
        }
      }),
    );
    const carousel = $("[data-nv-carousel]");
    const showSlide = (next, direction) => {
      next = (next + directions.length) % directions.length;
      const old = state.slide;
      if (next === old) return;
      state.slide = next;
      showPane(
        $("[data-nv-carousel-stack]"),
        next,
        direction ?? (next > old ? 1 : -1),
      );
      $$("[data-nv-slide]").forEach((b, i) =>
        b.setAttribute("aria-pressed", String(i === next)),
      );
      $("[data-nv-slide-count]").textContent = `0${next + 1} / 03`;
      say(
        "carousel",
        `Direction ${next + 1} of 3. ${directions[next].name}: ${directions[next].subtitle}`,
      );
    };
    carousel.addEventListener("click", (e) => {
      const b = e.target.closest("button");
      if (!b) return;
      if (b.hasAttribute("data-nv-slide")) showSlide(Number(b.dataset.nvSlide));
      else if (b.hasAttribute("data-nv-slide-prev"))
        showSlide(state.slide - 1, -1);
      else if (b.hasAttribute("data-nv-slide-next"))
        showSlide(state.slide + 1, 1);
      else if (b.hasAttribute("data-nv-use-direction"))
        say(
          "carousel",
          `${directions[state.slide].name} selected for your project. ${directions[state.slide].desc}`,
        );
    });
    carousel.addEventListener("keydown", (e) => {
      if (!["ArrowLeft", "ArrowRight", "Home", "End"].includes(e.key)) return;
      e.preventDefault();
      showSlide(
        e.key === "Home"
          ? 0
          : e.key === "End"
            ? 2
            : state.slide + (e.key === "ArrowRight" ? 1 : -1),
        e.key === "ArrowLeft" || e.key === "Home" ? -1 : 1,
      );
    });
    const positionMarkers = () => {
      if (!state.visible) return;
      moveMarker(
        $("[data-nv-dock-bar]"),
        $("[data-nv-dock-marker]"),
        dockButtons[state.dock],
        false,
      );
      const index = Math.max(0, state.menu);
      moveMarker(
        $("[data-nv-menu-bar]"),
        $("[data-nv-menu-marker]"),
        menuButtons[index],
        false,
      );
      const r = menu.getBoundingClientRect(),
        b = menuButtons[index].getBoundingClientRect();
      menuViewport.style.setProperty(
        "--nv-pointer",
        `${b.left + b.width / 2 - r.left}px`,
      );
    };
    const resize = new ResizeObserver(positionMarkers);
    const stop = () => {
      menuClose(false, false);
      comboToggle(false);
      root.getAnimations({ subtree: true }).forEach((a) => {
        try {
          a.finish();
        } catch {}
        a.cancel();
      });
      animations.forEach((a) => a.cancel());
      animations.clear();
      $$("[data-nv-pane]").forEach((el) => (el.style.visibility = ""));
    };
    const motionRoot = $(".pg-navigation") || root;
    const syncReduced = () => {
      motionRoot.dataset.nvReduced = String(S.reduced());
      if (S.reduced()) {
        root.getAnimations({ subtree: true }).forEach((a) => {
          try {
            a.finish();
          } catch {}
          a.cancel();
        });
        animations.clear();
        $$("[data-nv-pane]").forEach((el) => (el.style.visibility = ""));
      }
    };
    const motionChanges = new MutationObserver(syncReduced);
    const systemMotion = matchMedia("(prefers-reduced-motion: reduce)");
    systemMotion.addEventListener("change", () => {
      if (state.visible) syncReduced();
    });
    document.addEventListener("visibilitychange", () => {
      if (state.visible && document.hidden) stop();
    });
    instances.set(root, {
      enter() {
        state.visible = true;
        syncReduced();
        motionChanges.observe(document.documentElement, {
          attributes: true,
          attributeFilter: ["data-motion"],
        });
        resize.observe($("[data-nv-dock-bar]"));
        resize.observe($("[data-nv-menu-bar]"));
        positionMarkers();
      },
      leave() {
        state.visible = false;
        motionChanges.disconnect();
        resize.disconnect();
        stop();
      },
    });
  }
  S.page({
    id: "navigation",
    title: "Navigation",
    group: "Build",
    icon: pageIcon,
    order: 47,
    count: 8,
    description:
      "Eight working paths: travelling menus, search, breadcrumbs, pagination, file tree, dock, wizard and carousel.",
    render,
    mount,
    enter: (root) => instances.get(root)?.enter(),
    leave: (root) => instances.get(root)?.leave(),
  });
})();

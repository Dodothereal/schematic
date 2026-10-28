(() => {
  "use strict";
  const S = window.Schematic;
  const instances = new WeakMap();
  const icon = (name, size = 18) => S.icon(name, { size });
  const check = () =>
    `<svg class="pr-check" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true"><path pathLength="1" d="m5 12 4.5 4.5L19 7"/></svg>`;
  const pin = () =>
    `<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.6" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true"><path d="m9 4 6 0-1 6 4 4H6l4-4zM12 14v7"/></svg>`;
  const arrow = (up) =>
    `<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.6" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true"><path d="${up ? "M12 19V5m-5 5 5-5 5 5" : "M12 5v14m-5-5 5 5 5-5"}"/></svg>`;
  const note = (key, text) =>
    `<p class="pr-note" data-pr-status="${key}" role="status" aria-live="polite" aria-atomic="true">${text}</p>`;
  const demo = (key, title, desc, body, specs) =>
    S.demo({
      title,
      desc,
      body,
      specs,
      stage: "is-col is-plain",
      attrs: `data-pr-specimen="${key}"`,
    });
  const documents = [
    ["brief", "Release brief", "Direction & decisions", "document"],
    ["tokens", "Token inventory", "The shared language", "palette"],
    ["motion", "Motion studies", "Timing & transitions", "wave"],
    ["review", "Review notes", "Details worth keeping", "pencil"],
  ];
  const tasks = [
    ["audit", "Interface audit", "Find the gaps"],
    ["structure", "Layout & rhythm", "Make room for the content"],
    ["motion", "Motion review", "Connect each state"],
    ["handoff", "Final handoff", "Share the working reference"],
  ];
  const phrases = ["clear thinking.", "focused work.", "what comes next."];
  const pinRow = ([id, title, desc, ic], pinned = false) =>
    `<li class="pr-list-row" data-pr-pin-row="${id}" data-pr-pinned="${pinned}"><span class="pr-row-icon">${icon(ic)}</span><span class="pr-row-copy"><strong>${title}</strong><small>${desc}</small></span><button type="button" class="btn btn-ghost btn-icon pr-pin-button" data-pr-pin="${id}" aria-pressed="${pinned}" aria-label="${pinned ? "Unpin" : "Pin"} ${title}">${pin()}</button></li>`;
  const sortRow = ([id, title, desc], index) =>
    `<li class="pr-list-row" data-pr-sort-row="${id}"><span class="pr-position" data-pr-position aria-hidden="true">0${index + 1}</span><span class="pr-row-copy"><strong>${title}</strong><small>${desc}</small></span><span class="pr-move-controls"><button type="button" class="btn btn-ghost btn-icon btn-sm" data-pr-move="up" aria-label="Move ${title} up" ${index === 0 ? "disabled" : ""}>${arrow(true)}</button><button type="button" class="btn btn-ghost btn-icon btn-sm" data-pr-move="down" aria-label="Move ${title} down" ${index === tasks.length - 1 ? "disabled" : ""}>${arrow(false)}</button></span></li>`;
  const digits = () =>
    [..."128"]
      .map(
        (n) =>
          `<span class="pr-digit"><span class="pr-digit-strip" data-pr-digit style="--pr-digit:${n}">${[..."0123456789"].map((d) => `<span>${d}</span>`).join("")}</span></span>`,
      )
      .join("");

  const render = () => `<div class="pg-primitives" data-pr-motion="off">
    ${S.pageHead({ icon: "layers", eyebrow: "Build / The moving parts", title: "Primitives", lead: "Open a thought. Rearrange a plan. Give a number somewhere to go. Eight small building blocks, with every state considered.", meta: ['<span class="badge badge-accent">8 working specimens</span>', '<span class="badge">Manual, meaningful motion</span>', '<span class="badge">Keyboard ready</span>'] })}
    <div class="pr-intro"><span class="pr-intro-mark" aria-hidden="true">${icon("layers", 22)}</span><p>Nothing moves until you make a choice.<br><span class="muted">Try a second action before the first one finishes.</span></p><span class="pr-intro-index" aria-hidden="true">01 / 08</span></div>

    ${S.section({
      id: "pr-disclosure",
      num: "01",
      title: "A little more, when you need it",
      desc: "Keep the outline clear. Let detail occupy exactly the space it needs.",
      body: `<div class="grid cols-2 pr-disclosure-grid">
      ${demo(
        "accordion",
        "Accordion",
        "One thought open at a time. Arrow keys, Home and End move between headings.",
        `<div class="pr-accordion" data-pr-accordion><div class="pr-label-line"><span class="t-eyebrow">Project notes</span><span class="pr-mini-meta">03 topics</span></div>${[
          [
            "Start with the essentials",
            "A shared foundation makes the next decision easier. Agree on the language before adding another variation.",
            "Spacing / Type / Colour",
          ],
          [
            "Let the detail unfold",
            "Show the summary first, then make room for the reasoning. The heading stays in place while its content opens beneath it.",
            "Context, when it is useful",
          ],
          [
            "Keep the way back clear",
            "A good interaction works in both directions. You can close this thought, choose another, or change your mind while it is still moving.",
            "Reversible by design",
          ],
        ]
          .map(
            ([title, desc, tag], i) =>
              `<div class="pr-accordion-item" data-pr-accordion-item data-pr-open="${i === 0}"><h3><button type="button" class="pr-accordion-trigger" id="pr-accordion-trigger-${i}" data-pr-accordion-trigger="${i}" aria-expanded="${i === 0}" aria-controls="pr-accordion-panel-${i}"><span class="pr-ordinal" aria-hidden="true">0${i + 1}</span><span>${title}</span><span class="pr-disclosure-mark" aria-hidden="true">${icon("plus", 16)}</span></button></h3><div class="pr-grid-reveal" id="pr-accordion-panel-${i}" data-pr-accordion-panel role="region" aria-labelledby="pr-accordion-trigger-${i}" aria-hidden="${i !== 0}" ${i !== 0 ? "inert" : ""}><div class="pr-grid-clip"><div class="pr-accordion-copy"><p>${desc}</p><span class="pr-inline-label">${tag}</span></div></div></div></div>`,
          )
          .join(
            "",
          )}</div>${note("accordion", "Start with the essentials is open.")}`,
        ["single / reversible", "grid height"],
      )}
      ${demo("collapsible", "Collapsible", "A measured panel fits a changing checklist. Add the review notes while it is open.", `<div class="pr-packet" data-pr-collapsible data-pr-open="false"><div class="pr-label-line"><span class="t-eyebrow">Release packet</span><span class="badge badge-accent">Ready</span></div><h3 class="pr-card-title">A considered handoff.</h3><p class="pr-card-copy">The essentials up front. The reasoning, one step away.</p><dl class="pr-facts"><div><dt>Owner</dt><dd>Interface team</dd></div><div><dt>Scope</dt><dd>8 primitives</dd></div></dl><button type="button" class="pr-inline-trigger" data-pr-collapse aria-expanded="false" aria-controls="pr-checklist-panel"><span data-pr-collapse-label>Show checklist</span>${icon("chevron-down", 16)}</button><div class="pr-measured-panel" id="pr-checklist-panel" data-pr-collapse-panel aria-hidden="true" inert><div class="pr-panel-inset"><ul class="pr-checklist"><li>${check()}<span>Four appearances checked</span></li><li>${check()}<span>Keyboard paths reviewed</span></li><li>${check()}<span>Motion can change its mind</span></li></ul><button type="button" class="btn btn-ghost btn-sm pr-notes-button" data-pr-detail-toggle aria-expanded="false" aria-controls="pr-review-notes">Include review notes ${icon("plus", 14)}</button><div class="pr-review-notes" id="pr-review-notes" data-pr-detail-notes hidden><span class="t-eyebrow">For the next person</span><p>Keep state changes immediate and let the motion follow. A second click should feel like a new direction, even halfway through the first.</p></div></div></div></div>${note("collapsible", "Checklist collapsed. Review notes are optional.")}`, ["measured height", "variable content"])}
    </div>`,
    })}

    ${S.section({
      id: "pr-layout",
      num: "02",
      title: "A place for everything",
      desc: "Items keep their identity as their place changes. Follow the same row from one position to the next.",
      body: `<div class="grid cols-2 pr-layout-grid">
      ${demo(
        "pinned-list",
        "Pinned list",
        "Keep useful documents in reach. The original rows move between both lists.",
        `<div class="pr-pin-board" data-pr-pinned-list><div class="pr-list-label"><span>${pin()} Pinned</span><span class="pr-mini-meta" data-pr-pin-count>01</span></div><ul class="pr-document-list" data-pr-pin-zone="pinned" aria-label="Pinned documents">${pinRow(documents[0], true)}</ul><div class="pr-list-label pr-list-label-secondary"><span>All other documents</span><span class="pr-mini-meta" data-pr-unpin-count>03</span></div><ul class="pr-document-list" data-pr-pin-zone="unpinned" aria-label="Unpinned documents">${documents
          .slice(1)
          .map((d) => pinRow(d))
          .join(
            "",
          )}</ul></div>${note("pinned-list", "1 pinned document. Pin another to keep it at the top.")}`,
        ["real DOM movement", "FLIP / 460ms"],
      )}
      ${demo("sortable-list", "Sortable list", "Reorder a practical plan with move buttons. Focus travels with the item.", `<div class="pr-sort-board" data-pr-sortable-list><div class="pr-label-line"><span class="t-eyebrow">A plan for the details</span><span class="pr-mini-meta">04 steps</span></div><ol class="pr-task-list" data-pr-sort-list aria-label="Project plan">${tasks.map(sortRow).join("")}</ol><div class="pr-list-bottom"><span class="pr-mini-meta">Your order, kept locally on this page.</span><button type="button" class="btn btn-ghost btn-sm" data-pr-sort-reset>${icon("refresh", 14)} Reset</button></div></div>${note("sortable-list", "Interface audit is first. Use each row’s up and down buttons to reorder.")}`, ["keyboard controls", "FLIP / 460ms"])}
    </div>`,
    })}

    ${S.section({
      id: "pr-type",
      num: "03",
      title: "Words and numbers, with continuity",
      desc: "A new value arrives from a direction. The message remains readable while the characters move.",
      body: `<div class="grid cols-2 pr-type-grid">
      ${demo("text-swap", "Text swap", "Choose a message. Words leave and enter in the direction of your selection.", `<div class="pr-text-card" data-pr-text-swap><span class="t-eyebrow">A little space for</span><p class="pr-text-sentence" data-pr-whole-text><span class="pr-sr" data-pr-swap-accessible>Make room for clear thinking.</span><span class="pr-text-prefix" aria-hidden="true">Make room for</span><span class="pr-word-window" data-pr-word-window aria-hidden="true"><span class="pr-word" data-pr-word-value="0">clear thinking.</span></span></p><div class="pr-text-rule" aria-hidden="true"><span></span><span></span><span></span></div><div class="pr-message-options" role="group" aria-label="Choose a message">${["Clarity", "Focus", "Progress"].map((label, i) => `<button type="button" class="pr-message-option" data-pr-swap="${i}" aria-pressed="${i === 0}"><span class="pr-option-dot" aria-hidden="true"></span>${label}</button>`).join("")}</div></div>${note("text-swap", "Make room for clear thinking. Clarity selected.")}`, ["directional / 460ms", "no autoplay"])}
      ${demo("counter", "Animated counter", "A manual tally with three stable columns. The digits roll; the layout stays still.", `<div class="pr-counter-card" data-pr-counter-card><div class="pr-label-line"><span class="t-eyebrow">Your local tally</span><span class="pr-mini-meta">000–999</span></div><output class="pr-counter" data-pr-counter data-value="128"><span class="pr-sr" data-pr-counter-accessible>128 items</span><span class="pr-digits" aria-hidden="true">${digits()}</span><span class="pr-counter-unit" aria-hidden="true">items</span></output><div class="pr-counter-controls"><button type="button" class="btn btn-secondary btn-icon" data-pr-count="-1" aria-label="Decrease count by one">${icon("minus")}</button><button type="button" class="btn btn-primary" data-pr-count="1">${icon("plus", 16)} Add one</button><button type="button" class="btn btn-ghost btn-sm" data-pr-counter-reset>${icon("refresh", 14)} Reset</button></div><p class="pr-counter-hint">Try the carry from 129 to 130.</p></div>${note("counter", "128 items. This sample tally stays on this page.")}`, ["stable digit columns", "reversible roll"])}
    </div>`,
    })}

    ${S.section({
      id: "pr-surfaces",
      num: "04",
      title: "Another side to the story",
      desc: "Surfaces hold context. Open a little more, or turn one over, without losing your place.",
      body: `<div class="grid cols-2 pr-surfaces-grid">
      ${demo("flip-card", "Flip card", "Turn the card deliberately. Only the visible face is available to a keyboard or screen reader.", `<div class="pr-flip-specimen"><div class="pr-flip-card" data-pr-flip-card data-pr-flipped="false"><div class="pr-flip-rotor"><div class="pr-flip-face pr-flip-front" data-pr-face="front" aria-hidden="false"><div class="pr-label-line"><span class="t-eyebrow">Collected ideas / 008</span>${icon("layers", 20)}</div><div class="pr-flip-art" aria-hidden="true"><span></span><span></span><span></span></div><h3>A quieter place<br>to start.</h3><p>Less searching. More room for the next useful thought.</p><button type="button" class="pr-face-action" data-pr-flip-action="save" aria-pressed="false">${icon("plus", 16)}<span data-pr-save-label>Save to reading list</span></button></div><div class="pr-flip-face pr-flip-back" data-pr-face="back" aria-hidden="true" inert><div class="pr-label-line"><span class="t-eyebrow">Behind the idea</span><span class="pr-mini-meta">02 / 02</span></div><h3>Small decisions.<br>Shared direction.</h3><dl class="pr-flip-facts"><div><dt>Language</dt><dd>Warm, clear, considered</dd></div><div><dt>Behavior</dt><dd>Every action has an answer</dd></div><div><dt>Purpose</dt><dd>Make the next step easier</dd></div></dl><button type="button" class="pr-face-action" data-pr-flip-action="review" aria-pressed="false">${check()}<span data-pr-flip-review-label>Mark as reviewed</span></button></div></div></div><button type="button" class="btn btn-secondary pr-flip-control" data-pr-flip aria-pressed="false">${icon("refresh", 16)}<span data-pr-flip-label>Show reverse</span></button></div>${note("flip-card", "Front of card. Not saved to the reading list.")}`, ["explicit control", "inert hidden face"])}
      ${demo("expandable-surface", "Expandable surface", "Open the project’s context, then complete its review. Height and status follow the action.", `<div class="pr-context-card" data-pr-expandable data-pr-open="false" data-pr-reviewed="false"><button type="button" class="pr-context-trigger" data-pr-expand aria-expanded="false" aria-controls="pr-context-panel"><span class="pr-project-mark" aria-hidden="true">fn.</span><span class="pr-context-heading"><span class="pr-mini-meta">Project / 008</span><strong>Field notes</strong><span class="pr-context-subtitle">A publishing workflow, considered.</span></span><span class="pr-context-arrow" aria-hidden="true">${icon("chevron-down", 18)}</span></button><div class="pr-context-summary"><span class="pr-review-state">${check()}<span data-pr-review-state>Ready for review</span></span><span class="pr-mini-meta">3 contributors</span></div><div class="pr-measured-panel" id="pr-context-panel" data-pr-expand-panel aria-hidden="true" inert><div class="pr-context-detail"><p>A shared place for small observations that deserve a second look.</p><dl class="pr-context-facts"><div><dt>Next milestone</dt><dd>Editorial handoff</dd></div><div><dt>What changed</dt><dd>Structure, rhythm, motion</dd></div><div><dt>Review window</dt><dd>Today, 14:00–16:00</dd></div></dl><button type="button" class="pr-review-button" data-pr-review aria-pressed="false"><span class="pr-review-check">${check()}</span><span data-pr-review-label>Mark review complete</span></button></div></div><div class="pr-context-bottom"><span data-pr-context-hint>Open to see the next step</span><span aria-hidden="true">${icon("arrow-up-right", 14)}</span></div></div>${note("expandable-surface", "Project context closed. Review pending.")}`, ["measured expansion", "drawn confirmation"])}
    </div>`,
    })}
  </div>`;

  const $ = (state, selector) => state.root.querySelector(selector);
  const $$ = (state, selector) => [...state.root.querySelectorAll(selector)];
  const announce = (state, key, text) => {
    $(state, `[data-pr-status="${key}"]`).textContent = text;
  };
  const canMove = (state) => state.active && !document.hidden && !S.reduced();
  const cancel = (state, el) => {
    state.animations.get(el)?.cancel();
    state.animations.delete(el);
  };
  const animate = (
    state,
    el,
    frames,
    duration = 460,
    easing = S.curve.inOut,
    done,
  ) => {
    cancel(state, el);
    if (!canMove(state)) {
      done?.();
      return;
    }
    const animation = el.animate(frames, { duration: S.dur(duration), easing });
    state.animations.set(el, animation);
    animation.finished.then(
      () => {
        if (state.animations.get(el) !== animation) return;
        state.animations.delete(el);
        animation.cancel();
        done?.();
      },
      () => {},
    );
  };

  // Read the displayed height before cancellation: reversing never starts from
  // an obsolete endpoint. The natural height returns once the new state settles.
  const resizePanel = (state, panel, open, mutate = () => {}) => {
    const before = panel.getBoundingClientRect().height;
    cancel(state, panel);
    mutate();
    panel.inert = !open;
    panel.setAttribute("aria-hidden", String(!open));
    panel.dataset.prOpen = String(open);
    panel.style.height = open ? "auto" : "0px";
    const after = panel.getBoundingClientRect().height;
    if (Math.abs(before - after) < 0.5 || !canMove(state)) return;
    panel.style.height = `${after}px`;
    animate(
      state,
      panel,
      [{ height: `${before}px` }, { height: `${after}px` }],
      460,
      S.curve.out,
      () => {
        panel.style.height = panel.dataset.prOpen === "true" ? "auto" : "0px";
      },
    );
  };

  const accordion = (state, index) => {
    state.accordion = state.accordion === index ? -1 : index;
    $$(state, "[data-pr-accordion-item]").forEach((item, i) => {
      const open = i === state.accordion,
        trigger = item.querySelector("[data-pr-accordion-trigger]"),
        panel = item.querySelector("[data-pr-accordion-panel]");
      if (!open && panel.contains(document.activeElement))
        trigger.focus({ preventScroll: true });
      item.dataset.prOpen = String(open);
      trigger.setAttribute("aria-expanded", String(open));
      panel.inert = !open;
      panel.setAttribute("aria-hidden", String(!open));
    });
    const title =
      state.accordion < 0
        ? "All topics are closed."
        : `${$(state, `[data-pr-accordion-trigger="${state.accordion}"]`).children[1].textContent} is open.`;
    announce(state, "accordion", title);
  };

  const flipRows = (state, rows, mutate) => {
    if (canMove(state))
      S.flip(rows, mutate, {
        duration: 460,
        easing: S.curve.inOut,
        enter: false,
      });
    else {
      rows.forEach((el) => el.getAnimations().forEach((a) => a.cancel()));
      mutate();
    }
  };
  const pinDocument = (state, button) => {
    const id = button.dataset.prPin,
      row = button.closest("[data-pr-pin-row]"),
      isPinned = !state.pinned.has(id);
    const focused = document.activeElement === button;
    isPinned ? state.pinned.add(id) : state.pinned.delete(id);
    flipRows(state, $$(state, "[data-pr-pin-row]"), () => {
      const zone = $(
        state,
        `[data-pr-pin-zone="${isPinned ? "pinned" : "unpinned"}"]`,
      );
      if (isPinned) zone.append(row);
      else {
        const order = documents.findIndex((d) => d[0] === id);
        const next = [...zone.children].find(
          (el) =>
            documents.findIndex((d) => d[0] === el.dataset.prPinRow) > order,
        );
        zone.insertBefore(row, next || null);
      }
      row.dataset.prPinned = String(isPinned);
      button.setAttribute("aria-pressed", String(isPinned));
      button.setAttribute(
        "aria-label",
        `${isPinned ? "Unpin" : "Pin"} ${row.querySelector("strong").textContent}`,
      );
      $(state, "[data-pr-pin-count]").textContent = String(
        state.pinned.size,
      ).padStart(2, "0");
      $(state, "[data-pr-unpin-count]").textContent = String(
        documents.length - state.pinned.size,
      ).padStart(2, "0");
    });
    if (focused) button.focus({ preventScroll: true });
    announce(
      state,
      "pinned-list",
      `${row.querySelector("strong").textContent} ${isPinned ? "pinned" : "unpinned"}. ${state.pinned.size} pinned ${state.pinned.size === 1 ? "document" : "documents"}.`,
    );
  };
  const updateSort = (state) => {
    $$(state, "[data-pr-sort-row]").forEach((row, i, rows) => {
      row.querySelector("[data-pr-position]").textContent = String(
        i + 1,
      ).padStart(2, "0");
      row.querySelector('[data-pr-move="up"]').disabled = i === 0;
      row.querySelector('[data-pr-move="down"]').disabled =
        i === rows.length - 1;
    });
  };
  const sortTask = (state, button) => {
    const row = button.closest("[data-pr-sort-row]"),
      up = button.dataset.prMove === "up",
      next = up ? row.previousElementSibling : row.nextElementSibling;
    if (!next) return;
    const focused = document.activeElement === button;
    flipRows(state, $$(state, "[data-pr-sort-row]"), () => {
      row.parentElement.insertBefore(row, up ? next : next.nextElementSibling);
      updateSort(state);
    });
    if (focused)
      (button.disabled
        ? row.querySelector(`[data-pr-move="${up ? "down" : "up"}"]`)
        : button
      ).focus({ preventScroll: true });
    const position = [...row.parentElement.children].indexOf(row) + 1;
    announce(
      state,
      "sortable-list",
      `${row.querySelector("strong").textContent} moved to position ${position} of ${tasks.length}.`,
    );
  };

  const settleWords = (state) => {
    $$(state, "[data-pr-word-value]").forEach((el) => {
      if (Number(el.dataset.prWordValue) !== state.word) el.remove();
      else {
        el.style.opacity = "1";
        el.style.transform = "none";
      }
    });
  };
  const swapText = (state, index) => {
    if (index === state.word) return;
    const direction = index > state.word ? 1 : -1,
      window = $(state, "[data-pr-word-window]");
    const current = [...window.children];
    // Capture all visible layers first, so a rapid reversal continues from the
    // actual composite, including the word that was on its way out.
    const visible = new Map(
      current.map((el) => {
        const css = getComputedStyle(el);
        return [el, { opacity: css.opacity, transform: css.transform }];
      }),
    );
    current.forEach((el) => cancel(state, el));
    state.word = index;
    $$(state, "[data-pr-swap]").forEach((button) =>
      button.setAttribute(
        "aria-pressed",
        String(Number(button.dataset.prSwap) === index),
      ),
    );
    $(state, "[data-pr-swap-accessible]").textContent =
      `Make room for ${phrases[index]}`;
    announce(
      state,
      "text-swap",
      `Make room for ${phrases[index]} ${["Clarity", "Focus", "Progress"][index]} selected.`,
    );
    let target = current.find((el) => Number(el.dataset.prWordValue) === index);
    if (!target) {
      target = document.createElement("span");
      target.className = "pr-word";
      target.dataset.prWordValue = String(index);
      target.textContent = phrases[index];
      window.append(target);
      visible.set(target, {
        opacity: "0",
        transform: `translateY(${direction * 90}%)`,
      });
    }
    if (!canMove(state)) {
      settleWords(state);
      return;
    }
    [...window.children].forEach((el) => {
      const chosen = el === target,
        end = {
          opacity: chosen ? "1" : "0",
          transform: chosen
            ? "translateY(0%)"
            : `translateY(${direction * -90}%)`,
        };
      Object.assign(el.style, end);
      animate(state, el, [visible.get(el), end], 460, S.curve.inOut, () => {
        if (Number(el.dataset.prWordValue) !== state.word) el.remove();
      });
    });
  };

  const count = (state, value, reset = false) => {
    state.count = S.clamp(value, 0, 999);
    const digits = String(state.count).padStart(3, "0");
    $$(state, "[data-pr-digit]").forEach((strip, i) =>
      strip.style.setProperty("--pr-digit", digits[i]),
    );
    $(state, "[data-pr-counter]").dataset.value = String(state.count);
    $(state, "[data-pr-counter-accessible]").textContent =
      `${state.count} ${state.count === 1 ? "item" : "items"}`;
    $(state, '[data-pr-count="-1"]').disabled = state.count === 0;
    $(state, '[data-pr-count="1"]').disabled = state.count === 999;
    announce(
      state,
      "counter",
      `${state.count} ${state.count === 1 ? "item" : "items"}.${reset ? " Tally reset." : ""}`,
    );
  };
  const turnCard = (state) => {
    const button = $(state, "[data-pr-flip]"),
      oldFace = $(
        state,
        `[data-pr-face="${state.flipped ? "back" : "front"}"]`,
      );
    if (oldFace.contains(document.activeElement))
      button.focus({ preventScroll: true });
    state.flipped = !state.flipped;
    $(state, "[data-pr-flip-card]").dataset.prFlipped = String(state.flipped);
    button.setAttribute("aria-pressed", String(state.flipped));
    $(state, "[data-pr-flip-label]").textContent = state.flipped
      ? "Show front"
      : "Show reverse";
    $$(state, "[data-pr-face]").forEach((face) => {
      const hidden = (face.dataset.prFace === "back") !== state.flipped;
      face.inert = hidden;
      face.setAttribute("aria-hidden", String(hidden));
    });
    announce(
      state,
      "flip-card",
      state.flipped
        ? `Reverse of card. ${state.flipReviewed ? "Reviewed." : "Not reviewed yet."}`
        : `Front of card. ${state.saved ? "Saved to" : "Not saved to"} the reading list.`,
    );
  };
  const contextStatus = (state) =>
    announce(
      state,
      "expandable-surface",
      `Project context ${state.expanded ? "open" : "closed"}. Review ${state.reviewed ? "complete" : "pending"}.`,
    );

  const settle = (state) => {
    state.root.dataset.prMotion = "off";
    state.animations.forEach((animation) => animation.cancel());
    state.animations.clear();
    state.root
      .getAnimations({ subtree: true })
      .forEach((animation) => animation.cancel());
    $$(state, ".pr-measured-panel").forEach((panel) => {
      panel.style.height = panel.dataset.prOpen === "true" ? "auto" : "0px";
    });
    settleWords(state);
  };
  const syncMotion = (state) => {
    if (!canMove(state)) settle(state);
    else state.root.dataset.prMotion = "on";
  };

  const mount = (pageRoot) => {
    const state = {
      root: pageRoot.querySelector(".pg-primitives"),
      active: false,
      animations: new Map(),
      accordion: 0,
      pinned: new Set(["brief"]),
      word: 0,
      count: 128,
      flipped: false,
      saved: false,
      flipReviewed: false,
      expanded: false,
      reviewed: false,
      collapsed: true,
      notes: false,
    };
    instances.set(pageRoot, state);
    state.root.addEventListener("click", (event) => {
      const button = event.target.closest("button");
      if (!button || !state.root.contains(button) || button.disabled) return;
      if (button.matches("[data-pr-accordion-trigger]"))
        accordion(state, Number(button.dataset.prAccordionTrigger));
      else if (button.matches("[data-pr-collapse]")) {
        state.collapsed = !state.collapsed;
        const open = !state.collapsed,
          card = $(state, "[data-pr-collapsible]");
        if (
          !open &&
          $(state, "[data-pr-collapse-panel]").contains(document.activeElement)
        )
          button.focus({ preventScroll: true });
        card.dataset.prOpen = String(open);
        button.setAttribute("aria-expanded", String(open));
        $(state, "[data-pr-collapse-label]").textContent = open
          ? "Hide checklist"
          : "Show checklist";
        resizePanel(state, $(state, "[data-pr-collapse-panel]"), open);
        announce(
          state,
          "collapsible",
          `Checklist ${open ? "expanded" : "collapsed"}.${open ? " Three handoff checks complete." : ""}`,
        );
      } else if (button.matches("[data-pr-detail-toggle]")) {
        state.notes = !state.notes;
        button.setAttribute("aria-expanded", String(state.notes));
        resizePanel(state, $(state, "[data-pr-collapse-panel]"), true, () => {
          $(state, "[data-pr-detail-notes]").hidden = !state.notes;
          button.innerHTML = `${state.notes ? "Hide review notes" : "Include review notes"} ${icon(state.notes ? "minus" : "plus", 14)}`;
        });
        announce(
          state,
          "collapsible",
          `Checklist expanded. Review notes ${state.notes ? "included" : "hidden"}.`,
        );
      } else if (button.matches("[data-pr-pin]")) pinDocument(state, button);
      else if (button.matches("[data-pr-move]")) sortTask(state, button);
      else if (button.matches("[data-pr-sort-reset]")) {
        flipRows(state, $$(state, "[data-pr-sort-row]"), () => {
          tasks.forEach(([id]) =>
            $(state, "[data-pr-sort-list]").append(
              $(state, `[data-pr-sort-row="${id}"]`),
            ),
          );
          updateSort(state);
        });
        announce(
          state,
          "sortable-list",
          "Original order restored. Interface audit is first.",
        );
      } else if (button.matches("[data-pr-swap]"))
        swapText(state, Number(button.dataset.prSwap));
      else if (button.matches("[data-pr-count]"))
        count(state, state.count + Number(button.dataset.prCount));
      else if (button.matches("[data-pr-counter-reset]"))
        count(state, 128, true);
      else if (button.matches("[data-pr-flip]")) turnCard(state);
      else if (button.matches('[data-pr-flip-action="save"]')) {
        state.saved = !state.saved;
        button.setAttribute("aria-pressed", String(state.saved));
        $(state, "[data-pr-save-label]").textContent = state.saved
          ? "Saved to reading list"
          : "Save to reading list";
        announce(
          state,
          "flip-card",
          state.saved
            ? "Saved to the local reading list."
            : "Removed from the local reading list.",
        );
      } else if (button.matches('[data-pr-flip-action="review"]')) {
        state.flipReviewed = !state.flipReviewed;
        button.setAttribute("aria-pressed", String(state.flipReviewed));
        $(state, "[data-pr-flip-review-label]").textContent = state.flipReviewed
          ? "Reviewed"
          : "Mark as reviewed";
        announce(
          state,
          "flip-card",
          state.flipReviewed
            ? "Idea marked as reviewed."
            : "Idea review cleared.",
        );
      } else if (button.matches("[data-pr-expand]")) {
        state.expanded = !state.expanded;
        const panel = $(state, "[data-pr-expand-panel]");
        if (!state.expanded && panel.contains(document.activeElement))
          button.focus({ preventScroll: true });
        $(state, "[data-pr-expandable]").dataset.prOpen = String(
          state.expanded,
        );
        button.setAttribute("aria-expanded", String(state.expanded));
        resizePanel(state, panel, state.expanded);
        $(state, "[data-pr-context-hint]").textContent = state.expanded
          ? "All the context, in one place"
          : "Open to see the next step";
        contextStatus(state);
      } else if (button.matches("[data-pr-review]")) {
        state.reviewed = !state.reviewed;
        button.setAttribute("aria-pressed", String(state.reviewed));
        $(state, "[data-pr-expandable]").dataset.prReviewed = String(
          state.reviewed,
        );
        $(state, "[data-pr-review-label]").textContent = state.reviewed
          ? "Review complete · Undo"
          : "Mark review complete";
        $(state, "[data-pr-review-state]").textContent = state.reviewed
          ? "Reviewed"
          : "Ready for review";
        contextStatus(state);
      }
    });
    state.root.addEventListener("keydown", (event) => {
      const accordionButton = event.target.closest(
          "[data-pr-accordion-trigger]",
        ),
        swapButton = event.target.closest("[data-pr-swap]");
      const keys = accordionButton
        ? ["ArrowDown", "ArrowUp", "Home", "End"]
        : ["ArrowRight", "ArrowLeft", "Home", "End"];
      if ((!accordionButton && !swapButton) || !keys.includes(event.key))
        return;
      event.preventDefault();
      const buttons = $$(
        state,
        accordionButton ? "[data-pr-accordion-trigger]" : "[data-pr-swap]",
      );
      const current = buttons.indexOf(accordionButton || swapButton);
      const index =
        event.key === "Home"
          ? 0
          : event.key === "End"
            ? buttons.length - 1
            : (current +
                (["ArrowDown", "ArrowRight"].includes(event.key) ? 1 : -1) +
                buttons.length) %
              buttons.length;
      buttons[index].focus({ preventScroll: true });
      if (swapButton) swapText(state, index);
    });
    document.addEventListener("visibilitychange", () => syncMotion(state));
    const motionObserver = new MutationObserver(() => syncMotion(state));
    motionObserver.observe(document.documentElement, {
      attributes: true,
      attributeFilter: ["data-motion"],
    });
    matchMedia("(prefers-reduced-motion: reduce)").addEventListener(
      "change",
      () => syncMotion(state),
    );
    syncMotion(state);
  };

  S.page({
    id: "primitives",
    title: "Primitives",
    group: "Build",
    icon: "layers",
    order: 45,
    count: 8,
    description:
      "Eight animated building blocks for disclosure, layout, type and surfaces.",
    render,
    mount,
    enter(pageRoot) {
      const state = instances.get(pageRoot);
      if (state) {
        state.active = true;
        syncMotion(state);
      }
    },
    leave(pageRoot) {
      const state = instances.get(pageRoot);
      if (state) {
        state.active = false;
        settle(state);
      }
    },
  });
})();

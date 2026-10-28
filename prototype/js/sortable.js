(() => {
  "use strict";
  const S = window.Schematic;
  let sequence = 0;

  /** Direct-child lists. onChange receives live DOM order, including rollback.
   * groups: selector within container (omit for a single list).
   * move()/mutate() share interruptible layout motion with pointer/keyboard moves.
   * setEnabled(false) cancels pickup; destroy() removes all owned work/listeners.
   */
  S.sortable = (container, {
    items = "[data-sortable-item]", handle = "[data-sortable-handle]",
    groups, onChange, announce, label = item => item.querySelector("strong")?.textContent || item.textContent.trim(),
    threshold = 6,
  } = {}) => {
    const scope = new AbortController(), animations = new Map(), landings = new Set();
    const groupList = () => groups ? [...container.querySelectorAll(groups)] : [container];
    const rows = group => [...group.children].filter(el => el.matches(items) && !el.hasAttribute("data-sortable-ghost"));
    const allRows = () => groupList().flatMap(rows);
    let enabled = true, session = null, frame = 0, lastFrame = 0, suppressedClick = null;
    const id = `sortable-help-${++sequence}`;
    const help = document.createElement("span");
    help.className = "sortable-sr";
    help.id = id;
    help.textContent = "Space or Enter picks up and drops. Arrow keys move between positions and lists. Escape cancels.";
    container.after(help);
    const live = announce ? null : document.createElement("span");
    if (live) {
      live.className = "sortable-sr";
      live.setAttribute("role", "status");
      live.setAttribute("aria-live", "polite");
      help.after(live);
    }
    const say = text => announce ? announce(text) : live.textContent = text;
    container.dataset.sortable = "";
    groupList().forEach(group => group.dataset.sortableGroup = "");
    const handles = new Map();
    allRows().forEach(row => {
      const grip = row.querySelector(handle);
      if (!grip) return;
      const saved = Object.fromEntries(["aria-describedby", "aria-pressed", "aria-keyshortcuts", "data-sortable-handle"].map(name => [name, grip.getAttribute(name)]));
      handles.set(grip, saved);
      grip.dataset.sortableHandle = "";
      grip.setAttribute("aria-describedby", [saved["aria-describedby"], id].filter(Boolean).join(" "));
      grip.setAttribute("aria-pressed", "false");
      grip.setAttribute("aria-keyshortcuts", "Space Enter ArrowUp ArrowDown ArrowLeft ArrowRight Escape");
    });
    const position = row => ({ container: row.parentElement, index: rows(row.parentElement).indexOf(row) });
    const report = (row, from, reason) => onChange?.({ item: row || null, from: from || null, to: row ? position(row) : null, reason,
      groups: groupList().map(group => ({ container: group, items: rows(group) })) });
    const describe = row => {
      const group = row.parentElement, list = rows(group);
      return `${label(row)}, position ${list.indexOf(row) + 1} of ${list.length}${groups ? `, ${group.getAttribute("aria-label") || "list"}` : ""}.`;
    };
    const motion = () => enabled && !document.hidden && !S.reduced();
    const clearAnimations = () => {
      animations.forEach(animation => animation.cancel());
      animations.clear();
    };
    const settle = () => {
      clearAnimations();
      [...landings].forEach(landing => landing());
    };
    const layout = el => {
      const rect = el.getBoundingClientRect();
      const transform = new DOMMatrixReadOnly(getComputedStyle(el).transform);
      return { left: rect.left - transform.m41, top: rect.top - transform.m42, width: rect.width, height: rect.height };
    };
    const mutate = (change, row, reason = "move") => {
      const before = new Map(allRows().map(el => [el, el.getBoundingClientRect()]));
      const focused = document.activeElement, from = row && position(row);
      clearAnimations();
      // Also absorb a layout animation started by a caller before pickup.
      allRows().forEach(el => el.getAnimations().forEach(animation => animation.cancel()));
      change();
      if (focused?.isConnected && container.contains(focused) && document.activeElement !== focused)
        focused.focus({ preventScroll: true });
      if (motion()) allRows().forEach(el => {
        if (session?.mode === "pointer" && session.row === el) return;
        const first = before.get(el), after = el.getBoundingClientRect();
        if (!first) return;
        const x = first.left - after.left, y = first.top - after.top;
        if (Math.abs(x) < 0.5 && Math.abs(y) < 0.5) return;
        const animation = el.animate([
          { transform: `translate(${x}px, ${y}px)` }, { transform: "none" },
        ], { duration: S.dur(320), easing: S.curve.out });
        animations.set(el, animation);
        animation.finished.then(() => {
          if (animations.get(el) === animation) { animations.delete(el); animation.cancel(); }
        }, () => {});
      });
      report(row, from, reason);
    };
    const reorder = (row, group, index) => {
      const others = rows(group).filter(el => el !== row);
      const before = others[Math.max(0, index)] || null;
      if (row.parentElement === group && row.nextElementSibling === before) return false;
      if (row.parentElement === group && rows(group).indexOf(row) === index) return false;
      mutate(() => group.insertBefore(row, before), row);
      return true;
    };
    const scrollParent = () => {
      for (let el = container.parentElement; el; el = el.parentElement)
        if (/(auto|scroll)/.test(getComputedStyle(el).overflowY) && el.scrollHeight > el.clientHeight) return el;
      return document.scrollingElement;
    };
    const scrollBounds = el => el === document.scrollingElement
      ? { top: 0, bottom: innerHeight }
      : { top: Math.max(0, el.getBoundingClientRect().top), bottom: Math.min(innerHeight, el.getBoundingClientRect().bottom) };
    const velocity = () => {
      if (!session?.started || session.mode !== "pointer") return 0;
      const el = session.scroller, bounds = scrollBounds(el), band = Math.min(56, (bounds.bottom - bounds.top) / 4);
      const y = session.y;
      let speed = y < bounds.top + band ? -Math.min(1, (bounds.top + band - y) / band)
        : y > bounds.bottom - band ? Math.min(1, (y - bounds.bottom + band) / band) : 0;
      if ((speed < 0 && el.scrollTop <= 0) || (speed > 0 && el.scrollTop + el.clientHeight >= el.scrollHeight - 1)) speed = 0;
      return speed * 16;
    };
    const indicator = () => {
      if (!session?.indicator) return;
      const rect = layout(session.row), group = session.row.parentElement;
      groupList().forEach(el => el.toggleAttribute("data-sortable-target", el === group));
      Object.assign(session.indicator.style, { width: `${rect.width}px`, transform: `translate(${rect.left}px, ${rect.top}px)` });
    };
    const target = () => {
      const s = session;
      if (!s?.started || s.mode !== "pointer") return;
      let group, nearest = Infinity;
      for (const candidate of groupList()) {
        const rect = candidate.getBoundingClientRect();
        if (s.x < rect.left - 24 || s.x > rect.right + 24) continue;
        const distance = Math.max(rect.top - s.y, s.y - rect.bottom, 0);
        if (distance <= 56 && distance < nearest) { group = candidate; nearest = distance; }
      }
      if (group) {
        const others = rows(group).filter(el => el !== s.row);
        let index = others.findIndex(el => { const rect = layout(el); return s.y < rect.top + rect.height / 2; });
        if (index < 0) index = others.length;
        if (reorder(s.row, group, index)) say(describe(s.row));
      }
      indicator();
    };
    const autoScroll = time => {
      frame = 0;
      const speed = velocity();
      if (!speed) { lastFrame = 0; return; }
      const delta = speed * (lastFrame ? Math.min(time - lastFrame, 32) / (1000 / 60) : 1);
      lastFrame = time;
      const scroller = session.scroller, before = scroller.scrollTop;
      scroller.scrollTop += delta;
      target();
      if (session && scroller.scrollTop !== before) frame = requestAnimationFrame(autoScroll);
      else lastFrame = 0;
    };
    const requestScroll = () => {
      if (!frame && velocity()) frame = requestAnimationFrame(autoScroll);
      else if (frame && !velocity()) { cancelAnimationFrame(frame); frame = 0; lastFrame = 0; }
    };
    const makeGhost = s => {
      const rect = s.row.getBoundingClientRect(), ghost = s.row.cloneNode(true);
      // A visual copy must never duplicate IDs, selection hooks, or live regions.
      [ghost, ...ghost.querySelectorAll("*")].forEach(el => {
        [...el.attributes].forEach(attr => { if (attr.name === "id" || (attr.name.startsWith("data-") && !["data-i", "data-anim"].includes(attr.name))) el.removeAttribute(attr.name); });
        el.removeAttribute("aria-live");
      });
      ghost.dataset.sortableGhost = "";
      ghost.setAttribute("aria-hidden", "true");
      ghost.inert = true;
      Object.assign(ghost.style, { left: `${rect.left}px`, top: `${rect.top}px`, width: `${rect.width}px`, height: `${rect.height}px`, margin: "0", transform: "none" });
      container.append(ghost);
      s.ghost = ghost;
      s.rect = rect;
      s.indicator = document.createElement("span");
      s.indicator.dataset.sortableIndicator = "";
      s.indicator.setAttribute("aria-hidden", "true");
      container.append(s.indicator);
      s.row.dataset.sortableSource = "pointer";
    };
    const start = s => {
      settle();
      s.started = true;
      container.dataset.sortableActive = s.mode;
      s.grip.setAttribute("aria-pressed", "true");
      if (s.mode === "pointer") makeGhost(s);
      else s.row.dataset.sortableSource = "keyboard";
      say(`Picked up ${describe(s.row)}`);
      indicator();
    };
    const release = s => {
      s.events.abort();
      cancelAnimationFrame(frame); frame = 0; lastFrame = 0;
      try { if (container.hasPointerCapture(s.pointerId)) container.releasePointerCapture(s.pointerId); } catch {}
      s.grip.setAttribute("aria-pressed", "false");
      s.indicator?.remove();
      groupList().forEach(group => group.removeAttribute("data-sortable-target"));
      delete container.dataset.sortableActive;
    };
    const finish = (rollback = false, immediate = false) => {
      const s = session;
      if (!s) return;
      session = null;
      release(s);
      if (!s.started) return;
      if (s.mode === "pointer") suppressedClick = { id: s.pointerId, time: performance.now(), x: s.x, y: s.y };
      if (rollback) mutate(() => s.original.forEach(({ group, order }) => order.forEach(row => group.append(row))), s.row, "cancel");
      else report(s.row, s.from, "drop");
      if (immediate || !motion()) clearAnimations();
      let removed = false;
      const remove = () => {
        if (removed) return;
        removed = true;
        animation?.cancel();
        s.ghost?.remove();
        s.row.removeAttribute("data-sortable-source");
        landings.delete(remove);
      };
      let animation;
      if (s.ghost && motion() && !immediate) {
        const rect = layout(s.row), transform = getComputedStyle(s.ghost).transform;
        animation = s.ghost.animate([{ transform }, { transform: `translate(${rect.left - s.rect.left}px, ${rect.top - s.rect.top}px)` }],
          { duration: S.dur(220), easing: S.curve.out });
        landings.add(remove);
        animation.finished.then(remove, remove);
      } else remove();
      if (s.grip.isConnected && enabled && !document.hidden) s.grip.focus({ preventScroll: true });
      say(`${rollback ? "Cancelled. Restored" : "Dropped"} ${describe(s.row)}`);
    };
    const begin = (row, grip, mode, event) => {
      finish(true, true);
      settle();
      const events = new AbortController();
      const s = session = { row, grip, mode, events, from: position(row), started: false,
        original: groupList().map(group => ({ group, order: rows(group) })),
        x: event.clientX, y: event.clientY, startX: event.clientX, startY: event.clientY,
        pointerId: event.pointerId, scroller: scrollParent() };
      const listen = (name, fn, options = {}) => document.addEventListener(name, fn, { ...options, signal: events.signal });
      listen("keydown", e => {
        if (e.key === "Escape") { e.preventDefault(); e.stopPropagation(); finish(true); }
        else if (e.key === "Tab" && s.mode === "keyboard") finish();
      }, { capture: true });
      if (mode === "pointer") {
        listen("pointermove", e => {
          if (e.pointerId !== s.pointerId) return;
          s.x = e.clientX; s.y = e.clientY;
          if (!s.started && Math.hypot(s.x - s.startX, s.y - s.startY) < threshold) return;
          e.preventDefault();
          if (!s.started) start(s);
          s.ghost.style.transform = `translate(${s.x - s.startX}px, ${s.y - s.startY}px)`;
          target(); requestScroll();
        }, { capture: true, passive: false });
        listen("pointerup", e => { if (e.pointerId === s.pointerId) finish(); }, { capture: true });
        listen("pointercancel", e => { if (e.pointerId === s.pointerId) finish(true, true); }, { capture: true });
        listen("lostpointercapture", e => { if (e.pointerId === s.pointerId) finish(true, true); });
        listen("scroll", () => { if (s.started) indicator(); }, { capture: true });
        try { container.setPointerCapture(s.pointerId); } catch {}
      } else start(s);
      grip.focus({ preventScroll: true });
    };
    container.addEventListener("pointerdown", event => {
      if (!enabled || event.button !== 0 || event.isPrimary === false) return;
      const grip = event.target.closest(handle), row = grip?.closest(items);
      if (!row || !container.contains(row) || grip.disabled || !groupList().includes(row.parentElement)) return;
      event.preventDefault();
      begin(row, grip, "pointer", event);
    }, { signal: scope.signal });
    container.addEventListener("click", event => {
      if (suppressedClick && event.pointerId === suppressedClick.id && performance.now() - suppressedClick.time < 80
        && Math.hypot(event.clientX - suppressedClick.x, event.clientY - suppressedClick.y) < 2) {
        suppressedClick = null; event.preventDefault(); event.stopImmediatePropagation(); return;
      }
      if (session && !event.target.closest(handle)) finish(true);
    }, { capture: true, signal: scope.signal });
    container.addEventListener("keydown", event => {
      const grip = event.target.closest(handle), row = grip?.closest(items);
      if (!enabled || !row || !container.contains(row)) return;
      if ([" ", "Enter"].includes(event.key)) {
        event.preventDefault(); event.stopPropagation();
        if (event.repeat) return;
        if (session?.started) finish(); else begin(row, grip, "keyboard", event);
        return;
      }
      if (session?.mode !== "keyboard" || !["ArrowUp", "ArrowDown", "ArrowLeft", "ArrowRight", "Home", "End"].includes(event.key)) return;
      event.preventDefault(); event.stopPropagation();
      const group = row.parentElement, lists = groupList(), index = rows(group).indexOf(row), groupIndex = lists.indexOf(group);
      const up = event.key === "ArrowUp", down = event.key === "ArrowDown";
      let destination = group, position = index;
      if (event.key === "Home") position = 0;
      else if (event.key === "End") position = rows(group).length - 1;
      else if (up && index > 0) position--;
      else if (down && index < rows(group).length - 1) position++;
      else {
        const direction = ["ArrowUp", "ArrowLeft"].includes(event.key) ? -1 : 1;
        destination = lists[groupIndex + direction];
        if (!destination) return;
        position = up ? rows(destination).length : down ? 0 : Math.min(index, rows(destination).length);
      }
      if (reorder(row, destination, position)) say(describe(row));
      grip.focus({ preventScroll: true });
      row.scrollIntoView({ block: "nearest", inline: "nearest", behavior: "instant" });
    }, { signal: scope.signal });
    const syncMotion = () => { if (S.reduced()) settle(); };
    const preference = new MutationObserver(syncMotion);
    preference.observe(document.documentElement, { attributes: true, attributeFilter: ["data-motion"] });
    matchMedia("(prefers-reduced-motion: reduce)").addEventListener("change", syncMotion, { signal: scope.signal });
    document.addEventListener("visibilitychange", () => {
      if (document.hidden) { finish(true, true); settle(); }
    }, { signal: scope.signal });

    return {
      get active() { return !!session?.started; },
      cancel: () => finish(true, true),
      settle,
      setEnabled(value) { enabled = !!value; if (!enabled) { finish(true, true); settle(); } },
      move(row, group, index) { finish(true, true); settle(); return reorder(row, group, index); },
      mutate(change) { finish(true, true); settle(); mutate(change, null, "programmatic"); },
      destroy() {
        enabled = false; finish(true, true); settle(); scope.abort(); preference.disconnect(); help.remove(); live?.remove();
        delete container.dataset.sortable;
        groupList().forEach(group => group.removeAttribute("data-sortable-group"));
        handles.forEach((saved, grip) => {
          Object.entries(saved).forEach(([name, value]) => value === null ? grip.removeAttribute(name) : grip.setAttribute(name, value));
        });
      },
    };
  };
})();

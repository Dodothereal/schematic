/* Palette preferences extend the app without replacing its theme or hue logic. */
(() => {
  const S = window.Schematic;
  const root = document.documentElement;
  const current = () => root.dataset.palette === "mono" ? "mono" : "color";
  let control;

  S.setPalette = (palette) => {
    if (palette !== "color" && palette !== "mono") return;
    const changed = root.dataset.palette !== palette;
    root.dataset.palette = palette;
    S.store.set("palette", palette);
    control?.set(palette, { silent: true });
    if (changed) root.dispatchEvent(new CustomEvent("rootattr", {
      bubbles: true,
      detail: { attributeName: "data-palette", value: palette },
    }));
  };

  // Swatches, the hue slider and commands all keep the original implementation.
  // Choosing a hue is an explicit return to the Color interface.
  const setHue = S.setHue;
  S.setHue = function (...args) {
    S.setPalette("color");
    return setHue.apply(this, args);
  };

  // app.js registered boot first, so #tune-pop is populated before we extend it.
  document.addEventListener("DOMContentLoaded", () => {
    const panel = document.querySelector("#tune-pop");
    if (!panel) return;
    const section = document.createElement("div");
    section.className = "pop-section";
    section.innerHTML = `
      <div class="pop-label" id="palette-label">Interface palette</div>
      <div class="seg seg-block" id="palette-seg" role="group" aria-labelledby="palette-label" aria-describedby="palette-help">
        <button type="button" class="seg-btn" data-value="color">Color</button>
        <button type="button" class="seg-btn" data-value="mono">Mono</button>
      </div>
      <p class="t-caption" id="palette-help">Mono keeps chart series in color.</p>`;
    panel.prepend(section);
    const accent = panel.querySelector(".swatches")?.closest(".pop-section");
    if (accent) {
      accent.dataset.accentOptions = "";
      const inner = document.createElement("div"); inner.className = "palette-accent-inner";
      while (accent.firstChild) inner.append(accent.firstChild);
      accent.append(inner);
    }
    control = S.segmented(section.querySelector("#palette-seg"), {
      value: current(),
      onChange: S.setPalette,
    });
    // Keep external setter calls (and direct root-attribute changes) in sync.
    const sync = () => { control.set(current(), { silent: true }); if (accent) { accent.inert = current() === "mono"; accent.setAttribute("aria-hidden", String(accent.inert)); } };
    new MutationObserver(sync)
      .observe(root, { attributes: true, attributeFilter: ["data-palette"] });
    sync();
  });
})();

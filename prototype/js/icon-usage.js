/* Keep the shell's control icons on the same registry as the specimens.
   Include after icons.js. Decorative separators and the theme morph stay put. */
(() => {
  const S = window.Schematic;
  const controls = [
    ["#guide-btn", "book-open"],
    ["#menu-btn", "menu"],
    [".search-trigger", "search"],
    ["#tune-btn", "sliders"],
    [".topbar-actions > [data-open-cmdk]", "command"],
  ];
  const install = () => controls.forEach(([selector, name]) => {
    const host = document.querySelector(selector);
    const previous = host?.querySelector(":scope > svg");
    if (!previous) return;
    const template = document.createElement("template");
    template.innerHTML = S.icon(name, {
      size: Number(previous.getAttribute("width")) || 18,
      cls: [...previous.classList].filter((cls) => cls !== "ic" && cls !== "is-anim").join(" "),
    });
    const next = template.content.firstElementChild;
    for (const { name: attribute, value } of previous.attributes) {
      if (/^(id|role|tabindex|aria-.+|data-tip.*)$/.test(attribute)) next.setAttribute(attribute, value);
    }
    previous.replaceWith(next);
  });
  if (document.readyState === "loading") document.addEventListener("DOMContentLoaded", install, { once: true });
  else install();
})();

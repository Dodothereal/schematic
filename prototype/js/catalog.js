/* The catalog is derived from the actual specimens, so names and links cannot
   silently drift away from the working collection. No page is mounted here. */
(() => {
  const S = window.Schematic;
  let signature = "",
    entries = [];
  S.catalog = () => {
    const pages = S.pages.filter((page) => page.id !== "overview");
    const next = pages.map((page) => page.id).join("|");
    if (next === signature) return entries;
    signature = next;
    entries = pages.flatMap((page) => {
      const template = document.createElement("template");
      template.innerHTML = page.render();
      return [...template.content.querySelectorAll(".demo")].map(
        (demo, index) => ({
          id: `${page.id}-${index}`,
          page: page.id,
          collection: page.title,
          icon: page.icon,
          index,
          title:
            demo.querySelector(".demo-title")?.textContent.trim() || page.title,
          desc:
            demo.querySelector(".demo-desc")?.textContent.trim() ||
            page.description,
          section: demo.closest("section[id]")?.id || "",
          family: ["components", "forms", "navigation"].includes(page.id)
            ? "controls"
            : ["primitives", "motion", "icons"].includes(page.id)
              ? "motion"
              : page.id === "data"
                ? "data"
                : page.id === "overlays"
                  ? "feedback"
                  : "foundations",
        }),
      );
    });
    const order = [
      "primitives",
      "navigation",
      "components",
      "forms",
      "data",
      "overlays",
      "motion",
      "icons",
      "foundations",
    ];
    entries.sort((a, b) => order.indexOf(a.page) - order.indexOf(b.page));
    return entries;
  };
})();

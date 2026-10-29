/* The catalog is derived from the actual specimens, so names and links cannot
   silently drift away from the working collection. No page is mounted here. */
(() => {
  const S = window.Schematic;
  const specimenIcons = {
    "Coding workspace": "code",
    "Accordion": "chevrons-up-down",
    "Collapsible": "chevron-down",
    "Pinned list": "pin",
    "Sortable list": "grip",
    "Text swap": "type",
    "Animated counter": "plus",
    "Flip card": "refresh",
    "Expandable surface": "window",
    "Travelling navigation menu": "menu",
    "Searchable collection picker": "search",
    "Living breadcrumb trail": "folder",
    "Directional pagination": "arrow-right",
    "Keyboard file tree": "git-branch",
    "Workspace action dock": "grid",
    "Three-step workspace wizard": "check-circle",
    "Project direction carousel": "chevrons-up-down",
    "Button hierarchy": "cursor",
    "A size for the moment": "ruler",
    "Action button": "play",
    "Tabs": "window",
    "Animated icons": "cursor",
    "Pending → resolved": "check-circle",
    "Copy with certainty": "copy",
    "A deliberate confirmation": "lock",
    "A little appreciation": "heart",
    "One action, a few alternatives": "more",
    "A toolbar that travels with you": "sliders",
    "Workspace preferences": "sliders",
    "Segmented, connected": "sidebar",
    "Tabs with a sense of direction": "arrow-right",
    "Labels that make room": "bookmark",
    "People, with presence": "users",
    "Input sizes": "input",
    "States with something to say": "alert",
    "Validation, in context": "check-circle",
    "Auto-growing textarea": "type",
    "Show the requirements": "lock",
    "One code, six small steps": "code",
    "Let the platform help": "chevron-down",
    "Every step accounted for": "plus",
    "A single point of control": "sliders",
    "Two bounds, one interval": "sliders",
    "Local files / simulated progress": "upload",
    "A continuous story": "wave",
    "Change the relationship": "chart-bar",
    "Keep your place": "trending",
    "A part of the whole": "pie",
    "Progress that resolves": "gauge",
    "Origin morph": "window",
    "Modal dialog": "window",
    "Destructive confirmation": "trash",
    "Side drawer": "sidebar",
    "Bottom sheet": "arrow-up",
    "Dropdown + submenu": "menu",
    "Context menu": "cursor",
    "Intent hovercard": "user",
    "Tooltip toolbar": "info",
    "Toast tones": "bell",
    "Promise feedback": "check-circle",
    "Single accordion": "chevron-down",
    "Multiple accordion": "chevrons-up-down",
    "Show-more disclosure": "plus",
    "Skeleton → content": "document",
    "Easing workbench": "wave",
    "Spring bench": "wave",
    "Staggered arrival": "grid",
    "State continuity": "arrow-right",
    "Inside controls": "cursor",
    "Snippet": "code",
    "Spacing scale": "ruler",
    "Control rhythm": "grid",
  };
  /** Shared with specimen headings; every answer is an actual registry name. */
  S.specimenIcon = (title, page) => {
    const collection = typeof page === "string" ? S.pages.find((entry) => entry.id === page) : page;
    const icon = specimenIcons[title] || collection?.icon;
    return S.iconNames.includes(icon) ? icon : "cube";
  };
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
        (demo, index) => {
          const title = demo.querySelector(".demo-title")?.textContent.trim() || page.title;
          return {
            id: `${page.id}-${index}`,
            page: page.id,
            collection: page.title,
            icon: S.specimenIcon(title, page),
            index,
            title,
            desc:
              demo.querySelector(".demo-desc")?.textContent.trim() ||
              page.description,
            section: demo.closest("section[id]")?.id || "",
            family: page.id === "workspace"
              ? "examples"
              : ["components", "forms", "navigation"].includes(page.id)
                ? "controls"
                : ["primitives", "motion", "icons"].includes(page.id)
                  ? "motion"
                  : page.id === "data"
                    ? "data"
                    : page.id === "overlays"
                      ? "feedback"
                      : "foundations",
          };
        },
      );
    });
    const order = [
      "workspace",
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

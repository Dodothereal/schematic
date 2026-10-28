(() => {
  const S = window.Schematic;
  const colors = ["bg", "bg-sunken", "surface", "surface-2", "surface-3", "border", "border-strong", "text", "text-2", "accent", "accent-soft", "accent-ring", "success", "warning", "danger", "info", "chart-1", "chart-2", "chart-3", "chart-4"];
  const scales = [["6xl", "One language.", "Display"], ["4xl", "Designed to belong.", "Page title"], ["2xl", "A clear hierarchy", "Section title"], ["lg", "Motion follows intention.", "Lead / heading"], ["md", "A shared rhythm for every screen, every state and every platform.", "Body"], ["sm", "The details should feel inevitable.", "Supporting"], ["xs", "Schematic / reference edition", "Caption"]];
  const download = (name, text, type) => {
    const url = URL.createObjectURL(new Blob([text], { type }));
    const a = document.createElement("a"); a.href = url; a.download = name; a.click(); setTimeout(() => URL.revokeObjectURL(url), 1000);
  };
  const currentCSS = () => `/* Schematic: resolved snapshot of the current theme. */\n:root {\n${S.tokenNames().map((k) => `  ${k}: ${S.token(k)};`).join("\n")}\n}\n`;
  S.page({
    id: "foundations", title: "Foundations", group: "Foundations", icon: "droplet", order: 10,
    description: "Colour, typography, spacing, surfaces and the tokens that connect them.",
    render: () => `<div class="pg-foundations">
      ${S.pageHead({ icon: "droplet", eyebrow: "Foundations / 01", title: "Everything starts here.", lead: "Colors, typography, spacing, borders and shadows.", meta: ['<span class="badge badge-accent">4px rhythm</span>', '<span class="badge">OKLCH colour</span>', '<span class="badge">Semantic tokens</span>'] })}
      ${S.section({ id: "foundations-color", num: "01", title: "Colour has a job", body: `<div class="fd-palette">${colors.map((c) => `<button class="fd-swatch" data-token="${c}" aria-label="Copy ${c} token"><span class="fd-swatch-color" style="background:var(--${c})"></span><span class="t-mono">--${c}</span><span class="fd-copy">${S.icon("copy", { size: 12 })}</span></button>`).join("")}</div><div class="row fd-theme"><button class="btn btn-secondary" data-theme-choice="light">${S.icon("sun")} Light</button><button class="btn btn-secondary" data-theme-choice="dark">${S.icon("moon")} Dark</button><span class="t-caption">Both themes consume the same semantic names.</span></div>` })}
      ${S.section({ id: "foundations-type", num: "02", title: "A voice at every size", body: `<div class="card fd-type">${scales.map(([size, text, label]) => `<div class="fd-type-row"><div class="fd-type-meta"><span class="t-mono">--t-${size}</span><span class="t-caption">${label}</span></div><div class="fd-specimen" style="font-size:var(--t-${size});line-height:var(--lh-${size})">${text}</div></div>`).join("")}</div>` })}
      ${S.section({ id: "foundations-space", num: "03", title: "Space is the structure", body: `<div class="grid cols-2">${S.demo({ title: "Spacing scale", desc: "The same scale governs every page.", stage: "is-plain is-start", body: `<div class="fd-spaces">${[1, 2, 3, 4, 5, 6, 8, 10, 12, 16, 20, 24].map((n) => `<div class="fd-space-row"><code>--s-${n}</code><span class="fd-space-bar" style="width:var(--s-${n})"></span><span class="t-caption t-mono">${n * 4}px</span></div>`).join("")}</div>` })}${S.demo({ title: "Control rhythm", desc: "A shared height aligns buttons, inputs and selectors.", body: `<div class="stack fd-controls">${["sm", "md", "lg", "xl"].map((size) => `<div class="row"><span class="t-mono t-caption">${size}</span><input aria-label="${size} sample field" class="fd-input" style="height:var(--control-${size})" placeholder="Project name"><button class="btn btn-secondary ${size === "md" ? "" : "btn-" + size}" data-control-demo="${size}">${S.icon("plus")} Create</button></div>`).join("")}</div>` })}</div>` })}
      ${S.section({ id: "foundations-surfaces", num: "04", title: "Depth, with restraint", body: `<div class="grid cols-3">${["sm", "md", "lg"].map((s) => `<div class="card fd-elevation" style="box-shadow:var(--shadow-${s});border-radius:var(--r-${s})"><span class="fd-layer">${S.icon("layers", { size: 24 })}</span><h3 class="t-h3">Elevation ${s.toUpperCase()}</h3><code>--shadow-${s}</code><span class="t-caption">Radius: var(--r-${s})</span></div>`).join("")}</div><div class="row"><span class="t-small muted">Try a different character</span>${["sharp", "default", "round"].map((r) => `<button class="btn btn-secondary" data-radius-choice="${r}" aria-pressed="false">${r[0].toUpperCase() + r.slice(1)}</button>`).join("")}</div>` })}
      ${S.section({ id: "foundations-export", num: "05", title: "Take the system with you", body: `<div class="card fd-export"><div class="stack"><span class="badge badge-success">HTML reference</span><h3 class="t-h3">Use names. Keep meaning.</h3><pre><code>.your-control {
  color: var(--text);
  background: var(--surface);
  padding: var(--s-3) var(--s-4);
  border-radius: var(--r-md);
  transition: opacity var(--d-fast) var(--ease-out);
}</code></pre><div class="row"><button class="btn btn-primary" data-export="css">${S.icon("download")} Download CSS snapshot</button><button class="btn btn-secondary" data-export="json">${S.icon("code")} Export JSON</button><button class="btn btn-ghost" data-export="copy">${S.icon("copy")} Copy CSS</button></div><p class="t-caption" role="status" id="foundations-export-status">Framework adapters are planned. This working reference is plain HTML, CSS and JavaScript.</p></div></div>` })}
    </div>`,
    mount(root) {
      const status = S.$("#foundations-export-status", root);
      root.addEventListener("click", async (e) => {
        const token = e.target.closest("[data-token]");
        if (token) { const text = `var(--${token.dataset.token})`; const ok = await S.copy(text); S.toast({ title: ok ? "Token copied" : "Clipboard unavailable", desc: text, tone: ok ? "success" : "warning" }); }
        const theme = e.target.closest("[data-theme-choice]"); if (theme) S.setTheme(theme.dataset.themeChoice, theme);
        const radius = e.target.closest("[data-radius-choice]");
        if (radius) { S.setRadius(radius.dataset.radiusChoice); S.$$("[data-radius-choice]", root).forEach((b) => b.setAttribute("aria-pressed", String(b === radius))); }
        const control = e.target.closest("[data-control-demo]"); if (control) S.toast({ title: "Control selected", desc: `Size ${control.dataset.controlDemo.toUpperCase()} shares its height with the adjacent field.`, tone: "info" });
        const exp = e.target.closest("[data-export]"); if (!exp) return;
        if (exp.dataset.export === "css") { download("schematic-theme.css", currentCSS(), "text/css"); status.textContent = "CSS snapshot downloaded."; }
        if (exp.dataset.export === "json") { const tokens = Object.fromEntries(S.tokenNames().map((k) => [k.slice(2), S.token(k)])); download("schematic-tokens.json", JSON.stringify({ theme: document.documentElement.dataset.theme, tokens }, null, 2), "application/json"); status.textContent = "Token snapshot downloaded."; }
        if (exp.dataset.export === "copy") { const ok = await S.copy(currentCSS()); status.textContent = ok ? "CSS snapshot copied." : "Clipboard unavailable. Use Download CSS instead."; }
      });
    },
    enter(root) { S.$$("[data-radius-choice]", root).forEach((b) => b.setAttribute("aria-pressed", String(b.dataset.radiusChoice === (document.documentElement.dataset.radius || "default")))); },
  });
})();

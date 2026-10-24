"use strict";

// Each focused view retains the original Tease controls and model; it only
// hides unrelated fields and results when the demo is embedded in API docs.
(() => {
    const previews = {
        slider: ["input-1", "output-0"],
        dropdown: ["input-2", "output-1"],
        probabilities: ["input-0", "output-2"],
    };
    const name = new URLSearchParams(window.location.search).get("preview");
    if (!Object.hasOwn(previews, name)) return;

    document.documentElement.dataset.preview = name;
    document.addEventListener("DOMContentLoaded", () => {
        const [visibleInput, visibleOutput] = previews[name];
        for (const id of ["input-0", "input-1", "input-2"]) {
            if (id !== visibleInput) document.getElementById(id).closest(".field").hidden = true;
        }
        for (const id of ["output-0", "output-1", "output-2"]) {
            if (id !== visibleOutput) document.getElementById(id).closest(".field").hidden = true;
        }
        for (const selector of ["h1", ".description", ".live-control", "#submit", "#status", "#download-status", "main > p.help"]) {
            document.querySelector(selector).hidden = true;
        }
    }, { once: true });
})();

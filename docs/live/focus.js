"use strict";

// Each focused view retains the original Tease controls and model; it only
// hides unrelated fields and results when the demo is embedded in API docs.
(() => {
    const previews = {
        slider: { inputs: ["input-1"], outputs: ["output-0"] },
        dropdown: { inputs: ["input-2"], outputs: ["output-1"] },
        probabilities: { inputs: ["input-0"], outputs: ["output-2"] },
        hero: { inputs: ["input-1", "input-2"], outputs: ["output-0", "output-1"] },
    };
    const name = new URLSearchParams(window.location.search).get("preview");
    if (!Object.hasOwn(previews, name)) return;

    document.documentElement.dataset.preview = name;
    document.addEventListener("DOMContentLoaded", () => {
        const { inputs, outputs } = previews[name];
        for (const id of ["input-0", "input-1", "input-2"]) {
            if (!inputs.includes(id)) document.getElementById(id).closest(".field").hidden = true;
        }
        for (const id of ["output-0", "output-1", "output-2"]) {
            if (!outputs.includes(id)) document.getElementById(id).closest(".field").hidden = true;
        }
        for (const selector of ["h1", ".description", ".live-control", "#submit", "#status", "#download-status", "main > p.help"]) {
            document.querySelector(selector).hidden = true;
        }
    }, { once: true });
})();

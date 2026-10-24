"use strict";

// Browser-only counterpart to examples/preview.rs. The Tease controls and
// result renderer in index.html are generated from the crate's own HTML/JS.
const teasePreviewModel = (() => {
    function error(field, label, message) {
        return { kind: "error", field, message: `${label}: ${message}` };
    }

    function predict(inputs) {
        if (!Array.isArray(inputs) || inputs.length !== 3) {
            return { kind: "error", field: null, message: "Invalid request. Submit all fields again." };
        }
        if (!String(inputs[0]).trim()) return error(0, "Gain", "A value is required.");
        const gain = Number(inputs[0]);
        if (!Number.isFinite(gain)) return error(0, "Gain", "Enter a finite number.");
        if (gain < 0.1 || gain > 5) return error(0, "Gain", "Value is outside the allowed range.");

        if (!String(inputs[1]).trim()) return error(1, "Offset", "A value is required.");
        const offset = Number(inputs[1]);
        if (!Number.isFinite(offset)) return error(1, "Offset", "Enter a finite number.");
        if (offset < -3 || offset > 3) return error(1, "Offset", "Value is outside the slider range.");
        const steps = (offset + 3) / 0.1;
        if (Math.abs(steps - Math.round(steps)) > 1e-6) {
            return error(1, "Offset", "Value must follow the slider step.");
        }

        if (!["0", "1", "2"].includes(inputs[2])) {
            return error(2, "Plot style", "Choose one of the available options.");
        }
        const kind = ["line", "scatter", "bar"][Number(inputs[2])];
        const response = x => 1 / (1 + Math.exp(-gain * x - offset));
        const probability = response(1);
        const series = [
            {
                name: "response",
                points: Array.from({ length: 41 }, (_, i) => {
                    const x = -3 + i * 0.15;
                    return [x, response(x)];
                }),
            },
        ];
        if (kind !== "bar") {
            series.push({ name: "reference", points: [[-3, 0.5], [3, 0.5]] });
        }
        const plot = {
            kind,
            x_label: "Input x",
            y_label: "Response probability",
            series,
        };
        return {
            kind: "success",
            values: [
                probability.toFixed(4),
                { kind: "plot", plot },
                {
                    kind: "scores",
                    probabilities: true,
                    scores: [
                        { label: "Positive", score: probability },
                        { label: "Negative", score: 1 - probability },
                    ],
                },
            ],
        };
    }

    if (typeof window !== "undefined") {
        window.ipc = {
            postMessage(message) {
                if (message === "ready") {
                    queueMicrotask(() => window.tease.ready(null));
                    return;
                }
                const request = JSON.parse(message);
                const reply = predict(request.inputs);
                queueMicrotask(() => window.tease.receive({
                    request_id: request.request_id,
                    fatal: false,
                    ...reply,
                }));
            },
        };
    }
    return { predict };
})();

if (typeof module !== "undefined") module.exports = teasePreviewModel;

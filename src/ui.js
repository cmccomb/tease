"use strict";
(() => {
    const form = document.getElementById("tease-form");
    const inputs = Array.from(document.querySelectorAll("[data-input]"));
    const outputs = Array.from(document.querySelectorAll("[data-output]"));
    const button = document.getElementById("submit");
    const status = document.getElementById("status");
    const error = document.getElementById("error");
    const config = JSON.parse(document.getElementById("tease-config")?.textContent || "{}");
    const preset = document.getElementById("preset");
    const pin = document.getElementById("pin");
    const scenarios = document.getElementById("scenarios");
    const pinned = [];
    let current = null, submitted = null;
    const render = (output, value) => window.teaseView ? window.teaseView.render(output, value) : output.textContent = value;
    const read = input => input.type === "file" ? input._payload || "" : input.type === "checkbox" ? String(input.checked) : input.multiple ? JSON.stringify(Array.from(input.selectedOptions, option => Number(option.value))) : input.value;
    function describe(input) {
        if (input.type === "file") return input._fileName || "No file selected";
        if (input.multiple) return Array.from(input.selectedOptions, option => option.textContent).join(", ") || "None";
        if (input.tagName === "SELECT") return input.selectedOptions[0]?.textContent || "";
        return read(input);
    }
    function compare() {
        if (!scenarios) return;
        scenarios.replaceChildren();
        const node = window.teaseView.node;
        const all = pinned.map((snapshot, index) => ({snapshot, index}));
        if (current && pinned.length) all.push({snapshot:current, index:null});
        all.forEach(({snapshot,index}) => {
            const card = node("article", undefined, scenarios); card.className = "scenario";
            node("h3", index === null ? "Current result" : `Pinned ${index + 1}`, card);
            const details = node("details", undefined, card); node("summary", snapshot.name || "Inputs", details);
            snapshot.inputs.forEach((value,i) => node("p", `${config.inputLabels[i]}: ${value}`, details));
            snapshot.values.forEach((value,i) => {
                node("h4", config.outputs[i].label, card);
                render(node("div", undefined, card), value);
                if (config.outputs[i].numeric && pinned.length) {
                    const delta = Number(value) - Number(pinned[0].values[i]);
                    node("p", `Difference: ${delta >= 0 ? "+" : ""}${Number(delta.toPrecision(6))}`, card).className = "help";
                }
            });
            if (index !== null) {
                const remove = node("button", "Remove", card); remove.type = "button";
                remove.addEventListener("click", () => { pinned.splice(index,1); compare(); if (pin) pin.disabled = !current || pinned.length >= 3; });
            }
        });
    }
    if (pin) pin.addEventListener("click", () => {
        if (!current || pinned.length >= 3) return;
        pinned.push(JSON.parse(JSON.stringify(current))); pin.disabled = pinned.length >= 3; compare();
    });
    if (preset) preset.addEventListener("change", () => {
        const scenario = config.presets[Number(preset.value)];
        if (preset.value === "" || !scenario) return;
        scenario.inputs.forEach((value,i) => {
            const input = inputs[i];
            if (input.type === "file") { input.value = ""; window.teaseFiles.apply(input,value,document.getElementById(`preview-${i}`)); }
            else if (input.type === "checkbox") input.checked = value === "true";
            else if (input.multiple) { const selected = JSON.parse(value); Array.from(input.options).forEach(option => option.selected = selected.includes(Number(option.value))); }
            else input.value = value;
            if (input.type === "range") input.nextElementSibling.textContent = input.value;
        });
        clearErrors(); status.textContent = `Loaded ${scenario.name}. Select Predict to run.`;
        enable(initialized && !unavailable && activeRequest === null, activeRequest !== null);
    });
    let initialized = false;
    let unavailable = false;
    let activeRequest = null;
    let nextRequest = 0;
    function enable(enabled, busy = false) {
        button.disabled = !enabled || inputs.some(input => input._reading);
        inputs.forEach(input => { input.disabled = !enabled; });
        if (preset) preset.disabled = !enabled;
        form.setAttribute("aria-busy", String(busy));
    }
    function clearErrors() {
        error.textContent = "";
        inputs.forEach((input, index) => {
            input.removeAttribute("aria-invalid");
            document.getElementById(`error-${index}`).textContent = "";
        });
    }
    function ready(message) {
        if (initialized) return;
        initialized = true;
        unavailable = Boolean(message);
        enable(!message);
        status.textContent = message ? "Model unavailable." : "Ready.";
        if (message) error.textContent = message;
    }
    function receive(response) {
        // A late or duplicate response must not overwrite newer state or unlock a job.
        if (response.request_id !== activeRequest) return;
        if (activeRequest === null && response.request_id !== null) return;
        activeRequest = null;
        unavailable = Boolean(response.fatal);
        enable(!response.fatal);
        clearErrors();
        if (response.kind === "success") {
            outputs.forEach((output, index) => render(output, response.values[index]));
            current = { ...submitted, values: response.values };
            if (pin) pin.disabled = pinned.length >= 3;
            compare();
            status.textContent = "Prediction complete.";
        } else {
            outputs.forEach(output => { output.textContent = ""; });
            status.textContent = response.fatal ? "Model unavailable." : "Prediction not completed.";
            error.textContent = response.message;
            if (!response.fatal && Number.isInteger(response.field) && inputs[response.field]) {
                const input = inputs[response.field];
                input.setAttribute("aria-invalid", "true");
                document.getElementById(`error-${response.field}`).textContent = response.message;
                input.focus();
            } else {
                error.focus();
            }
        }
    }
    let exporting = false;
    function downloadFinished(message) {
        exporting = false;
        const notice = document.getElementById("download-status"); if (notice) notice.textContent = message;
    }
    window.tease = { ready, receive, downloadFinished, exportCsv(csv) {
        if (exporting) return;
        exporting = true;
        try { window.ipc.postMessage(JSON.stringify({export_csv:csv})); }
        catch (_) { downloadFinished("Could not request CSV export. Please try again."); }
    } };
    inputs.forEach((input,index) => {
        if (input.type === "file") input.addEventListener("change", async () => {
            const ticket = input._ticket = (input._ticket || 0) + 1;
            input._payload = ""; input._reading = true; button.disabled = true;
            const file = input.files[0];
            const preview = document.getElementById(`preview-${index}`); preview.replaceChildren();
            try {
                const raw = await window.teaseFiles.read(file,input.dataset.fileKind);
                if (input._ticket !== ticket) return;
                window.teaseFiles.apply(input,raw,preview,file?.name || "No file selected",false);
                clearErrors(); status.textContent = raw ? "File loaded. Select Predict to run." : "Select a file.";
            } catch (failure) {
                if (input._ticket !== ticket) return;
                input._payload = ""; input._reading = false;
                error.textContent = failure.message; status.textContent = "File not loaded.";
            }
            enable(initialized && !unavailable && activeRequest === null);
        });
        input.addEventListener("change", () => { if (preset) preset.value = ""; });
        if (input.type === "range") {
            input.addEventListener("input", () => { input.nextElementSibling.textContent = input.value; });
        }
    });
    form.addEventListener("submit", event => {
        event.preventDefault();
        if (!initialized || button.disabled || activeRequest !== null) return;
        clearErrors();
        outputs.forEach(output => { output.textContent = ""; });
        submitted = { inputs: inputs.map(describe), name: preset && preset.value !== "" ? config.presets[Number(preset.value)].name : "Inputs" };
        current = null; if (pin) pin.disabled = true; compare();
        activeRequest = ++nextRequest;
        enable(false, true);
        status.textContent = "Computing…";
        try {
            window.ipc.postMessage(JSON.stringify({ request_id: activeRequest, inputs: inputs.map(read) }));
        } catch (_) {
            receive({ request_id: activeRequest, kind: "error", field: null, message: "Could not submit the prediction. Please try again." });
        }
    });
    enable(false, true);
    status.textContent = "Preparing model…";
    try { window.ipc.postMessage("ready"); }
    catch (_) { ready("Could not connect to the model. Restart the demo."); }
})();

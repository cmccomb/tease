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
        changed();
    });
    const live = document.getElementById("live");
    let initialized = false, unavailable = false;
    let activeRequest = null, nextRequest = 0;
    let revision = 0, pending = null, timer = null;
    const isLive = () => Boolean(live?.checked);
    const filesReady = () => !inputs.some(input => input._reading || (input.type === "file" && !input._payload));
    function cancelPending() {
        if (timer !== null) clearTimeout(timer);
        timer = null; pending = null;
    }
    function enable(enabled, busy = false) {
        button.disabled = !enabled || inputs.some(input => input._reading);
        const editable = enabled || (isLive() && initialized && !unavailable);
        inputs.forEach(input => { input.disabled = !editable; });
        if (preset) preset.disabled = !editable;
        if (live) live.disabled = !initialized || unavailable;
        form.setAttribute("aria-busy", String(busy && !editable));
        document.getElementById("results")?.setAttribute("aria-busy", String(busy));
    }
    function clearErrors() {
        error.textContent = "";
        inputs.forEach((input,index) => {
            input.removeAttribute("aria-invalid"); document.getElementById(`error-${index}`).textContent = "";
        });
    }
    function snapshot() {
        return { raw: inputs.map(read), inputs: inputs.map(describe), revision,
            name: preset && preset.value !== "" ? config.presets[Number(preset.value)].name : "Inputs" };
    }
    function schedule() {
        if (!isLive() || !initialized || unavailable || !filesReady() || inputs.some(input => input._composing)) return;
        if (timer !== null) clearTimeout(timer);
        timer = setTimeout(() => {
            timer = null;
            if (!isLive() || unavailable || !filesReady()) return;
            const job = snapshot();
            if (activeRequest !== null) { pending = job; status.textContent = "Waiting to run the latest inputs…"; }
            else send(job);
        }, config.liveDelay);
    }
    function changed() {
        revision++;
        cancelPending();
        schedule();
    }
    function ready(message) {
        if (initialized) return;
        initialized = true; unavailable = Boolean(message);
        enable(!message);
        status.textContent = message ? "Model unavailable." : "Ready.";
        if (message) error.textContent = message;
        else schedule();
    }
    function send(job) {
        if (activeRequest !== null || unavailable || !initialized || inputs.some(input => input._reading)) return;
        clearErrors(); outputs.forEach(output => { output.textContent = ""; });
        submitted = {inputs:job.inputs, name:job.name, revision:job.revision};
        current = null; if (pin) pin.disabled = true; compare();
        activeRequest = ++nextRequest;
        enable(false,true); status.textContent = "Computing…";
        try { window.ipc.postMessage(JSON.stringify({request_id:activeRequest, inputs:job.raw})); }
        catch (_) { receive({request_id:activeRequest,kind:"error",field:null,message:"Could not submit the prediction. Please try again."}); }
    }
    function flush() {
        if (!pending || !isLive() || unavailable || !filesReady()) return;
        const job = pending; pending = null;
        if (job.revision === revision) send(job);
    }
    function receive(response) {
        if (response.request_id !== activeRequest) return;
        if (activeRequest === null && response.request_id !== null) return;
        activeRequest = null; unavailable = Boolean(response.fatal);
        if (unavailable) cancelPending();
        enable(!unavailable);
        // Matching a request ID is insufficient: controls can change while it runs.
        if (!unavailable && live && submitted && submitted.revision !== revision) {
            status.textContent = inputs.some(input => input._reading) ? "Reading file…" : !filesReady() ? "Select a valid file to continue." : isLive() ? "Updating to the latest inputs…" : "Inputs changed. Select Predict to run.";
            flush(); return;
        }
        clearErrors();
        if (response.kind === "success") {
            outputs.forEach((output,index) => render(output,response.values[index]));
            current = {...submitted, values:response.values};
            if (pin) pin.disabled = pinned.length >= 3;
            compare(); status.textContent = "Prediction complete.";
        } else {
            outputs.forEach(output => { output.textContent = ""; });
            status.textContent = unavailable ? "Model unavailable." : "Prediction not completed.";
            error.textContent = response.message;
            if (!unavailable && Number.isInteger(response.field) && inputs[response.field]) {
                const input = inputs[response.field]; input.setAttribute("aria-invalid","true");
                document.getElementById(`error-${response.field}`).textContent = response.message;
                if (!isLive()) input.focus();
            } else if (unavailable || !isLive()) error.focus();
        }
        flush();
    }
    let exporting = false;
    function downloadFinished(message) {
        exporting = false;
        const notice = document.getElementById("download-status"); if (notice) notice.textContent = message;
    }
    window.tease = {ready,receive,downloadFinished,exportCsv(csv) {
        if (exporting) return;
        exporting = true;
        try { window.ipc.postMessage(JSON.stringify({export_csv:csv})); }
        catch (_) { downloadFinished("Could not request CSV export. Please try again."); }
    }};
    if (live) live.addEventListener("change", () => {
        cancelPending();
        enable(initialized && !unavailable && activeRequest === null, activeRequest !== null);
        if (isLive()) schedule();
        else if (activeRequest === null) status.textContent = "Live prediction paused. Select Predict to run.";
    });
    inputs.forEach((input,index) => {
        if (input.type === "file") input.addEventListener("change", async () => {
            changed();
            const ticket = input._ticket = (input._ticket || 0) + 1;
            input._payload = ""; input._reading = true; button.disabled = true;
            const file = input.files[0], preview = document.getElementById(`preview-${index}`); preview.replaceChildren();
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
            enable(initialized && !unavailable && activeRequest === null, activeRequest !== null);
            changed();
        });
        input.addEventListener("compositionstart", () => {input._composing = true; revision++; cancelPending();});
        input.addEventListener("compositionend", () => {input._composing = false; changed();});
        input.addEventListener("input", () => {
            if (input.type === "range") input.nextElementSibling.textContent = input.value;
            if (preset) preset.value = "";
            if (input.type !== "file") changed();
        });
        input.addEventListener("change", () => { if (preset) preset.value = ""; if (input.type !== "file") changed(); });
    });
    form.addEventListener("submit", event => {
        event.preventDefault();
        if (!initialized || unavailable || button.disabled || activeRequest !== null) return;
        cancelPending(); send(snapshot());
    });
    enable(false,true); status.textContent = "Preparing model…";
    try { window.ipc.postMessage("ready"); }
    catch (_) { ready("Could not connect to the model. Restart the demo."); }
})();

"use strict";
(() => {
    const form = document.getElementById("tease-form");
    const inputs = Array.from(document.querySelectorAll("[data-input]"));
    const outputs = Array.from(document.querySelectorAll("[data-output]"));
    const button = document.getElementById("submit");
    const status = document.getElementById("status");
    const error = document.getElementById("error");
    let initialized = false;
    let activeRequest = null;
    let nextRequest = 0;
    function enable(enabled, busy = false) {
        button.disabled = !enabled;
        inputs.forEach(input => { input.disabled = !enabled; });
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
        enable(!message);
        status.textContent = message ? "Model unavailable." : "Ready.";
        if (message) error.textContent = message;
    }
    function receive(response) {
        // A late or duplicate response must not overwrite newer state or unlock a job.
        if (response.request_id !== activeRequest) return;
        if (activeRequest === null && response.request_id !== null) return;
        activeRequest = null;
        enable(!response.fatal);
        clearErrors();
        if (response.kind === "success") {
            outputs.forEach((output, index) => { output.textContent = response.values[index]; });
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
    window.tease = { ready, receive };
    inputs.forEach(input => {
        if (input.type === "range") {
            input.addEventListener("input", () => { input.nextElementSibling.textContent = input.value; });
        }
    });
    form.addEventListener("submit", event => {
        event.preventDefault();
        if (!initialized || button.disabled || activeRequest !== null) return;
        clearErrors();
        outputs.forEach(output => { output.textContent = ""; });
        activeRequest = ++nextRequest;
        enable(false, true);
        status.textContent = "Computing…";
        try {
            window.ipc.postMessage(JSON.stringify({ request_id: activeRequest, inputs: inputs.map(input => input.value) }));
        } catch (_) {
            receive({ request_id: activeRequest, kind: "error", field: null, message: "Could not submit the prediction. Please try again." });
        }
    });
    enable(false, true);
    status.textContent = "Preparing model…";
    try { window.ipc.postMessage("ready"); }
    catch (_) { ready("Could not connect to the model. Restart the demo."); }
})();

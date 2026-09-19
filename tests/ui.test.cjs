const { test } = require('node:test');
const assert = require('node:assert/strict');
const { readFileSync } = require('node:fs');
const { runInNewContext } = require('node:vm');

// Exercise the shipped event handlers, with only the DOM operations they use.
function fixture() {
    function element(value = '') {
        return { value, textContent: '', disabled: false, attributes: {}, listeners: {},
            setAttribute(key, value) { this.attributes[key] = value; },
            removeAttribute(key) { delete this.attributes[key]; },
            addEventListener(type, callback) { this.listeners[type] = callback; },
            focus() { this.focused = true; } };
    }
    const inputs = [element('3'), element('2')];
    inputs[1].type = 'range'; inputs[1].nextElementSibling = element();
    const outputs = [element()];
    const nodes = Object.fromEntries(['tease-form', 'submit', 'status', 'error', 'error-0', 'error-1'].map(id => [id, element()]));
    const messages = [];
    const window = { ipc: { postMessage(message) { messages.push(message); } } };
    const document = { getElementById: id => nodes[id], querySelectorAll: selector => selector === '[data-input]' ? inputs : outputs };
    runInNewContext(readFileSync(`${__dirname}/../src/ui.js`, 'utf8'), { window, document });
    function submit() {
        let prevented = false;
        nodes['tease-form'].listeners.submit({ preventDefault() { prevented = true; } });
        assert.ok(prevented);
    }
    return { inputs, outputs, nodes, messages, window, submit };
}

test('preparation and active work disable submissions and all inputs', () => {
    const f = fixture();
    assert.deepEqual(f.messages, ['ready']);
    f.submit(); assert.equal(f.messages.length, 1);
    f.window.tease.ready(null);
    f.submit(); f.submit();
    assert.equal(f.messages.length, 2);
    assert.deepEqual(JSON.parse(f.messages[1]), { request_id: 1, inputs: ['3', '2'] });
    assert.ok(f.inputs.every(input => input.disabled));
    assert.equal(f.nodes.status.textContent, 'Computing…');
    assert.equal(f.nodes['tease-form'].attributes['aria-busy'], 'true');
});

test('late and duplicate replies neither replace results nor unlock newer work', () => {
    const f = fixture(); f.window.tease.ready(null); f.submit();
    f.window.tease.receive({ request_id: 99, kind: 'success', values: ['stale'] });
    assert.ok(f.nodes.submit.disabled); assert.equal(f.outputs[0].textContent, '');
    f.window.tease.receive({ request_id: 1, kind: 'success', values: ['9.00'] });
    assert.equal(f.outputs[0].textContent, '9.00'); assert.ok(!f.nodes.submit.disabled);
    f.submit();
    f.window.tease.receive({ request_id: 1, kind: 'success', values: ['stale'] });
    assert.ok(f.nodes.submit.disabled); assert.equal(f.outputs[0].textContent, '');
    f.window.tease.receive({ request_id: 2, kind: 'success', values: ['16.00'] });
    f.window.tease.receive({ request_id: 2, kind: 'success', values: ['duplicate'] });
    assert.equal(f.outputs[0].textContent, '16.00');
});

test('field and model errors preserve values and allow a corrected resubmission', () => {
    const f = fixture(); f.window.tease.ready(null); f.submit();
    f.window.tease.receive({ request_id: 1, kind: 'error', field: 0, message: 'Value is required.' });
    assert.ok(f.inputs[0].focused); assert.equal(f.inputs[0].attributes['aria-invalid'], 'true');
    assert.ok(f.inputs.every(input => !input.disabled));
    assert.equal(f.inputs[1].value, '2');
    f.inputs[0].value = '4'; f.submit();
    f.window.tease.receive({ request_id: 2, kind: 'error', field: null, message: '<script>bad()</script>' });
    assert.equal(f.nodes.error.textContent, '<script>bad()</script>');
    f.submit();
    f.window.tease.receive({ request_id: 3, kind: 'success', values: ['16.00'] });
    assert.equal(f.nodes.error.textContent, ''); assert.equal(f.outputs[0].textContent, '16.00');
    assert.equal(f.inputs[0].attributes['aria-invalid'], undefined);
});

test('transport failures recover; fatal model failures leave controls disabled', () => {
    const f = fixture(); f.window.tease.ready(null);
    f.window.ipc.postMessage = () => { throw new Error('disconnected'); };
    f.submit(); assert.ok(!f.nodes.submit.disabled);
    assert.match(f.nodes.error.textContent, /Could not submit/);
    f.window.ipc.postMessage = () => {};
    f.submit();
    f.window.tease.receive({ request_id: 2, kind: 'error', fatal: true, message: 'Model panicked.' });
    assert.ok(f.nodes.submit.disabled); assert.ok(f.inputs.every(input => input.disabled));
    assert.equal(f.nodes.status.textContent, 'Model unavailable.');
    assert.equal(f.nodes['tease-form'].attributes['aria-busy'], 'false');
});

test('initialization errors remain visible and sliders update their readout', () => {
    const f = fixture(); f.window.tease.ready('Missing model.'); f.submit();
    assert.equal(f.messages.length, 1); assert.equal(f.nodes.error.textContent, 'Missing model.');
    assert.ok(f.nodes.submit.disabled);
    const healthy = fixture(); healthy.window.tease.ready(null);
    healthy.inputs[1].value = '4'; healthy.inputs[1].listeners.input();
    assert.equal(healthy.inputs[1].nextElementSibling.textContent, '4');
});

const { test } = require('node:test');
const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');
const { predict } = require('../docs/live/model.js');

test('browser preview computes the documented default and changed response', () => {
    const baseline = predict(['1', '0', '0']);
    assert.equal(baseline.kind, 'success');
    assert.equal(baseline.values[0], '0.7311');
    assert.equal(baseline.values[1].plot.kind, 'line');
    assert.equal(baseline.values[1].plot.series[0].points.length, 41);
    assert.equal(baseline.values[2].scores[0].score.toFixed(3), '0.731');

    const changed = predict(['3', '0', '1']);
    assert.equal(changed.values[0], '0.9526');
    assert.equal(changed.values[1].plot.kind, 'scatter');
    assert.equal(changed.values[2].scores[1].score.toFixed(3), '0.047');
});

test('browser preview validates bad requests and retains Tease live controls', () => {
    assert.deepEqual(predict(['', '0', '0']), {
        kind: 'error', field: 0, message: 'Gain: A value is required.',
    });
    assert.equal(predict(['1', '0', '9']).field, 2);
    const html = fs.readFileSync(path.join(__dirname, '../docs/live/index.html'), 'utf8');
    assert.match(html, /<script src="model\.js"><\/script>/);
    assert.match(html, /"liveDelay":200/);
    assert.match(html, /id="live" type="checkbox" checked/);
    for (const asset of ['ui.css', 'ui.js', 'render.js', 'files.js']) {
        assert.ok(html.includes(fs.readFileSync(path.join(__dirname, '../src', asset), 'utf8')),
            `${asset} in the browser preview has drifted from Tease`);
    }
});

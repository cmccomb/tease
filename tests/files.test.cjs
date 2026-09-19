const {test}=require('node:test');const assert=require('node:assert/strict');const {Element,fixture}=require('./dom.cjs');
test('CSV previews preserve quoted delimiters, Unicode, and embedded newlines',()=>{
    const f=fixture(); const result=f.window.teaseFiles.parseCSV('a,b\r\n"🦀,x","two\nlines with ""quotes"""\r\n');
    assert.deepEqual(JSON.parse(JSON.stringify(result)),{columns:['a','b'],rows:[['🦀,x','two\nlines with "quotes"']]});
    for(const value of ['a,b\n1\n','a,a\n1,2\n','a\n"unfinished']) assert.throws(()=>f.window.teaseFiles.parseCSV(value));
});
test('file presets supply actual CSV content and clear earlier selections',()=>{
    const input=new Element('input');input.type='file';input.dataset.fileKind='table';
    const f=fixture({presets:[{name:'CSV',inputs:['x\n1\n2\n']}],outputs:[{label:'Rows'}]},[input]);
    f.window.tease.ready(null);f.nodes.preset.value='0';f.nodes.preset.fire('change');f.submit();
    assert.equal(JSON.parse(f.messages[1]).inputs[0],'x\n1\n2\n');assert.match(f.nodes['preview-0'].textContent,/2 rows/);
});
test('table results remain literal and preview is bounded',()=>{
    const f=fixture(); const table={columns:['<b>header</b>'],rows:Array.from({length:101},()=>['<img onerror=bad()>'])};
    f.window.teaseView.render(f.outputs[0],{kind:'table',table});assert.match(f.outputs[0].textContent,/101 rows \(showing first 100\)/);
    assert.match(f.outputs[0].textContent,/<img onerror/);assert.equal(f.outputs[0].children[0].children[0].children[1].children.length,100);
});

test('malformed file previews recover and later file selection succeeds',async()=>{
    const input=new Element('input');input.type='file';input.dataset.fileKind='table';input.files=[{name:'bad.csv'}];
    const f=fixture({},[input]);f.window.tease.ready(null);
    f.window.teaseFiles.read=()=>Promise.resolve('a,b\n1\n');await input.fireAsync('change');
    assert.equal(f.nodes.submit.disabled,false);assert.match(f.nodes.error.textContent,/CSV/);assert.equal(input._payload,'');
    f.window.teaseFiles.read=()=>Promise.resolve('x\n3\n');await input.fireAsync('change');f.submit();
    assert.equal(JSON.parse(f.messages[1]).inputs[0],'x\n3\n');
});
test('a newer file and a preset each invalidate older asynchronous reads',async()=>{
    const input=new Element('input');input.type='file';input.dataset.fileKind='table';input.files=[{name:'one.csv'}];
    const f=fixture({presets:[{name:'Preset',inputs:['x\n9\n']}]},[input]);f.window.tease.ready(null);
    const complete=[];f.window.teaseFiles.read=()=>new Promise(resolve=>complete.push(resolve));
    const first=input.fireAsync('change'),second=input.fireAsync('change');
    complete[1]('x\n2\n');await second;complete[0]('x\n1\n');await first;assert.equal(input._payload,'x\n2\n');
    const third=input.fireAsync('change');assert.equal(f.nodes.submit.disabled,true);
    f.nodes.preset.value='0';f.nodes.preset.fire('change');assert.equal(f.nodes.submit.disabled,false);
    complete[2]('x\n3\n');await third;assert.equal(input._payload,'x\n9\n');
});
test('export requests are bounded to one open dialog and cancellation permits retry',()=>{
    const f=fixture();f.window.tease.exportCsv('x\n1\n');f.window.tease.exportCsv('x\n2\n');
    assert.equal(f.messages.length,2);assert.equal(JSON.parse(f.messages[1]).export_csv,'x\n1\n');
    f.window.tease.downloadFinished('CSV export canceled.');f.window.tease.exportCsv('x\n3\n');assert.equal(f.messages.length,3);
});

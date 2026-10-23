const {test}=require('node:test'); const assert=require('node:assert/strict');
const {Element,fixture}=require('./dom.cjs');
function find(root,tag) { return [root,...root.children.flatMap(c=>find(c,tag))].filter(n=>n.tagName===tag.toUpperCase()); }
test('plots draw line/scatter/bar with accessible point labels and literal legend text',()=>{
    const f=fixture();
    for(const kind of ['line','scatter','bar']) {
        f.window.teaseView.render(f.outputs[0],{kind:'plot',plot:{kind,x_label:'x',y_label:'y',series:[{name:'<script>unsafe</script>',points:[[0,-1],[1,2]]}]}});
        assert.equal(find(f.outputs[0],'svg').length,1); assert.equal(find(f.outputs[0],'title').length,2);
        assert.equal(find(f.outputs[0],'table').length,1); assert.equal(find(f.outputs[0],'script').length,0);
        assert.match(f.outputs[0].textContent, /<script>unsafe/);
    }
});
test('probability curves use readable y-axis ticks',()=>{
    const f=fixture();
    f.window.teaseView.render(f.outputs[0],{kind:'plot',plot:{kind:'line',x_label:'x',y_label:'probability',series:[{name:'response',points:[[-3,0.047],[0,0.5],[3,0.953]]}]}});
    const labels=find(find(f.outputs[0],'svg')[0],'text').map(node=>node.textContent);
    for(const tick of ['0','0.25','0.5','0.75','1']) assert.ok(labels.includes(tick));
});
test('class bars sort supplied probabilities without inventing scores',()=>{
    const f=fixture(); f.window.teaseView.render(f.outputs[0],{kind:'scores',probabilities:true,scores:[{label:'Low',score:0.2},{label:'High',score:0.8}]});
    assert.equal(find(f.outputs[0],'meter')[0].value,0.8); assert.match(f.outputs[0].textContent,/80.0%/);
});
test('presets, checkbox and multi-select serialize typed values and pinned inputs remain snapshots',()=>{
    const text=new Element('textarea'), flag=new Element('input'), multi=new Element('select'); flag.type='checkbox'; multi.multiple=true;
    for(let i=0;i<2;i++){const option=new Element('option'); option.value=String(i);option.textContent=['A','B'][i];multi.append(option);}
    const f=fixture({comparison:true,inputLabels:['Notes','Flag','Choices'],outputs:[{label:'Result',numeric:true}],presets:[{name:'Sample',inputs:['example','true','[1]']}]},[text,flag,multi]);
    f.window.tease.ready(null);f.nodes.preset.value='0';f.nodes.preset.fire('change');
    assert.equal(text.value,'example'); assert.equal(flag.checked,true); assert.equal(multi.selectedOptions[0].value,'1');
    f.submit(); assert.deepEqual(JSON.parse(f.messages[1]).inputs,['example','true','[1]']);
    f.window.tease.receive({request_id:1,kind:'success',values:['2.00']}); f.nodes.pin.fire('click');
    text.value='changed';text.fire('change');f.submit();f.window.tease.receive({request_id:2,kind:'success',values:['3.50']});
    assert.match(f.nodes.scenarios.textContent,/example/);assert.match(f.nodes.scenarios.textContent,/changed/);assert.match(f.nodes.scenarios.textContent,/Difference: \+1.5/);
    find(f.nodes.scenarios,'button')[0].fire('click');assert.equal(f.nodes.scenarios.textContent,'');
});

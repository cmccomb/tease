const {test}=require('node:test');const assert=require('node:assert/strict');const {Element,fixture}=require('./dom.cjs');
function setup() {
    let now=0,id=0;const timers=new Map();
    const clock={setTimeout(fn,ms){timers.set(++id,{fn,at:now+ms});return id;},clearTimeout(id){timers.delete(id);},tick(ms){now+=ms;for(const [key,job] of [...timers]) if(job.at<=now){timers.delete(key);job.fn();}}};
    const input=new Element('input');input.value='2';
    const f=fixture({liveDelay:200},[input],clock);f.window.tease.ready(null);
    return {...f,clock,edit(value){input.value=value;input.fire('input');},reply(id,value,extra={}){f.window.tease.receive({request_id:id,kind:'success',values:[value],...extra});}};
}
test('rapid edits debounce and only the latest pending snapshot runs',()=>{
    const f=setup();f.edit('3');f.clock.tick(100);f.edit('4');f.clock.tick(199);assert.equal(f.messages.length,1);f.clock.tick(1);
    assert.deepEqual(JSON.parse(f.messages[1]),{request_id:1,inputs:['4']});assert.equal(f.inputs[0].disabled,false);assert.equal(f.nodes.submit.disabled,true);assert.equal(f.nodes['tease-form'].attributes['aria-busy'],'false');
    f.edit('5');f.clock.tick(200);f.edit('6');f.clock.tick(200);assert.equal(f.messages.length,2);
    f.reply(1,'16');assert.equal(f.outputs[0].textContent,'');assert.deepEqual(JSON.parse(f.messages[2]),{request_id:2,inputs:['6']});
    f.reply(1,'stale');assert.equal(f.nodes.submit.disabled,true);f.reply(2,'36');assert.equal(f.outputs[0].textContent,'36');
});
test('completion before debounce expires cannot render obsolete inputs',()=>{
    const f=setup();f.clock.tick(200);f.edit('8');f.reply(1,'4');assert.equal(f.outputs[0].textContent,'');assert.equal(f.messages.length,2);
    f.clock.tick(200);assert.equal(JSON.parse(f.messages[2]).inputs[0],'8');f.reply(2,'64');assert.equal(f.outputs[0].textContent,'64');
});
test('pausing discards pending automatic work while manual submission still works',()=>{
    const f=setup();f.clock.tick(200);f.edit('5');f.clock.tick(200);f.nodes.live.checked=false;f.nodes.live.fire('change');f.reply(1,'4');f.clock.tick(1000);
    assert.equal(f.messages.length,2);assert.equal(f.outputs[0].textContent,'');f.submit();f.reply(2,'25');assert.equal(f.outputs[0].textContent,'25');
});
test('recoverable errors accept new edits, fatal errors cancel pending work',()=>{
    const f=setup();f.clock.tick(200);f.reply(1,'',{kind:'error',field:0,message:'Bad input'});assert.equal(f.inputs[0].focused,undefined);
    f.edit('3');f.clock.tick(200);f.edit('4');f.clock.tick(200);f.reply(2,'',{kind:'error',fatal:true,message:'Model failed'});f.clock.tick(1000);
    assert.equal(f.messages.length,3);assert.equal(f.nodes.live.disabled,true);assert.equal(f.inputs[0].disabled,true);
});
test('text composition waits for committed text and canceled initial work stays canceled',()=>{
    const f=setup();f.inputs[0].fire('compositionstart');f.edit('draft');f.clock.tick(1000);assert.equal(f.messages.length,1);
    f.inputs[0].fire('compositionend');f.clock.tick(200);assert.equal(JSON.parse(f.messages[1]).inputs[0],'draft');
});

const {readFileSync}=require('node:fs');
const {runInNewContext}=require('node:vm');
class Element {
    constructor(tag='div') { this.tagName=tag.toUpperCase(); this.children=[]; this.attributes={}; this.listeners={}; this.style={}; this.value=''; this.disabled=false; this._text=''; this.dataset={}; }
    set textContent(text) { this._text=String(text); this.children=[]; }
    get textContent() { return this._text+this.children.map(c=>c.textContent).join(''); }
    append(...children) { this.children.push(...children); }
    replaceChildren(...children) { this.children=children; this._text=''; }
    setAttribute(k,v) { this.attributes[k]=v; }
    removeAttribute(k) { delete this.attributes[k]; }
    addEventListener(k,fn) { (this.listeners[k]??=[]).push(fn); }
    fire(k) { (this.listeners[k]||[]).forEach(fn=>fn({preventDefault(){},target:this})); }
    focus() { this.focused=true; }
    get options() { return this.children; }
    get selectedOptions() { return this.children.filter(c=>c.selected); }
}
function fixture(config={}, inputs=[new Element('input')]) {
    const nodes=Object.fromEntries(['tease-form','submit','status','error','tease-config',...inputs.map((_,i)=>`error-${i}`)].map(id=>[id,new Element()]));
    if(config.presets?.length) nodes.preset=new Element('select');
    if(config.comparison) { nodes.pin=new Element('button'); nodes.scenarios=new Element(); }
    nodes['tease-config'].textContent=JSON.stringify(config);
    const outputs=(config.outputs||[{}]).map(()=>new Element('output'));
    const messages=[]; const window={ipc:{postMessage:m=>messages.push(m)}};
    const document={getElementById:id=>nodes[id],querySelectorAll:s=>s==='[data-input]'?inputs:outputs,createElement:tag=>new Element(tag),createElementNS:(_,tag)=>new Element(tag)};
    const context={window,document,console};
    for(const file of ['render.js','ui.js']) runInNewContext(readFileSync(`${__dirname}/../src/${file}`,'utf8'),context);
    return {window,nodes,outputs,inputs,messages,submit:()=>nodes['tease-form'].fire('submit')};
}
module.exports={Element,fixture};

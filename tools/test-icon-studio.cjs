// Dependency-free smoke test for the standalone Icon Studio.
// A minimal DOM and Canvas stand-in tests real script startup, drawing,
// undo/redo, JSON persistence/export and PNG ZIP creation with Node.
const fs = require("node:fs");
const vm = require("node:vm");
const assert = require("node:assert/strict");

const html = fs.readFileSync("web/icon-studio.html", "utf8");
const js = html.match(/<script>\s*([\s\S]*?)\s*<\/script>/i)?.[1];
assert(js, "Inline app script not found");
new vm.Script(js);

class Node {
  constructor(tag = "div") {
    this.tagName = tag.toUpperCase();
    this.children = [];
    this.dataset = {};
    this.style = {};
    this.events = {};
    this.attrs = {};
    this.className = "";
    this.value = "";
    this.disabled = false;
    this._text = "";
    this.files = [];
    this.classList = { toggle: (name, on) => {
      const x = new Set(this.className.split(" ").filter(Boolean));
      if (on) x.add(name); else x.delete(name);
      this.className = [...x].join(" ");
    }};
  }
  set textContent(t) {
    this._text = String(t);
    if (t === "") this.children = [];
  }
  get textContent() { return this._text; }
  append(...children) {
    children.forEach(x => this.children.push(...(x.tagName === "FRAGMENT" ? x.children : [x])));
  }
  addEventListener(type, callback) { (this.events[type] ||= []).push(callback); }
  fire(type, event = {}) {
    return Promise.all((this.events[type] || []).map(fn => fn.call(this, event)));
  }
  click() { return this.fire("click"); }
  setAttribute(name, value) { this.attrs[name] = String(value); }
  querySelector(selector) {
    return this.children.find(x => selector === "canvas" ? x.tagName === "CANVAS" :
      selector === ".filename" ? x.className === "filename" : false);
  }
  querySelectorAll(selector) {
    if (selector === ".tile") return this.children.filter(x => x.className.includes("tile"));
    return [];
  }
  getContext() {
    if (!this.context) this.context = {
      createImageData: (w,h) => ({data: new Uint8ClampedArray(w*h*4)}),
      putImageData: data => {this.lastImage = data;},
      clearRect: () => {}, drawImage: () => {},
      getImageData: () => ({data:new Uint8ClampedArray(24*24*4)})
    };
    return this.context;
  }
  getBoundingClientRect() { return {left:0,top:0,width:240,height:240}; }
  setPointerCapture() {}
  toBlob(callback) { callback(new Blob([Buffer.from([137,80,78,71])], {type:"image/png"})); }
  remove() {}
}
const ids = [...html.matchAll(/\bid="([^"]+)"/g)].map(x => x[1]);
const elements = new Map(ids.map(id => [id, new Node(id === "canvas" ? "canvas" : "div")]));
for (const id of ["pen","erase","fill","pick"]) elements.get(id).dataset.tool = id;
elements.get("customColor").value = "#ffffff";
elements.get("gridCheck").checked = true;
const doc = {
  body: new Node("body"),
  activeElement: null,
  getElementById: id => elements.get(id),
  createElement: tag => new Node(tag),
  createDocumentFragment: () => new Node("fragment"),
  querySelectorAll: selector => selector === "[data-tool]" ?
    ["pen","erase","fill","pick"].map(id => elements.get(id)) :
    selector === ".swatch" ? elements.get("palette").children : [],
  addEventListener: () => {}
};
const local = new Map();
let downloaded;
const context = {
  document:doc,
  window:{addEventListener:()=>{}},
  localStorage:{getItem:k=>local.get(k)||null,setItem:(k,v)=>local.set(k,v)},
  Blob, Buffer, TextEncoder, Uint8ClampedArray, Uint8Array, DataView,
  URL:{createObjectURL:b=>{downloaded=b;return "blob:fake";},revokeObjectURL:()=>{}},
  setTimeout:()=>1,clearTimeout:()=>{},confirm:()=>true,alert:x=>{throw Error(x)},
  console
};
vm.runInNewContext(js, context);
const get = id => elements.get(id);
assert.equal(get("tiles").children.length,128);
assert.equal(get("palette").children.length,4,"four Harbour palette swatches");
assert.equal(get("progress").textContent,"0 / 128");
(async () => {
  const canvas=get("canvas");
  const event={pointerType:"pen",pointerId:1,clientX:35,clientY:45,preventDefault:()=>{}};
  await canvas.fire("pointerdown",event);
  await canvas.fire("pointerup",event);
  assert.equal(get("progress").textContent,"1 / 128");
  assert.equal(canvas.lastImage.data[(4*24+3)*4+3],255,"painted pixel opacity");
  await get("undo").click();
  assert.equal(get("progress").textContent,"0 / 128");
  await get("redo").click();
  assert.equal(get("progress").textContent,"1 / 128");
  get("iconName").value="My-test-icon";
  await get("iconName").fire("input");
  await get("exportJson").click();
  const project=JSON.parse(await downloaded.text());
  assert.equal(project.icons.length,128);
  assert.equal(project.icons[0].name,"My-test-icon");
  assert.equal(project.icons[0].pixels[4*24+3],"#382209");
  assert(local.has("harbour-icon-studio:v1"),"browser persistence");
  await get("exportZip").click();
  const zip=new Uint8Array(await downloaded.arrayBuffer());
  assert.equal(String.fromCharCode(...zip.slice(0,2)),"PK","ZIP signature");
  assert(zip.length>15000,"128 PNGs + manifest exported");
  assert.equal(get("exportZip").disabled,false);
  console.log("Icon Studio smoke test passed: 128 slots, touch drawing, undo/redo, JSON, storage, ZIP");
})().catch(e => {console.error(e);process.exitCode=1;});

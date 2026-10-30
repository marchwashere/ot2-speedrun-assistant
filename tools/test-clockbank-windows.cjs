const fs=require('node:fs'),vm=require('node:vm'),assert=require('node:assert/strict');
const source=fs.readFileSync(require('node:path').join(__dirname,'../viewer/viewer.js'),'utf8');
const start=source.indexOf('          if(i.clockbank_free) {');
const end=source.indexOf('          if(i.clockbank_adjustment) {',start);
assert(start>=0&&end>start);
const nodes={};const ctx={i:{clockbank_free:{windows:[
 {label:'Main town → district',switches:0,end_time:2},
 {label:'Before first factory',switches:1,end_time:4},
],return_to_day:true}},$:id=>nodes[id]??=( {hidden:true,innerHTML:''}),escapeHtml:s=>s,instructionBoxHtml:lines=>lines.join('\n')};
vm.runInNewContext(source.slice(start,end),ctx);
assert.equal(nodes['clockbank-bg-result'].hidden,false);
const text=nodes['clockbank-bg-routine'].innerHTML;
assert.match(text,/Time switches: 0 · finish in daytime/);
assert.match(text,/Time switches: 1 · finish in nighttime/);
assert.match(text,/After second factory exit: switch to day/);
ctx.i.clockbank_free.return_to_day=false;
vm.runInNewContext(source.slice(start,end),ctx);
assert.doesNotMatch(nodes['clockbank-bg-routine'].innerHTML,/After second factory exit/);
assert(source.includes('if(i.approach.agnea_story.world_after.galdera_route) maybeWarnAdjustmentCost('));
console.log('PASS: free-window counts, night entries, conditional day restoration and Galdera-only warning');

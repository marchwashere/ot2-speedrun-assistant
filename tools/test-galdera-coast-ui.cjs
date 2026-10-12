'use strict';
const fs=require('node:fs'),vm=require('node:vm'),assert=require('node:assert/strict');
const source=fs.readFileSync(require('node:path').join(__dirname,'../viewer/viewer.js'),'utf8');
const start=source.indexOf('                  if(g.coast) {');
const end=source.indexOf('\n                }\n              }',start);
assert(start>=0&&end>start);
const elements=new Map();
const $=id=>{if(!elements.has(id))elements.set(id,{});return elements.get(id);};
const entry={trace:[],after:{draws:0,state:0}};
const coast={day_before_gravell:entry,conning:entry,coast_entry:entry,coast_exit:entry,eastern_sai:entry,
  movement:{run_percent:43.6}};
const context={$,g:{coast},conningStages:[],conningSpans:[],instructionBoxHtml(parts){
  assert(Array.isArray(parts),'Instructions must be an array of lines');return parts.join('\n');
}};
vm.runInNewContext(source.slice(start,end),context);
assert.equal($('galdera-coast-result').hidden,false);
assert.match($('galdera-coast-routine').innerHTML,/Run first 43\.6% · walk remaining 56\.4%/);
assert.match($('galdera-coast-routine').innerHTML,/No night\/day adjustment/);
assert(!context.conningStages.some(([label])=>label==='Conning Creek · switched to night'));
coast.conning_night=entry;coast.coast_day=entry;context.conningStages=[];
vm.runInNewContext(source.slice(start,end),context);
assert.match($('galdera-coast-routine').innerHTML,/switch to night immediately after landing/);
assert.match($('galdera-coast-routine').innerHTML,/switch to day immediately after entering/);
assert(context.conningStages.some(([label])=>label==='Conning Creek · switched to night'));
assert(context.conningStages.some(([label])=>label==='Western Conning Creek Coast · switched to day'));
console.log('PASS: both coast timing branches, running percentage, and detailed checkpoints');
coast.southern_sai={arrival:entry,field_return:entry,battle_entry:entry.after,
  menu:{first_actor:2,rng:entry.after},escape:{chance_percent:100,trace:[]},
  encounter:{checkpoints:[{label:'entry',draws:0,state:0}]}};
context.names=['Throné','Osvald','Agnea','Partitio'];context.encounterLabel=()=> 'Encounter: Southern Sai formation';context.escapeHtml=String;context.conningPoints=[];
vm.runInNewContext(source.slice(start,end),context);
assert.equal($('southern-sai-result').hidden,false);
assert.match($('southern-sai-routine').innerHTML,/T1 · Agnea · 1st command menu/);
assert.match($('southern-sai-routine').innerHTML,/Escape \(100%\)/);
assert(context.conningStages.some(([label])=>label==='Southern Sai Sands · escaped, field restored'));
console.log('PASS: Southern Sai actual actor, guaranteed Escape and field-return checkpoints');

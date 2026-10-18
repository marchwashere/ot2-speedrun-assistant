'use strict';
const fs=require('node:fs'),vm=require('node:vm'),assert=require('node:assert/strict');
const source=fs.readFileSync(require('node:path').join(__dirname,'../viewer/viewer.js'),'utf8');
const start=source.indexOf("                  $('gravell-routine').innerHTML=");
const end=source.indexOf('                  conningSpans.push(...g.fight.trace',start);
assert(start>=0&&end>start);
const helpers=source.slice(source.indexOf('function menuOrdinal('),source.indexOf('function safeDefendsHtml('));
for(const coast of [false,true]) for(const firstMerchant of [1,3]) {
 const element={},rendered=[];
 const secondMerchant=firstMerchant===1?3:1;
 const names=['Throné',coast?'Osvald':'Hikari','Agnea','Partitio'];
 const actions=[
  {action:{donate_bp:{target:secondMerchant}},round:1,actor:firstMerchant,inputs:[1]},
  {action:{pomegranate:{target:2}},round:1,actor:0,inputs:[]},
  {action:'guards',round:1,actor:secondMerchant,inputs:[1,2,3]},
  {action:'defend',round:1,actor:2,inputs:[]},
  {action:'bewildering_grace',round:2,actor:2,inputs:[1,2,1,2,3]},
  {action:'thunder_soulstone',round:2,actor:0,inputs:[1,0]},
 ];
 const c={
  $:()=>element,g:{coast,encounter:{},fight:{actions}},names,conningPoints:[],
  escapeHtml:String,encounterLabel:()=> 'Chubby Cait',
  instructionBoxHtml:(parts,command)=>{rendered.push({inputs:[],command});return command;},
  boostStepsHtml:(inputs,command)=>{rendered.push({inputs,command});return command;},
 };
 vm.runInNewContext(helpers,c);
 vm.runInNewContext(source.slice(start,end),c);
 assert.deepEqual(rendered.map(r=>r.command),[
  '+1 Donate BP → '+names[secondMerchant],'Energizing Pomegranate (M) → Agnea',
  '+3 Hired Help: Guards','Defend','+3 Bewildering Grace','Thunder Soulstone (L)',
 ]);
 for(let i=0;i<actions.length;i++)assert.deepEqual(rendered[i].inputs,actions[i].inputs);
 assert.match(element.innerHTML,/T1 · Throné · 2nd command menu/);
 assert.match(element.innerHTML,/T1 · Agnea · 4th command menu/);
 assert.match(element.innerHTML,/T2 · Agnea · 1st command menu/);
 assert.match(element.innerHTML,/T2 · Throné · 2nd command menu/);
 assert.equal(c.conningPoints.length,actions.length);
 // BG may win immediately: show only the actions actually taken.
 actions.pop();rendered.length=0;c.conningPoints=[];
 vm.runInNewContext(source.slice(start,end),c);
 assert.equal(rendered.length,5);
 assert.equal(rendered.at(-1).command,'+3 Bewildering Grace');
 assert.doesNotMatch(element.innerHTML,/Thunder Soulstone/);
 assert.doesNotMatch(element.innerHTML,/T2 · Throné · 2nd command menu/);
}
console.log('PASS: complete Any% and Galdera EXP100 routines, both merchant orders, targets, exact boosts and menu ordinals');

'use strict';
const fs=require('node:fs'),vm=require('node:vm'),assert=require('node:assert/strict');
const source=fs.readFileSync(require('node:path').join(__dirname,'../viewer/viewer.js'),'utf8');
const start=source.indexOf("                  $('gravell-routine').innerHTML=");
const end=source.indexOf('                  conningSpans.push(...g.fight.trace',start);
assert(start>=0&&end>start);
for(const coast of [false,true]) {
 const element={},rendered=[];
 const actions=[
  {action:'defend',round:1,actor:0,inputs:[]},
  {action:'guards',round:1,actor:3,inputs:[1,2,3]},
  {action:'bewildering_grace',round:2,actor:2,inputs:[1,2,1,2,3]},
  {action:'thunder_soulstone',round:2,actor:0,inputs:[]},
 ];
 vm.runInNewContext(source.slice(start,end),{
  $:()=>element,g:{coast,encounter:{},fight:{actions}},ordinals:{},
  names:['Throné','Osvald','Agnea','Partitio'],conningPoints:[],
  escapeHtml:String,encounterLabel:()=> 'Chubby Cait',menuOrdinal:n=>String(n),
  boostStepsHtml:(inputs,command)=>{rendered.push({inputs,command});return command;},
 });
 assert.deepEqual(rendered.map(r=>r.command),[
  '+3 Hired Help: Guards','+3 Bewildering Grace',...(coast?['Thunder Soulstone (L)']:[])
 ]);
 assert.equal(rendered[0].inputs,actions[1].inputs);
 assert.equal(rendered[1].inputs,actions[2].inputs);
 assert.match(element.innerHTML,/T1 · Partitio · 2 command menu/);
 assert.match(element.innerHTML,/T2 · Agnea · 1 command menu/);
}
console.log('PASS: Any% and Galdera Chubby action labels, boost inputs, and menu ordinals');

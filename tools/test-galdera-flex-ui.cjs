'use strict';
const fs=require('fs'),vm=require('vm'),assert=require('assert/strict'),path=require('path');
const js=fs.readFileSync(path.join(__dirname,'../viewer/viewer.js'),'utf8');
const start=js.indexOf('                                      if(q.flex) {'),end=js.indexOf('                                      if(q.gravell)',start);
assert(start>0&&end>start);
const elements=new Map(),$=id=>{if(!elements.has(id))elements.set(id,{});return elements.get(id);};
const rng={draws:0,state:0},e={after:rng,trace:[]},encounter={checkpoints:[{draws:0,state:0,label:'entry'}]};
const f={arrival:e,encounter,field_return:e,menu:{first_actor:2,rng},escape:{trace:[]}};
const q={flex:{order:[3,2,1],segments:[{name:'Boat approach',extra_switches:0,stages:[],encounters:[f],escape_boost_inputs:[1,0],movement:{run_percent:62.3}}],boat:{extra_switches:2,name:'Boat and Roque Harbor',stages:[['Harbor',e]],encounters:[]},encounter,rewards:{awarded:{jp:12760},trace:[]},field_return:e,remaining_errands:[],fight:{trace:[],actions:[{actor:0,round:1,action:{dagger:{target:4}},inputs:[],before:rng},{actor:3,round:1,action:{donate_bp_unboosted:{target:2}},inputs:[],before:rng},{actor:2,round:2,action:'bewildering_grace',inputs:[1,2,1,2,3],before:rng},{actor:1,round:2,action:'fireball_two',inputs:[1,2],before:rng}]}}};
const c={q,$,party:['Throné','Osvald','Temenos','Partitio'],conningStages:[],conningSpans:[],conningPoints:[],encounterLabel:()=> 'Winged Landwalker ×2',instructionBoxHtml:xs=>xs.join('\n'),boostStepsHtml:(inputs,name)=>inputs.join(' → ')+' '+name};
vm.runInNewContext(js.slice(start,end),c);
const html=$('galdera-flex-routine').innerHTML;
for(const re of [/Run 62.3% after the encounter/,/Dagger attack → default target/,/Donate BP → Temenos \(unboosted\)/,/Bewildering Grace/,/Fireball \+2 BP/,/2 extra time switches/])assert.match(html,re);
assert.doesNotMatch(html,/Pom|Hired Help|After Firestarter|<h2>Harbor<|Collect|Restore|Sell|Buy/);
assert.equal($('galdera-flex-result').hidden,false);assert(c.conningPoints.length>=4);
q.flex.fight.actions[1]={actor:3,round:1,action:{donate_bp_unboosted:{target:2}},inputs:[],before:rng};
vm.runInNewContext(js.slice(start,end),c);
assert.match($('galdera-flex-routine').innerHTML,/Donate BP → Temenos \(unboosted\)/);
console.log('PASS: separate manipulation cards, movement percentage, targets and boosts');

assert.equal((html.match(/<section class="panel route-card">/g)||[]).length,3);
assert.doesNotMatch(html,/Escape \(100%\)|JP per character|All four survive|Lychee|cutscene|no encounter/i);
assert.match(html,/<h2>Beasting Bay: Anchorage<\/h2>/);
for(const location of ['Roque Island Harbor','Nameless Isle','Shipwreck'])for(const switches of [0,2]){
  q.flex.location=location;q.flex.boat.extra_switches=switches;
  vm.runInNewContext(js.slice(start,end),c);
  const rendered=$('galdera-flex-routine').innerHTML;
  const marker='<section class="panel route-card jp-destination">';
  assert.equal(rendered.split(marker).length-1,1);
  assert(rendered.includes('<h2>JP ×100 · '+location+'</h2>'));
  assert(rendered.includes(location==='Roque Island Harbor'?'Switch to night upon entering Roque Harbor.':'Stay during day at Roque Harbor.'));
  assert(rendered.indexOf(marker)>rendered.indexOf('<h2>Beasting Bay: Anchorage</h2>'));
  assert(rendered.indexOf(marker)<rendered.indexOf('<h2>'+location+' · JP ×100</h2>'));
  if(switches)assert.match(rendered,/2 extra time switches<\/section><section class="panel route-card jp-destination">/);
}
console.log('PASS: red JP destination follows Tropu switches for all locations, including zero switches');

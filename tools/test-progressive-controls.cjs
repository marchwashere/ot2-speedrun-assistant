'use strict';
const assert=require('node:assert/strict'),fs=require('node:fs'),vm=require('node:vm'),path=require('node:path');
// Load the real viewer, renderers, event handlers and streaming client together.
// The compact fixture has both timing branches; no simulation search is mocked
// into a selector click, and every stream chunk is a fresh JSON object.
const elements=new Map(),pending=[],windowHandlers=new Map();
function element(id) {
  if(!elements.has(id)) {
    const attrs=new Map(),events=new Map();let html='';
    elements.set(id,{textContent:'',value:'0',hidden:false,disabled:false,dataset:{},style:{},writes:0,openedRow:false,
      classList:{toggle(){},add(){},remove(){}},focus(){},reportValidity(){return true;},querySelectorAll(){return [];},
      setAttribute(k,v){attrs.set(k,v);},getAttribute(k){return attrs.get(k);},
      addEventListener(type,fn){events.set(type,fn);},fire(type,event={}){return events.get(type)?.(event);},
      get innerHTML(){return html;},set innerHTML(v){html=v;this.writes++;this.openedRow=false;},
    });
  }
  return elements.get(id);
}
class CustomEvent extends Event {constructor(type,options){super(type);this.detail=options.detail;}}
const context=vm.createContext({console,URLSearchParams,TextDecoder,Event,CustomEvent,AbortController,setTimeout,
  requestAnimationFrame:fn=>fn(),localStorage:{getItem(){return null;},setItem(){}},
  document:{title:'',getElementById:element,querySelector:element,querySelectorAll(){return [];},addEventListener(){}},
  window:{scrollX:0,scrollY:0,scrollTo(){},addEventListener(type,fn){windowHandlers.set(type,fn);},dispatchEvent(e){windowHandlers.get(e.type)?.(e);}},
  fetch:(url,{signal})=>new Promise((resolve,reject)=>{
    const request={url,signal,resolve,reject};pending.push(request);
    signal.addEventListener('abort',()=>reject(new DOMException('Aborted','AbortError')),{once:true});
  }),
});
for(const file of ['viewer.js','voice.js'])vm.runInContext(fs.readFileSync(path.join(__dirname,'../viewer',file),'utf8'),context);
const run=code=>vm.runInContext(code,context),flush=()=>new Promise(setImmediate);
const rng=n=>({draws:n,state:n+1});
const entry=n=>({after:rng(n),world_after:{time:2,encounter_counter:900},trace:[]});
const encounter=name=>({enemies:[{enemy:{name}}],situation:'Normal',counter_updates:5,checkpoints:[]});
const escape=n=>({menu:{rng:rng(n),first_actor:0},attempt:{trace:[]},inputs:{levels:[1,0]}});
function winter(name,n) {
  const voyage=label=>({arrival:{encounter:encounter(label)},escape:escape(n),run_percent:100,cropdale_reach:{run_percent:100}});
  return {first_encounter:encounter(name),first_escape:escape(n),first_field_return:entry(n),
    predicted_second_encounter:false,second_encounter:encounter(name+' second'),without_second:entry(n),
    with_second:{escape:escape(n+1),field_return:entry(n+1),town:entry(n+1)},
    voyage_without_second:voyage(name+' harbor'),voyage_with_second:voyage(name+' second harbor'),verification:'fixture'};
}
function exit(name,n) {
  return {escape:escape(n),field_return:entry(n),onward:{start:'Encounter',running:{reaches_reset:true}},
    eastern_entry:entry(n),eastern_encounter:encounter(name+' eastern'),winterbloom_after_eastern:winter(name,n)};
}
function snapshot(seed=20,partial=true) {
  const branch=name=>({non_surprise:true,encounter:encounter(name),world:{time:2},start:rng(2),before_battle_initialization:rng(3)});
  return {identified_entry:{},initial_seed:seed,partial,crt_setup:{battle_entry_advances:0},
    state:{rng:rng(1)},initial_state:{queue:[0]},actors:[{name:'Throné',side:'player'}],
    steps:[{round:3,actors_after:[],report:{actor:0,rng_before:rng(1),rng_after:rng(2),events:[{kind:'action',name:'defend'}]}}],
    post_battle_field:{flats_escape:{before_battle_initialization:rng(1),arrival:rng(1),encounter:encounter('Flats'),
      early_cait:{snows_approach:{predicted_before_osvald:false,measured_running_updates:64,before_osvald:branch('Before'),after_osvald:branch('After')},
        movement:{mixed:{running_distance_percent:100},running:{reaches_reset:true}},clerics_present:true,
        village_visit:{after:rng(1),trace:[]},after_flats:rng(1),after_night:rng(1),after_day:rng(1),trace:[],
        osvald:{after_loading:rng(1),at_osvald:rng(1),after_recruitment:rng(2),trace:[]}},
      snows_escape:{before_osvald:exit('Before',10),after_osvald:exit('After',20)}}}};
}
function galderaSnapshot(seed,partial=true) {
  const result=snapshot(seed,partial);
  // Galdera skips the guild detour: exactly one Western Snows encounter.
  for(const exit of Object.values(result.post_battle_field.flats_escape.snows_escape)) {
    const west=exit.winterbloom_after_eastern;
    west.with_second=null;west.second_encounter=null;west.voyage_with_second=null;
    west.first_field_return.world_after.galdera_route=true;
    west.without_second.world_after.galdera_route=true;
  }
  return result;
}
async function start(seed=20,route='any_hikari') {
  run(`activeRoute='${route}';voiceStage='lines';voiceFirst='throne';voiceHp=197;voiceCandidate={current_state:${seed}};`);
  const task=run('simulateIdentified()');
  const request=pending.shift();assert(request?.url.startsWith('/api/identified-route?'));
  let controller;
  const body=new ReadableStream({start(c){controller=c;}});
  request.signal.addEventListener('abort',()=>controller.error(new DOMException('Aborted','AbortError')),{once:true});
  request.resolve(new Response(body));
  return {task,request,send(type,result,message='Searching later section'){
    controller.enqueue(new TextEncoder().encode(JSON.stringify({type,result,message})+'\n'));
  },close(){controller.close();}};
}
(async()=>{
  const stream=await start();
  stream.send('progress',snapshot());await flush();
  assert.equal(run('running'),true);
  assert.equal(element('southern-encounter-name').textContent,'Encounter: After');
  const actionWrites=element('actions').writes;element('actions').openedRow=true;
  element('snows-encounter-before').fire('click');
  assert.equal(run('snowsEncounterTiming'),'before','Osvald choice must work during EXP/JP search');
  assert.equal(element('southern-encounter-name').textContent,'Encounter: Before');
  assert.match(element('winter-crackridge-encounter').textContent,/Before harbor/);
  element('western-second-yes').fire('click');
  assert.equal(run('westernSecondOverride'),true);
  assert.match(element('winter-crackridge-encounter').textContent,/Before second harbor/);
  stream.send('progress',snapshot());await flush();
  stream.send('complete',snapshot(20,false));stream.close();await stream.task;
  assert.equal(run('snowsEncounterTiming'),'before');assert.equal(run('westernSecondOverride'),true);
  assert.equal(element('snows-encounter-before').getAttribute('aria-pressed'),'true');
  assert.match(element('winter-crackridge-encounter').textContent,/Before second harbor/);
  assert.equal(element('actions').writes,actionWrites,'Unchanged actions must not be replaced');
  assert.equal(element('actions').openedRow,true,'Expanded action survives progress and selector changes');
  assert.equal(pending.length,0,'Selectors must not start a new RNG search');
  assert.equal(element('export').disabled,false);
  console.log('PASS: Any% Hikari selectors update early and onward widgets; selections and open actions survive completion');

  const failed=await start(30);
  assert.equal(run('result'),null);assert.equal(run('snowsEncounterTiming'),null);assert.equal(run('westernSecondOverride'),null);
  failed.send('progress',snapshot(30));await flush();
  element('snows-encounter-before').fire('click');element('western-second-yes').fire('click');
  element('detail-mode').fire('click');assert.equal(run('activeScreen'),'detail');
  element('manipulation-mode').fire('click');assert.equal(run('activeScreen'),'manipulation');
  assert.equal(failed.request.signal.aborted,false,'Returning from Detailed log must keep the search running');
  failed.send('error',null,'later search failed');failed.close();await failed.task;
  assert.equal(run('snowsEncounterTiming'),'before');assert.equal(run('westernSecondOverride'),true);
  assert.match(element('outcome').textContent,/remaining route failed/);assert.equal(element('export').disabled,true);
  element('snows-encounter-after').fire('click');assert.equal(run('westernSecondOverride'),null);
  assert.equal(element('southern-encounter-name').textContent,'Encounter: After');
  console.log('PASS: detail navigation during search and usable selected prefix after a later error');

  const galdera=await start(35,'galdera');
  galdera.send('progress',galderaSnapshot(35));await flush();
  element('snows-encounter-before').fire('click');
  assert.equal(run('snowsEncounterTiming'),'before');
  element('detail-mode').fire('click');element('galdera-mode').fire('click');
  assert.equal(run('activeScreen'),'manipulation');assert.equal(galdera.request.signal.aborted,false);
  for(const type of ['progress','complete']) {
    galdera.send(type,galderaSnapshot(35,type!=='complete'));await flush();
    assert.equal(run('snowsEncounterTiming'),'before');
    assert.match(element('winter-crackridge-encounter').textContent,/Before harbor/);
    for(const id of ['western-second-no','western-second-yes']) {
      assert.equal(element(id).hidden,true,'Galdera must not offer a second Winterbloom encounter');
      element(id).fire('click');assert.equal(run('westernSecondOverride'),null);
    }
    assert.equal(element('western-second-routine').innerHTML,'');
  }
  galdera.close();await galdera.task;
  console.log('PASS: Galdera preserves Osvald timing while keeping both second-encounter controls hidden');

  const reset=await start(40);reset.send('progress',snapshot(40));await flush();
  element('voice-reset').fire('click');await reset.task;
  assert(reset.request.signal.aborted);assert.equal(run('voiceStage'),'order');
  assert.equal(run('result'),null);assert.equal(element('results').hidden,true);
  assert.equal(element('voice-status').textContent,'Choose turn-3 order: Left = Throné · Right = Pirro');
  // A failure before the first new snapshot must never revive a previous run.
  const empty=await start(50);empty.send('error',null,'no new prefix');empty.close();await empty.task;
  assert.equal(run('result'),null);assert.equal(element('results').hidden,true);
  console.log('PASS: Start over aborts the pending search; new-run failure cannot resurrect stale results');

  context.fixture=snapshot(60);run('result=fixture;running=true;render();');
  run('result.post_battle_field.flats_escape.early_cait.southern_rare_target=true;render();');
  element('snows-encounter-before').fire('click');assert.equal(run('snowsEncounterTiming'),null);
  run('result.post_battle_field.flats_escape.early_cait.southern_rare_target=false;result.post_battle_field.flats_escape.early_cait.snows_approach.predicted_before_osvald=true;render();');
  element('snows-encounter-after').fire('click');assert.equal(run('snowsEncounterTiming'),null);
  assert.equal(element('southern-encounter-name').textContent,'Encounter: Before');
  console.log('PASS: route-imposed unavailable alternatives remain blocked during search');
})().catch(e=>{console.error(e);process.exitCode=1;});

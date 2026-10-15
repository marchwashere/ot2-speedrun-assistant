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
assert.match($('galdera-coast-routine').innerHTML,/switch to night after the Head Cook steal/);
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
coast.eastern_sai_encounter={...coast.southern_sai,menu:{first_actor:0,rng:entry.after}};
coast.sai=entry;context.conningStages=[];
vm.runInNewContext(source.slice(start,end),context);
assert.equal($('eastern-sai-result').hidden,false);
assert.match($('eastern-sai-routine').innerHTML,/T1 · Throné · 1st command menu/);
assert.match($('eastern-sai-routine').innerHTML,/Escape \(100%\)/);
assert(context.conningStages.some(([label])=>label==='Sai · loaded, chapter start discarded'));
console.log('PASS: separate Eastern and Southern encounter cards, followed by Sai');
coast.oresrush_supplies={warrior:entry,oresrush:entry,night:entry,hire:entry,day:entry,pickaxe:entry};
context.conningStages=[];
vm.runInNewContext(source.slice(start,end),context);
assert.deepEqual(context.conningStages.slice(-6).map(([label])=>label),[
 'Sai · Warrior License acquired','Oresrush · fast travel, chapter start discarded','Oresrush · switched to night',
 'Oresrush · Smooth Talker Peddler hired · 1,600 leaves','Oresrush · switched to day after hire','Oresrush · Sturdy Pickaxe stolen · 100% · change path action, then close without dialogue']);
console.log('PASS: Warrior License, night Hire and Pickaxe Entreat in Detailed log');
coast.oresrush_supplies.augmentor={sai:entry,night:entry,skull_helm:entry,east_district:entry,entreat:entry,day:entry,jam_steal:entry};
context.conningStages=[];
vm.runInNewContext(source.slice(start,end),context);
assert.equal(context.conningStages.at(-3)[0],'Sai: East District · Elemental Augmentor entreated · change path action, then close without dialogue');
assert.equal(context.conningStages.at(-7)[0],'Sai · fast travel back during day, chapter start discarded');
assert.equal(context.conningStages.at(-6)[0],'Sai · switch to night before Skull Helm Entreat');
console.log('PASS: Sai return, East District, night Entreat with dialogue skip');
coast.oresrush_supplies.winterbloom={arrival:entry,night:entry,sword_quest:entry,soldier_entreat:entry};
context.conningStages=[];
vm.runInNewContext(source.slice(start,end),context);
assert.deepEqual(context.conningStages.slice(-3).map(([label])=>label),[
 'Winterbloom · switch to night before pickaxe delivery',
 'Winterbloom · deliver Sturdy Pickaxe · Rusty Sword, Revitalizing Jam, 7,000 leaves',
 'Winterbloom · Fortifying Nut (L) entreated from Soldier · change path action, then close without dialogue']);
console.log('PASS: Winterbloom quest and Soldier Entreat with dialogue skip');
coast.oresrush_supplies.wellgrove={sai:entry,day:entry,southern_sai:entry,
 western_sai:{...coast.southern_sai,menu:{first_actor:1,rng:entry.after}},eastern_wellgrove:entry};
context.conningStages=[];
vm.runInNewContext(source.slice(start,end),context);
assert.equal($('western-sai-result').hidden,false);
assert.match($('western-sai-routine').innerHTML,/T1 · Osvald · 1st command menu/);
assert.match($('western-sai-routine').innerHTML,/Escape \(100%\)/);
assert.deepEqual(context.conningStages.slice(-6).map(([label])=>label),[
 'Sai · fast travel from Winterbloom, chapter start discarded','Sai · switched to day immediately',
 'Southern Sai Sands · crossed without encounter','Western Sai Sands · loaded',
 'Western Sai Sands · escaped, field restored','Eastern Wellgrove Trail · loaded']);
console.log('PASS: return to Sai daytime, no Southern fight, one Western fight before Wellgrove');
coast.oresrush_supplies.wellgrove.additional_time_switches=4;
vm.runInNewContext(source.slice(start,end),context);
assert.match($('western-sai-routine').innerHTML,/switch night → day 2 more time\(s\)/);
console.log('PASS: additional Sai time cycles are shown only when needed to avoid ambush');
coast.oresrush_supplies.wellgrove.western_escape_boost_cycles=3;
coast.oresrush_supplies.wellgrove.champion={approach:entry,steal:entry,exit:entry,wellgrove:entry,
 encounter:{...coast.southern_sai,menu:{first_actor:0,rng:entry.after}}};
context.conningStages=[];
vm.runInNewContext(source.slice(start,end),context);
assert.match($('western-sai-routine').innerHTML,/Boost 0 → 1 → 0 × 3/);
assert.equal($('eastern-wellgrove-result').hidden,false);
assert.match($('eastern-wellgrove-routine').innerHTML,/T1 · Throné · 1st command menu/);
assert.match($('eastern-wellgrove-routine').innerHTML,/Escape \(100%\)/);
assert.deepEqual(context.conningStages.slice(-5).map(([label])=>label),[
 'Eastern Wellgrove Trail · run 100% to Man with Horses without encounter',
 'Eastern Wellgrove Trail · Champion Seeker and Strengthening Serum stolen · 100% · change path action, then close without dialogue',
 'Eastern Wellgrove Trail · after Champion Seeker steal · escaped, field restored',
 'Eastern Wellgrove Trail · run 100% to Wellgrove without a second encounter',
 'Wellgrove · entered, chapter start discarded']);
console.log('PASS: Western boosts, steal dialogue skip, later encounter and Wellgrove entry');
coast.oresrush_supplies.wellgrove.champion.supplies={pom_steal:entry,night:entry,mother_steal:entry,magic_entreat:entry,day:entry,dancer_license:entry};
context.conningStages=[];
vm.runInNewContext(source.slice(start,end),context);
assert.deepEqual(context.conningStages.slice(-6).map(([label])=>label),[
 'Wellgrove · Merchant: steal 2 Energizing Pomegranates (M) · change path action, then close without dialogue',
 "Wellgrove · Girl's Mother: steal Empowering Lychee (M) · 100% · change path action, then close without dialogue",
 'Wellgrove · switched to night after Lychee steal',
 'Wellgrove · Mysterious Merchant north of tavern: Magic Nut (L) entreated · change path action, then close without dialogue',
 'Wellgrove · switch to day immediately after Magic Nut (L) Entreat',
 'Wellgrove · Dancer License acquired']);
console.log('PASS: daytime Pom M x2 Steal, night switch, Entreats and Dancer License in Detailed log');
coast.oresrush_supplies.wellgrove.champion.supplies.al_return={new_delsta:entry,highroad:entry,delivery:entry};
context.conningStages=[];
vm.runInNewContext(source.slice(start,end),context);
assert.deepEqual(context.conningStages.slice(-3).map(([label])=>label),[
 'New Delsta · fast travel after Dancer License, chapter start discarded',
 'New Delsta Highroad · entered for Al',
 'Al · recovered bag delivered · 1,500 leaves, Slippery Nut, Healing Grape ×3']);
console.log('PASS: New Delsta day switch, Highroad and Al delivery in Detailed log');
coast.oresrush_supplies.wellgrove.champion.supplies.al_return.flamechurch={highroad_exit:entry,clockbank_entry:entry,
 additional_time_switches:0,encounter:{...coast.southern_sai,menu:{first_actor:3,rng:entry.after}},clockbank_exit:entry,eastern_flamechurch:entry};
context.conningStages=[];
vm.runInNewContext(source.slice(start,end),context);
assert.equal($('galdera-clockbank-result').hidden,false);
assert.match($('galdera-clockbank-routine').innerHTML,/T1 · Partitio · 1st command menu/);
assert.match($('galdera-clockbank-routine').innerHTML,/run 100%/);
assert.match($('galdera-clockbank-routine').innerHTML,/Escape \(100%\)/);
assert(!context.conningStages.some(([label])=>label==='Western Clockbank Highroad · night/day adjustment'));
assert.equal(context.conningStages.at(-1)[0],'Eastern Flamechurch Pass · entered');
coast.oresrush_supplies.wellgrove.champion.supplies.al_return.flamechurch.additional_time_switches=2;
coast.oresrush_supplies.wellgrove.champion.supplies.al_return.flamechurch.adjustment=entry;
vm.runInNewContext(source.slice(start,end),context);
assert.match($('galdera-clockbank-routine').innerHTML,/switch night → day 1 time\(s\)/);
console.log('PASS: Western Clockbank full-run encounter and optional ambush adjustment before Flamechurch');

const fc=coast.oresrush_supplies.wellgrove.champion.supplies.al_return.flamechurch;fc.escape_boost_cycles=2;fc.movement={run_percent:45.8};fc.pass_exit=entry;fc.flamechurch=entry;
vm.runInNewContext(source.slice(start,end),context);
assert.match($("galdera-clockbank-routine").innerHTML,/Boost 0 → 1 → 0 × 2 before Escape/);
assert.match($("galdera-clockbank-routine").innerHTML,/run first 45.8% · walk remaining 54.2%/);
assert.equal(context.conningStages.at(-1)[0],"Flamechurch · entered, chapter start discarded");

fc.temenos={introduction:entry,field_return:entry,recruitment:entry,battle:{actor:1,boost_cycles:2,at_command_menu:entry.after,after:entry.after,trace:[]}};
vm.runInNewContext(source.slice(start,end),context);
assert.equal($('galdera-temenos-result').hidden,false);
assert.match($('galdera-temenos-routine').innerHTML,/Osvald/);
assert.match($('galdera-temenos-routine').innerHTML,/Boost 0 → 1 → 0 × 2/);
assert.match($('galdera-temenos-routine').innerHTML,/Boost to \+1 BP → regular attack/);
assert.match($('galdera-temenos-routine').innerHTML,/replace Agnea/);

fc.temenos.quicksand={setup:entry,sai:entry,southern_sai:entry,western_sai:{...coast.southern_sai,menu:{first_actor:2,rng:entry.after}},quicksand:entry};
vm.runInNewContext(source.slice(start,end),context);
assert.equal($('galdera-quicksand-result').hidden,false);
assert.match($('galdera-quicksand-routine').innerHTML,/T1 · Temenos/);
assert.match($('galdera-quicksand-routine').innerHTML,/Osvald: learn Collect, then → Warrior · Temenos → Dancer/);
assert.match($('galdera-quicksand-routine').innerHTML,/Escape \(100%\)/);
assert.equal(context.conningStages.at(-1)[0],'Quicksand Gaol · entered');

assert.doesNotMatch($('galdera-quicksand-routine').innerHTML,/switch night|time adjustment/);

fc.temenos.quicksand.lord_sands={night:entry,prompt:entry,field_return:entry,victory:entry,battle:{at_command_menu:entry.after,after:entry.after,trace:[],money_spent:15000,rewards:{exp:533,jp:37,money:1188},actions:[{turn:1,actor:2,bp:0,name:'Holy Light',report:{rng_before:entry.after,rng_after:entry.after}},{turn:2,actor:3,bp:3,name:'Hired Help: Beastlings',report:{rng_before:entry.after,rng_after:entry.after}}]}};
vm.runInNewContext(source.slice(start,end),context);
assert.equal($('galdera-lord-sands-result').hidden,false);
assert.match($('galdera-lord-sands-routine').innerHTML,/T1 · Temenos · Holy Light/);
assert.match($('galdera-lord-sands-routine').innerHTML,/T2 · Partitio · \+3 BP/);
assert.equal(context.conningStages.at(-1)[0],'Lord of the Sands · defeat event complete');

fc.temenos.quicksand.lord_sands.finisher_boost_cycles=2;
fc.temenos.quicksand.loot={day:entry,rusty_axe:entry,scorched_bone_spear:entry,encounter:{...coast.southern_sai,menu:{first_actor:2,rng:entry.after}}};
vm.runInNewContext(source.slice(start,end),context);
assert.match($('galdera-lord-sands-routine').innerHTML,/Boost 0 → 1 → 0 × 2/);
assert.equal($('galdera-quicksand-loot-result').hidden,false);
assert.match($('galdera-quicksand-loot-routine').innerHTML,/Rusty Axe/);
assert.match($('galdera-quicksand-loot-routine').innerHTML,/Temenos · Escape/);
assert.match($('galdera-quicksand-loot-routine').innerHTML,/Scorched Bone Spear/);
assert.equal(context.conningStages.at(-1)[0],'Quicksand Gaol · Scorched Bone Spear obtained');
fc.temenos.quicksand.gravell={arrival:entry,supplies:entry,setup:entry,night:entry,ambush:entry,restoration:entry,day:entry,sales:entry,serum_purchase:entry,lychee_steal:entry,jam_steal:entry,sale_proceeds:250000};
vm.runInNewContext(source.slice(start,end),context);
assert.match(context.conningStages.at(-1)[0],/Reinforcing Jam stolen \(100%\); switch path action/);
assert(context.conningStages.some(([label])=>/250,000 leaves/.test(label)));

assert(context.conningStages.some(([label])=>/Old Man Empowering Lychee \(L\) and Sharp Nut \(L\)/.test(label)));
assert(context.conningStages.some(([label])=>/buy Diffusing Serum at Provisioner · 3,000 leaves/.test(label)));
assert(!context.conningStages.some(([label])=>/Wellgrove tavern/.test(label)));
console.log('PASS: relocated Sharp L and serum appear in Gravell, with no Wellgrove tavern stop');

context.boostStepsHtml=(inputs,label)=>label;
const segment={name:'Travel',stages:[['Travel entry',entry]],encounters:[],extra_switches:0,escape_boost_inputs:[]};
const x=fc.temenos.quicksand.flex={location:'Nameless Isle',order:[2,1,3,4],segments:[],boat:segment,continuation:[segment],
 encounter:coast.southern_sai.encounter,encounter_arrival:entry,
 fight:{trace:[],actions:[{round:1,actor:0,inputs:[],before:entry.after,action:{springy_boots:{target:1}}},{round:2,actor:1,inputs:[1],before:entry.after,action:'fireball'}]},
 rewards:{awarded:{jp:12210},trace:[]},field_return:entry,remaining_errands:[],
 nameless_jp:{setup:entry,water_entry:entry,movement:{run_percent:75}}};
vm.runInNewContext(source.slice(start,end),context);
assert.match($('galdera-flex-routine').innerHTML,/JP ×100: Nameless Isle/);
assert.match($('galdera-flex-routine').innerHTML,/Springy Boots → Osvald/);
assert.match($('galdera-flex-routine').innerHTML,/Fireball \+1 BP/);
assert.doesNotMatch($('galdera-flex-routine').innerHTML,/Winged Landwalker/);
x.location='Shipwreck';x.order.push(5);x.nameless_jp=null;
x.nameless={setup:entry,approach:{encounter:coast.southern_sai,escape_inputs:[],run_percent:75,water_entry:entry,gigantes_entry:entry},night:entry,prompt:entry,field_return:entry,victory:entry,battle:{trace:[]},loot:segment,finisher_boost_cycles:1};
x.shipwreck={travel:segment};x.fight.actions[1].action='fireball_max';
vm.runInNewContext(source.slice(start,end),context);
assert.match($('galdera-flex-routine').innerHTML,/JP ×100: Shipwreck/);
assert.match($('galdera-flex-routine').innerHTML,/Gigantes/);
assert.match($('galdera-flex-routine').innerHTML,/Fireball \+3 BP/);
assert(context.conningStages.some(([label])=>label==='Shipwreck · JP ×100 victory and field return'));
console.log('PASS: Nameless and Shipwreck selected routes, Boots target, Fireball BP, and Gigantes continuation');
const defended={...coast.southern_sai,defends:[{actor:0,rng_before:entry.after},{actor:1,rng_before:entry.after}],defend_rounds:[1,2],escape_round:2};
x.segments=[{...segment,encounters:[defended]}];
x.nameless.approach.encounter=defended;
vm.runInNewContext(source.slice(start,end),context);
assert.match($('galdera-flex-routine').innerHTML,/T1 · Throné · Defend/);
assert.match($('galdera-flex-routine').innerHTML,/T2 · Osvald · Defend/);
assert.match($('galdera-flex-routine').innerHTML,/T2 · Temenos · Escape/);
assert(context.conningPoints.some(([label])=>/Nameless T2 Osvald Defend/.test(label)));
console.log('PASS: Defend actors/rounds, later Escape menu and detailed checkpoints');



fc.temenos.quicksand.pre_gravell=[{...segment,name:'Sai / Firestarter',stages:[['Firestarter chest',entry]],encounters:[]},{...segment,name:'Crackridge book',stages:[['From the Far Reaches of Hell chest',entry]],encounters:[]}];
fc.temenos.quicksand.flex=null;
vm.runInNewContext(source.slice(start,end),context);
const prefixHtml=$('galdera-quicksand-loot-routine').innerHTML;
assert(prefixHtml.indexOf('Firestarter chest')<prefixHtml.indexOf('From the Far Reaches of Hell chest'));
assert(prefixHtml.indexOf('From the Far Reaches of Hell chest')<prefixHtml.indexOf('Teleport to Gravell'));
console.log('PASS: Firestarter/book/Gravell instructions visible before JP search completes');

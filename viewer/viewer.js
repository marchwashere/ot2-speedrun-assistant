'use strict';
const $ = id => document.getElementById(id);
const escapeHtml = value => String(value).replace(/[&<>"']/g, c => ({'&':'&amp;','<':'&lt;','>':'&gt;','"':'&quot;',"'":'&#39;'}[c]));
const encounterLabel = prediction => {
  const counts=new Map();
  for(const slot of prediction?.enemies ?? []) {
    const name=slot.enemy.name;counts.set(name,(counts.get(name)??0)+1);
  }
  return counts.size?'Encounter: '+[...counts].map(([name,n])=>name+(n>1?` ×${n}`:'')).join(' · '):'';
};
const hex = value => '0x' + value.toString(16).toUpperCase().padStart(8, '0');
let result = null;
let snowsEncounterTiming = null;
let westernSecondOverride = null;
let running = false;
let stopped = false;
const repeatSettings = [
  {key:'early_clerics_max_boost_repeats',id:'early-clerics-boost-repeats',storage:'ot2-fork-early-clerics-boost-repeats',label:'Early clerics',requiredDefault:true},
  {key:'early_approach_max_boost_repeats',id:'early-cait-boost-repeats',storage:'ot2-fork-early-cait-boost-repeats',label:'Avoid surprise before or after Osvald',requiredDefault:true},
  {key:'mansion_run_max_boost_repeats',id:'mansion-run-boost-repeats',storage:'ot2-assistant-mansion-run-boost-repeats',label:'Mansion no second encounter',requiredDefault:true},
  {key:'mug_clerics_max_boost_repeats',id:'mug-clerics-boost-repeats',storage:'ot2-assistant-mug-clerics-boost-repeats',label:'Black-market clerics',requiredDefault:true},
  {key:'crackridge_approach_max_boost_repeats',id:'crackridge-approach-boost-repeats',storage:'ot2-assistant-crackridge-approach-boost-repeats',label:'Crackridge Harbor approach',requiredDefault:true},
  {key:'encounter_max_boost_repeats',id:'encounter-boost-repeats',storage:'ot2-assistant-encounter-boost-repeats',label:'Encounter avoidance'},
  {key:'second_fight_max_boost_repeats',id:'second-fight-boost-repeats',storage:'ot2-assistant-second-fight-boost-repeats',label:'Second fight',legacy:'ot2-assistant-max-boost-repeats'},
  {key:'mansion_approach_max_boost_repeats',id:'mansion-approach-boost-repeats',storage:'ot2-assistant-mansion-approach-boost-repeats',label:'Gold Ring and mansion approach'},
  {key:'mansion_escape_max_boost_repeats',id:'mansion-escape-boost-repeats',storage:'ot2-assistant-mansion-escape-boost-repeats',label:'Mansion Escape',strategy:true},
  {key:'pirro_victory_max_boost_repeats',id:'pirro-victory-boost-repeats',storage:'ot2-assistant-pirro-victory-boost-repeats',label:'Pirro victory'},
  {key:'pirro_no_grape_max_boost_repeats',id:'pirro-no-grape-boost-repeats',storage:'ot2-assistant-pirro-no-grape-boost-repeats',label:'Pirro no Healing Grape',legacy:'ot2-assistant-pirro-boost-repeats'},
  {key:'highroad_approach_max_boost_repeats',id:'highroad-approach-boost-repeats',storage:'ot2-assistant-highroad-approach-boost-repeats',label:'Highroad approach'},
  {key:'highroad_escape_max_boost_repeats',id:'highroad-escape-boost-repeats',storage:'ot2-assistant-highroad-escape-boost-repeats',label:'Highroad Escape',strategy:true},
  {key:'highroad_flats_max_boost_repeats',id:'highroad-flats-boost-repeats',storage:'ot2-assistant-highroad-flats-boost-repeats',label:'Flats walking'},
  {key:'highroad_flats_approach_max_boost_repeats',id:'highroad-flats-approach-boost-repeats',storage:'ot2-assistant-highroad-flats-approach-boost-repeats',label:'Flats approach'},
  {key:'flats_escape_max_boost_repeats',id:'flats-escape-boost-repeats',storage:'ot2-assistant-flats-escape-boost-repeats',label:'Flats Escape',strategy:true},
  {key:'flats_snow_max_boost_repeats',id:'flats-snow-boost-repeats',storage:'ot2-assistant-flats-snow-boost-repeats',label:'Post-village walk',requiredDefault:true},
  {key:'flats_osvald_max_boost_repeats',id:'flats-osvald-boost-repeats',storage:'ot2-assistant-flats-osvald-boost-repeats',label:'Osvald walk',requiredDefault:true}
];
const hardNoSurprise = new Set(['mansion_approach_max_boost_repeats','highroad_approach_max_boost_repeats','highroad_flats_approach_max_boost_repeats','early_approach_max_boost_repeats','crackridge_approach_max_boost_repeats']);
let eventRepeatCaps = {};
const lastRepeatCaps = {};
const validRepeatCap = value => /^[0-9]+$/.test(value) && Number(value) <= 256;
const repeatCapLabel = value => value == null ? 'Required' : value;
const repeatCapQuery = (value,key) => value === null ? (repeatSettings.find(s=>s.key===key)?.strategy ? 'boost_only' : 'required') : (value ?? 4);
for (const setting of repeatSettings) {
  eventRepeatCaps[setting.key] = setting.requiredDefault ? null : 4;
  lastRepeatCaps[setting.key] = 4;
  try {
    // Each legacy setting migrates only to its named target.
    const saved = hardNoSurprise.has(setting.key)?'required':localStorage.getItem(setting.storage) ?? (setting.legacy ? localStorage.getItem(setting.legacy) : null);
    const last = localStorage.getItem(`${setting.storage}-last-limit`) ?? (setting.legacy ? localStorage.getItem(`${setting.legacy}-last-limit`) : null);
    if (last !== null && validRepeatCap(last)) lastRepeatCaps[setting.key] = Number(last);
    if (['unlimited','required','boost_only'].includes(saved)) eventRepeatCaps[setting.key] = null;
    else if (saved !== null && validRepeatCap(saved)) eventRepeatCaps[setting.key] = lastRepeatCaps[setting.key] = Number(saved);
  } catch { /* Storage may be unavailable; retain this event's default. */ }
}
// The user enabled cost-based breaking for this route. Migrate the previous
// boost-only preference once; later explicit Boost only selections still work.
try {
  if(localStorage.getItem('ot2-weighted-escape-strategies-v1')!=='1') {
    for(const s of repeatSettings.filter(s=>s.strategy)) {
      eventRepeatCaps[s.key]=4;localStorage.setItem(s.storage,'4');
    }
    localStorage.setItem('ot2-weighted-escape-strategies-v1','1');
  }
} catch {for(const s of repeatSettings.filter(s=>s.strategy)) eventRepeatCaps[s.key]=4;}
const names = {attack_sword:'Sword',attack_dagger:'Dagger',attack_axe:'Axe',defend:'Defend',ABI_ENE_0041:'Attack',ABI_ENE_0500:'Slash'};
Object.assign(names,{ABI_SET_THI_020:'Darkest Night',ABI_SET_GST_0170:'Slice',ABI_SET_GST_0190:'Pierce Through',ABI_ENE_0200:'Double Thrust',ABI_ENE_0010:'Attack'});
const reasons = {actor_initialization:'Actor initialization',current_order_shuffle:'First order shuffle',battle_start_speaker:'Opening speaker',battle_start_voice:'Opening voice check',ability_voice:'Ability voice check',break_voice:'Break reaction check',kill_voice:'Kill reaction check',next_order_shuffle:'Next order shuffle',order_voice:'Turn-start voice check',boost_voice:'Boost voice check',normal_accuracy:'Cached accuracy check',attack_voice:'Attack voice check',last_attack_voice:'Final-hit voice check',critical_hit_and_damage:'Critical / hit / damage',damage_notification_timer:'Damage notification timer',miss_notification_timer:'Miss notification timer',guard_disease:'Guard status checks',guard_voice:'Guard voice check',round_timing_revision:'Round timing revision',enemy_ai_and_target:'Enemy AI and target',enemy_drop:'Enemy item drop',field_encounter_reset_b_then_a:'Area counter reset (B then A)'};
const actorName = id => result.actors[id]?.name ?? '—';
reasons.boost_reduce = 'Reduce boost (no CRT draw)';
Object.assign(reasons, {break_slow_timer:'Break timer',broken_hit_slow_timer:'Broken-hit timer',round_bp_timer:'Round-start BP timer',post_battle_event_encounter_reset_b_then_a:'Post-battle event counter reset (B then A)'});
Object.assign(reasons, {checkpoint_pirro_npc_frame:'Checkpoint: Pirro NPC starting frame',checkpoint_scaracci_npc_frame:'Checkpoint: Scaracci NPC starting frame',checkpoint_counter_reset_b_then_a:'Checkpoint counter reset (B then A)',checkpoint_ambush_popup:'Ambush tutorial (no additional draws)'});
Object.assign(reasons, {ambush_menu_action_timer:'Open path-action menu: action timer',ambush_scripted_dialogue:'Ambush: scripted dialogue (lottery bypassed)',ambush_counter_reset_b_then_a:'Ambush event counter reset (B then A)',rejoin_counter_reset_b_then_a:'Skipped rejoin cutscene: counter reset (B then A)'});
reasons.second_fight_serial_event_start='Serial event → battle (no counter reset)';
Object.assign(reasons, {
});
const shortActorName = id => actorName(id).split(' · ')[0];
const safeName = id => escapeHtml(actorName(id));
const totalAdvances = rng => (BigInt(result.crt_setup.battle_entry_advances) + BigInt(rng.draws)).toString();
const setupDescription = setup => `${setup.advances} at title + ${12 + Number(setup.recreate_shared_level)} opening draws`;
function showError(message) { $('error').textContent = message; $('error').hidden = false; }

function validateSeed(text) {
  if (!/^(?:[0-9]+|0x[0-9a-f]+)$/i.test(text)) throw new Error('Enter a decimal seed or 0x-prefixed hexadecimal seed.');
  const seed = Number(text);
  if (!Number.isInteger(seed) || seed < 0 || seed > 0xFFFFFFFF) throw new Error('Seed must be between 0 and 4294967295 (0xFFFFFFFF).');
  return seed;
}

let activeRoute = 'any_hikari';
const routeResults = new Map();
const routeNames = () => ['Throné',activeRoute === 'galdera' ? 'Osvald' : 'Hikari','Agnea','Partitio'];
function readSetup() {
  const seed = validateSeed($('seed').value.trim());
  const text = $('advances').value.trim();
  if (!/^[0-9]+$/.test(text)) throw new Error('Advances must be a nonnegative whole number.');
  const advances = BigInt(text);
  const recreate = $('recreate-shared-level').getAttribute('aria-pressed') === 'true';
  if (advances + 12n + BigInt(recreate) > 18446744073709551615n) throw new Error('Title advances plus opening transition exceed 18446744073709551615.');
  return {route:activeRoute,seed,advances:advances.toString(),recreate_shared_level:recreate,route_end:'osvald_encounter',...eventRepeatCaps};
}
const setupQuery = setup => {
  const query = new URLSearchParams({route:setup.route ?? activeRoute,seed:setup.seed,advances:setup.advances,recreate_shared_level:setup.recreate_shared_level,route_end:setup.route_end ?? 'osvald_encounter',boost_menu:setup.boost_menu ?? 0,boost_advances:setup.boost_advances ?? 0});
  for (const {key} of repeatSettings) query.set(key,repeatCapQuery(setup[key],key));
  for (const key of ['boost_repeat_cap','second_boost_menu','second_boost_advances','second_boost_repeat_cap']) {
    if(setup[key]!=null) query.set(key,setup[key]);
  }
  return query.toString()+['pirro_boost_inputs','pirro_r8_boost_inputs'].map(key=>setup[key]==null?'':`&${key}=${setup[key].join(',')}`).join('');
};
function renderSetup(setup) {
  $('prepared-state').textContent = `${setup.battle_entry_advances} advances`;
  $('prepared-help').textContent = `${hex(setup.battle_seed)} · ${setupDescription(setup)} · advances since initial seed`;
}
function markSetupChanged() {
  $('error').hidden = true;
  $('results').hidden = true;
  $('status').textContent = 'Inputs changed — run simulation.';
  $('simulate').textContent = 'Run simulation ↗';
  // Detailed log remains the last completed result, including its own setup.
}

let activeScreen = 'manipulation';
const screenScrollPositions = new Map();
function showScreen(screen) {
  if (!['manipulation','detail'].includes(screen)) return;
  if (screen === activeScreen) return;
  const currentPosition = {left:window.scrollX, top:window.scrollY};
  screenScrollPositions.set(activeScreen, currentPosition);
  const position = screenScrollPositions.get(screen) ?? currentPosition;
  activeScreen = screen;
  for (const name of ['manipulation','detail','field','settings']) {
    $(`${name}-panel`).hidden = name !== screen;
    $(`${name}-mode`).setAttribute('aria-pressed', String(name === screen && (name !== 'manipulation' || activeRoute === 'any_hikari')));
  }
  $('galdera-mode').setAttribute('aria-pressed',String(screen === 'manipulation' && activeRoute === 'galdera'));
  window.dispatchEvent(new CustomEvent('screen-changed', {detail:screen}));
  window.scrollTo({...position, behavior:'instant'});
  requestAnimationFrame(() => {
    if (activeScreen === screen) window.scrollTo({...position, behavior:'instant'});
  });
}
function selectRoute(route) {
  if (running || stopped || (typeof voiceBusy !== 'undefined' && voiceBusy)) return;
  const previous=activeRoute;
  routeResults.set(activeRoute,{result,snowsEncounterTiming,westernSecondOverride});
  activeRoute=route;
  ({result=null,snowsEncounterTiming=null,westernSecondOverride=null}=routeResults.get(route)||{});
  $('manipulation-title').textContent=route==='galdera'?'Galdera':'Any% Hikari';
  showScreen('manipulation');
  $('manipulation-mode').setAttribute('aria-pressed',String(route==='any_hikari'));
  $('galdera-mode').setAttribute('aria-pressed',String(route==='galdera'));
  $('error').hidden=true;
  window.dispatchEvent(new CustomEvent('route-changed',{detail:{route,previous}}));
  if(result){if(result.crt_setup)renderSetup(result.crt_setup);render();}else{
    $('results').hidden=true;
    $('detail-empty').hidden=false;
    $('detail-results').hidden=true;
    $('status').textContent='Run simulation for this route.';
  }
}
$('manipulation-mode').addEventListener('click',()=>selectRoute('any_hikari'));
$('galdera-mode').addEventListener('click',()=>selectRoute('galdera'));
for (const name of ['detail','field','settings']) $(`${name}-mode`).addEventListener('click',()=>showScreen(name));

async function simulate(event) {
  event?.preventDefault();
  if (running || stopped) return;
  $('error').hidden = true;
  let setup;
  try { setup = readSetup(); }
  catch(error) { showError(error.message); return; }
  $('seed').value = hex(setup.seed);
  running = true;
  $('simulate').disabled = true;
  $('quit').disabled = true;
  $('seed').disabled = true;
  $('advances').disabled = true;
  $('recreate-shared-level').disabled = true;
  updateRepeatControls();
  $('save-settings').disabled = true;
  $('simulate').textContent = 'Simulating…';
  $('status').textContent = 'Simulating…';
  try {
    const response = await fetch(`/api/boost-search?${setupQuery(setup)}`);
    const calculation = await response.json();
    if (!response.ok) throw new Error(calculation.error || 'Simulation failed.');
    if (!calculation.prediction) {
      if(calculation.search.event_caps.second_fight == null) throw new Error('Required outcome unavailable: checkpoint safety and one-action second fight.');
      const cap = calculation.search.event_caps.encounter;
      throw new Error(cap == null ? 'Required checkpoint outcome unavailable after exhausting supported candidates.' : `No encounter-dodge plan found within its ${cap}-repeat limit.`);
    }
    const data = calculation.prediction;
    data.boost_search = calculation.search;
    result = data;
    snowsEncounterTiming = null;
    westernSecondOverride = null;
    renderSetup(data.crt_setup);
    render();
    window.dispatchEvent(new Event('battle-simulated'));
    $('status').textContent = '';
  } catch(error) {
    showError(error instanceof TypeError ? 'Cannot reach the local simulator. Relaunch OT2 Battle Viewer and try again.' : error.message);
    $('results').hidden = true;
    $('status').textContent = 'Simulation failed. Previous result remains in Detailed log.';
  } finally {
    running = false;
    $('simulate').disabled = false;
    $('quit').disabled = false;
    $('seed').disabled = false;
    $('advances').disabled = false;
    $('recreate-shared-level').disabled = false;
    updateRepeatControls();
    $('save-settings').disabled = false;
    $('simulate').textContent = 'Run simulation ↗';
  }
}

function render() {
  if(result.identified_entry) {renderIdentified();return;}
  const state = result.state;
  const after = state.opening_crt.after_battle;
  $('results').hidden = false;
  const unmet=result.required_outcomes?.unmet || [];
  $('required-outcomes-error').hidden=!unmet.length;
  const blocked=result.required_outcomes?.blocker;
  const unevaluated=result.required_outcomes?.not_evaluated || [];
  $('required-outcomes-error').textContent=unmet.length ? (blocked ? 'Route simulation stopped · '+blocked : 'Required target not reached · '+unmet[0].label)+'. '+(unevaluated.length?'Later required outcomes were not evaluated. ':'')+'See Detailed log.' : '';
  $('detail-results').hidden = false;
  $('detail-empty').hidden = true;
  $('outcome').textContent = state.outcome === 'victory' ? 'Predicted victory' : `Predicted ${state.outcome}`;
  $('outcome').className = 'pill ' + (state.outcome === 'victory' ? 'success' : 'failure');
  const setup = result.crt_setup;
  $('result-context').textContent = `Seed ${hex(setup.initial_seed)} + ${setupDescription(setup)} · Start ${hex(result.initial_seed)} · Round ${state.round} · ${result.steps.length} actions`;
  $('final-hex').textContent = hex(state.rng.state);
  $('copy').textContent = 'Copy hex';
  $('final-advances').textContent = totalAdvances(state.rng);
  const checkpoint = result.post_battle_field?.checkpoint;
  const ambush = result.post_battle_field?.ambush;
  const second = result.post_battle_field?.second_battle;
  renderSecond(second);
  $('checkpoint-advances').textContent = checkpoint ? totalAdvances(checkpoint.after_popup) : 'Not reached';
  $('checkpoint-hex').textContent = checkpoint ? hex(checkpoint.after_popup.state) : '—';
  const received = result.steps.flatMap(s=>s.report.events).filter(e=>e.kind==='damage' && result.actors[e.target]?.side==='player').reduce((sum,e)=>sum+e.amount,0);
  const damageFree = state.outcome === 'victory' && received === 0;
  $('damage-free').textContent = damageFree ? 'Yes' : 'No';
  $('damage-free').className = damageFree ? 'text-good' : 'text-bad';
  const enemyTurns = result.battle_assessment.enemy_turns.length;
  $('damage-help').textContent = received || enemyTurns || state.outcome!=='victory' ? `${received} damage · ${enemyTurns} enemy turn${enemyTurns===1?'':'s'}${state.outcome==='victory'?'':' · '+state.outcome}` : '';
  const reach = result.post_battle_field?.reset_reach;
  const mixed = reach?.mixed;
  $('avoid-verdict').textContent = !reach ? 'Unavailable' : !mixed ? 'Encounter unavoidable · adjust the checkpoint search in Settings' : `Run ≈${mixed.max_running_percent}%`;
  $('avoid-verdict').className = 'run-state ' + (reach?.walking.reaches_reset ? 'text-good' : 'text-bad');
  $('avoid-help').textContent = mixed ? `${mixed.max_running_updates} run + ${mixed.walking_updates} walk updates` : '';
  $('draws').textContent = state.rng.draws;
  $('draws-help').textContent = `${setup.battle_entry_advances} before battle · ${totalAdvances(state.rng)} total from initial seed`;
  $('counter').textContent = after?.encounter_steps ?? '—';
  renderBoostPlan();
  $('drops').textContent = after ? `${after.healing_grapes} Healing Grape${after.healing_grapes === 1 ? '' : 's'} dropped` : 'No victory rewards';
  $('route-checks').innerHTML = battleAssessmentHtml() + (result.post_battle_field ? resetReachHtml(result.post_battle_field.reset_reach) : '<p>No completed field frame available.</p>');
  const checkpoints = [
    ['Before initialization', {state:result.initial_seed,draws:0}],
    ...result.steps.filter(step => result.actors[step.report.actor].side === 'player')
      .map(step => [`R${step.round} · ${shortActorName(step.report.actor)} command menu`, step.report.rng_before]),
    ...(after ? [['Fight ended',after.rng_before_rewards],['After drops',after.rng_after_rewards],['After first area reset',after.rng_after_field_reset],['After post-battle event reset',after.rng_after_post_battle_event]] : [['Battle ended',state.rng]]),
    ...(checkpoint ? [['Checkpoint arrival · before cutscene',checkpoint.arrival],['After checkpoint cutscene + Ambush popup · expected',checkpoint.after_popup]] : []),
    ...(ambush ? [['Path-action menu open · before confirming Ambush',ambush.at_menu],['After Ambush · rejoin cutscene starts',ambush.after_ambush],['After skipped rejoin cutscene · expected',ambush.after_rejoin]] : []),
    ...(second ? [
      ['Second fight · before initialization · expected',second.entry],
      ...second.steps.filter(s=>second.actors[s.report.actor].side==='player').map(s=>[`Second fight R${s.round} · ${second.actors[s.report.actor].name} command menu`,secondRng(second,s.report.rng_before)]),
      ['Second fight · after drops · expected',second.after]
    ] : [])
  ];
  $('checkpoints').innerHTML = setup.transition_checkpoints.map(c=>`<tr><td>${escapeHtml(c.label)}</td><td>${c.total_advances}</td><td>${hex(c.state)}</td><td>—</td></tr>`).join('') + checkpoints.map(([label,rng]) => `<tr><td>${escapeHtml(label)}</td><td>${totalAdvances(rng)}</td><td>${hex(rng.state)}</td><td>${rng.draws}</td></tr>`).join('');
  $('party').innerHTML = result.actors.map((meta,i) => {
    if (meta.side !== 'player') return '';
    const r = state.actors[i].resources;
    return `<div class="member"><div class="member-heading"><span>${escapeHtml(meta.name)}</span><span>${r.hp} / ${meta.max_hp} HP</span></div><progress max="${meta.max_hp}" value="${r.hp}" aria-label="${escapeHtml(meta.name)} HP"></progress><small>${r.sp} / ${meta.max_sp} SP · ${r.bp} BP</small></div>`;
  }).join('');
  $('enemy-outcome').textContent = result.actors.map((meta,i) => meta.side === 'enemy' ? `${shortActorName(i)}: ${state.actors[i].resources.hp} HP` : '').filter(Boolean).join(' · ');
  $('initial-order').textContent = result.initial_state.queue.map(shortActorName).join(' → ');
  $('log-count').textContent = `${result.steps.length} actions · ${state.rng.draws} CRT draws`;
  $('actions').innerHTML = result.steps.map((s,i)=>renderAction(s,i)).join('');
  const titleTrace = setup.transition_checkpoints.slice(1).map((c,i)=>{
    const before=setup.transition_checkpoints[i];
    return `<tr><td>${escapeHtml(c.label)}</td><td>—</td><td>${before.total_advances}</td><td>${c.total_advances}</td><td>+${c.transition_draws-before.transition_draws}</td><td>${hex(before.state)}</td><td>${hex(c.state)}</td></tr>`;
  }).join('');
  $('trace').innerHTML = titleTrace + [...state.opening_crt.trace,...(checkpoint?.trace || []),...(ambush?.trace || [])].map(span => `<tr><td>${escapeHtml(reasons[span.reason] || span.reason.replaceAll('_',' '))}</td><td>${span.actor === null ? '—' : safeName(span.actor)}</td><td>${totalAdvances(span.before)}</td><td>${totalAdvances(span.after)}</td><td>+${span.after.draws-span.before.draws}</td><td>${hex(span.before.state)}</td><td>${hex(span.after.state)}</td></tr>`).join('');
  if(second) {
    const spans=[...second.transition_trace,...second.state.opening_crt.trace.map(s=>({...s,before:secondRng(second,s.before),after:secondRng(second,s.after)}))];
    $('trace').innerHTML += spans.map(s=>`<tr><td>Second fight · ${escapeHtml(reasons[s.reason] || s.reason.replaceAll('_',' '))}</td><td>${s.actor===null?'—':escapeHtml(second.actors[s.actor].name)}</td><td>${totalAdvances(s.before)}</td><td>${totalAdvances(s.after)}</td><td>+${BigInt(s.after.draws)-BigInt(s.before.draws)}</td><td>${hex(s.before.state)}</td><td>${hex(s.after.state)}</td></tr>`).join('');
    $('trace').innerHTML += (second.city?.trace || []).map(s=>`<tr><td>${escapeHtml(reasons[s.reason] || s.reason.replaceAll('_',' '))}</td><td>—</td><td>${totalAdvances(s.before)}</td><td>${totalAdvances(s.after)}</td><td>+${BigInt(s.after.draws)-BigInt(s.before.draws)}</td><td>${hex(s.before.state)}</td><td>${hex(s.after.state)}</td></tr>`).join('');
    if(second.city) $('checkpoints').innerHTML += `<tr><td>After New Delsta radar popup</td><td>${totalAdvances(second.city.after_popup)}</td><td>${hex(second.city.after_popup.state)}</td><td>${second.city.after_popup.draws}</td></tr>`;
    if(second.parlor) {
      const p=second.parlor;
      $('trace').innerHTML += p.trace.map(s=>`<tr><td>${escapeHtml(s.reason)}</td><td>—</td><td>${totalAdvances(s.before)}</td><td>${totalAdvances(s.after)}</td><td>+${BigInt(s.after.draws)-BigInt(s.before.draws)}</td><td>${hex(s.before.state)}</td><td>${hex(s.after.state)}</td></tr>`).join('');
      $('checkpoints').innerHTML += [['Entered parlor',p.after_entry],['After first parlor cutscene · before next trigger / Yes',p.after_first_scene],['After two further cutscenes · returned to backstreets',p.after_return]].map(([label,rng])=>`<tr><td>${label}</td><td>${totalAdvances(rng)}</td><td>${hex(rng.state)}</td><td>${rng.draws}</td></tr>`).join('');
    }
    if(second.parlor_revisit) {
      const p=second.parlor_revisit;
      $('trace').innerHTML += p.trace.map(s=>`<tr><td>${escapeHtml(s.reason)}</td><td>—</td><td>${totalAdvances(s.before)}</td><td>${totalAdvances(s.after)}</td><td>+${BigInt(s.after.draws)-BigInt(s.before.draws)}</td><td>${hex(s.before.state)}</td><td>${hex(s.after.state)}</td></tr>`).join('');
      $('checkpoints').innerHTML += [['Second parlor visit · entered',p.after_entry],['Second parlor visit · backstreets arrival before dialogue/tutorial',p.after_return],['Second parlor visit · after Steal popup',p.after_popup]].map(([label,rng])=>`<tr><td>${label}</td><td>${totalAdvances(rng)}</td><td>${hex(rng.state)}</td><td>${rng.draws}</td></tr>`).join('');
    }
    if(second.mansion) {
      const p=second.mansion;
      $('trace').innerHTML += p.trace.map(s=>`<tr><td>${escapeHtml(s.reason)}</td><td>—</td><td>${totalAdvances(s.before)}</td><td>${totalAdvances(s.after)}</td><td>+${BigInt(s.after.draws)-BigInt(s.before.draws)}</td><td>${hex(s.before.state)}</td><td>${hex(s.after.state)}</td></tr>`).join('');
      $('checkpoints').innerHTML += [...(p.gold_ring_before_clothes&&p.after_gold_ring?[['Gold Ring stolen',p.after_gold_ring]]:[]),['Clothes stolen · after Save popup',p.after_clothes],...(!p.gold_ring_before_clothes&&p.after_gold_ring?[['Gold Ring stolen',p.after_gold_ring]]:[]),['Boy · Shadow Soulstone stolen',p.after_boy],['New Delsta main area loaded',p.after_main_load],['Artist · three soulstones stolen',p.after_artist],['Mansion entry · control returned',p.after_entry]].map(([label,rng])=>`<tr><td>${label}</td><td>${totalAdvances(rng)}</td><td>${hex(rng.state)}</td><td>${rng.draws}</td></tr>`).join('');
      const a=p.lookout_ambush;
      $('trace').innerHTML += a.trace.map(s=>`<tr><td>${escapeHtml(s.reason)}</td><td>—</td><td>${totalAdvances(s.before)}</td><td>${totalAdvances(s.after)}</td><td>+${BigInt(s.after.draws)-BigInt(s.before.draws)}</td><td>${hex(s.before.state)}</td><td>${hex(s.after.state)}</td></tr>`).join('');
      $('checkpoints').innerHTML += [['Mansion lookout · Ambush menu open',a.at_menu],['Mansion lookout · after Ambush',a.after]].map(([label,rng])=>`<tr><td>${label}</td><td>${totalAdvances(rng)}</td><td>${hex(rng.state)}</td><td>${rng.draws}</td></tr>`).join('');
    }
  }
  renderMansionEscape();
  renderPirro();
  renderPirroAftermath();
  renderHighroad();
  renderFlats();
  renderHarborReturn();
  renderFlatsEscape();
  renderVillageVisit();
  renderVillageReturn();
  renderOsvald();
  renderEarlyCait();
  renderCapeCold();
  renderMug();
  renderCrackridgeVoyage();
  renderActionVisibility();
}

// Keep every simulated action event visible, including zero-adjustment plans.
// Diagnostics remain in Detailed log.
function renderActionVisibility() {
  const field=result.post_battle_field;
  for(const node of document.querySelectorAll('[data-diagnostic-event]')) {
    const key=node.dataset.diagnosticEvent;
    node.hidden=!(key==='village_return'?field?.flats_escape?.village_return:field?.[key]);
  }
  $('first-battle-result').hidden=!!result.identified_entry;
  $('checkpoint-run-result').hidden=!field?.reset_reach;
  const second=field?.second_battle;
  $('second-battle-result').hidden=!second;
  const mansion=field?.mansion_escape;
  // Escape and the onward run are player instructions even with zero extra boosts.
  $('mansion-escape-result').hidden=!mansion;
  const pirroInstructions=!!$('pirro-boost-routine').textContent.trim();
  $('pirro-boost-heading').hidden=!pirroInstructions;
  $('pirro-result').hidden=!field?.pirro_battle;
  const highroad=field?.highroad;
  $('highroad-result').hidden=!highroad;
  const flats=field?.flats_escape;
  $('flats-escape-result').hidden=!flats;
}

function attackLabel(name,boost=0) {
  return `${boost>0?'+'+boost+' ':''}${name}`;
}
function stepActionLabel(step,simulation) {
  if(!step) return '';
  const events=step.report.events;
  const action=events.find(e=>e.kind==='action');
  const bp=events.find(e=>e.kind==='resource_changed' && e.resource==='bp' && e.actor===step.report.actor);
  const weapon=action?.name?.match(/^attack_(.+)$/)?.[1];
  const name=weapon?weapon[0].toUpperCase()+weapon.slice(1):names[action?.name] || simulation.action_names?.[action?.name] || action?.name || 'Action';
  const label=attackLabel(name,bp?Math.max(0,bp.before-bp.after):0);
  const actors=simulation.actors || [];
  const playerCount=actors.filter(a=>a.side==='player').length;
  const enemyCount=actors.filter(a=>a.side==='enemy').length;
  const target=(weapon || action?.name==='ABI_SET_THI_020')?singleTargetPosition(events,playerCount,enemyCount):'';
  return target?label+' → '+target:label;
}
function instructionBoxHtml(parts,command='') {
  const lines=command?[...parts,command]:parts;
  return lines.length?'<div class="boost-instructions">'+lines.map(c=>`<span class="boost-step">${escapeHtml(c)}</span>`).join('')+'</div>':'';
}
function boostStepsHtml(levels,command='') {
  const parts=groupBoostRoutine(levels || []).map(c=>`${c.from} → ${c.to}${c.count>1?' ×'+c.count:''}`);
  if(!(levels?.length) || levels.length===levels[levels.length-1]) parts.unshift('No extra boost adjustment needed');
  return instructionBoxHtml(parts,command);
}
function menuOrdinal(n) { return n===1?'1st':n===2?'2nd':n===3?'3rd':n+'th'; }
function safeDefendsHtml(b) {
  return (b.defends||[]).map(d=>`<p><strong>R${d.round} · ${d.actor===1?'Osvald':'Throné'} · ${menuOrdinal(d.menu_ordinal)} command menu</strong></p>${instructionBoxHtml(['Defend'])}`).join('');
}
function singleTargetPosition(events,playerCount,enemyCount) {
  if(enemyCount<=1) return '';
  const targets=[...new Set(events.filter(e=>['damage','miss'].includes(e.kind) && Number.isInteger(e.target) && e.target>=playerCount).map(e=>e.target-playerCount))];
  if(targets.length!==1) return '';
  const slot=targets[0];
  if(slot<0 || slot>=enemyCount) return '';
  return slot===0?'rightmost enemy':slot===enemyCount-1?'leftmost enemy':enemyCount===3?'middle enemy':menuOrdinal(slot+1)+' enemy from right';
}
function breakAttackLabel(b,menu) {
  const names={ABI_SET_THI_020:'Darkest Night',ABI_SET_SCH_010:'Fireball',ABI_SET_SCH_020:'Icewind'};
  const events=b.report?.events || [];
  const action=events.find(e=>e.kind==='action' && e.actor===b.attack_actor)?.name || b.skill;
  const weapon=action?.match(/^attack_(.+)$/)?.[1] || b.weapon;
  const label=attackLabel(names[action] || names[b.skill] || (weapon[0].toUpperCase()+weapon.slice(1)),b.boost);
  const aoe=['ABI_SET_SCH_010','ABI_SET_SCH_020','ABI_SET_SCH_030'].includes(action);
  const playerCount=menu?.party_levels?.length;
  const target=!aoe && playerCount?singleTargetPosition(events,playerCount,menu.current_order.length-playerCount):'';
  return target?label+' → '+target:label;
}
function breakEscapeHtml(s) {
  const b=s.break_action;
  return `<p><strong>R1 · Throné · 1st command menu</strong></p>${boostStepsHtml(b.attack_inputs,breakAttackLabel(b,s.menu))}${safeDefendsHtml(b)}<p><strong>R${b.escape_round} · ${b.escape_actor===1?'Osvald':'Throné'} · ${menuOrdinal(b.escape_menu_ordinal||(b.escape_round===1?2:1))} command menu</strong></p>${boostStepsHtml(b.escape_inputs,'Escape')}`;
}
function renderHighroad() {
  const p=result.post_battle_field?.highroad;
  $('highroad-result').hidden=!p;
  $('highroad-encounter-name').textContent=encounterLabel(p?.encounter);
  $('highroad-boost-routine').innerHTML='';
  $('highroad-escape-diagnostics').hidden=true;
  if(!p) return;
  if(p.error) { $('highroad-boost-routine').textContent=`Unavailable · ${p.error}`; return; }
  const s=p.escape, search=p.approach_search;
  if(search && !search.target_met) $('pirro-boost-routine').innerHTML+='<p class="text-bad">R8 adjustment unavailable · see Detailed log</p>';
  if(s?.break_action) {
    $('highroad-boost-routine').innerHTML=breakEscapeHtml(s);
  } else if(s?.inputs) {
    $('highroad-boost-routine').innerHTML='<p><strong>R1 · Throné · 1st command menu</strong></p>'+boostStepsHtml(s.inputs.levels,'Escape');
  } else $('highroad-boost-routine').textContent='Adjustment unavailable · see Detailed log';
  if(s?.flats_search && !s.flats_search.target_met) $('highroad-boost-routine').innerHTML+='<p class="text-bad">Flats walking target not met · increase its repeat limit in Settings</p>';
  if(s?.flats_approach_search && !s.flats_approach_search.target_met) $('highroad-boost-routine').innerHTML+='<p class="text-bad">Flats surprise target not met · increase its repeat limit in Settings</p>';
  const checkpoint=(label,rng)=>`<tr><td>${escapeHtml(label)}</td><td>${totalAdvances(rng)}</td><td>${hex(rng.state)}</td><td>${rng.draws}</td></tr>`;
  $('checkpoints').innerHTML+=checkpoint('Al introduction · popup dismissed',p.after_al)+checkpoint('Highroad encounter · before initialization',p.before_battle_initialization);
  if(s) $('checkpoints').innerHTML+=checkpoint('Highroad R1 · Throné · 1st command menu',s.menu.rng);
  if(s?.break_action) $('checkpoints').innerHTML+=checkpoint(`Highroad R${s.break_action.escape_round} · Escape menu after break`,s.break_action.at_escape_menu);
  if(s?.attempt) $('checkpoints').innerHTML+=checkpoint('Highroad Escape · after first check',s.attempt.after);
  if(p.field_return) {
    const r=p.field_return.after;
    $('highroad-escape-diagnostics').hidden=false;
    $('highroad-escape-advances').textContent=totalAdvances(r);
    $('highroad-escape-hex').textContent=hex(r.state);
    $('checkpoints').innerHTML+=checkpoint('Highroad Escape · returned to field',r);
  }
  let traces=[...p.trace], previous=p.after_al;
  for(const c of p.encounter.checkpoints) {
    const after={state:c.state,draws:BigInt(p.after_al.draws)+BigInt(c.draws)};
    traces.push({reason:c.label,before:previous,after});previous=after;
  }
  traces.push(...(s?.attempt?.trace || s?.menu.trace || []));
  traces.push(...(p.field_return?.trace || []));
  $('trace').innerHTML+=traces.map(t=>`<tr><td>Highroad · ${escapeHtml(reasons[t.reason] || t.reason)}</td><td>—</td><td>${totalAdvances(t.before)}</td><td>${totalAdvances(t.after)}</td><td>+${BigInt(t.after.draws)-BigInt(t.before.draws)}</td><td>${hex(t.before.state)}</td><td>${hex(t.after.state)}</td></tr>`).join('');
  $('pirro-verification').textContent+=` Highroad: Lv${p.level} · ${p.encounter.situation} · ${p.encounter.enemies.map(e=>e.enemy.name).join(', ')}. ${search?.status || ''}. ${s ? `${s.status}; Escape ${s.baseline.chance_percent}%.` : ''} ${s?.flats_search?.status || ''}. ${s?.flats_approach_search?.status || ''}. ${p.verification}`;
  if(p.formation_search) $('pirro-verification').textContent+=` Formation search: ${p.formation_search.baseline_cost} → ${p.formation_search.selected_cost} input-equivalents. ${p.formation_search.verification}`;
  if(p.formation_search_error) $('pirro-verification').textContent+=` Formation lookahead failed; retaining the valid baseline: ${p.formation_search_error}`;
}

function renderFlatsEscape() {
  const p=result.post_battle_field?.flats_escape;
  $('flats-escape-result').hidden=!p;
  $('flats-encounter-name').textContent=encounterLabel(p?.encounter);
  $('flats-escape-routine').innerHTML='';
  $('flats-escape-run').textContent='';
  $('flats-escape-diagnostics').hidden=true;
  if(!p) return;
  if(p.error) { $('flats-escape-routine').textContent=`Unavailable · ${p.error}`; return; }
  const s=p.escape;
  if(s?.break_action) {
    $('flats-escape-routine').innerHTML=breakEscapeHtml(s);
  } else if(s?.inputs) {
    $('flats-escape-routine').innerHTML='<p><strong>R1 · Throné · 1st command menu</strong></p>'+boostStepsHtml(s.inputs.levels,'Escape');
  } else $('flats-escape-routine').textContent=p.status;
  const checkpoint=(label,rng)=>`<tr><td>${escapeHtml(label)}</td><td>${totalAdvances(rng)}</td><td>${hex(rng.state)}</td><td>${rng.draws}</td></tr>`;
  $('checkpoints').innerHTML+=checkpoint('Flats encounter · before initialization',p.before_battle_initialization);
  if(s) $('checkpoints').innerHTML+=checkpoint('Flats R1 · Throné · 1st command menu',s.menu.rng);
  if(s?.break_action) $('checkpoints').innerHTML+=checkpoint('Flats · Escape menu after break',s.break_action.at_escape_menu);
  if(p.field_return) {
    const r=p.field_return.after;
    $('flats-escape-diagnostics').hidden=false;
    $('flats-escape-advances').textContent=totalAdvances(r);
    $('flats-escape-hex').textContent=hex(r.state);
    $('flats-escape-run').textContent=`Run ${p.remaining_updates} updates · ${p.remaining_run.counter_before_reset} counter left at village entrance`;
    $('checkpoints').innerHTML+=checkpoint('Flats Escape · returned to field',r)+checkpoint('Abandoned Village entrance · before loading',p.at_village_entrance);
  }
  const traces=[]; let previous=p.arrival;
  for(const c of p.encounter.checkpoints) {
    const after={state:c.state,draws:BigInt(p.arrival.draws)+BigInt(c.draws)};
    traces.push({reason:c.label,before:previous,after}); previous=after;
  }
  traces.push(...(s?.attempt?.trace || s?.menu.trace || []),...(p.field_return?.trace || []));
  $('trace').innerHTML+=traces.map(t=>`<tr><td>Flats · ${escapeHtml(reasons[t.reason] || t.reason)}</td><td>—</td><td>${totalAdvances(t.before)}</td><td>${totalAdvances(t.after)}</td><td>+${BigInt(t.after.draws)-BigInt(t.before.draws)}</td><td>${hex(t.before.state)}</td><td>${hex(t.after.state)}</td></tr>`).join('');
  $('pirro-verification').textContent+=` Flats: Lv${p.level} · ${p.encounter.situation} · ${p.encounter.enemies.map(e=>e.enemy.name).join(', ')}. Encounter at update ${p.first_encounter_update}/${p.run_updates}; reset counter must be ≥${p.required_counter}. ${s?.status || ''}. ${p.verification}`;
}

function renderVillageVisit() {
  const p=result.post_battle_field?.flats_escape?.village_visit;
  $('village-visit-log').hidden=!p;
  if(!p) return;
  $('village-visit-advances').textContent=totalAdvances(p.after);
  $('village-visit-hex').textContent=hex(p.after.state);
  $('village-visit-counter').textContent=`Counter ${p.world_after.encounter_counter} · daytime · Throné only · village tagged`;
  $('checkpoints').innerHTML+=`<tr><td>First Abandoned Village visit · tagged, tutorial dismissed</td><td>${totalAdvances(p.after)}</td><td>${hex(p.after.state)}</td><td>${p.after.draws}</td></tr>`;
  $('trace').innerHTML+=p.trace.map(t=>`<tr><td>${escapeHtml(t.reason)}</td><td>—</td><td>${totalAdvances(t.before)}</td><td>${totalAdvances(t.after)}</td><td>+${BigInt(t.after.draws)-BigInt(t.before.draws)}</td><td>${hex(t.before.state)}</td><td>${hex(t.after.state)}</td></tr>`).join('');
}

function renderEarlyCait() {
  const p=result.post_battle_field?.flats_escape?.early_cait;
  $('early-route-result').hidden=!p;
  $('early-route-log').hidden=!p;
  $('osvald-cait-result').hidden=!p?.snows_approach;
  $('snows-onward-result').hidden=true;
  $('eastern-snows-log').hidden=true;
  $('eastern-escape-result').hidden=true;
  $('eastern-escape-routine').innerHTML='';
  $('rare-fight-result').hidden=true;
  $('winterbloom-log').hidden=true;
  $('progression-log').hidden=true;
  $('eastern-onward-run').hidden=true;
  $('snows-escape-routine').innerHTML='';
  if(!p) return;
  const approach=p.snows_approach;
  const timing=p.southern_rare_target?'after':approach?.predicted_before_osvald?'before':snowsEncounterTiming ?? 'after';
  const branch=approach?.[timing+'_osvald'];
  const beforeOsvald=timing==='before';
  $('osvald-run-instruction').textContent=p.walk_to_osvald?'Walk to Osvald':'Run to Osvald';
  $('snows-encounter-predicted').textContent=approach?`Predicted: ${approach.predicted_before_osvald?'before':'after'} Osvald · ${p.walk_to_osvald?'70 walking':approach.measured_running_updates+' running'} updates`:'';
  $('snows-encounter-before').setAttribute('aria-pressed',String(beforeOsvald));
  $('snows-encounter-after').setAttribute('aria-pressed',String(!beforeOsvald));
  $('snows-encounter-after').disabled=!!approach?.predicted_before_osvald;
  $('snows-encounter-before').disabled=!!p.southern_rare_target;
  $('snows-encounter-status').textContent=branch?`${snowsEncounterTiming?'Selected':'Predicted'} encounter: ${beforeOsvald?'before':'after'} Osvald · ${branch.non_surprise?'No surprise':'Surprise'}`:'';
  const m=p.movement.mixed;
  $('early-route-run').textContent=!m?'Walking target not met':p.movement.running.reaches_reset?'Run 100%':`Run ≈${m.running_distance_percent}% · walk ${m.walking_distance_percent}%`;
  $('early-route-run').classList.toggle('text-bad',!m);
  $('early-route-switches').hidden=true;
  $('early-route-switches').textContent='After clerics time switch: 1 → day';
  $('early-route-warning').hidden=p.clerics_present && !!p.osvald?.after_recruitment;
  $('early-route-warning').textContent=!p.clerics_present?'Clerics target not met':!p.osvald?.after_recruitment?'Snows route not reached':'';
  const e=branch?.encounter ?? p.encounter;
  $('southern-encounter-name').textContent=encounterLabel(e);
  const encounterWorld=branch?.world ?? p.world_at_encounter;
  $('early-route-encounter').textContent=e?`${e.enemies.map(slot=>slot.enemy.name).join(' · ')} · ${e.situation} · ${encounterWorld?.time===4?'night':'day'} · after ${e.counter_updates} running updates from ${beforeOsvald?'Snows entry':'recruitment'}`:'First encounter not reached';
  const f=result.post_battle_field.flats_escape;
  $('early-route-cost').textContent=f.early_weighted_input_cost==null?'':`Estimated cost: ${f.early_weighted_input_cost} boost-input units · ${f.escape?.inputs?.input_count ?? 0} boost inputs + 2 cleric time switches ×7 · ${f.early_tested_projections} cached route candidates`;
  $('early-route-status').textContent=`Clerics: ${p.clerics_present?'yes':'no'} · no surprise before Osvald: ${approach?.before_osvald.non_surprise?'yes':'no'}${approach?.after_osvald_required?` · no surprise after Osvald: ${approach.after_osvald.non_surprise?'yes':'no'}`:' · post-Osvald opening not a search target'}`;
  const endpoint=branch?.before_battle_initialization ?? p.before_battle_initialization;
  $('early-cait-advances').textContent=endpoint?totalAdvances(endpoint):'—';
  $('early-cait-hex').textContent=endpoint?hex(endpoint.state):'—';
  const milestones=[['First Abandoned Village visit',p.village_visit.after],['Flats return · before early clerics',p.after_flats],['Early clerics · after night switch',p.after_night],['Early clerics · purchases complete, fixed switch to day',p.after_day],['Southern Cape Cold Snows · after loading',p.osvald?.after_loading],...(!beforeOsvald?[['Osvald · before recruitment',p.osvald?.at_osvald],['Osvald · recruited, tutorials dismissed',p.osvald?.after_recruitment]]:[]),[`First Snows encounter ${timing} Osvald · composition finalized, before enemy actor initialization`,endpoint]];
  $('checkpoints').innerHTML+=milestones.filter(([,r])=>r).map(([label,r])=>`<tr><td>${escapeHtml(label)}</td><td>${totalAdvances(r)}</td><td>${hex(r.state)}</td><td>${r.draws}</td></tr>`).join('');
  const osvaldTrace=(p.osvald?.trace||[]).filter(t=>!beforeOsvald || BigInt(t.after.draws)<=BigInt(p.osvald.after_loading.draws));
  const trace=[...p.village_visit.trace,...p.trace,...osvaldTrace,...(!beforeOsvald?(p.encounter_time_trace||[]):[])];
  if(e) {
    const at_encounter=branch?.start ?? p.after_osvald_switches ?? p.osvald.after_recruitment;
    let before=at_encounter;
    for(const c of e.checkpoints) {
      const after={state:c.state,draws:BigInt(at_encounter.draws)+BigInt(c.draws)};
      trace.push({reason:`First Snows encounter: ${c.label}`,before,after}); before=after;
    }
  }
  const exit=f.snows_escape?.[timing+'_osvald'];
  const exitError=f.snows_escape?.[timing+'_error'];
  const appendRare=v=>{
    const points=[['Rare fight · first command menu',v.battle.at_command_menu],['Rare fight · victory and drops',v.battle.after],['Rare fight · field return, two skills learned and Evasive Maneuvers equipped',v.after]];
    $('checkpoints').innerHTML+=points.map(([label,r])=>`<tr><td>${label}</td><td>${totalAdvances(r)}</td><td>${hex(r.state)}</td><td>${r.draws}</td></tr>`).join('');
    trace.push(...v.battle.trace,...v.trace);
    const ledger=v.world_after.character_progression;
    if(ledger) {
      $('progression-log').hidden=false;
      $('progression-details').innerHTML=v.world_after.party.map(id=>{
        const p=ledger[id],c=p.character;
        return `<p><strong>${id===7?'Throné':'Osvald'} · Lv${p.level}</strong><br>EXP ${p.experience} · JP ${p.jp_earned-p.jp_spent} (${p.jp_earned} earned / ${p.jp_spent} spent)<br>HP ${c.resources.hp}/${c.stats.max_hp} · SP ${c.resources.sp}/${c.stats.max_sp} · Speed ${c.stats.speed}</p>`;
      }).join('');
    }
  };
  const escapeRoutine=s=>{
    const b=s.break_action,traveler=i=>i===1?'Osvald':'Throné';
    if(b) return `<p><strong>R1 · ${traveler(b.attack_actor)} · 1st command menu</strong></p>${boostStepsHtml(b.attack_inputs,breakAttackLabel(b,s.menu))}${safeDefendsHtml(b)}<p><strong>R${b.escape_round} · ${traveler(b.escape_actor)} · ${menuOrdinal(b.escape_menu_ordinal||(b.escape_round===1?2:1))} command menu</strong></p>${boostStepsHtml(b.escape_inputs,'Escape')}`;
    return `<p><strong>R1 · ${traveler(s.menu.first_actor)} · 1st command menu</strong></p>${boostStepsHtml(s.inputs?.levels,'Escape')}`;
  };
  if(p.rare_victory) {
    $('snows-escape-routine').innerHTML=instructionBoxHtml(['SS M']);
    appendRare(p.rare_victory);
    const onward=p.after_rare;
    if(onward) {
      const entry=onward.eastern_entry,e=onward.eastern_encounter,w=onward.passage;
      $('snows-onward-result').hidden=false;
      $('snows-onward-title').textContent='Encounter exit → Eastern Cape Cold Snows';
      $('snows-onward-run').textContent=onward.onward.running.reaches_reset?'Run 100%':`Run ≈${onward.onward.running_distance_percent}% · walk ${onward.onward.walking_distance_percent}%`;
      $('eastern-escape-result').hidden=false;
      $('eastern-encounter-name').textContent=encounterLabel(e);
      $('eastern-onward-run').hidden=false;
      $('eastern-escape-routine').innerHTML=escapeRoutine(w.escape);
      $('eastern-snows-log').hidden=false;
      $('eastern-snows-advances').textContent=totalAdvances(entry.after);
      $('eastern-snows-hex').textContent=hex(entry.after.state);
      $('eastern-snows-counter').textContent=`Encounter counter: ${entry.world_after.encounter_counter} · Evasive Maneuvers · First encounter: ${e.enemies.map(e=>e.enemy.name).join(', ')} · ${e.situation}`;
      const points=[['Eastern Cape Cold Snows · after loading',entry.after],['Eastern Escape · command menu',w.escape.menu.rng],...(w.escape.break_action?[['Eastern Escape · menu after break',w.escape.break_action.at_escape_menu]]:[]),['Eastern Escape · field returned',w.field_return.after],['Western Winterbloom Snows · after loading',w.western_entry.after]];
      $('checkpoints').innerHTML+=points.map(([label,r])=>`<tr><td>${label}</td><td>${totalAdvances(r)}</td><td>${hex(r.state)}</td><td>${r.draws}</td></tr>`).join('');
      trace.push(...entry.trace);
      let before=entry.after;
      for(const c of e.checkpoints.slice(1)) {
        const after={state:c.state,draws:BigInt(entry.after.draws)+BigInt(c.draws)};
        trace.push({reason:`Eastern first encounter: ${c.label}`,before,after});before=after;
      }
      trace.push(...w.escape.attempt.trace,...w.field_return.trace,...w.western_entry.trace);
    }
  }
  if(exit) {
    const s=exit.escape,b=s.break_action;
    const traveler=i=>i===1?'Osvald':'Throné';
    if(b) {
      const weapon=b.weapon[0].toUpperCase()+b.weapon.slice(1);
      $('snows-escape-routine').innerHTML=`<p><strong>R1 · ${traveler(b.attack_actor)} · 1st command menu</strong></p>${boostStepsHtml(b.attack_inputs,breakAttackLabel(b,s.menu))}${safeDefendsHtml(b)}<p><strong>R${b.escape_round} · ${traveler(b.escape_actor)} · ${menuOrdinal(b.escape_menu_ordinal||(b.escape_round===1?2:1))} command menu</strong></p>${boostStepsHtml(b.escape_inputs,'Escape')}`;
    } else {
      $('snows-escape-routine').innerHTML=`<p><strong>R1 · ${traveler(s.menu.first_actor)} · 1st command menu</strong></p>${boostStepsHtml(s.inputs?.levels,'Escape')}`;
    }
    const move=exit.onward,entry=exit.eastern_entry;
    $('snows-onward-result').hidden=false;
    $('snows-onward-title').textContent=`${move.start} → Eastern Cape Cold Snows`;
    $('snows-onward-run').textContent=move.running.reaches_reset?'Run 100%':`Run ≈${move.running_distance_percent}% · walk ${move.walking_distance_percent}%`;
    $('eastern-snows-log').hidden=false;
    $('eastern-snows-advances').textContent=totalAdvances(entry.after);
    $('eastern-snows-hex').textContent=hex(entry.after.state);
    $('eastern-snows-counter').textContent=`Encounter counter: ${entry.world_after.encounter_counter}`;
    const points=[['Snows R1 · first command menu',s.menu.rng],...(b?[[`Snows R${b.escape_round} · Escape menu after break`,b.at_escape_menu]]:[]),['Snows Escape · field returned',exit.field_return.after],...(exit.recruitment?[['Osvald · recruited after Snows Escape',exit.recruitment.after]]:[]),['Eastern Cape Cold Snows · after loading',entry.after]];
    $('checkpoints').innerHTML+=points.map(([label,r])=>`<tr><td>${escapeHtml(label)}</td><td>${totalAdvances(r)}</td><td>${hex(r.state)}</td><td>${r.draws}</td></tr>`).join('');
    trace.push(...s.attempt.trace,...exit.field_return.trace,...(exit.recruitment?.trace||[]),...entry.trace);
    const eastern=exit.eastern_encounter;
    if(eastern) {
      $('eastern-escape-result').hidden=false;
      $('eastern-encounter-name').textContent=encounterLabel(eastern);
      $('eastern-snows-counter').textContent+=` · First encounter: ${eastern.enemies.map(e=>e.enemy.name).join(', ')} · ${eastern.situation}`;
      let before=entry.after;
      for(const c of eastern.checkpoints.slice(1)) {
        // Encounter checkpoints count from map entry; route checkpoints count from battle entry.
        const after={state:c.state,draws:BigInt(entry.after.draws)+BigInt(c.draws)};
        trace.push({reason:`Eastern first encounter: ${c.label}`,before,after});before=after;
      }
      $('checkpoints').innerHTML+=`<tr><td>Eastern Cape Cold Snows · before battle initialization</td><td>${totalAdvances(before)}</td><td>${hex(before.state)}</td><td>${before.draws}</td></tr>`;
    }
    if(exit.western) {
      const w=exit.western;
      $('eastern-escape-result').hidden=false;
      $('eastern-onward-run').hidden=false;
      $('eastern-escape-routine').innerHTML=escapeRoutine(w.escape);
      const points=[['Eastern Escape · command menu',w.escape.menu.rng],['Eastern Escape · field returned',w.field_return.after],['Western Winterbloom Snows · after loading',w.western_entry.after],['Western first encounter · before battle initialization',{state:w.western_encounter.final_state,draws:BigInt(w.western_entry.after.draws)+BigInt(w.western_encounter.draws)}]];
      $('checkpoints').innerHTML+=points.map(([label,r])=>`<tr><td>${label}</td><td>${totalAdvances(r)}</td><td>${hex(r.state)}</td><td>${r.draws}</td></tr>`).join('');
      trace.push(...w.escape.attempt.trace,...w.field_return.trace,...w.western_entry.trace);
      let before=w.western_entry.after;
      for(const c of w.western_encounter.checkpoints.slice(1)) {
        const after={state:c.state,draws:BigInt(w.western_entry.after.draws)+BigInt(c.draws)};
        trace.push({reason:`Western first encounter: ${c.label}`,before,after});before=after;
      }
      $('eastern-snows-counter').textContent+=` · Western: ${w.western_encounter.enemies.map(e=>e.enemy.name).join(', ')} · ${w.western_encounter.situation}`;
    }
    const victory=exit.rare_victory ?? exit.western?.rare_victory;
    if(victory) {
      if(exit.western) {
        $('rare-fight-result').hidden=false;
        $('western-encounter-name').textContent=encounterLabel(exit.western.western_encounter);
      } else {
        $('eastern-escape-routine').innerHTML=instructionBoxHtml(['SS M']);
      }
      appendRare(victory);
    }
  } else if(exitError) {
    $('snows-escape-routine').innerHTML=`<p class="text-bad">${escapeHtml(exitError)}</p>`;
  }
  const west=p.after_rare?.passage?.winterbloom ?? exit?.winterbloom_after_eastern ?? exit?.western?.winterbloom;
  if(west) {
    $('rare-fight-result').hidden=false;
    $('western-encounter-name').textContent=encounterLabel(west.first_encounter);
    $('western-first-routine').innerHTML=west.first_is_rare?instructionBoxHtml(['SS M']):escapeRoutine(west.first_escape);
    const second=west.predicted_second_encounter || westernSecondOverride===true;
    $('western-second-no').disabled=west.predicted_second_encounter;
    $('western-second-no').setAttribute('aria-pressed',String(!second));
    $('western-second-yes').setAttribute('aria-pressed',String(second));
    $('western-second-status').textContent=second?encounterLabel(west.second_encounter):'No second encounter predicted';
    $('western-second-routine').innerHTML=second?escapeRoutine(west.with_second.escape):'';
    const destination=second?west.with_second.town:west.without_second;
    $('winterbloom-log').hidden=false;
    $('winterbloom-advances').textContent=totalAdvances(destination.after);
    $('winterbloom-hex').textContent=hex(destination.after.state);
    $('winterbloom-verification').textContent=west.verification;
    const appendEscape=(s,returned,label)=>{
      const points=[[label+' · first command menu',s.menu.rng],[label+' · field returned',returned.after]];
      $('checkpoints').innerHTML+=points.map(([label,r])=>`<tr><td>${label}</td><td>${totalAdvances(r)}</td><td>${hex(r.state)}</td><td>${r.draws}</td></tr>`).join('');
      trace.push(...s.attempt.trace,...returned.trace);
    };
    if(west.first_escape) appendEscape(west.first_escape,west.first_field_return,'Western first Escape');
    if(second) appendEscape(west.with_second.escape,west.with_second.field_return,'Western second Escape');
    trace.push(...destination.trace);
    const winterPopup=destination.trace.find(t=>t.kind==='boundary' && t.reason==='Winterbloom: story prompt dismissed without starting chapter')?.after;
    for(const [label,r] of [['Winterbloom · arrival story popup declined',winterPopup],['Winterbloom · after night switch',destination.after]]) {
      if(r) $('checkpoints').innerHTML+=`<tr><td>${label}</td><td>${totalAdvances(r)}</td><td>${hex(r.state)}</td><td>${r.draws}</td></tr>`;
    }
  }
  $('trace').innerHTML+=trace.map(t=>`<tr><td>${escapeHtml(t.reason)}</td><td>—</td><td>${totalAdvances(t.before)}</td><td>${totalAdvances(t.after)}</td><td>+${BigInt(t.after.draws)-BigInt(t.before.draws)}</td><td>${hex(t.before.state)}</td><td>${hex(t.after.state)}</td></tr>`).join('');
  $('pirro-verification').textContent+=` ${p.verification}`;
}
for(const timing of ['before','after']) {
  $('snows-encounter-'+timing).addEventListener('click',()=>{
    if(!result || running) return;
    if(timing==='after' && result.post_battle_field?.flats_escape?.early_cait?.snows_approach?.predicted_before_osvald) return;
    snowsEncounterTiming=timing;
    westernSecondOverride=null;
    // Redraw from the immutable alternatives; never advance CRT or re-search
    // earlier controls merely because the player corrected encounter timing.
    render();
  });
}
for(const [id,value] of [['western-second-no',false],['western-second-yes',true]]) {
  $(id).addEventListener('click',()=>{if(!result||running||$(id).disabled)return;westernSecondOverride=value;render();});
}

function renderVillageReturn() {
  const p=result.post_battle_field?.flats_escape?.village_return;
  $('village-return-result').hidden=!p;
  if(!p) return;
  $('village-return-advances').textContent=totalAdvances(p.after);
  $('village-return-hex').textContent=hex(p.after.state);
  const r=p.snow_reach,m=r.mixed;
  const label=!m?'Walking target not met':r.running.reaches_reset?'Run 100%':`Run ≈${m.running_distance_percent}% · walk ${m.walking_distance_percent}%`;
  $('village-snow-run').textContent=label;
  $('village-snow-run').className='run-state '+(m?'text-good':'text-bad');
  $('village-snow-detail').textContent=`Counter ${r.initial_counter} · ${m?`${m.estimated_counter_at_exit} left · route-distance estimate`:'Walking needs counter 649'}`;
  for(const [label,rng] of [['Abandoned Village · tagged, popup dismissed',p.at_village],['Flats · returned from Abandoned Village',p.after],...(p.at_snow_entrance?[['Southern Cape Cold Snows entrance · before loading',p.at_snow_entrance]]:[])]) {
    $('checkpoints').innerHTML+=`<tr><td>${label}</td><td>${totalAdvances(rng)}</td><td>${hex(rng.state)}</td><td>${rng.draws}</td></tr>`;
  }
  $('trace').innerHTML+=p.trace.map(t=>`<tr><td>${escapeHtml(t.reason)}</td><td>—</td><td>${totalAdvances(t.before)}</td><td>${totalAdvances(t.after)}</td><td>+${BigInt(t.after.draws)-BigInt(t.before.draws)}</td><td>${hex(t.before.state)}</td><td>${hex(t.after.state)}</td></tr>`).join('');
  $('pirro-verification').textContent+=` ${p.verification} ${r.verification} ${result.post_battle_field.flats_escape.snow_search?.status||''}.`;
}

function renderOsvald() {
  const p=result.post_battle_field?.flats_escape?.osvald;
  $('osvald-result').hidden=!p;
  if(!p) return;
  const safe=p.walking.reaches_reset;
  $('osvald-walk').textContent=safe?'Walk to Osvald · no encounter':'Walking target not met';
  $('osvald-walk').className='run-state '+(safe?'text-good':'text-bad');
  $('osvald-counter').textContent=`Counter ${p.world_at_entry.encounter_counter} · ${safe?`${p.walking.counter_before_reset} left after 70 updates`:'walking needs 631'}`;
  $('osvald-diagnostics').hidden=!p.after_recruitment;
  if(p.after_recruitment) {
    $('osvald-advances').textContent=totalAdvances(p.after_recruitment);
    $('osvald-hex').textContent=hex(p.after_recruitment.state);
  }
  for(const [label,rng] of [['Southern Cape Cold Snows · after loading',p.after_loading],['Osvald · before recruitment',p.at_osvald],['Osvald · recruited, tutorials dismissed',p.after_recruitment]]) {
    if(rng) $('checkpoints').innerHTML+=`<tr><td>${label}</td><td>${totalAdvances(rng)}</td><td>${hex(rng.state)}</td><td>${rng.draws}</td></tr>`;
  }
  $('trace').innerHTML+=p.trace.map(t=>`<tr><td>Osvald · ${escapeHtml(t.reason)}</td><td>—</td><td>${totalAdvances(t.before)}</td><td>${totalAdvances(t.after)}</td><td>+${BigInt(t.after.draws)-BigInt(t.before.draws)}</td><td>${hex(t.before.state)}</td><td>${hex(t.after.state)}</td></tr>`).join('');
  $('pirro-verification').textContent+=` ${p.verification} ${result.post_battle_field.flats_escape.osvald_search?.status||''}.`;
}

function renderCapeCold() {
  const p=result.post_battle_field?.cape_cold;
  $('cape-cold-result').hidden=!p;
  $('cape-cold-error').hidden=!p?.error;
  $('cape-cold-error').textContent=p?.error?`Prediction unavailable · ${p.error}`:'';
  $('cape-cold-movement').hidden=!p || !!p.error;
  $('cape-cold-diagnostics').hidden=!p?.after_loading;
  if(!p || p.error) return;
  const r=p.reach,m=r.mixed;
  $('cape-cold-run').textContent=!m?'Walking target not met':r.running.reaches_reset?'Run 100%':`Run ≈${m.running_distance_percent}% · walk ${m.walking_distance_percent}%`;
  $('cape-cold-run').className='run-state '+(m?'text-good':'text-bad');
  $('cape-cold-counter').textContent=`Counter ${r.initial_counter} · ${m?`${m.estimated_counter_at_exit} left · route-distance estimate`:'Walking needs counter 487'}`;
  if(p.after_loading) {
    $('cape-cold-advances').textContent=totalAdvances(p.after_loading);
    $('cape-cold-hex').textContent=hex(p.after_loading.state);
  }
  for(const [label,rng] of [['Cape Cold entrance · before loading',p.at_entrance],['Cape Cold · after loading',p.after_loading]]) {
    if(rng) $('checkpoints').innerHTML+=`<tr><td>${escapeHtml(label)}</td><td>${totalAdvances(rng)}</td><td>${hex(rng.state)}</td><td>${rng.draws}</td></tr>`;
  }
  $('trace').innerHTML+=p.trace.map(t=>`<tr><td>Cape Cold · ${escapeHtml(t.reason)}</td><td>—</td><td>${totalAdvances(t.before)}</td><td>${totalAdvances(t.after)}</td><td>+${BigInt(t.after.draws)-BigInt(t.before.draws)}</td><td>${hex(t.before.state)}</td><td>${hex(t.after.state)}</td></tr>`).join('');
  $('pirro-verification').textContent+=` ${r.verification} ${p.verification}`;
}

function renderMug() {
  const p=result.post_battle_field?.cape_cold_mug;
  const market=result.post_battle_field?.black_market;
  const search=market?.search;
  const reach=market?.cleric_reach;
  $('cleric-approach-result').hidden=!reach;
  $('cleric-approach-run').textContent=!reach?'':reach.running.reaches_reset?'Run 100%':reach.mixed?`Run ${reach.mixed.max_running_percent}% · walk ${(100-reach.mixed.max_running_percent).toFixed(1).replace(/\.0$/,'')}%`:'Encounter unavoidable while walking';
  $('cleric-approach-run').classList.toggle('text-bad',!!reach && !reach.mixed);
  $('cleric-approach-detail').textContent=reach?`Flats entry → cleric spawns: ${reach.walking.updates} updates at either speed (40 measured + 2 for player error). Entry counter ${reach.initial_counter} · walking cost ${reach.walking.cost} · running cost ${reach.running.cost}. ${reach.mixed?`Safe mix: ${reach.mixed.max_running_updates} run + ${reach.mixed.walking_updates} walk updates; ${reach.mixed.counter_before_reset} counter left.`:'Walking triggers an encounter.'} World state retains the walking reference counter.`:'';
  const warning=market?.error || (search && !search.met ? 'Clerics target not met · adjust its search in Settings.' : '');
  $('mug-log').hidden=!p;
  $('mug-details').hidden=!p || !!p.error;
  $('mug-error').hidden=!p?.error;
  $('mug-error').textContent=p?.error?`Prediction unavailable · ${p.error}`:'';
  $('mug-result').hidden=!p;
  const instructions=[];
  if(search?.extra_advances) {
    const parts=groupBoostRoutine(search.inputs.levels).map(c=>c.from+'→'+c.to+(c.count>1?' ×'+c.count:''));
    instructions.push('<h3>Boost adjustment</h3><p><strong>R2 · Osvald · 1st command menu</strong></p>'+instructionBoxHtml(parts));
  } else instructions.push('<p><strong>R2 · Osvald · 1st command menu</strong></p>'+instructionBoxHtml(['No extra boost adjustment needed']));
  if(p?.mug?.survived_round_two) instructions.push('<p><strong>R3 · Osvald · 1st command menu</strong></p>'+instructionBoxHtml([],'Staff'));
  if(search?.cape_time_switches) {
    const n=search.cape_time_switches;
    instructions.push('<p class="action-context">After Mug · Cape Cold</p><p class="run-action">Cape Cold time switch: '+n+'</p>');
    if(!market.switched_at_village) instructions.push('<p class="action-note">Skip time switch in Abandoned Village</p>');
  }
  if(warning) instructions.push('<p class="text-bad">'+escapeHtml(warning)+'</p>');
  $('mug-adjustment').innerHTML=p?.error?'<p class="text-bad">Prediction unavailable · '+escapeHtml(p.error)+'</p>':instructions.join('');
  $('mug-actions').innerHTML='';
  if(!p || p.error) return;
  const battle=p.mug;
  const mugNames={ABI_SET_SCH_020:'Icewind',ABI_SET_SCH_010:'Fireball',attack_staff:'Staff attack',ABI_NPB_2570:'Stand Ground'};
  const actor=id=>id===0?'Osvald':id===1?'Man':'—';
  $('mug-context').textContent=`Lv1 Osvald · ${battle.survived_round_two?'Staff finisher needed':'Man defeated after Fireball'}`;
  $('mug-advances').textContent=totalAdvances(p.after);
  $('mug-hex').textContent=hex(p.after.state);
  $('mug-verification').textContent=p.verification;
  let round=1, hp=280;
  $('mug-actions').innerHTML=battle.steps.map(s=>{
    const action=s.events.find(e=>e.kind==='action');
    const bp=s.events.find(e=>e.kind==='resource_changed' && e.resource==='bp' && e.actor===s.actor);
    const damage=s.events.filter(e=>e.kind==='damage').map(e=>e.amount);
    const health=s.events.find(e=>e.kind==='resource_changed' && e.resource==='hp' && e.actor===1);
    if(health) hp=health.after;
    const label=(mugNames[action?.name]||action?.name||'Action')+(s.actor===0?` · ${bp?bp.before-bp.after:0} BP`:'');
    const row=`<tr><td>R${round}</td><td>${actor(s.actor)}</td><td>${escapeHtml(label)}</td><td>${damage.length?damage.join(' + '):'—'}</td><td>${hp}</td><td>${totalAdvances(s.rng_before)}</td><td>${hex(s.rng_before.state)}</td></tr>`;
    const next=s.events.find(e=>e.kind==='round_started');
    if(next) round=next.round;
    return row;
  }).join('');
  const checkpoints=[['Cape Cold · night switch',p.after_night],['Mug · before battle initialization',p.before_battle],...battle.menus.map(m=>[`Mug · R${m.round} Osvald command menu`,m.rng]),['Mug · victory and drop',battle.after],['Mug · loot dismissed, field returned',p.after]];
  if(market && !market.error) {
    for(const [i,s] of (market.cape_switches||[]).entries()) checkpoints.push([`Cape Cold · time switch ${i+1} · ${s.time===2?'day':'night'}`,s.after]);
    checkpoints.push(['Fast travel · Abandoned Village',market.after_travel],[market.switched_at_village?'Abandoned Village · day switch':'Abandoned Village · already day, no switch',market.after_day],['Black market route · Flats entry',market.after_flats],['Cleric spawns · before night switch',market.at_clerics],['Flats · night switch near market',market.after_night]);
    $('mug-verification').textContent+=' '+(market.clerics_present?'Clerics present. ':'Clerics not present. ')+market.verification;
  }
  $('checkpoints').innerHTML+=checkpoints.map(([label,rng])=>`<tr><td>${escapeHtml(label)}</td><td>${totalAdvances(rng)}</td><td>${hex(rng.state)}</td><td>${rng.draws}</td></tr>`).join('');
  // Field trace includes both pre-battle and return callbacks. Merge by draw
  // boundary so return draws follow the battle, including zero-draw spans.
  const trace=[...p.trace,...battle.trace,...(market?.trace||[])].sort((a,b)=>Number(a.before.draws)-Number(b.before.draws)||Number(a.after.draws)-Number(b.after.draws));
  $('trace').innerHTML+=trace.map(t=>`<tr><td>Mug · ${escapeHtml(reasons[t.reason]||t.reason.replaceAll('_',' '))}</td><td>${actor(t.actor)}</td><td>${totalAdvances(t.before)}</td><td>${totalAdvances(t.after)}</td><td>+${BigInt(t.after.draws)-BigInt(t.before.draws)}</td><td>${hex(t.before.state)}</td><td>${hex(t.after.state)}</td></tr>`).join('');
}

function renderCrackridgeVoyage() {
  $('cropdale-walk-result').hidden=true;
  $('cropdale-return-escape-result').hidden=true;
  $('oresrush-walk-result').hidden=true;
  $('oresrush-movement-log').hidden=true;
  $('oresrush-route-log').hidden=true;
  $('ryu-walk-result').hidden=true;
  $('ryu-town-walk-result').hidden=true;
  $('ryu-route-log').hidden=true;
  $('hikari-recruit-result').hidden=true;
  $('hikari-recruit-log').hidden=true;
  $('scholar-setup-log').hidden=true;
  $('canalbrine-route-log').hidden=true;
  $('canalbrine-water-result').hidden=true;
  $('canalbrine-water-log').hidden=true;
  for(const id of ['new-delsta-bg-result','crackridge-bg-result','clockbank-bg-result','oresrush-time-result','cropdale-time-result','conning-north-result','conning-west-result','conning-steal-result','inventor-escape-result','clockbank-movement-result','crackridge-return-result','southern-crackridge-result','western-crackridge-result','western-crackridge-return-result','gravell-result','conning-log']) $(id).hidden=true;
  const early=result.post_battle_field?.flats_escape?.early_cait;
  const timing=early?.southern_rare_target?'after':early?.snows_approach?.predicted_before_osvald?'before':snowsEncounterTiming ?? 'after';
  const exit=result.post_battle_field?.flats_escape?.snows_escape?.[timing+'_osvald'];
  const west=early?.after_rare?.passage?.winterbloom ?? exit?.winterbloom_after_eastern ?? exit?.western?.winterbloom;
  if(west) {
    const second=west.predicted_second_encounter || westernSecondOverride===true;
    const v=second?west.voyage_with_second:west.voyage_without_second;
    $('crackridge-switch-result').hidden=true;
    $('crackridge-travel-title').textContent='Winterbloom → New Delsta Harbor → Crackridge Harbor';
    $('crackridge-switch-instruction').textContent='Teleport: Winterbloom → New Delsta Harbor · Boat → Crackridge Harbor';
    $('crackridge-escape-result').hidden=false;
    $('winter-crackridge-encounter').textContent=encounterLabel(v.arrival.encounter);
    $('crackridge-escape-routine').innerHTML=v.escape.break_action?breakEscapeHtml(v.escape):`<p class="text-bad">Required break unavailable</p>`;
    $('winter-crackridge-run').textContent=`Crackridge Harbor encounter exit → Eastern Cropdale Trail · Run ${v.run_percent===100?'100':('≈'+v.run_percent.toFixed(1))}%`;
    $('cropdale-walk-result').hidden=false;
    const crop=v.cropdale_reach;
    $('cropdale-running-limit').textContent=crop.run_percent===100?'Run 100%':`Run ≈${crop.run_percent.toFixed(1)}% · walk ${(100-crop.run_percent).toFixed(1)}%`;
    $('crackridge-voyage-result').hidden=true;
    const switches=v.cropdale.time_switches||[];
    $('cropdale-time-result').hidden=true;
    const onward=v.cropdale_exit;
    $('cropdale-return-escape-result').hidden=false;
    $('cropdale-return-encounter').textContent=encounterLabel(onward.encounter);
    const actor=['Throné','Osvald','Agnea'][onward.escape.menu.first_actor];
    $('cropdale-return-escape-routine').innerHTML=`<p><strong>R1 · ${actor} · 1st command menu</strong></p>${boostStepsHtml(onward.escape.inputs?.levels,'Escape')}`;
    const oresrush=onward.oresrush_reach;
    $('oresrush-walk-result').hidden=false;
    $('oresrush-running-limit').textContent=!oresrush?'Walking target not met':oresrush.run_percent===100?'Run 100%':`Run ≈${oresrush.run_percent.toFixed(1)}% · walk ${(100-oresrush.run_percent).toFixed(1)}%`;
    $('oresrush-running-limit').classList.toggle('text-bad',!oresrush);
    $('oresrush-movement-log').hidden=false;
    $('oresrush-movement-budget').textContent=oresrush
      ? `Initial counter ${oresrush.initial_counter} · walking ${oresrush.walking_updates} updates / ${oresrush.walking_cost} counter · running ${oresrush.running_updates} updates / ${oresrush.running_cost} counter · estimated counter at exit ${oresrush.estimated_counter_at_exit}. Mixed running share estimates route distance from both measurements, reserves one counter point and rounds down to 0.1%.`
      : `Initial counter ${onward.southern.world_after.encounter_counter} · walking target not met.`;
    const oresrushRoute=onward.oresrush;
    $('oresrush-time-result').hidden=!oresrushRoute;
    if(oresrushRoute) $('oresrush-time-routine').innerHTML=instructionBoxHtml([
      'Before Partitio: '+((oresrushRoute.switches_before_partitio||[]).length?'time switch ×'+oresrushRoute.switches_before_partitio.length:'No time switch needed'),
      'After Partitio: '+((oresrushRoute.switches_after_partitio||[]).length?'time switch ×'+oresrushRoute.switches_after_partitio.length:'No time switch needed'),
      ...(oresrushRoute.southern_day_switch?['Immediately upon entering Southern Oresrush Wilds: switch to day']:[])]);
    const oresrushStages=oresrushRoute?[
      ['Southern Oresrush Wilds · loaded',oresrushRoute.southern_entry],
      ['Oresrush · loaded',oresrushRoute.town_entry],
      ['Oresrush · Mother route accepted',oresrushRoute.mother_route],
      ...(oresrushRoute.switches_before_partitio||[]).map((e,i)=>['Oresrush · before Partitio switch '+(i+1),e]),
      ['Partitio · recruited, Chapter 1 skipped',oresrushRoute.recruitment],
      ...(oresrushRoute.switches_after_partitio||[]).map((e,i)=>['Oresrush · after Partitio switch '+(i+1),e]),
      ['Southern Oresrush Wilds · returned from Oresrush',oresrushRoute.southern_return],
      ...(oresrushRoute.southern_day_switch?[['Southern Oresrush Wilds · switched to day before movement',oresrushRoute.southern_day_switch]]:[]),
    ]:[];
    const ryuRoute=oresrushRoute?.ryu;
    const ryuStages=ryuRoute?[
      ['Northern Ryu Sands · loaded',ryuRoute.northern_entry],
      ['Ryu · loaded',ryuRoute.town_entry],
    ].filter(([,entry])=>entry):[];
    const hikariRoute=ryuRoute?.hikari;
    const hikariSearch=ryuRoute?.hikari_search;
    const scholar=ryuRoute?.throne_scholar;
    const canalbrine=ryuRoute?.canalbrine;
    const water=canalbrine?.water;
    const waterPoints=water?[
      ['Canalbrine Bridge · canoe boarded',water.boarding.after],
      ['Canalbrine Bridge · water battle initialization',water.battle_entry],
      ['Canalbrine Bridge · first command menu',water.escape.menu.rng],
      ['Canalbrine Bridge · Escape check after boosts',water.escape.attempt.before_check],
      ['Canalbrine Bridge · Escape succeeded',water.escape.attempt.after],
      ['Canalbrine Bridge · Escape field returned',water.field_return.after],
    ]:[];
    const canalbrineStages=canalbrine?[
      ['Northern Ryu Sands · returned from Ryu',canalbrine.northern_return],
      ['Canalbrine Bridge · loaded',canalbrine.bridge_entry],
    ]:[];
    const scholarStages=scholar?[
      ['Throné · Scholar assigned',scholar.assignment],
      ['Throné · Lightning Bolt + Elemental Barrage learned',scholar.learning],
      ['Throné · Evasive Maneuvers equipped, menu closed',scholar.after],
    ]:[];
    const hikariStages=hikariRoute?[
      ['Hikari · before recruit battle',hikariRoute.introduction],
      ['Hikari · victory field return',hikariRoute.field_return],
      ['Hikari · joined, Osvald replaced, tutorials dismissed',hikariRoute.recruitment],
    ]:[];
    if(hikariRoute) {
      $('hikari-recruit-result').hidden=false;
      $('hikari-recruit-log').hidden=false;
      const final=hikariRoute.recruitment;
      $('hikari-recruit-advances').textContent=totalAdvances(final.after);
      $('hikari-recruit-hex').textContent=hex(final.after.state);
      $('hikari-recruit-counter').textContent=`Counter ${final.world_after.encounter_counter} · Evasive Maneuvers inactive while Osvald is benched`;
      $('hikari-recruit-verification').textContent=hikariRoute.verification;
      const menuNames=['Throné','Osvald','Agnea','Partitio'];
      const lastMenu=hikariRoute.battle.menus[1];
      const weapon=['Dagger','Staff','Dagger','Lance'][lastMenu.actor];
      $('hikari-recruit-routine').innerHTML=`<p><strong>R1 · ${menuNames[lastMenu.actor]} · 2nd player command menu</strong></p>${boostStepsHtml(hikariSearch?.inputs?.levels)}`;
      $('hikari-search-detail').textContent=hikariSearch
        ? `${hikariSearch.status} · ${hikariSearch.inputs.extra_input_count} Hikari boost inputs · ${hikariSearch.total_input_cost} total Hikari + water + coast boost inputs · ${hikariSearch.tested_hikari_candidates} Hikari candidates / ${hikariSearch.tested_escape_attempts} Escape checks. The selected route to Ryu is preserved.`
        : '';
      const names={1:'Hikari',4:'Partitio',6:'Osvald',7:'Throné',8:'Agnea'};
      const w=final.world_after;
      $('hikari-recruit-ledger').innerHTML=[...w.party,...w.bench].map(id=>{
        const p=w.character_progression[id];
        return `<tr><td>${names[id]}</td><td>${w.party.includes(id)?'Active':'Bench'}</td><td>${p.level}</td><td>${p.experience}</td><td>${p.jp_earned-p.jp_spent} (${p.jp_earned} earned / ${p.jp_spent} spent)</td></tr>`;
      }).join('');
      $('hikari-recruit-decisions').innerHTML=hikariStages.flatMap(([stage,entry])=>entry.decisions.map(d=>`<tr><td>${escapeHtml(stage)}</td><td>${escapeHtml(d.callback)}</td><td>${escapeHtml(d.subject)}</td><td>${escapeHtml(d.result)}</td><td>${totalAdvances(d.rng)}</td><td>${hex(d.rng.state)}</td><td>${d.rng.draws}</td></tr>`)).join('');
      const actors=['Throné','Osvald','Agnea','Partitio','Soldier leader','Soldier 1','Soldier 2'];
      const actions={ice_soulstone:'Ice Soulstone',attack_dagger:'Dagger',attack_staff:'Staff',attack_lance:'Lance'};
      $('hikari-recruit-battle').innerHTML=hikariRoute.battle.steps.map(s=>{
        const action=s.events.find(e=>e.kind==='action');
        const damage=s.events.filter(e=>e.kind==='damage').map(e=>`${actors[e.target]} ${e.amount}`).join(' · ');
        return `<tr><td>R1</td><td>${actors[s.actor]}</td><td>${escapeHtml(actions[action?.name]||action?.name||'Action')}</td><td>${damage||'—'}</td><td>${totalAdvances(s.rng_before)}</td><td>${hex(s.rng_before.state)}</td></tr>`;
      }).join('');
    }
    if(scholar) {
      $('scholar-setup-log').hidden=false;
      const final=scholar.after;
      const p=final.world_after.character_progression[7];
      const c=p.character;
      $('scholar-setup-advances').textContent=totalAdvances(final.after);
      $('scholar-setup-hex').textContent=hex(final.after.state);
      $('scholar-setup-counter').textContent=`Counter ${final.world_after.encounter_counter} · Evasive Maneuvers active on Throné · CRT and counter unchanged`;
      $('scholar-setup-jp').textContent=`Lightning Bolt 30 JP + Elemental Barrage 100 JP · ${scholar.jp_after} JP remaining`;
      $('scholar-setup-stats').textContent=`Level ${p.level} · EXP ${p.experience} · HP ${c.resources.hp}/${c.stats.max_hp} · SP ${c.resources.sp}/${c.stats.max_sp} · elemental attack ${c.combat.magic} · elemental defense ${c.combat.magic_defense} · original default weapon retained`;
      $('scholar-setup-verification').textContent=scholar.verification;
    }
    if(canalbrine) {
      $('canalbrine-route-log').hidden=false;
      $('canalbrine-route-checkpoints').innerHTML=canalbrineStages.map(([label,entry])=>`<tr><td>${label}</td><td><strong>${totalAdvances(entry.after)}</strong></td><td>${hex(entry.after.state)}</td><td>${entry.world_after.encounter_counter}</td></tr>`).join('');
      $('canalbrine-route-verification').textContent=canalbrine.verification;
      $('canalbrine-route-decisions').innerHTML=canalbrineStages.flatMap(([stage,entry])=>entry.decisions.map(d=>`<tr><td>${escapeHtml(stage)}</td><td>${escapeHtml(d.callback)}</td><td>${escapeHtml(d.subject)}</td><td>${escapeHtml(d.result)}</td><td>${totalAdvances(d.rng)}</td><td>${hex(d.rng.state)}</td></tr>`)).join('');
    }
    if(water) {
      $('canalbrine-water-result').hidden=false;
      $('canalbrine-water-log').hidden=false;
      const actor=routeNames()[water.escape.menu.first_actor];
      $('canalbrine-water-encounter').textContent=encounterLabel(water.encounter);
      $('canalbrine-water-routine').innerHTML=`<p><strong>R1 · ${actor} · 1st command menu</strong></p>${boostStepsHtml(water.escape.inputs?.levels,'Escape')}`;
      $('canalbrine-water-detail').textContent=`${encounterLabel(water.encounter)} · ${water.encounter.situation} · ${water.escape.attempt.chance_percent}% Escape chance before the check · ${water.escape.weighted_input_cost} boost inputs. First Escape succeeds.`;
      $('canalbrine-water-checkpoints').innerHTML=waterPoints.map(([label,rng])=>`<tr><td>${escapeHtml(label)}</td><td>${totalAdvances(rng)}</td><td>${hex(rng.state)}</td></tr>`).join('');
      $('canalbrine-water-return').textContent=`Field control restored · counter ${water.field_return.world_after.encounter_counter} · Evasive Maneuvers active on Throné.`;
      $('canalbrine-water-decisions').innerHTML=water.boarding.decisions.map(d=>`<tr><td>${escapeHtml(d.callback)}</td><td>${escapeHtml(d.subject)}</td><td>${escapeHtml(d.result)}</td><td>${totalAdvances(d.rng)}</td><td>${hex(d.rng.state)}</td></tr>`).join('');
    }
    const conning=canalbrine?.conning;
    $('conning-steal-result').hidden=true;
    const conningStages=[],conningSpans=[],conningPoints=[];
    if(conning) {
      $('conning-log').hidden=false;
      $('conning-verification').textContent=conning.verification;
      for(const [key,label,fight] of [['north','Northern Conning Creek Coast',conning.northern],['west','Western Conning Creek Coast',conning.western]]) {
        $('conning-'+key+'-result').hidden=false;
        $('conning-'+key+'-encounter').textContent=encounterLabel(fight.arrival.encounter);
        const actor=routeNames()[fight.menu.first_actor];
        $('conning-'+key+'-routine').innerHTML='<p><strong>R1 · '+actor+' · 1st command menu</strong></p>'+boostStepsHtml(fight.inputs.levels,fight.victory?'Shadow Soulstone (M)':'Escape');
        for(const [name,entry] of [['loaded',fight.arrival.entry],['night switch',fight.arrival.night],['field returned',fight.field_return],['day switch',fight.day]]) conningStages.push([label+' · '+name,entry]);
        conningPoints.push([label+' · first command menu',fight.menu.rng],[label+' · battle completed',(fight.victory||fight.escape).after]);
        conningSpans.push(...fight.arrival.entry.trace,...fight.arrival.night.trace);
        let previous=fight.arrival.night.after;
        const origin=BigInt(previous.draws)-BigInt(fight.arrival.encounter.checkpoints[0].draws);
        for(const c of fight.arrival.encounter.checkpoints) {const after={state:c.state,draws:origin+BigInt(c.draws)};conningSpans.push({reason:label+' · '+c.label,before:previous,after});previous=after;}
        conningSpans.push(...(fight.victory||fight.escape).trace,...fight.field_return.trace,...fight.day.trace);
      }
      conningStages.push(['Conning Creek · loaded; no chapter started',conning.town]);
      conningSpans.push(...conning.town.trace);
      if(conning.harbor) {
        conningStages.push(['Conning Creek: Harbor · loaded',conning.harbor]);
        conningSpans.push(...conning.harbor.trace);
      }
      if(conning.pomegranate) {
        const p=conning.pomegranate;
        $('conning-steal-result').hidden=false;
        $('conning-steal-routine').innerHTML=instructionBoxHtml([
          'Conning Creek time switch: '+p.town_switches.length,
          'Conning Creek: Harbor time switch: '+p.harbor_switches.length,
          'After stealing both items: change selected path action, then close'
        ]);
        for(const [zone,entries] of [['Conning Creek',p.town_switches],['Conning Creek: Harbor',p.harbor_switches]]) {
          entries.forEach((e,i)=>{conningStages.push([zone+' · time switch '+(i+1),e]);conningSpans.push(...e.trace);});
        }
        conningStages.push(['Townsperson · Pomegranate (M) then Wind Soulstone (L) · Throné Lv'+p.throne_level+' · '+p.chance+'% each · rolls '+p.roll+'/'+p.wind_roll,p.steal]);
        conningSpans.push(...p.steal.trace);
        if(p.voyage) {
          for(const [label,e] of [['New Delsta Harbor · fast travel',p.voyage.new_delsta],['Toto’haha Beasting Bay: Anchorage · boat arrival',p.voyage.anchorage],['Canalbrine · boat arrival',p.voyage.canalbrine]]) {
            conningStages.push([label,e]);conningSpans.push(...e.trace);
          }
          if(p.voyage.castti) {conningStages.push(['Castti recruited · Chapter 1 skipped · benched',p.voyage.castti]);conningSpans.push(...p.voyage.castti.trace);}
        }
        if(p.inventor) {
          const i=p.inventor,actor=i.menu ? routeNames()[i.menu.first_actor] : null;
          if(i.new_delsta_adjustment) {
            const c=i.new_delsta_adjustment;
            $('new-delsta-bg-result').hidden=false;
            $('new-delsta-bg-routine').innerHTML=instructionBoxHtml(['New Delsta main town time switches: '+c.switches+' (finish in '+(c.departure.world_after.time===4?'nighttime':'daytime')+')',...(i.approach.highroad_day_switch?['Immediately upon entering Eastern New Delsta Highroad: switch to day']:[])]);
            conningStages.push(['New Delsta · BG adjustment complete ('+(c.departure.world_after.time===4?'night':'day')+')',c.departure]);
            conningSpans.push(...c.departure.trace);
            conningPoints.push(['Gravell · targeted executed EXP ×100 lottery',c.target]);
          }
          $('inventor-escape-result').hidden=false;
          $('clockbank-movement-result').hidden=!i.clockbank;
          const reach=i.clockbank_reach;
          $('clockbank-movement-routine').innerHTML=instructionBoxHtml(reach ? [
            'Start fast: first ≈'+reach.run_percent.toFixed(1)+'% of total route',
            'Then slow: remaining ≈'+(100-reach.run_percent).toFixed(1)+'%'
          ] : ['Encounter unavoidable even at slow speed']);
          const movement=i.approach.thief_reach;
          $('inventor-escape-routine').innerHTML=movement ? instructionBoxHtml([
            'Run first '+movement.run_percent.toFixed(1)+'% · walk remaining '+(100-movement.run_percent).toFixed(1)+'%'
          ]) : '<p>'+escapeHtml(encounterLabel(i.approach.encounter))+'</p><p><strong>R1 · '+actor+' · 1st command menu</strong></p>'+boostStepsHtml(i.inputs.levels,'Escape');
          const thiefStages=i.thief ? [['Thief · switched to night',i.thief.night],['Thief · scripted battle initialization',i.thief.introduction],['Thief · victory and field return',i.thief.field_return],["Al’s Bag recovered · return to Al later",i.thief.bag],['Highroad · returned to day before Arkar',i.thief.day]] : [];
          if(i.thief) { $('inventor-escape-routine').innerHTML+='<p><strong>Thief · T1 · '+routeNames()[i.thief.battle.actor]+' · Shadow Soulstone (M)</strong></p>'; conningSpans.push(...(i.thief.battle.trace||[])); }
          if(i.inventor_reach) { const r=i.inventor_reach; $('inventor-escape-routine').innerHTML+='<p><strong>Thief → Inventor house</strong></p>'+instructionBoxHtml([
            'Run first '+r.run_percent.toFixed(1)+'% · walk remaining '+(100-r.run_percent).toFixed(1)+'%'
          ]); }
          for(const [label,e] of [['New Delsta · fast travel',i.approach.town],[i.approach.agnea_story.world_after.galdera_route?'New Delsta · Agnea Chapter 2 left inactive':'Agnea Chapter 2 · story accepted, opening skipped',i.approach.agnea_story],['Eastern New Delsta Highroad · loaded',i.approach.highroad],['Eastern New Delsta Highroad · switched to day before encounter',i.approach.highroad_day_switch],[i.approach.thief_reach?'Highroad · reached thief without an encounter':'Inventor Highroad · escaped',i.field_return],...thiefStages,...(i.inventor_arrival?[['Inventor house · reached without an encounter',i.inventor_arrival]]:[]),['Arkar · Proof of the Inventor',i.license],['Elemental Bomb Bottle crafted',i.bomb],['Critical Scope crafted',i.scope],['Western Clockbank Highroad · loaded',i.western_clockbank],['Southern Clockbank Highroad · loaded (no intervening encounter)',i.southern_clockbank],['Clockbank · loaded',i.clockbank],['Partitio Chapter 2 · story accepted, opening skipped',i.partitio_story]]) {if(e){conningStages.push([label,e]);conningSpans.push(...e.trace);}}
          if(i.crackridge_return) {
            const r=i.crackridge_return,actor=routeNames()[r.menu.first_actor];
            $('crackridge-return-result').hidden=false;
            $('crackridge-return-title').textContent=i.scope.world_after.galdera_route?'Crackridge Harbor · after Inventor crafting':'Crackridge Harbor · after Clockite delivery';
            $('crackridge-return-routine').innerHTML=lateRouteInstructions(r);
          }
          for(const [label,e] of i.clockite_stages || []) {conningStages.push([label,e]);conningSpans.push(...e.trace);}
          if(i.clockbank_adjustment) {
            const c=i.clockbank_adjustment;
            $('clockbank-bg-result').hidden=false;
            $('clockbank-bg-routine').innerHTML=instructionBoxHtml(['Clockbank Industrial District time switches: '+c.switches]);
            conningStages.push(['Clockbank · BG adjustment complete (day)',c.departure]);
            conningSpans.push(...c.departure.trace);
            conningPoints.push(['Gravell · targeted executed EXP ×100 lottery',c.target]);
          }
          if(i.crackridge_return) {
            const r=i.crackridge_return;
            for(const [label,e] of [[i.scope.world_after.galdera_route?'Crackridge Harbor · fast travel after Inventor':'Crackridge Harbor · fast travel after Clockite',r.arrival],['Crackridge Harbor return · escaped',r.field_return]]) {conningStages.push([label,e]);conningSpans.push(...e.trace);}
            conningPoints.push(['Crackridge Harbor return · first command menu',r.menu.rng]);
            if(r.southern) {
              const a=r.southern,actor=routeNames()[a.menu.first_actor];
              $('southern-crackridge-result').hidden=false;
              $('southern-crackridge-routine').innerHTML=lateRouteInstructions(a);
              for(const [label,e] of [['Southern Crackridge Wilds · loaded',a.arrival],['Southern Crackridge Wilds · escaped',a.field_return]]) {conningStages.push([label,e]);conningSpans.push(...e.trace);}
              conningPoints.push(['Southern Crackridge Wilds · first command menu',a.menu.rng]);
              conningSpans.push(...a.escape.trace);
              for(const [key,label,b] of [['western-crackridge','Western Crackridge Wilds · before town',a.southern],['western-crackridge-return','Western Crackridge Wilds · after town',a.southern?.southern]]) {
                if(!b) continue;
                if(b.bg_adjustment) {
                  const c=b.bg_adjustment;
                  $('crackridge-bg-result').hidden=false;
                  $('crackridge-bg-routine').innerHTML=instructionBoxHtml(['Crackridge time switches: '+c.switches+' (finish in daytime)']);
                  conningStages.push(['Crackridge · BG adjustment complete (day)',c.departure]);
                  conningSpans.push(...c.departure.trace);
                  conningPoints.push(['Gravell · targeted executed EXP ×100 lottery',c.target]);
                }
                const actor=routeNames()[b.menu.first_actor];
                $(key+'-result').hidden=false;
                $(key+'-routine').innerHTML=lateRouteInstructions(b);
                for(const [name,e] of [...(b.approach_stages||[]),[label+' · loaded',b.arrival],[label+(b.victory?' · Wind Soulstone (L) victory':' · escaped'),b.field_return]]) {conningStages.push([name,e]);conningSpans.push(...e.trace);}
                conningPoints.push([label+' · first command menu',b.menu.rng]);conningSpans.push(...(b.victory||b.escape).trace);
                if(b.gravell) {
                  const g=b.gravell;
                  $('gravell-result').hidden=false;
                  const names=routeNames();
                  const ordinals={};
                  $('gravell-routine').innerHTML='<p><strong>'+escapeHtml(encounterLabel(g.encounter))+'</strong></p>'+g.fight.actions.map(a=>{
                    const ordinal=ordinals[a.round]=(ordinals[a.round]||0)+1;
                    conningPoints.push([`Gravell · T${a.round} · ${names[a.actor]} · command menu`,a.before]);
                    if(!['bewildering_grace','guards'].includes(a.action)) return '';
                    return `<p><strong>T${a.round} · ${names[a.actor]} · ${menuOrdinal(ordinal)} command menu</strong></p>`+boostStepsHtml(a.inputs,a.action==='guards'?'+3 Hired Help: Guards':'+3 Bewildering Grace');
                  }).join('');
                  conningSpans.push(...g.fight.trace,...g.rewards.trace);
                  for(const [name,e] of [['Merchant License acquired',g.license],['Gravell jobs, skills and supports equipped',g.setup],['Western Gravell Wilds · loaded',g.arrival],['Western Gravell Wilds · switched to night',g.night]]) {conningStages.push([name,e]);conningSpans.push(...e.trace);}
                  conningPoints.push(['Western Gravell Wilds · before battle initialization',g.battle_entry]);
                  conningStages.push(['Western Gravell Wilds · EXP ×100 victory · field return',g.field_return]);conningSpans.push(...g.field_return.trace);
                  if(g.town){conningStages.push(['Gravell · loaded (night)',g.town]);conningSpans.push(...g.town.trace);}
                }
              }
            }
          }
          if(i.menu && i.approach.encounter) {
          conningPoints.push(['Inventor Highroad · first command menu · ambush chance '+i.approach.surprise_percent+'%',i.menu.rng]);
          let previous=(i.approach.highroad_day_switch||i.approach.highroad).after;
          const origin=BigInt(previous.draws)-BigInt(i.approach.encounter.checkpoints[0].draws);
          for(const c of i.approach.encounter.checkpoints) {const after={state:c.state,draws:origin+BigInt(c.draws)};conningSpans.push({reason:'Inventor Highroad · '+c.label,before:previous,after});previous=after;}
          conningSpans.push(...i.escape.trace);
          }
        }
      }
      $('conning-checkpoints').innerHTML=[...conningStages.map(([label,e])=>[label,e.after,e.world_after.encounter_counter]),...conningPoints.map(([label,r])=>[label,r,'—'])].sort((a,b)=>Number(BigInt(a[1].draws)-BigInt(b[1].draws))).map(([label,r,c])=>'<tr><td>'+escapeHtml(label)+'</td><td><strong>'+totalAdvances(r)+'</strong></td><td>'+hex(r.state)+'</td><td>'+c+'</td></tr>').join('');
    }
    if(oresrushRoute) {
      const town=oresrushRoute.town_reach;
      const ryu=oresrushRoute.ryu_reach;
      $('ryu-walk-result').hidden=false;
      $('ryu-running-limit').textContent=!ryu?'Walking target not met':ryu.run_percent===100?'Run 100%':`Run ≈${ryu.run_percent.toFixed(1)}% · walk ${(100-ryu.run_percent).toFixed(1)}%`;
      $('ryu-running-limit').classList.toggle('text-bad',!ryu);
      const ryuTown=ryuRoute?.town_reach;
      $('ryu-town-walk-result').hidden=false;
      $('ryu-town-running-limit').textContent=!ryuTown?'Walking target not met':ryuTown.run_percent===100?'Run 100%':`Run ≈${ryuTown.run_percent.toFixed(1)}% · walk ${(100-ryuTown.run_percent).toFixed(1)}%`;
      $('ryu-town-running-limit').classList.toggle('text-bad',!ryuTown);
      $('oresrush-route-log').hidden=false;
      $('oresrush-town-movement-budget').textContent=`Southern Oresrush Wilds entry → Oresrush · initial counter ${town.initial_counter} · measured ${town.running_updates} running updates / ${town.running_cost} counter · counter at town entry ${town.estimated_counter_at_exit}. The running budget keeps a positive counter at town entry.`;
      $('ryu-movement-budget').textContent=ryu
        ? `Southern Oresrush Wilds return → Northern Ryu Sands · initial counter ${ryu.initial_counter} · walking ${ryu.walking_updates} updates / ${ryu.walking_cost} counter · running ${ryu.running_updates} updates / ${ryu.running_cost} counter · estimated counter at exit ${ryu.estimated_counter_at_exit}. Each speed's update count includes three updates of leeway. Mixed running share estimates route distance from both measurements, reserves one counter point and rounds down to 0.1%.`
        : `Southern Oresrush Wilds return → Northern Ryu Sands · initial counter ${oresrushRoute.southern_return.world_after.encounter_counter} · walking target not met.`;
      $('oresrush-route-verification').textContent=oresrushRoute.verification;
      $('oresrush-route-decisions').innerHTML=oresrushStages.flatMap(([stage,entry])=>entry.decisions.map(d=>`<tr><td>${escapeHtml(stage)}</td><td>${escapeHtml(d.callback)}</td><td>${escapeHtml(d.subject)}</td><td>${escapeHtml(d.result)}</td><td>${totalAdvances(d.rng)}</td><td>${hex(d.rng.state)}</td><td>${d.rng.draws}</td></tr>`)).join('');
      if(ryuRoute) {
        $('ryu-route-log').hidden=false;
        $('ryu-town-movement-budget').textContent=ryuTown
          ? `Initial counter ${ryuTown.initial_counter} · walking ${ryuTown.walking_updates} updates / ${ryuTown.walking_cost} counter · running ${ryuTown.running_updates} updates / ${ryuTown.running_cost} counter · estimated counter at town entry ${ryuTown.estimated_counter_at_exit}. Each speed's update count includes three updates of leeway. Mixed running share estimates route distance from both measurements, reserves one counter point and rounds down to 0.1%.`
          : `Initial counter ${ryuRoute.northern_entry.world_after.encounter_counter} · walking target not met.`;
        $('ryu-route-verification').textContent=ryuRoute.verification;
        $('ryu-route-decisions').innerHTML=ryuStages.flatMap(([stage,entry])=>entry.decisions.map(d=>`<tr><td>${escapeHtml(stage)}</td><td>${escapeHtml(d.callback)}</td><td>${escapeHtml(d.subject)}</td><td>${escapeHtml(d.result)}</td><td>${totalAdvances(d.rng)}</td><td>${hex(d.rng.state)}</td><td>${d.rng.draws}</td></tr>`)).join('');
      }
    }
    const points=[['New Delsta Harbor · fast travel',v.arrival.harbor.after],['Crackridge Harbor · boat arrival',v.arrival.crackridge.after],['Crackridge Harbor · first command menu',v.escape.menu.rng],['Crackridge Harbor · Escape field returned',v.field_return.after],['Eastern Cropdale Trail · loaded',v.destination.after],['Cropdale · loaded',v.cropdale.entry.after],['Agnea · recruited, Chapter 1 skipped',v.cropdale.recruitment.after],['Eastern Cropdale Trail · returned from Cropdale',onward.trail.after],['Eastern Cropdale return · first command menu',onward.escape.menu.rng],['Eastern Cropdale return · Escape field returned',onward.field_return.after],['Southern Cropdale Trail · loaded',onward.southern.after]];
    points.push(...[...oresrushStages,...ryuStages].map(([label,entry])=>[label,entry.after]));
    if(hikariRoute) points.push(
      ['Hikari · before recruit battle',hikariRoute.introduction.after],
      ...hikariRoute.battle.menus.map(m=>[`Hikari recruit · R${m.round} ${['Throné','Osvald','Agnea','Partitio'][m.actor]} command menu`,m.rng]),
      ['Hikari recruit · victory and drops',hikariRoute.battle.after],
      ...hikariStages.slice(1).map(([label,entry])=>[label,entry.after]),
    );
    points.push(...scholarStages.map(([label,entry])=>[label,entry.after]));
    points.push(...canalbrineStages.map(([label,entry])=>[label,entry.after]));
    points.push(...waterPoints,...switches.map((e,i)=>['Cropdale · switch '+(i+1)+(i===0?' before Agnea':' after Agnea'),e.after]),...conningStages.map(([label,e])=>[label,e.after]),...conningPoints);
    points.sort((a,b)=>Number(BigInt(a[1].draws)-BigInt(b[1].draws)));
    $('checkpoints').innerHTML+=points.map(([label,r])=>`<tr><td>${label}</td><td>${totalAdvances(r)}</td><td>${hex(r.state)}</td><td>${r.draws}</td></tr>`).join('');
    let before=v.arrival.crackridge.after;
    const encounterSpans=v.arrival.encounter.checkpoints.map(c=>{
      const after={state:c.state,draws:BigInt(v.arrival.crackridge.after.draws)+BigInt(c.draws)};
      const span={reason:'Crackridge encounter · '+c.label,before,after};before=after;return span;
    });
    before=onward.trail.after;
    const onwardSpans=onward.encounter.checkpoints.map(c=>{
      const after={state:c.state,draws:BigInt(onward.trail.after.draws)+BigInt(c.draws)};
      const span={reason:'Eastern Cropdale return encounter · '+c.label,before,after};before=after;return span;
    });
    const hikariSpans=hikariRoute?[...hikariRoute.introduction.trace,...hikariRoute.battle.trace,...hikariRoute.field_return.trace,...hikariRoute.recruitment.trace]:[];
    const waterSpans=[];
    if(water) {
      waterSpans.push(...water.boarding.trace);
      before=water.boarding.after;
      const origin=BigInt(water.boarding.after.draws)-BigInt(water.encounter.checkpoints[0].draws);
      for(const c of water.encounter.checkpoints) {
        const after={state:c.state,draws:origin+BigInt(c.draws)};
        waterSpans.push({reason:'Canalbrine water encounter · '+c.label,before,after});before=after;
      }
      waterSpans.push(...water.escape.attempt.trace,...water.field_return.trace);
    }
    const spans=[...v.arrival.harbor.trace,...v.arrival.crackridge.trace,...encounterSpans,...v.escape.attempt.trace,...v.field_return.trace,...v.destination.trace,...v.cropdale.entry.trace,...(switches[0]?.trace||[]),...v.cropdale.recruitment.trace,...switches.slice(1).flatMap(e=>e.trace),...onward.trail.trace,...onwardSpans,...onward.escape.attempt.trace,...onward.field_return.trace,...onward.southern.trace,...oresrushStages.flatMap(([,entry])=>entry.trace),...ryuStages.flatMap(([,entry])=>entry.trace),...hikariSpans,...scholarStages.flatMap(([,entry])=>entry.trace),...canalbrineStages.flatMap(([,entry])=>entry.trace),...waterSpans,...conningSpans];
    $('trace').innerHTML+=spans.map(t=>`<tr><td>${escapeHtml(t.reason)}</td><td>—</td><td>${totalAdvances(t.before)}</td><td>${totalAdvances(t.after)}</td><td>+${BigInt(t.after.draws)-BigInt(t.before.draws)}</td><td>${hex(t.before.state)}</td><td>${hex(t.after.state)}</td></tr>`).join('');
    return;
  }
  $('crackridge-travel-title').textContent='Cleric purchase → Crackridge Harbor';
  $('winter-crackridge-encounter').textContent='';
  $('winter-crackridge-run').textContent='';
  const p=result.post_battle_field?.crackridge_voyage;
  const e=result.post_battle_field?.crackridge_escape;
  $('crackridge-switch-result').hidden=!p || !!p.error;
  $('crackridge-switch-instruction').textContent=p?.day_switch_location==='flats'?'Switch to day in New Delsta Flats · before teleporting to Harbor':'Switch to day in New Delsta Harbor · after teleporting';
  $('crackridge-escape-result').hidden=!e;
  const actor=e?.menu?.first_actor===1?'Osvald':'Throné';
  $('crackridge-escape-routine').innerHTML=e?.error?`<p class="text-bad">${escapeHtml(e.error)}</p>`:e?.attempt?.success?`<p><strong>R1 · ${actor} · 1st command menu</strong></p>${boostStepsHtml(e.inputs?.levels,'Escape')}`:`<p class="text-bad">${escapeHtml(e?.status||'')}</p>`;
  $('crackridge-voyage-result').hidden=!p;
  $('crackridge-voyage-error').hidden=!p?.error;
  $('crackridge-voyage-error').textContent=p?.error?`Prediction unavailable · ${p.error}`:'';
  $('crackridge-voyage-diagnostics').hidden=!p || !!p.error;
  $('crackridge-voyage-verification').textContent='';
  if(!p || p.error) return;
  $('crackridge-voyage-advances').textContent=totalAdvances(p.after);
  $('crackridge-voyage-hex').textContent=hex(p.after.state);
  $('crackridge-voyage-verification').textContent=p.verification;
  const dayLabel=p.day_switch_location==='flats'?'New Delsta Flats · switched to day':'New Delsta Harbor · switched to day';
  const travel=['Fast travel · New Delsta Harbor',p.at_new_delsta_harbor];
  const day=[dayLabel,p.after_day_switch];
  const checkpoints=[['Clerics · soulstones bought, shop dismissed',p.after_shopping],...(p.day_switch_location==='flats'?[day,travel]:[travel,day]),['Boat to Crackridge · before voyage',p.before_boat],['Crackridge Harbor · boat finished, field control',p.after]];
  if(e?.menu) checkpoints.push([`Crackridge Harbor · R1 ${actor} command menu`,e.menu.rng]);
  if(e?.attempt) checkpoints.push(['Crackridge Harbor · before Escape check',e.attempt.before_check],['Crackridge Harbor · successful Escape',e.attempt.after]);
  if(e?.field_return) checkpoints.push(['Crackridge Harbor · field returned',e.field_return.after]);
  $('checkpoints').innerHTML+=checkpoints.map(([label,rng])=>`<tr><td>${escapeHtml(label)}</td><td>${totalAdvances(rng)}</td><td>${hex(rng.state)}</td><td>${rng.draws}</td></tr>`).join('');
  $('trace').innerHTML+=p.trace.map(t=>`<tr><td>Crackridge voyage · ${escapeHtml(t.reason)}</td><td>—</td><td>${totalAdvances(t.before)}</td><td>${totalAdvances(t.after)}</td><td>+${BigInt(t.after.draws)-BigInt(t.before.draws)}</td><td>${hex(t.before.state)}</td><td>${hex(t.after.state)}</td></tr>`).join('');
  if(e?.encounter) $('checkpoints').innerHTML+=e.encounter.checkpoints.map(t=>`<tr><td>Crackridge encounter · ${escapeHtml(t.label)}</td><td>${totalAdvances({draws:t.draws})}</td><td>${hex(t.state)}</td><td>${t.draws}</td></tr>`).join('');
  const trace=[...(e?.attempt?.trace||[]),...(e?.field_return?.trace||[])];
  $('trace').innerHTML+=trace.map(t=>`<tr><td>Crackridge Escape · ${escapeHtml(t.reason)}</td><td>${t.actor===0?'Throné':t.actor===1?'Osvald':'—'}</td><td>${totalAdvances(t.before)}</td><td>${totalAdvances(t.after)}</td><td>+${BigInt(t.after.draws)-BigInt(t.before.draws)}</td><td>${hex(t.before.state)}</td><td>${hex(t.after.state)}</td></tr>`).join('');
  if(e?.verification) $('crackridge-voyage-verification').textContent+=' '+e.verification;
}

function renderFlats() {
  const p=result.post_battle_field?.flats;
  $('flats-result').hidden=!p;
  $('flats-diagnostics').hidden=!p || !!p.error;
  $('flats-error').hidden=!p?.error;
  $('flats-error').textContent=p?.error ? `Prediction unavailable · ${p.error}` : '';
  $('flats-harbor').innerHTML='';
  $('flats-harbor-detail').textContent='';
  if(!p || p.error) return;
  $('flats-advances').textContent=totalAdvances(p.after);
  $('flats-hex').textContent=hex(p.after.state);
  const reach=p.harbor_reach;
  if(reach) {
    const mixed=reach.mixed;
    const label=!mixed?'Encounter unavoidable':reach.running.reaches_reset?'Run 100%':`Run ≈${mixed.running_distance_percent}% · walk ${mixed.walking_distance_percent}%`;
    const detail=reach.running.reaches_reset?`${reach.running.counter_before_reset} left at harbor · 56 running updates`:mixed?`Route-distance estimate · ${mixed.estimated_counter_at_harbor} left`:'Walking needs counter 550';
    $('flats-harbor').innerHTML=`<h3>${reach.running.reaches_reset?'Running limit':'Running limit · estimate'}</h3><p class="run-state ${mixed?'text-good':'text-bad'}">${label}</p>`;
    $('flats-harbor-detail').textContent=`Counter ${reach.initial_counter} · ${detail}`;
  }
  $('checkpoints').innerHTML+=`<tr><td>New Delsta Flats · after loading</td><td>${totalAdvances(p.after)}</td><td>${hex(p.after.state)}</td><td>${p.after.draws}</td></tr>`;
  $('trace').innerHTML+=p.trace.map(s=>`<tr><td>${escapeHtml(s.reason)}</td><td>—</td><td>${totalAdvances(s.before)}</td><td>${totalAdvances(s.after)}</td><td>+${BigInt(s.after.draws)-BigInt(s.before.draws)}</td><td>${hex(s.before.state)}</td><td>${hex(s.after.state)}</td></tr>`).join('');
  $('pirro-verification').textContent+=` ${p.verification} ${reach?.verification || ''}`;
}

function renderHarborReturn() {
  const p=result.post_battle_field?.harbor_return;
  $('harbor-return-result').hidden=!p;
  $('harbor-return-diagnostics').hidden=!p || !!p.error;
  $('harbor-return-error').hidden=!p?.error;
  $('harbor-return-error').textContent=p?.error ? `Prediction unavailable · ${p.error}` : '';
  if(!p || p.error) return;
  $('harbor-return-advances').textContent=totalAdvances(p.after);
  $('harbor-return-hex').textContent=hex(p.after.state);
  $('harbor-return-counter').textContent=`Counter ${p.world_after.encounter_counter}`;
  for(const [label,rng] of [['New Delsta Harbor · tagged',p.at_harbor],['New Delsta Flats · returned from harbor',p.after]]) {
    $('checkpoints').innerHTML+=`<tr><td>${label}</td><td>${totalAdvances(rng)}</td><td>${hex(rng.state)}</td><td>${rng.draws}</td></tr>`;
  }
  $('trace').innerHTML+=p.trace.map(s=>`<tr><td>${escapeHtml(s.reason)}</td><td>—</td><td>${totalAdvances(s.before)}</td><td>${totalAdvances(s.after)}</td><td>+${BigInt(s.after.draws)-BigInt(s.before.draws)}</td><td>${hex(s.before.state)}</td><td>${hex(s.after.state)}</td></tr>`).join('');
  $('pirro-verification').textContent+=` ${p.verification}`;
}

function renderPirroAftermath() {
  const p=result.post_battle_field?.pirro_aftermath;
  $('pirro-aftermath-result').hidden=!p || !!p.error;
  $('pirro-aftermath-error').hidden=!p?.error;
  $('pirro-aftermath-error').textContent=p?.error ? `Post-Pirro prediction unavailable: ${p.error}` : '';
  if(!p || p.error) return;
  $('pirro-aftermath-advances').textContent=totalAdvances(p.after);
  $('pirro-aftermath-hex').textContent=hex(p.after.state);
  $('pirro-verification').textContent+=` ${p.verification}`;
  $('checkpoints').innerHTML+=`<tr><td>After Pirro cutscenes · Eastern New Delsta Highroad control</td><td>${totalAdvances(p.after)}</td><td>${hex(p.after.state)}</td><td>${p.after.draws}</td></tr>`;
  $('trace').innerHTML+=p.trace.map(s=>`<tr><td>${escapeHtml(s.reason)}</td><td>—</td><td>${totalAdvances(s.before)}</td><td>${totalAdvances(s.after)}</td><td>+${BigInt(s.after.draws)-BigInt(s.before.draws)}</td><td>${hex(s.before.state)}</td><td>${hex(s.after.state)}</td></tr>`).join('');
}

function renderPirro() {
  const p=result.post_battle_field?.pirro_battle;
  $('pirro-result').hidden=!result.post_battle_field?.mansion_escape;
  $('pirro-log').hidden=!p || !!p.error;
  $('pirro-boost-routine').innerHTML='';
  if(!p || p.error) {
    $('pirro-boost-routine').textContent='Unavailable · no Pirro prediction';return;
  }
  const won=p.state.outcome==='victory' && !p.routine_error;
  const search=p.boost_search, m=p.manipulation;
  if(won) {
    const instruction=[m,p.r8_manipulation].filter(adjustment=>adjustment?.inputs.extra_input_count>0).map(adjustment=>{
      const parts=(result.identified_entry && adjustment.round===3 ? identifiedBoostGroups(adjustment.inputs.levels) : groupBoostRoutine(adjustment.inputs.levels)).map(c=>`${c.from} → ${c.to}${c.count>1?' ×'+c.count:''}`);
      const step=p.steps.find(s=>s.round===adjustment.round && p.actors[s.report.actor].side==='player');
      return `<p><strong>R${adjustment.round} · Throné · 1st command menu</strong></p>`+instructionBoxHtml(parts);
    }).join('');
    const r3Step=p.steps.find(s=>s.round===3 && p.actors[s.report.actor].side==='player');
    const r3Notice=m?.inputs.extra_input_count>0?'':'<p><strong>R3 · Throné · 1st command menu</strong></p>'+instructionBoxHtml(['No extra boost adjustment needed']);
    const r8Notice=p.r8_manipulation?.inputs.extra_input_count>0?'':'<p><strong>R8 · Throné · 1st command menu</strong></p>'+instructionBoxHtml(['No extra boost adjustment needed']);
    $('pirro-boost-routine').innerHTML=r3Notice+instruction+r8Notice;
  } else {
    $('pirro-boost-routine').textContent='No boost adjustment found · see Detailed log';
  }
  $('pirro-context').textContent=`${p.group} · ${p.state.outcome} · entry ${totalAdvances(p.entry)} (${hex(p.entry.state)}) · ${p.initial_state.actors[0].resources.hp} HP / ${p.initial_state.actors[0].resources.sp} SP`;
  if(search) {
    const policy=search.settings.no_grape_max_boost_repeats===null
      ? `Required victory without Healing Grape · ${search.victory_repeats} joint repeats`
      : `Victory: ${repeatCapLabel(search.settings.victory_max_boost_repeats)} (${search.victory_repeats} repeats used) · Optional no-grape: ${search.settings.no_grape_max_boost_repeats} additional repeats`;
    $('pirro-context').textContent+=` · R3 search: ${search.tested_candidates} candidates · ${policy}${m?` · +${m.extra_advances} CRT draws · ${m.inputs.extra_input_count} extra button presses`:''}`;
  }
  $('pirro-error').hidden=!p.routine_error;$('pirro-error').textContent=p.routine_error || '';
  $('pirro-verification').textContent=p.verification;
  $('pirro-actions').innerHTML=p.steps.map((s,i)=>renderAction(s,i,p)).join('');
  const checkpoint=(label,rng)=>`<tr><td>${escapeHtml(label)}</td><td>${totalAdvances(rng)}</td><td>${hex(rng.state)}</td><td>${rng.draws}</td></tr>`;
  $('checkpoints').innerHTML+=checkpoint(result.identified_entry?'Pirro R3 · RNG identified (3 BP selected)':'Pirro · cutscene skipped, before initialization',p.entry);
  let round=0,menu=0;
  for(const s of p.steps.filter(s=>p.actors[s.report.actor].side==='player')) {
    if(s.round!==round){round=s.round;menu=0;}menu++;
    $('checkpoints').innerHTML+=checkpoint(`Pirro R${round} · Throné · command menu ${menu}`,secondRng(p,s.report.rng_before));
  }
  $('checkpoints').innerHTML+=checkpoint(p.after_victory?'Pirro · after victory drops':`Pirro · ${p.state.outcome}`,p.after);
  const trace=[...p.transition_trace,...p.state.opening_crt.trace.map(s=>({...s,before:secondRng(p,s.before),after:secondRng(p,s.after)}))];
  $('trace').innerHTML+=trace.map(s=>`<tr><td>Pirro · ${escapeHtml(reasons[s.reason] || s.reason.replaceAll('_',' '))}</td><td>${s.actor===null?'—':escapeHtml(p.actors[s.actor].name)}</td><td>${totalAdvances(s.before)}</td><td>${totalAdvances(s.after)}</td><td>+${BigInt(s.after.draws)-BigInt(s.before.draws)}</td><td>${hex(s.before.state)}</td><td>${hex(s.after.state)}</td></tr>`).join('');
}

function renderMansionEscape() {
  const p=result.post_battle_field?.mansion_escape;
  $('mansion-escape-result').hidden=!p;
  $('mansion-encounter-name').textContent=encounterLabel(p?.first_encounter);
  if(!p) return;
  $('mansion-escape-status').hidden=!!p.solution;
  $('mansion-escape-status').textContent=p.error || p.status;
  $('mansion-escape-status').className='run-state '+(p.solution?'text-good':'text-bad');
  $('mansion-escape-position').textContent=p.error?'':`Diamante Guard I · ${p.first_encounter.situation} · update ${p.first_encounter_update}/${p.run_updates}`;
  $('mansion-escape-routine').innerHTML='';
  $('mansion-escape-diagnostics').innerHTML='';
  if(p.error) return;
  const s=p.solution;
  const menuLabel=`R${p.round} · Throné · 1st command menu`;
  const checkpoint=(label,rng)=>`<tr><td>${escapeHtml(label)}</td><td>${totalAdvances(rng)}</td><td>${hex(rng.state)}</td><td>${rng.draws}</td></tr>`;
  $('checkpoints').innerHTML+=checkpoint('Mansion encounter · before enemy initialization',p.before_battle_initialization)+checkpoint(menuLabel,p.at_command_menu);
  if(s) {
    $('mansion-escape-routine').innerHTML=`<p><strong>${menuLabel}</strong></p>${boostStepsHtml(s.inputs.levels,s.break_then_escape?attackLabel('Dagger',1):'Escape')}${s.break_then_escape?`<p><strong>R${s.attempt.state.round} · Throné · 1st command menu</strong></p>${boostStepsHtml(s.escape_inputs?.levels,'Escape')}`:''}`;
    if(!s.reaches_pirro) $('mansion-escape-routine').innerHTML+='<p class="text-bad">No-second-encounter target not met. Increase its budget or select Required outcome.</p>';
    $('mansion-escape-diagnostics').innerHTML=`<span class="diagnostic-label">CRT at command menu · before boosts</span><div class="diagnostic-value"><strong>${totalAdvances(p.at_command_menu)}</strong><span>advances since initial seed</span></div><span class="rng-secondary">${hex(p.at_command_menu.state)}</span><div class="run-diagnostics"><span class="diagnostic-label">CRT after Escape · field returned</span><div class="diagnostic-value"><strong>${totalAdvances(s.after_field_return)}</strong><span>advances since initial seed</span></div><span class="rng-secondary">${hex(s.after_field_return.state)}</span></div><p class="muted small">Counter ${s.counter} · run ${p.remaining_updates} updates · ${s.counter_at_trigger} left at Pirro trigger</p>`;
    if(s.at_escape_menu) $('checkpoints').innerHTML+=checkpoint(`Mansion R${s.attempt.state.round} · command menu after break`,s.at_escape_menu);
    $('checkpoints').innerHTML+=checkpoint('Mansion Escape · after check',s.after_escape)+checkpoint('Mansion Escape · returned to field / Pirro trigger before cutscene',s.after_field_return);
  } else {
    $('mansion-escape-routine').textContent=p.status==='No plan within repeat limit' ? 'No boost routine found within the limit. Increase the optional budget or select Required outcome in Settings.' : 'No boost routine found for this remaining run.';
    $('mansion-escape-diagnostics').textContent=`Needs counter ≥ ${p.required_counter} for ${p.remaining_updates} remaining updates.`;
  }
  let previous=p.after_lookout;
  for(const c of p.first_encounter.checkpoints) {
    const after={state:c.state,draws:BigInt(p.after_lookout.draws)+BigInt(c.draws)};
    $('trace').innerHTML+=`<tr><td>Mansion · ${escapeHtml(c.label)}</td><td>—</td><td>${totalAdvances(previous)}</td><td>${totalAdvances(after)}</td><td>+${BigInt(after.draws)-BigInt(previous.draws)}</td><td>${hex(previous.state)}</td><td>${hex(after.state)}</td></tr>`;
    previous=after;
  }
  for(const t of s?.trace || p.initial_trace) $('trace').innerHTML+=`<tr><td>Mansion · ${escapeHtml(reasons[t.reason] || t.reason.replaceAll('_',' '))}</td><td>${t.actor==null?'—':t.actor===0?'Throné':'Diamante Guard I'}</td><td>${totalAdvances(t.before)}</td><td>${totalAdvances(t.after)}</td><td>+${BigInt(t.after.draws)-BigInt(t.before.draws)}</td><td>${hex(t.before.state)}</td><td>${hex(t.after.state)}</td></tr>`;
  $('second-verification').textContent+=' '+p.verification;
}
const secondRng = (second,rng) => ({state:rng.state,draws:(BigInt(second.entry.draws)+BigInt(rng.draws)).toString()});
function renderSecond(second) {
  const won = second?.state.outcome==='victory';
  $('second-results').hidden=!won;
  $('second-empty').hidden=won;
  $('second-empty').textContent=second?'Battle not completed':'Not reached';
  $('second-log').hidden=!second;
  if(!second){$('early-steals-result').hidden=true;return;}
  $('second-context').textContent=`Party Lv${second.party_level} · English · 2× · skip the preceding cutscene`;
  $('second-outcome').textContent=second.state.outcome==='victory'?'Victory':second.state.outcome;
  $('second-outcome').className=second.state.outcome==='victory'?'text-good':'text-bad';
  const a=second.assessment;
  $('second-one-action').textContent=a.only_one_enemy_action?'Yes':'No';
  $('second-one-action').className=a.only_one_enemy_action?'text-good':'text-bad';
  const counts=`${a.enemy_actions} enemy action${a.enemy_actions===1?'':'s'} · ${a.damage_hits_received} damage hit${a.damage_hits_received===1?'':'s'}`;
  $('second-action-count').textContent=counts;
  $('second-damage').textContent=`${counts} · ${a.damage_received} damage received · Round ${second.state.round}`;
  const conditions=a.order_conditions;
  const checks=[
    ['R1 · Throné before right grunt',conditions.throne_before_right_round_one],
    ['R1 · Scaracci before left grunt',conditions.scaracci_before_left_round_one],
    ['R2 · Throné before middle leader',conditions.throne_before_leader_round_two],
  ];
  $('second-order-checks').innerHTML=`<table><thead><tr><th>One-action conditions</th><th>Result</th></tr></thead><tbody>${checks.map(([label,pass])=>`<tr><td>${label}</td><td class="${pass?'text-good':'text-bad'}">${pass?'Pass':'Fail'}</td></tr>`).join('')}</tbody></table>`;
  $('early-steals-result').hidden=!second.mansion;
  renderSecondBoost(second);
  $('second-after-advances').textContent=totalAdvances(second.after);
  $('second-after-hex').textContent=hex(second.after.state);
  $('city-result').hidden=!second.city;
  $('city-after-advances').textContent=second.city ? totalAdvances(second.city.after_popup) : '—';
  $('city-after-hex').textContent=second.city ? hex(second.city.after_popup.state) : '—';
  $('gold-ring-order').textContent=second.mansion ? (second.mansion.gold_ring_before_clothes?'Gold Ring → Brothel Girl’s Clothes':'Brothel Girl’s Clothes → Gold Ring') : '';
  $('second-error').hidden=!second.routine_error;
  $('second-error').textContent=second.routine_error||'';
  $('second-verification').textContent=second.verification+(second.city ? ' '+second.city.verification : '')+(second.parlor ? ' '+second.parlor.verification : '')+(second.parlor_revisit ? ' '+second.parlor_revisit.verification : '')+(second.mansion ? ' '+second.mansion.verification+' '+second.mansion.lookout_ambush.verification : '');
  $('second-order').textContent=second.initial_state.queue.map(id=>second.actors[id].name).join(' → ');
  $('second-actions').innerHTML=second.steps.map((s,i)=>renderAction(s,i,second)).join('');
}

function battleAssessmentHtml() {
  const a=result.battle_assessment;
  const turns=a.enemy_turns.map(t=>`R${t.round} · ${shortActorName(t.actor)}`).join('; ');
  return `<h2>Opening battle: enemies never act?</h2><p><span class="pill ${a.no_enemy_actions?'success':'failure'}">${a.no_enemy_actions?'Yes — no enemy actions':'No — '+a.enemy_turns.length+' enemy turn'+(a.enemy_turns.length===1?'':'s')}</span></p><p class="muted small route-help">${a.no_enemy_actions?'Both enemies are stopped before acting and the routine wins.':escapeHtml(turns || 'The routine did not win.')} Checks the entire simulated routine, including breaks and misses. The first boosted attack targets enemy 1 (right).</p>`;
}

function resetReachHtml(r) {
  const row=(label,m)=>`<tr><td>${label}</td><td>${m.cost} (${r.assumed_updates} × ${m.decrement})</td><td><span class="pill ${m.reaches_reset?'success':'failure'}">${m.reaches_reset?'Yes':'No'}</span></td><td>${m.reaches_reset?m.counter_before_reset+' left':`Encounter on update ${m.encounter_on_update} / ${r.assumed_updates}`}</td></tr>`;
  const m=r.mixed;
  const recommendation=m ? `Estimated running share: ${m.max_running_percent}% (${m.max_running_updates} updates), with ${m.walking_updates} walking updates.` : 'Neither running nor walking reaches the reset under this estimate.';
  const mixedRow=m ? `<tr><td>Estimated mixed running · ${m.max_running_percent}%</td><td>${m.cost} (${m.max_running_updates} × ${r.running.decrement} + ${m.walking_updates} × ${r.walking.decrement})</td><td><span class="pill success">Yes, under estimate</span></td><td>${m.counter_before_reset} left</td></tr>` : '';
  return `<h2 class="route-subtitle">Reach the next cutscene without an encounter?</h2><p class="route-help">${recommendation} Starting counter: <b>${r.initial_counter}</b>.</p><div class="table-scroll"><table><thead><tr><th>Movement</th><th>Counter cost</th><th>Reaches reset</th><th>Result</th></tr></thead><tbody>${row('Running · estimated',r.running)}${row('Walking · measured',r.walking)}${mixedRow}</tbody></table></div><p class="muted small route-help">Opening walk measured: 830 → 146, consuming 684 points over ${r.assumed_updates} updates. Running and mixed budgets still assume 57 updates; their actual counts are unmeasured. The game discards excess displacement after each decrement, so changing speed can change update counts and the result can depend on where speeds change. This percentage is an estimate of update share, not measured route distance or elapsed time. Flats has its own separate 61-walk/56-run measurements; that ratio is not applied here.</p>`;
}

// Compress the exact button path, including the action's normal boost. A
// repeated climb lowers back to its start between repetitions, then stays up.
function groupBoostRoutine(levels) {
  const path = [0,...levels];
  const best = Array(path.length);
  best[path.length-1] = [];
  for (let i=path.length-2; i>=0; i--) {
    const from = path[i];
    const direction = Math.sign(path[i+1]-from);
    for (let j=i+1; j<path.length && path[j]-path[j-1]===direction; j++) {
      const to = path[j];
      const width = Math.abs(to-from);
      const repeat = direction>0 ? [
        ...Array.from({length:width},(_,k)=>to-k-1),
        ...Array.from({length:width},(_,k)=>from+k+1),
      ] : [];
      let end = j;
      let count = 1;
      while (true) {
        const candidate = [{from,to,count},...best[end]];
        if (!best[i] || candidate.length<=best[i].length) best[i] = candidate;
        if (!repeat.length || end+repeat.length>=path.length ||
            !repeat.every((bp,k)=>path[end+k+1]===bp)) break;
        end += repeat.length;
        count++;
      }
    }
  }
  return best[0];
}

function renderSecondBoost(second) {
  const search=second.mansion_approach_search;
  const m=second.manipulation;
  const node=$('second-boost-routine');
  node.innerHTML='';
  if(!search) {node.innerHTML=instructionBoxHtml(['No extra boost adjustment needed']);return;}
  const status=search.target_met ? 'Mansion opening · '+search.situation : search.limit_reached ? 'Gold Ring/mansion target not found within repeat limit.' : 'Mansion surprise-avoidance target not reached.';
  let routine='';
  if(m?.inputs.extra_input_count>0) {
    const step=second.steps.filter(s=>second.actors[s.report.actor].side==='player')[m.plan.player_menu-1];
    const action=step.report.events.find(e=>e.kind==='action');
    const targets=[...new Set(step.report.events.filter(e=>e.kind==='damage' || e.kind==='miss').map(e=>second.actors[e.target].name))].join(', ');
    const ordinal=['1st','2nd','3rd'][m.round_menu-1] ?? m.round_menu+'th';
    const heading='R'+m.round+' · '+second.actors[m.actor].name+' · '+ordinal+' command menu';
    const parts=groupBoostRoutine(m.inputs.levels).map(c=>c.from+'→'+c.to+(c.count>1?' ×'+c.count:''));
    routine='<p><strong>'+escapeHtml(heading)+'</strong></p>'+instructionBoxHtml(parts);
  }
  node.innerHTML=(routine?'<h3>Boost adjustment</h3>'+routine:instructionBoxHtml(['No extra boost adjustment needed']))+(!search.target_met?'<p class="text-bad">'+escapeHtml(status)+' Adjust its search in Settings.</p>':'');
}

function renderBoostPlan() {
  const m = result.manipulation;
  const limited = result.boost_search?.second_fight_limit_reached;
  const exhausted = result.boost_search?.second_fight_search_exhausted;
  $('boost-cap-note').hidden = !exhausted;
  $('boost-cap-note').textContent = limited ? `Fight-two target skipped (${result.crt_setup.second_fight_max_boost_repeats}-repeat limit).` : exhausted ? 'Fight-two target not found.' : '';
  $('boost-detail').hidden = !m;
  $('boost-help').hidden = !m;
  if (!m) {
    $('boost-result').textContent = result.post_battle_field?.reset_reach.walking.reaches_reset
      ? 'No extra boost adjustment needed'
      : 'Not calculated.';
    return;
  }
  const n = m.plan.advances;
  const playerSteps = result.steps.filter(s=>result.actors[s.report.actor].side==='player');
  const step = playerSteps[m.plan.player_menu-1];
  // Count actual player menus in this round, including Defend and excluding enemies.
  const roundMenu = playerSteps.slice(0,m.plan.player_menu).filter(s=>s.round===m.round).length;
  const menuOrdinal = ['1st','2nd','3rd'][roundMenu-1] ?? `${roundMenu}th`;
  const events = step.report.events;
  const action = events.find(e=>e.kind==='action');
  const inputs = m.inputs;
  const targets = [...new Set(events.filter(e=>e.kind==='damage' || e.kind==='miss').map(e=>shortActorName(e.target)))].join(', ');
  const command = stepActionLabel(step,result);
  const path = levels => levels.join('→')+' BP';
  const parts = groupBoostRoutine(inputs.levels)
    .map(({from,to,count})=>`${from}→${to}${count>1 ? ` ×${count}` : ''}`);
  const heading = `R${m.round} · ${shortActorName(m.actor)} · ${menuOrdinal} command menu`;
  const instruction = `${heading}: ${parts.join('; ')}. ${command}.`;
  $('boost-result').innerHTML = inputs.extra_input_count
    ? `<b>${escapeHtml(heading)}</b>${instructionBoxHtml(parts)}`
    : instructionBoxHtml(['No extra boost adjustment needed']);
  $('boost-help').textContent = `${inputs.extra_repeats} extra repeat${inputs.extra_repeats===1?'':'s'} · ${inputs.input_count} boost inputs · +${n} CRT advances`;
  const search=result.boost_search;
  const budget=search ? (result.crt_setup.second_fight_max_boost_repeats===null
    ? ` Required checkpoint safety and one-action second fight: ${inputs.extra_repeats} repeats in the joint plan.`
    : ` Checkpoint plan: ${search.encounter_dodge_repeats} repeats (${repeatCapLabel(result.crt_setup.encounter_max_boost_repeats)}). Optional second fight: up to ${result.crt_setup.second_fight_max_boost_repeats} additional repeats; ${exhausted?'target skipped':`${search.second_fight_extra_repeats} used`}.`) : '';
  $('boost-detail').textContent = `${instruction} Full input path: ${path([0,...inputs.levels])}. Repeated climbs return to their starting BP between repetitions; the last repetition stays at its ending BP. The routine includes the action's final boost. Menu #${m.plan.player_menu} overall: ${totalAdvances(m.rng_before)} (${hex(m.rng_before.state)}) → ${totalAdvances(m.rng_after_inputs)} (${hex(m.rng_after_inputs.state)}) after all boost inputs.${budget} Required outcomes are solved together before optional improvements. Optional misses preserve the required plan. Candidates replay the checkpoint, Ambush and both battles, minimizing extra button presses at one first-fight menu. Ties: fewer draws, then latest menu. First-fight damage-less is not required. Repeated boost inputs await live validation.`;

}

function renderAction(step, index, simulation=result) {
  const actorName=id=>simulation.actors[id]?.name??'—';
  const shortActorName=id=>actorName(id).split(' · ')[0];
  const safeName=id=>escapeHtml(actorName(id));
  const totalAdvances=rng=>(BigInt(result.crt_setup.battle_entry_advances)+BigInt(simulation.entry?.draws??0)+BigInt(rng.draws)).toString();
  const report = step.report;
  const events = report.events;
  const action = events.find(e => e.kind === 'action');
  const bp = events.find(e => e.kind === 'resource_changed' && e.resource === 'bp' && e.actor === report.actor);
  const hits = events.filter(e => e.kind === 'damage' || e.kind === 'miss');
  const damage = hits.filter(e => e.kind === 'damage').map(e => e.amount);
  const boost = bp ? bp.before-bp.after : 0;
  const isPlayer = simulation.actors[report.actor].side === 'player';
  const rngLabel = isPlayer ? 'Command menu · before boost' : 'Before enemy action';
  const latent=events.some(e=>e.kind==='latent_activated');
  const description = (latent?'Latent → ':'') + stepActionLabel(step,simulation);
  const targets = [...new Set(hits.map(e => shortActorName(e.target)))].join(', ');
  const changes = events.map(e => {
    switch(e.kind) {
      case 'shield_changed': return `${actorName(e.actor)}: shields ${e.before} → ${e.after}`;
      case 'broken': return `${actorName(e.actor)} breaks; recovery would be in round ${e.recover_round}.`;
      case 'knocked_out': return `${actorName(e.actor)} is defeated.`;
      case 'resource_changed': return `${actorName(e.actor)}: ${e.resource.toUpperCase()} ${e.before} → ${e.after}`;
      case 'battle_ended': return `Battle ended: ${e.outcome}.`;
      default: return null;
    }
  }).filter(Boolean);
  const resources = step.actors_after.map((actor,i) => `<span><b>${safeName(i)}</b>${actor.resources.hp} HP · ${actor.resources.sp} SP · ${actor.resources.bp} BP${simulation.actors[i].side === 'enemy' ? ` · ${actor.shields} shield` : ''}</span>`).join('');
  return `<details class="action"><summary><span class="round">R${step.round} · #${index+1}</span><span><b>${safeName(report.actor)}</b><span class="sub">${escapeHtml(description)}</span></span><span class="rolls">${escapeHtml(targets || 'Self')}<span class="sub">${damage.length ? `${damage.join(' + ')} damage rolled` : hits.length ? 'Miss' : action?.name==='defend' ? 'Guard applied' : 'Action completed'}</span></span><span class="action-rng">${totalAdvances(report.rng_before)} advances<span class="sub">${rngLabel}</span><span class="sub">From initial seed</span></span></summary><div class="action-body"><div class="hit-list">${hits.map((hit,i) => `<span class="hit"><strong>Hit ${i+1}</strong> ${hit.kind === 'miss' ? 'Miss' : `${hit.amount} · ${hit.broken_before ? 'Broken target' : hit.weak ? 'Weakness' : 'Neutral'}${hit.critical ? ' · Critical' : ''}`}</span>`).join('')}</div><p class="small muted">${rngLabel}: ${totalAdvances(report.rng_before)} advances from initial seed · CRT ${hex(report.rng_before.state)}. After action and transition: ${totalAdvances(report.rng_after)} advances · CRT ${hex(report.rng_after.state)}. Includes the next actor's turn-start voice or victory callbacks.</p>${changes.length ? `<ul class="event-list">${changes.map(text => `<li>${escapeHtml(text)}</li>`).join('')}</ul>` : ''}<div class="after-resources">${resources}</div></div></details>`;
}

function selectTab(selected) {
  for (const name of ['actions','trace']) {
    const active = selected === name;
    $(`${name}-tab`).setAttribute('aria-selected',String(active));
    $(`${name}-tab`).tabIndex = active ? 0 : -1;
    $(`${name}-view`).hidden = !active;
  }
}
for (const name of ['actions','trace']) {
  $(`${name}-tab`).addEventListener('click',() => selectTab(name));
  $(`${name}-tab`).addEventListener('keydown',event => {
    if (['ArrowLeft','ArrowRight','Home','End'].includes(event.key)) {
      event.preventDefault();
      const next = event.key === 'Home' ? 'actions' : event.key === 'End' ? 'trace' : name === 'actions' ? 'trace' : 'actions';
      selectTab(next); $(`${next}-tab`).focus();
    }
  });
}
$('copy').addEventListener('click',async () => {
  if (!result) return;
  try { await navigator.clipboard.writeText(hex(result.state.rng.state)); $('copy').textContent = 'Copied'; }
  catch { showError('Clipboard unavailable. Select and copy the final CRT value above.'); }
});
$('export').addEventListener('click',() => {
  if (!result) return;
  const link = document.createElement('a');
  if(result.identified_entry) {
    const url=URL.createObjectURL(new Blob([JSON.stringify(result,null,2)],{type:'application/json'}));
    link.href=url;link.download=`ot2-pirro-identified-${hex(result.initial_seed)}.json`;
    document.body.append(link);link.click();link.remove();setTimeout(()=>URL.revokeObjectURL(url),1000);return;
  }
  const setup = result.crt_setup;
  link.href = `/api/export?${setupQuery({...setup,seed:setup.initial_seed})}`;
  const suffix = setup.recreate_shared_level ? '-recreated' : '';
  const boostSuffix = setup.boost_advances ? `-boost-menu-${setup.boost_menu}-advances-${setup.boost_advances}` : '';
  link.download = `ot2-throne-seed-${setup.initial_seed}-title-advances-${setup.advances}${suffix}${boostSuffix}.json`;
  document.body.append(link); link.click(); link.remove();
});
$('quit').addEventListener('click',async () => {
  if (running || stopped) return;
  $('quit').disabled = true;
  try {
    const response = await fetch('/api/shutdown',{method:'POST',headers:{'X-OT2-Viewer':'1'}});
    if (!response.ok) throw new Error('Could not stop the viewer.');
    stopped = true;
    for (const name of ['manipulation','detail','field','settings']) $(`${name}-panel`).hidden = true;
    document.querySelector('.mode-switch').hidden = true;
    $('closed').hidden = false;
    $('quit').textContent = 'Stopped'; $('status').textContent = 'Local simulator stopped.';
  } catch(error) { showError(error.message); $('quit').disabled = false; }
});
$('seed-form').addEventListener('submit',simulate);
$('seed').addEventListener('input',markSetupChanged);
$('advances').addEventListener('input',markSetupChanged);
$('recreate-shared-level').addEventListener('click',() => {
  const enabled = $('recreate-shared-level').getAttribute('aria-pressed') !== 'true';
  $('recreate-shared-level').setAttribute('aria-pressed',String(enabled));
  $('recreate-state').textContent = enabled ? 'On' : 'Off';
  markSetupChanged();
});
function updateRepeatControls() {
  for (const setting of repeatSettings) {
    const unlimited = $(`${setting.id}-unlimited`);
    unlimited.disabled = running || stopped || hardNoSurprise.has(setting.key);
    $(setting.id).disabled = running || stopped || unlimited.checked;
  }
}
for (const setting of repeatSettings) {
  $(setting.id).value = String(lastRepeatCaps[setting.key]);
  $(`${setting.id}-unlimited`).checked = eventRepeatCaps[setting.key] === null;
  $(`${setting.id}-unlimited`).addEventListener('change',updateRepeatControls);
}
updateRepeatControls();
$('settings-form').addEventListener('submit',event => {
  event.preventDefault();
  if (running || stopped) return;
  $('settings-error').hidden = true;
  $('settings-status').textContent = '';
  const updated = {};
  for (const setting of repeatSettings) {
    const value = $(setting.id).value.trim();
    const unlimited = hardNoSurprise.has(setting.key) || $(`${setting.id}-unlimited`).checked;
    if (!unlimited && !validRepeatCap(value)) {
      $('settings-error').textContent = `${setting.label}: enter a whole number from 0 to 256.`;
      $('settings-error').hidden = false;
      $(setting.id).focus();
      return;
    }
    updated[setting.key] = unlimited ? null : Number(value);
  }
  eventRepeatCaps = updated;
  let persisted = true;
  try {
    for (const setting of repeatSettings) {
      localStorage.setItem(setting.storage,repeatCapQuery(eventRepeatCaps[setting.key],setting.key));
      const value = $(setting.id).value.trim();
      if (validRepeatCap(value)) localStorage.setItem(`${setting.storage}-last-limit`,value);
    }
  } catch { persisted = false; }
  markSetupChanged();
  $('settings-status').textContent = persisted ? 'Saved. Run simulation to apply.' : 'Saved for this session. Browser storage is unavailable.';
});
// This branch waits for voice identification; never replay early manipulation on load.

function lateRouteInstructions(r) {
  const names=routeNames();
  const defends=r.defends||[];
  let html='<p>'+escapeHtml(encounterLabel(r.encounter))+'</p>';
  const ordinals={};
  defends.forEach((d,i)=>{
    const round=r.defend_rounds?.[i]??1;
    const ordinal=ordinals[round]=(ordinals[round]||0)+1;
    html+='<p><strong>T'+round+' · '+names[d.actor]+' · '+menuOrdinal(ordinal)+' command menu</strong></p>'+instructionBoxHtml(['Defend']);
  });
  const round=r.escape_round??1;
  return html+'<p><strong>T'+round+' · '+names[r.menu.first_actor]+' · '+menuOrdinal((ordinals[round]||0)+1)+' command menu</strong></p>'+boostStepsHtml(r.inputs.levels,r.victory?'Wind Soulstone (L)':'Escape');
}

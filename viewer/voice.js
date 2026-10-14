'use strict';
let voiceStage='order', voiceFirst=null, voiceHp=null;
let voiceLocked=false;
let voiceTranscript='', voiceBusy=false, voiceCandidate=null, voiceRevision=0, voiceAudio=null;
const voiceRouteStates=new Map();
window.addEventListener('route-changed',({detail:{route,previous}})=>{
  voiceRouteStates.set(previous,{voiceStage,voiceFirst,voiceHp,voiceLocked,voiceTranscript,voiceCandidate,status:$('voice-status').textContent});
  ({voiceStage='order',voiceFirst=null,voiceHp=null,voiceLocked=false,voiceTranscript='',voiceCandidate=null}=voiceRouteStates.get(route)||{});
  voiceRevision++;
  $('voice-hp').value=voiceHp??'';
  $('voice-status').textContent=voiceRouteStates.get(route)?.status??'Choose turn-3 order: Left = Throné · Right = Pirro';
  voiceControls();
});
function voiceControls() {
  for(const id of ['voice-normal','voice-special']) $(id).disabled=voiceStage!=='lines' || stopped || voiceLocked;
  $('voice-undo').disabled=stopped || voiceLocked || !voiceTranscript.length;
  $('voice-lock').disabled=stopped;
  $('voice-lock').textContent=voiceLocked?'Unlock input':'Lock input · Enter';
  $('voice-lock').setAttribute('aria-pressed',String(voiceLocked));
  $('voice-lock-status').textContent=voiceLocked?'Input locked':'Input open · Enter to lock';
  $('voice-reset').disabled=running;
  $('voice-order-step').hidden=voiceStage!=='order';
  $('voice-hp-step').hidden=voiceStage!=='hp';
  $('voice-lines-step').hidden=voiceStage!=='lines';
  $('voice-hp').disabled=running;
  $('voice-setup-summary').textContent=voiceFirst ? `R3: ${voiceFirst==='throne'?'Throné':'Pirro'} first${voiceHp!==null?' · Throné '+voiceHp+' HP':''}` : '';
  $('voice-transcript').textContent=voiceTranscript ? `${voiceTranscript.length} observations · ${voiceTranscript}` : '';
}
function recordVoice(symbol) {
  if(voiceStage!=='lines' || stopped || voiceLocked) return;
  try {voiceAudio ??= new (window.AudioContext || window.webkitAudioContext)();voiceAudio.resume();} catch {}
  voiceTranscript+=symbol;voiceRevision++;voiceCandidate=null;
  clearVoiceResult();voiceControls();
  if(voiceBusy || running) $('voice-status').textContent='Extra input recorded · updating prediction…';
  filterVoices();
}
function clearVoiceResult() {
  $('results').hidden=true;$('detail-results').hidden=true;$('detail-empty').hidden=false;
  $('error').hidden=true;
}
function voiceNotification(seed) {
  $('voice-status').textContent=`RNG identified: ${hex(seed)} · Stop boosting at 3 BP. Starting simulation…`;
  document.title='RNG identified · OT2';
  if(voiceAudio) try {
    const o=voiceAudio.createOscillator(),g=voiceAudio.createGain();
    o.frequency.value=880;g.gain.value=0.08;o.connect(g);g.connect(voiceAudio.destination);
    o.start();o.stop(voiceAudio.currentTime+0.2);
  } catch {}
}
async function filterVoices() {
  if(voiceStage!=='lines' || voiceBusy || running) return;
  voiceBusy=true;
  const revision=voiceRevision, transcript=voiceTranscript, route=activeRoute;
  $('voice-status').textContent=transcript?'Filtering RNG candidates…':'Ready · Left = normal · Right = special';
  try {
    const response=await fetch('/api/identify-observed?'+new URLSearchParams({voices:transcript,r3_first:voiceFirst,hp:String(voiceHp)}));
    const data=await response.json();
    if(revision!==voiceRevision || route!==activeRoute) return;
    if(!response.ok) throw new Error(data.error || 'Identification failed');
    $('voice-status').textContent=data.count ? `${data.count.toLocaleString()} candidates · ${data.observations} observations` : 'No matching RNG. Check the voicelines, turn-3 order and HP.';
    voiceCandidate=data.candidate;
    if(voiceCandidate) {voiceNotification(voiceCandidate.current_state);await simulateIdentified();}
  } catch(e) {
    if(revision===voiceRevision) $('voice-status').textContent=e.message;
  } finally {
    voiceBusy=false;voiceControls();
    if(revision!==voiceRevision) filterVoices();
  }
}
async function simulateIdentified() {
  if(voiceStage!=='lines' || !voiceFirst || voiceHp===null || !voiceCandidate || running) return;
  const query=new URLSearchParams({route:activeRoute,voices:voiceTranscript,r3_first:voiceFirst,hp:String(voiceHp),stream:'1'});
  const revision=voiceRevision, candidate=voiceCandidate, route=activeRoute;
  running=true;voiceControls();$('quit').disabled=true;clearVoiceResult();
  $('status').textContent='Simulating from identified RNG…';
  try {
    const response=await fetch('/api/identified-route?'+query);
    if(!response.ok) throw new Error((await response.json()).error || 'Simulation failed');
    await readRouteStream(response,event=>{
      // A late chunk must never overwrite another tab or a newer transcript.
      if(revision!==voiceRevision || route!==activeRoute) return;
      if(event.type==='error') throw new Error(event.message);
      result=event.result;snowsEncounterTiming=null;westernSecondOverride=null;
      $('seed').value=hex(candidate.current_state);$('advances').value='0';
      render();$('status').textContent=event.message;
      if(event.type==='complete') window.dispatchEvent(new Event('battle-simulated'));
    });
    if(revision!==voiceRevision || route!==activeRoute) return;
    $('voice-status').textContent=`Identified seed ${hex(candidate.current_state)} · 0 advances immediately after the last recorded boost`;
    $('status').textContent='';
  } catch(e) {
    if(revision!==voiceRevision || route!==activeRoute) return;
    if(result?.partial){result.partial_error=e.message;render();}
    $('voice-status').textContent=`RNG identified: ${hex(candidate.current_state)}. ${e.message}`;
    $('status').textContent=result?.partial?'Completed sections remain available. Remaining route failed: '+e.message:'Route prediction not available yet.';
  } finally {running=false;$('quit').disabled=false;voiceControls();}
}
async function readRouteStream(response,onEvent) {
  const reader=response.body.getReader(),decoder=new TextDecoder();
  let pending='',complete=false;
  const consume=line=>{
    if(!line.trim())return;
    const event=JSON.parse(line);
    if(!['progress','complete','error'].includes(event.type))throw new Error('Invalid route stream event');
    onEvent(event);
    if(event.type==='error')throw new Error(event.message);
    if(event.type==='complete')complete=true;
  };
  try {
    for(;;) {
      const {value,done}=await reader.read();
      pending+=decoder.decode(value,{stream:!done});
      let end;while((end=pending.indexOf('\n'))>=0){consume(pending.slice(0,end));pending=pending.slice(end+1);}
      if(done)break;
    }
    if(pending.trim())consume(pending);
    if(!complete)throw new Error('Connection ended before the remaining route finished');
  } finally {await reader.cancel().catch(()=>{});reader.releaseLock();}
}
$('voice-normal').addEventListener('click',()=>recordVoice('N'));
$('voice-special').addEventListener('click',()=>recordVoice('S'));
function undoVoice() {
  if(voiceStage!=='lines' || stopped || voiceLocked || !voiceTranscript.length)return;
  voiceTranscript=voiceTranscript.slice(0,-1);voiceRevision++;voiceCandidate=null;
  clearVoiceResult();voiceControls();filterVoices();
}
$('voice-undo').addEventListener('click',undoVoice);
function lockVoiceInput() {
  if(voiceStage!=='lines' || stopped)return;
  voiceLocked=true;voiceControls();
}
$('voice-lock').addEventListener('click',()=>{
  if(voiceStage!=='lines' || stopped)return;
  voiceLocked=!voiceLocked;voiceControls();
});
$('voice-reset').addEventListener('click',()=>{
  if(running)return;voiceTranscript='';voiceRevision++;voiceCandidate=null;
  voiceLocked=false;
  voiceStage='order';voiceFirst=null;voiceHp=null;$('voice-hp').value='';
  document.title='OT2 Speedrun Assistant';clearVoiceResult();voiceControls();
  $('voice-status').textContent='Choose turn-3 order: Left = Throné · Right = Pirro';
  $('status').textContent='Identify RNG to start prediction.';
  $('voice-throne-first').focus();
});
function chooseVoiceOrder(first) {
  if(voiceStage!=='order' || running || stopped)return;
  voiceFirst=first;voiceStage='hp';voiceControls();
  $('voice-status').textContent='Enter Throné’s HP at her turn-3 command menu, then press Enter.';
  $('voice-hp').focus();
}
$('voice-throne-first').addEventListener('click',()=>chooseVoiceOrder('throne'));
$('voice-pirro-first').addEventListener('click',()=>chooseVoiceOrder('pirro'));
$('voice-hp-step').addEventListener('submit',e=>{
  e.preventDefault();
  if(voiceStage!=='hp' || running || !$('voice-hp').reportValidity())return;
  voiceHp=Number($('voice-hp').value);voiceStage='lines';voiceControls();
  $('voice-status').textContent='Ready · Left = normal · Right = special';
  $('voice-normal').focus();
});
document.addEventListener('keydown',e=>{
  if(e.repeat || e.altKey || e.ctrlKey || e.metaKey || e.shiftKey || activeScreen!=='manipulation' ||
    e.target.closest('input,textarea,select,[contenteditable="true"]')) return;
  if(e.key==='Enter' && voiceStage==='lines') {
    // Enter only locks; it must not activate a focused voiceline/Unlock button.
    e.preventDefault();lockVoiceInput();
  } else if(e.key==='Backspace' && voiceStage==='lines') {
    e.preventDefault();undoVoice();
  } else if(e.key==='ArrowLeft' || e.key==='ArrowRight') {
    e.preventDefault();
    if(voiceStage==='order')chooseVoiceOrder(e.key==='ArrowLeft'?'throne':'pirro');
    else if(voiceStage==='lines')recordVoice(e.key==='ArrowLeft'?'N':'S');
  }
});
// A resumed R3 sequence starts at selected BP3, unlike normal menu paths.
function identifiedBoostGroups(levels) {
  const groups=[];let previous=3;
  for(const level of levels) {
    const last=groups[groups.length-1];
    if(last && last.from===previous && last.to===level)last.count++;
    else groups.push({from:previous,to:level,count:1});
    previous=level;
  }
  return groups;
}
function renderIdentified() {
  $('export').disabled=Boolean(result.partial);
  $('results').hidden=false;$('detail-results').hidden=false;$('detail-empty').hidden=true;
  $('required-outcomes-error').hidden=true;
  $('result-context').textContent=`Identified seed ${hex(result.initial_seed)} · Pirro R3 (3 BP selected)`;
  $('outcome').textContent=result.partial?(result.partial_error?'Completed sections · remaining route failed':'Completed sections · remaining route calculating'):'Prediction from identification';
  $('final-hex').textContent=hex(result.state.rng.state);$('final-advances').textContent=result.state.rng.draws;
  $('checkpoints').innerHTML='';$('trace').innerHTML='';
  $('actions').innerHTML=result.steps.map((s,i)=>renderAction(s,i)).join('');
  $('log-count').textContent='Pirro continuation';
  $('initial-order').textContent=result.initial_state.queue.map(shortActorName).join(' → ');
  $('party').innerHTML='';$('enemy-outcome').textContent='';$('route-checks').innerHTML='';
  renderPirro();renderPirroAftermath();renderHighroad();renderFlats();renderHarborReturn();
  renderFlatsEscape();renderVillageVisit();renderVillageReturn();renderOsvald();renderEarlyCait();
  renderCapeCold();renderMug();renderCrackridgeVoyage();renderActionVisibility();
}
voiceControls();

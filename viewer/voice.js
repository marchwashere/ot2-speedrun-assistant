'use strict';
let voiceStage='order', voiceFirst=null, voiceHp=null;
let voiceTranscript='', voiceBusy=false, voiceCandidate=null, voiceRevision=0, voiceAudio=null;
function voiceControls() {
  for(const id of ['voice-normal','voice-special']) $(id).disabled=voiceStage!=='lines' || running || !!voiceCandidate;
  $('voice-undo').disabled=running || !voiceTranscript.length;
  $('voice-reset').disabled=running;
  $('voice-order-step').hidden=voiceStage!=='order';
  $('voice-hp-step').hidden=voiceStage!=='hp';
  $('voice-lines-step').hidden=voiceStage!=='lines';
  $('voice-hp').disabled=running;
  $('voice-setup-summary').textContent=voiceFirst ? `R3: ${voiceFirst==='throne'?'Throné':'Pirro'} first${voiceHp!==null?' · Throné '+voiceHp+' HP':''}` : '';
  $('voice-transcript').textContent=voiceTranscript ? `${voiceTranscript.length} observations · ${voiceTranscript}` : '';
}
function recordVoice(symbol) {
  if(voiceStage!=='lines' || running || voiceCandidate || stopped) return;
  try {voiceAudio ??= new (window.AudioContext || window.webkitAudioContext)();voiceAudio.resume();} catch {}
  voiceTranscript+=symbol;voiceRevision++;clearVoiceResult();voiceControls();filterVoices();
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
  const revision=voiceRevision, transcript=voiceTranscript;
  $('voice-status').textContent=transcript?'Filtering RNG candidates…':'Ready · Left = normal · Right = special';
  try {
    const response=await fetch('/api/identify-observed?'+new URLSearchParams({voices:transcript,r3_first:voiceFirst,hp:String(voiceHp)}));
    const data=await response.json();
    if(revision!==voiceRevision) return;
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
  const query=new URLSearchParams({voices:voiceTranscript,r3_first:voiceFirst,hp:String(voiceHp)});
  running=true;voiceControls();$('quit').disabled=true;clearVoiceResult();
  $('status').textContent='Simulating from identified RNG…';
  try {
    const response=await fetch('/api/identified-route?'+query);
    const data=await response.json();
    if(!response.ok) throw new Error(data.error || 'Simulation failed');
    result=data;snowsEncounterTiming=null;westernSecondOverride=null;
    $('seed').value=hex(voiceCandidate.current_state);$('advances').value='0';
    render();window.dispatchEvent(new Event('battle-simulated'));
    $('voice-status').textContent=`Identified seed ${hex(voiceCandidate.current_state)} · 0 advances immediately after the last recorded boost`;
    $('status').textContent='';
  } catch(e) {
    $('voice-status').textContent=`RNG identified: ${hex(voiceCandidate.current_state)}. ${e.message}`;
    $('status').textContent='Route prediction not available yet.';
  } finally {running=false;$('quit').disabled=false;voiceControls();}
}
$('voice-normal').addEventListener('click',()=>recordVoice('N'));
$('voice-special').addEventListener('click',()=>recordVoice('S'));
function undoVoice() {
  if(voiceStage!=='lines' || running || !voiceTranscript.length)return;
  voiceTranscript=voiceTranscript.slice(0,-1);voiceRevision++;voiceCandidate=null;
  clearVoiceResult();voiceControls();filterVoices();
}
$('voice-undo').addEventListener('click',undoVoice);
$('voice-reset').addEventListener('click',()=>{
  if(running)return;voiceTranscript='';voiceRevision++;voiceCandidate=null;
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
  if(e.key==='Backspace' && voiceStage==='lines') {
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
  $('results').hidden=false;$('detail-results').hidden=false;$('detail-empty').hidden=true;
  $('required-outcomes-error').hidden=true;
  $('result-context').textContent=`Identified seed ${hex(result.initial_seed)} · Pirro R3 (3 BP selected)`;
  $('outcome').textContent='Prediction from identification';
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

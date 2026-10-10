'use strict';
const assert=require('node:assert/strict'),fs=require('node:fs'),vm=require('node:vm');
const elements=new Map(),pending=[],rendered=[],handlers={};
const element=id=>{if(!elements.has(id))elements.set(id,{addEventListener(type,fn){handlers[id+':'+type]=fn;},focus(){},reportValidity(){return true;}});return elements.get(id);};
const context=vm.createContext({$:element,running:false,stopped:false,activeScreen:'manipulation',
  URLSearchParams,Event,window:{dispatchEvent(){}},document:{addEventListener(type,fn){handlers[type]=fn;},title:''},
  hex:n=>'0x'+n.toString(16),render(){rendered.push(vm.runInContext('result',context));},
  result:null,snowsEncounterTiming:null,westernSecondOverride:null,
  fetch:url=>new Promise(resolve=>pending.push({url,resolve})),console});
vm.runInContext(fs.readFileSync(require('node:path').join(__dirname,'../viewer/voice.js'),'utf8'),context);
const run=code=>vm.runInContext(code,context);
const flush=()=>new Promise(resolve=>setImmediate(resolve));
async function reply(data,ok=true){assert(pending.length);pending.shift().resolve({ok,json:async()=>data});await flush();}
(async()=>{
  run("voiceStage='lines';voiceFirst='pirro';voiceHp=154;recordVoice('N')");
  await reply({count:1,observations:1,candidate:{current_state:10}});
  assert(pending[0].url.startsWith('/api/identified-route?'));
  assert.equal(element('voice-normal').disabled,false);
  let prevented=false;
  handlers.keydown({key:'Enter',target:{closest:()=>null},preventDefault(){prevented=true;}});
  assert(prevented);assert.equal(run('voiceLocked'),true);
  run("recordVoice('S');undoVoice()");
  assert.equal(run('voiceTranscript'),'N');
  assert.equal(element('voice-normal').disabled,true);
  handlers['voice-lock:click']();
  assert.equal(run('voiceLocked'),false);
  run("recordVoice('S');recordVoice('N');undoVoice()");
  assert.equal(run('voiceTranscript'),'NS');
  assert.equal(element('results').hidden,true);
  await reply({initial_seed:10}); // Old route must not be rendered.
  assert.equal(rendered.length,0);
  assert(pending[0].url.includes('voices=NS'));
  await reply({count:1,observations:2,candidate:{current_state:20}});
  run('lockVoiceInput()');
  await reply({initial_seed:20});
  assert.equal(rendered.length,1);assert.equal(rendered[0].initial_seed,20);
  assert.equal(run('voiceLocked'),true); // Completion does not unlock inputs.
  handlers['voice-lock:click']();
  run("recordVoice('N')"); // Still editable after simulation completes.
  assert.equal(run('voiceTranscript'),'NSN');
  await reply({count:1,observations:3,candidate:{current_state:30}});
  run('undoVoice()');
  await reply({error:'stale failure'},false);
  assert.equal(rendered.length,1);
  assert(pending[0].url.includes('voices=NS'));
  await reply({count:1,observations:2,candidate:{current_state:20}});
  await reply({initial_seed:20});
  assert.equal(rendered.length,2);
  assert.equal(run('running'),false);
  console.log('PASS: extra inputs, undo, queued latest transcript, stale success/error suppression');
})().catch(e=>{console.error(e);process.exitCode=1;});

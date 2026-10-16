'use strict';
const assert=require('node:assert/strict'),fs=require('node:fs'),vm=require('node:vm');
let honorAbort=false;
const elements=new Map(),pending=[],rendered=[],handlers={};
const element=id=>{if(!elements.has(id))elements.set(id,{addEventListener(type,fn){handlers[id+':'+type]=fn;},focus(){},setAttribute(){},reportValidity(){return true;}});return elements.get(id);};
const context=vm.createContext({$:element,running:false,stopped:false,activeScreen:'manipulation',activeRoute:'galdera',
  URLSearchParams,Event,TextDecoder,AbortController,window:{dispatchEvent(){},addEventListener(){}},document:{addEventListener(type,fn){handlers[type]=fn;},title:''},
  hex:n=>'0x'+n.toString(16),render(){rendered.push(vm.runInContext('result',context));},
  result:null,snowsEncounterTiming:null,westernSecondOverride:null,
  fetch:(url,options)=>new Promise((resolve,reject)=>{const request={url,resolve,signal:options?.signal};pending.push(request);if(honorAbort)options.signal.addEventListener('abort',()=>{const i=pending.indexOf(request);if(i>=0)pending.splice(i,1);reject(new DOMException('Aborted','AbortError'));},{once:true});}),console});
vm.runInContext(fs.readFileSync(require('node:path').join(__dirname,'../viewer/voice.js'),'utf8'),context);
const run=code=>vm.runInContext(code,context);
const flush=()=>new Promise(resolve=>setImmediate(resolve));
async function reply(data,ok=true){assert(pending.length);const request=pending.shift();request.resolve(request.url.startsWith('/api/identified-route?')&&ok?new Response(JSON.stringify({type:'complete',result:data,message:''})+'\n'):{ok,json:async()=>data});await flush();}
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
  const streamed=run('simulateIdentified()');
  let controller;
  pending.shift().resolve(new Response(new ReadableStream({start(c){controller=c;}})));
  const send=event=>controller.enqueue(new TextEncoder().encode(JSON.stringify(event)+'\n'));
  send({type:'progress',result:{initial_seed:20,partial:true},message:'Searching JP'});
  await flush();
  assert.equal(rendered.length,3);assert.equal(run('running'),true);
  assert.equal(element('status').textContent,'Searching JP');
  send({type:'error',message:'test interruption'});controller.close();await streamed;
  assert.equal(run('result.initial_seed'),20);
  assert.equal(run('result.partial_error'),'test interruption');
  assert.match(element('status').textContent,/Completed sections remain available/);
  const stale=run('simulateIdentified()');
  pending.shift().resolve(new Response(new ReadableStream({start(c){controller=c;}})));
  run('voiceRevision++');
  const count=rendered.length;
  send({type:'progress',result:{initial_seed:999,partial:true}});
  send({type:'complete',result:{initial_seed:999,partial:false}});controller.close();await stale;
  assert.equal(rendered.length,count,'Ignore stale progress and completion');
  honorAbort=true;
  run("recordVoice('S')");
  await reply({count:1,observations:3,candidate:{current_state:40}});
  const old=pending[0];assert(old.url.startsWith('/api/identified-route?'));
  run("recordVoice('N')");await flush();
  assert(old.signal.aborted,'Extra input must abort the in-flight route');
  assert(pending[0].url.startsWith('/api/identify-observed?'),'Latest identification starts without waiting for old route');
  await reply({count:0,observations:4});
  assert.equal(run('running'),false);
  console.log('PASS: extra inputs, undo, queued latest transcript, stale success/error suppression');
  console.log('PASS: immediate partial rendering, retained results on failure, stale stream suppression');
})().catch(e=>{console.error(e);process.exitCode=1;});

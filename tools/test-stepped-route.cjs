'use strict';
const fs=require('node:fs'),vm=require('node:vm'),assert=require('node:assert/strict'),path=require('node:path');
const voice=fs.readFileSync(path.join(__dirname,'../viewer/voice.js'),'utf8');
const parser=voice.slice(voice.indexOf('async function readRouteStream('),voice.indexOf("$('voice-normal').addEventListener"));
const context=vm.createContext({TextDecoder});vm.runInContext(parser,context);
const read=context.readRouteStream;
(async()=>{
 let controller;const body=new ReadableStream({start(c){controller=c;}}),events=[];
 const pending=read({body},e=>events.push(e));
 const bytes=new TextEncoder().encode(JSON.stringify({type:'progress',message:'EXP ×100'})+'\n');
 for(const byte of bytes)controller.enqueue(Uint8Array.of(byte));
 await new Promise(setImmediate);
 assert.equal(events.length,1,'Progress must be delivered before stream closes');
 assert.equal(events[0].message,'EXP ×100');
 controller.enqueue(new TextEncoder().encode('{"type":"complete"}\n'));controller.close();await pending;
 await assert.rejects(read(new Response('{"type":"progress"}\n'),()=>{}),/Connection ended/);
 await assert.rejects(read(new Response('{"type":"error","message":"search failed"}\n'),()=>{}),/search failed/);
 console.log('PASS: incremental UTF-8 chunks, early delivery, truncation and error handling');
 if(!process.argv[2])return;
 const start=Date.now(),query=new URLSearchParams({route:'galdera',voices:'SSSSSSNSNSNSNSNSNSNSNSNSNNNN',r3_first:'pirro',hp:'154',stream:'1'});
 const response=await fetch(process.argv[2]+'/api/identified-route?'+query);assert(response.ok);
 const snapshots=[];
 await read(response,e=>{
   assert.notEqual(e.type,'error',e.message);snapshots.push(e);
   fs.writeFileSync(path.join(__dirname,`../target/stepped-${snapshots.length}.json`),JSON.stringify(e.result));
   console.log(`${((Date.now()-start)/1000).toFixed(1)}s ${e.type}: ${e.message}`);
 });
 assert.equal(snapshots.length,4);assert.equal(snapshots[3].type,'complete');
 // Everything published earlier must be unchanged in the final plan.
 function subset(a,b){
   if(a===null)return;
   if(typeof a!=='object'){assert.deepEqual(a,b);return;}
   for(const [key,value]of Object.entries(a)){if(key!=='partial')subset(value,b[key]);}
 }
 for(const s of snapshots.slice(0,-1))subset(s.result,snapshots[3].result);
 console.log('PASS: every committed prefix preserved exactly through final JP selection');
})().catch(e=>{console.error(e);process.exitCode=1;});

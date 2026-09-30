'use strict';
// Small trace excerpts, retaining the repeat capture's state discontinuity.
const fs=require('node:fs'), path=require('node:path'), crypto=require('node:crypto');
const dir=path.resolve(__dirname,'../../OT2BattleEngineRust/data/source');
const captures=[];
for(const [name,start,count,advances,recreate] of [
  ['live-throne-title-to-pirro-20260926-trace.json',0,13,4916,true],
  ['live-throne-repeat-20260926-trace.json',202,12,4917,false],
]) {
  const b=fs.readFileSync(path.join(dir,name)), trace=JSON.parse(b);
  captures.push({source:'OT2BattleEngineRust/data/source/'+name,sha256:crypto.createHash('sha256').update(b).digest('hex'),start_record:start,title_advances:advances,recreate_shared_level:recreate,
    records:trace.records.slice(start,start+count).map(r=>({before:r.before,return_rva:r.returnRva}))});
}
fs.writeFileSync(path.resolve(__dirname,'../data/source/title-transitions.json'),JSON.stringify({
  route:'Title -> New Game as Throné -> skip both cutscenes -> battle initialization',captures,
  limits:['These offsets model the captured skipped route, not arbitrary watching, menu activity or reseeds.',
    'The repeat had an untraced state change from its earlier title marker (0xEA4F203D) to 0x5D3D7B67. Its 12-call interval starts from that latter state, at advance 4917. The model does not insert or explain that unrecorded state change.',
    'Shared Btl_SinExpose attribution comes from native loading conditions; trace callers establish six versus five streamed-level draws.'],
  documentation:'OT2BattleEngineRust/docs/opening-cutscenes.md'
},null,2)+'\n');

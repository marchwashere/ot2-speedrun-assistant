'use strict';
const fs=require('node:fs'),path=require('node:path'),assert=require('node:assert/strict'),s=require('./source.cjs');
const {blueprint}=require('../../OT2BattleEngineRust/tools/audit-pirro.cjs');
const sources=[];function table(n){const t=s.table(n);sources.push({source:t.source,sha256:t.sha256});return t.rows;}
const quest=table('Story/Database/SubStoryTask').find(r=>r.id==='SS_TUT_003');
assert.equal(quest.RewardMoney,1500);assert.deepEqual(quest.RewardParam,['ITM_CSM_0680','1','ITM_CSM_0010','3','None','None']);
const al=table('Placement/Database/PlacementData').find(r=>r.id==='NPC_SS_TCity13_Tutorial_0100');
assert.equal(al.EventLabel_A,'SC_FC13_Tutorial_0400');assert.equal(al.EventStartFlag_A,20510);assert.equal(al.SpawnEndFlag,20511);
const event=table('Event/Database/EventList').find(r=>r.id===al.EventLabel_A);assert.equal(event.MapID,'46');assert(event.EncountReset);
const world=JSON.parse(fs.readFileSync(path.join(__dirname,'../data/town-loading.json')));
const grant=world.events.SC_FC13_Tutorial_0310.commands[29];assert.equal(grant.cmd,2000);assert.deepEqual(grant.opt,[al.EventParam_A_1,'1']);
const delivery=world.events[event.id].commands[4];assert.equal(delivery.cmd,5500);assert.deepEqual(delivery.opt,['4','57']);
const bp=blueprint('Event/BP/EventManagerBP',['UpdateSubDeliveryDialog','CreateEventCharacter','GetSpawnNPCAnimationLabel']);assert(bp.functions.UpdateSubDeliveryDialog.includes('SubDeliveryItem'));
sources.push({source:'/Game/Event/BP/EventManagerBP',sha256:bp.sha256});
const runtime={event:event.id,npc:al.ResourceLabel,bag:al.EventParam_A_1,reward_money:quest.RewardMoney,rewards:[{item:quest.RewardParam[0],quantity:Number(quest.RewardParam[1])},{item:quest.RewardParam[2],quantity:Number(quest.RewardParam[3])}]};
fs.writeFileSync(path.join(__dirname,'../data/galdera-al-return.json'),JSON.stringify(runtime,null,2)+'\n');
fs.writeFileSync(path.join(__dirname,'../data/source/galdera-al-return.json'),JSON.stringify({sources,quest,al,event,grant,delivery,blueprint:bp},null,2)+'\n');
console.log(JSON.stringify(runtime));

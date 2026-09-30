'use strict';
const fs=require('fs'),path=require('path'),assert=require('node:assert/strict');
const s=require('./source.cjs'),{blueprint}=require('../../OT2BattleEngineRust/tools/audit-pirro.cjs');
const sources=[];
function table(name){const t=s.table(name);sources.push({source:t.source,sha256:t.sha256});return t.rows;}
const npc=table('Character/Database/NPCData').find(r=>r.id==='NPC_Twn_Snw_1_2_A_TALK_1100');
const purchase=table('Character/Database/NPCPurchaseData').find(r=>r.id===npc.FCmd_Purchase_ID);
assert(purchase);assert.equal(purchase.RobBeforeEventID,'None');assert.equal(purchase.RobEnemyGroupID,'None');
const fc=table('FieldCommand/Database/FieldCommandData').find(r=>r.id==='FIELD_COMMAND_ROB');
assert.deepEqual(fc.BranchEventWeight,[.5,.5]);
const events=table('Event/Database/EventList'),scripts=[];
for(const id of [...fc.BeforeEventLabel,fc.SuccessEventLabel[0]]){
 const event=events.find(r=>r.id===id);assert(event&&!event.EncountReset&&event.MapID==='None');
 const bytes=fs.readFileSync(path.join(s.assets.CONTENT,'Event/json',id));
 const commands=Object.values(JSON.parse(bytes));assert(commands.every(c=>[4000,42,10000].includes(c.cmd)));
 sources.push({source:'/Game/Event/json/'+id,sha256:s.assets.hash(bytes)});scripts.push({event,commands});
}
const bps=[
 blueprint('FieldCommand/BPv2/Flow/BP_FCv2_FieldCommandFlow_Rob',['UpdateFlow','StartBattle','PostProcess','OnFinishFieldCommand']),
 blueprint('FieldCommand/BPv2/Flow/BP_FCv2_FieldCommandFlowBase',['DrawEventLabel','PostProcess','OnFinishFieldCommand']),
 blueprint('Battle/BP/EncountManagerBP',['StartRobBattle']),
 blueprint('Battle/BP/BattleManagerBP',['Start','ExecSpecialAction_Preparation','PlayVictoryVoice','LotteryDropItem']),
 blueprint('Battle/BP/BattleEnemyBP',['InitializeEnemy','OpenWeakIcon']),
 blueprint('Character/BP/BattleAI/NPC/BattleAI_NPC_Pattern',['UpdateThink']),
 blueprint('Battle/BP/BattleCharacterBaseBP',['GetDEFDiseaseRatio','GetMDEFDiseaseRatio']),
];
assert(bps[0].functions.UpdateFlow.includes('LocalVirtualFunction DrawEventLabel'));
assert(bps[1].functions.DrawEventLabel.includes('RandomFloatInRange'));
const config={npc:npc.id,night_placement:'NPC_Twn_Snw_1_2_A_TALK_1100_N000',enemy_group:npc.FCmd_EnemyGroup,before_events:fc.BeforeEventLabel,before_weights:fc.BranchEventWeight,success_event:fc.SuccessEventLabel[0]};
fs.writeFileSync(path.join(__dirname,'../data/cape_cold_mug.json'),JSON.stringify(config,null,2)+'\n');
fs.writeFileSync(path.join(__dirname,'../data/source/capecold-mug-audit.json'),JSON.stringify({sources,npc,purchase,fc,scripts,bps},null,2)+'\n');
console.log(config);

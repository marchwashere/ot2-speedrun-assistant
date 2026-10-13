'use strict';
const fs=require('fs'),path=require('path'),assert=require('assert/strict'),s=require('./source.cjs');
const {blueprint}=require('../../OT2BattleEngineRust/tools/audit-pirro.cjs');
const sources=[];function table(n){const t=s.table(n);sources.push({source:t.source,sha256:t.sha256});return t.rows;}
const npcs=table('Character/Database/NPCData'),purchase=table('Character/Database/NPCPurchaseData'),items=table('Shop/Database/PurchaseItemTable');
const soldier=npcs.find(r=>r.id==='NPC_Twn_Snw_2_1_A_TALK_1500');assert(soldier);
assert.equal(purchase.find(r=>r.id===soldier.FCmd_Purchase_ID).BegSuccessTalkID,'None');
const nut=items.find(r=>r.id==='NPC_Twn_Snw_2_1_A_TALK_1500_NPCBUY_03');assert.equal(nut.ItemLabel,'ITM_CSM_0550');assert.equal(nut.ProperBeg,26);
const quest=table('Story/Database/SubStoryTask').find(r=>r.id==='SS_SNW2_0200');assert(quest);
assert.deepEqual(quest.RewardParam,['ITM_TRE_WPM_01','1','ITM_CSM_0110','1','None','None']);assert.equal(quest.RewardMoney,7000);
const placements=table('Placement/Database/PlacementData');const woman=placements.find(r=>r.id==='NPC_SS_TSnow21_0200_0100');assert(woman);
assert.equal(woman.EventType_A,'ePARTY_MEMBER');assert.equal(woman.EventParam_A_1,'8');assert.equal(woman.EventType_B,'eDELIVERY_ITEM');assert.equal(woman.EventLabel_B,'SC_SS_TSnow21_0200_02A0');
const p=path.join(__dirname,'../data/town-loading.json'),world=JSON.parse(fs.readFileSync(p)),events=table('Event/Database/EventList');
const name='SC_SS_TSnow21_0200_02A0',row=events.find(r=>r.id===name);assert.equal(row.MapID,'170');assert(row.EncountReset);
const raw=fs.readFileSync(path.join(s.assets.CONTENT,'Event/json',name));sources.push({source:'/Game/Event/json/'+name,sha256:s.assets.hash(raw)});
const cs=Object.entries(JSON.parse(raw)).map(([index,c])=>({index:Number(index),cmd:c.cmd,target:c.target,text:c.text,opt:c.opt}));
assert.equal(cs[8].cmd,5500);assert.deepEqual(cs[8].opt,['8','71']);assert.equal(cs[68].cmd,9990);assert.equal(cs[70].cmd,1310);
world.events[name]={map_id:170,reset_counter:true,completion:null,commands:cs};fs.writeFileSync(p,JSON.stringify(world,null,2)+'\n');
const bp=[];for(const [a,names]of [
 ['Event/BP/EventManagerBP',['UpdateSubDeliveryDialog','SpawnCharacter','CreateEventCharacter']],
 ['Event/BP/EventFunctionBP',['IsCheckPartyMemberNum']],
 ['Character/BP/KSCharacterManagerBP',['SpawnChara','AddEventCharacter']],
 ]){const b=blueprint(a,names);bp.push(b);sources.push({source:'/Game/'+a,sha256:b.sha256});}
assert(bp[0].functions.UpdateSubDeliveryDialog.includes('SubDeliveryItem'));
const runtime={event:name,quest_npc:woman.ResourceLabel,pickaxe:woman.EventParam_B_1,sword:quest.RewardParam[0],jam:quest.RewardParam[2],reward_money:quest.RewardMoney,
 soldier:{npc:soldier.id,item:nut.ItemLabel,required_level:nut.ProperBeg}};
fs.writeFileSync(path.join(__dirname,'../data/galdera-winterbloom.json'),JSON.stringify(runtime,null,2)+'\n');
fs.writeFileSync(path.join(__dirname,'../data/source/galdera-winterbloom.json'),JSON.stringify({sources,quest,woman,soldier,nut,blueprints:bp},null,2)+'\n');console.log(JSON.stringify(runtime));

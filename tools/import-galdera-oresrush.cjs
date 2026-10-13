'use strict';
const fs=require('fs'),path=require('path'),assert=require('assert/strict'),s=require('./source.cjs');
const {blueprint}=require('../../OT2BattleEngineRust/tools/audit-pirro.cjs');
const sources=[];
function table(name){const t=s.table(name);sources.push({source:t.source,sha256:t.sha256});return t.rows;}
function one(rows,id){const r=rows.find(r=>r.id===id);assert(r,id);return r;}
const npcs=table('Character/Database/NPCData'),leads=table('Character/Database/NPCLeadData'),purchases=table('Character/Database/NPCPurchaseData');
const items=table('Shop/Database/PurchaseItemTable'),params=table('GameParam/GameParamDefineTable'),commands=table('FieldCommand/Database/FieldCommandData');
const guild=one(table('Shop/Database/GuildData'),'GUILD_000');
assert.equal(guild.JobLicenseData[0].NeedMoney,0);assert.equal(guild.JobLicenseData[0].NeedItemLabel,'None');
const peddler=one(npcs,'NPC_SS_TW11_0100_0100'),lead=one(leads,peddler.FCmd_Lure_ID);
assert.equal(lead.SpActionMerchant,'SPACT_MERCHANT_014');assert.equal(lead.HirePrice,1600);assert(lead.CommandEnable&4);
assert(lead.FollowingFlags.every(x=>x===0)&&lead.UnFollowingFlags.every(x=>x===0));assert.equal(lead.HireSuccesTalkID,'None');
const miner=one(npcs,'NPC_SS_TSn21_0200_0200'),purchase=one(purchases,miner.FCmd_Purchase_ID);
assert.equal(purchase.BegSuccessTalkID,'None');assert(purchase.CommandEnable&4);
const pickaxe=one(items,'NPC_SS_TSn21_0200_0200_NPCBUY_01');assert.equal(pickaxe.ProperBeg,30);
const augmentorNpc=one(npcs,'NPC_Twn_Dst_2_1_B_TALK_1800');
const augmentorPurchase=one(purchases,augmentorNpc.FCmd_Purchase_ID);assert.equal(augmentorPurchase.BegSuccessTalkID,'None');
const augmentorItem=one(items,'NPC_Twn_Dst_2_1_B_TALK_1800_NPCBUY_02');assert.equal(augmentorItem.ItemLabel,'ITM_EQP_ACS_250');assert.equal(augmentorItem.ProperBeg,25);
const over=Number(one(params,'STEAL_PROBABILITY_LEVEL_OVER').Init);assert.equal(over,11);
const jamNpc=one(npcs,'NPC_Twn_Dst_2_1_B_TALK_0400');
const jamPurchase=one(purchases,jamNpc.FCmd_Purchase_ID);
const jamItem=one(items,jamPurchase.ShopID+'_02');
assert.equal(jamItem.ItemLabel,'ITM_CSM_0101');assert.equal(jamItem.ProperSteal,35);
assert.equal(jamPurchase.StealSuccessTalkID,'None');assert(jamPurchase.CommandEnable&1);
assert.equal(Number(one(params,'STEAL_PROBABILITY_OVER').Init),100);
const hire=one(commands,'FIELD_COMMAND_HIRE'),steal=one(commands,'FIELD_COMMAND_STEAL');
assert.deepEqual(hire.BranchEventWeight,[.5,.5]);assert.deepEqual(steal.BranchEventWeight,[.5,.5]);
const ep=path.join(__dirname,'../data/town-loading.json'),world=JSON.parse(fs.readFileSync(ep));
const events=table('Event/Database/EventList');
for(const name of [guild.FistTaskEvent,guild.CompleteEvent[0],...hire.SuccessEventLabel,...steal.SuccessEventLabel]){
 const row=one(events,name),raw=fs.readFileSync(path.join(s.assets.CONTENT,'Event/json',name));
 assert.equal(row.EncountReset,false);assert.equal(row.MapID,'None');
 sources.push({source:'/Game/Event/json/'+name,sha256:s.assets.hash(raw)});
 const cs=Object.entries(JSON.parse(raw)).map(([index,c])=>({index:Number(index),cmd:c.cmd,target:c.target,text:c.text,opt:c.opt}));
 assert(cs.every(c=>[40,42,70,10000].includes(c.cmd)));
 world.events[name]={map_id:0,reset_counter:false,completion:null,commands:cs};
}
fs.writeFileSync(ep,JSON.stringify(world,null,2)+'\n');
const bp=[];
for(const [asset,names]of [
 ['FieldCommand/BPv2/FCv2_FieldCommandManager',['UpdatePreSequence']],
 ['FieldCommand/BPv2/Flow/BP_FCv2_FieldCommandFlowBase',['DrawEventLabel','Follow Npc']],
 ['FieldCommand/BPv2/Flow/BP_FCv2_FieldCommandFlow_Hire',['UpdateFlow']],
 ['FieldCommand/BPv2/Flow/BP_FCv2_FieldCommandFlow_Steal',['UpdateFlow']],
 ['FieldCommand/BP/LibFieldCommand',['SetupFollowNPC']],
 ['Character/BP/KSCharacterManagerBP',['RefreshFollowCharacter']],
 ['UserInterface/FieldCommand/BP/FieldCommandWisgetBeg',['BegItemExec']],
 ['FieldCommand/BPv2/Flow/BP_FCv2_FieldCommandFlow_Beg',['UpdateFlow']],
 ]){
 // Manager function name differs across cooked builds; inspect exports below.
 const pkg=s.assets.loadPackage(path.join(s.assets.CONTENT,asset+'.uasset'));
 const available=pkg.exports.filter(e=>s.assets.objectPath(pkg,e.classIndex)==='/Script/CoreUObject.Function').map(e=>e.name);
 const selected=names.filter(n=>available.includes(n));
 if(asset.endsWith('FCv2_FieldCommandManager')&&!selected.length)selected.push(...available.filter(n=>/PreSequence/.test(n)));
 assert(selected.length,asset);const b=blueprint(asset,selected);bp.push(b);sources.push({source:'/Game/'+asset,sha256:b.sha256});
}
assert(bp.some(b=>Object.values(b.functions).some(v=>v.includes('K2_SetTimerDelegate'))));
assert(bp.some(b=>Object.values(b.functions).some(v=>v.includes('RandomFloatInRange'))));
const begCode=bp.flatMap(b=>Object.entries(b.functions)).find(([n])=>n==='BegItemExec')[1];
assert(!/Random|K2_SetTimer/.test(begCode));assert(begCode.includes('AddBackpackItem'));
const runtime={warrior_job:2,guild_flag:411,peddler:{npc:peddler.id,unique_id:peddler.NPC_UnqIndex,supporter:lead.SupporterID,talent:lead.SpActionMerchant,price:lead.HirePrice,events:hire.SuccessEventLabel},
 pickaxe:{npc:miner.id,item:pickaxe.ItemLabel,required_level:pickaxe.ProperBeg},
 augmentor:{npc:augmentorNpc.id,item:augmentorItem.ItemLabel,required_level:augmentorItem.ProperBeg}};
runtime.jam={npc:jamNpc.id,item:jamItem.ItemLabel,proper_steal:jamItem.ProperSteal,guaranteed_level:jamItem.ProperSteal+over};
fs.writeFileSync(path.join(__dirname,'../data/galdera-oresrush.json'),JSON.stringify(runtime,null,2)+'\n');
fs.writeFileSync(path.join(__dirname,'../data/source/galdera-oresrush.json'),JSON.stringify({sources,guild,lead,peddler,miner,purchase,pickaxe,jamNpc,jamPurchase,jamItem,runtime,blueprints:bp},null,2)+'\n');
console.log(JSON.stringify(runtime));

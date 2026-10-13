'use strict';
const fs=require('fs'),path=require('path'),assert=require('assert/strict'),s=require('./source.cjs');
const sources=[];function table(n){const t=s.table(n);sources.push({source:t.source,sha256:t.sha256});return t.rows;}
const npcs=table('Character/Database/NPCData'),purchase=table('Character/Database/NPCPurchaseData'),items=table('Shop/Database/PurchaseItemTable'),placements=table('Placement/Database/PlacementData');
const one=(rs,id)=>{const r=rs.find(x=>x.id===id);assert(r,id);return r;};
const steals={};
for(const [key,id,labels]of [['gravell','NPC_Twn_Wld_3_1_A_TALK_0100',['ITM_CSM_0070','ITM_MRL_COM_0020']],['gravell_lychee','NPC_Twn_Wld_3_1_A_TALK_0200',['ITM_CSM_0240']],['mason','NPC_Twn_Wld_2_1_A_TALK_1100',['ITM_MRL_REV_0040']]]){
 const npc=one(npcs,id),p=one(purchase,npc.FCmd_Purchase_ID);assert.equal(p.StealSuccessTalkID,'None');
 steals[key]={npc:id,items:labels.map(label=>{const r=items.find(x=>x.id.startsWith(p.ShopID+'_')&&x.ItemLabel===label);assert(r);return {id:label,level:r.ProperSteal+11};})};
}
const scholar=one(npcs,'NPC_Twn_Wld_2_1_A_TALK_1900'),battle=one(table('Character/Database/NPCBattleData'),scholar.FCmd_Battle_ID);
const ambush=one(table('FieldCommand/Database/FieldCommandData'),'FIELD_COMMAND_ASSASSINATE');
const chests=['Treasure_SS_TMnt21_0200_010','Treasure_Fld_Dst_2_2_02','Treasure_Fld_Dst_3_2_04'].map(id=>one(table('Environment/Database/ObjectData'),id));
for(const c of chests){assert(!c.IsMoney);assert(c.RandomItemCandidates.every(x=>x.ItemLabel==='None'));}
const boat=one(npcs,'NPC_SHO_EX1_0120'),boat_purchase=one(purchase,boat.FCmd_Purchase_ID);
const events=table('Event/Database/EventList'),world=JSON.parse(fs.readFileSync(path.join(__dirname,'../data/town-loading.json'))),scripts={};
for(const n of ['0100','0110','0200','0400','0410','0500','0600','0700'].map(x=>'MS_SHO_EX1_'+x)){
 const e=one(events,n),raw=fs.readFileSync(path.join(s.assets.CONTENT,'Event/json',n));sources.push({source:'/Game/Event/json/'+n,sha256:s.assets.hash(raw)});
 const commands=Object.entries(JSON.parse(raw)).map(([i,c])=>({index:Number(i),cmd:c.cmd,target:c.target,text:c.text,opt:c.opt}));scripts[n]=commands;
 world.events[n]={map_id:Number(e.MapID),reset_counter:e.EncountReset,completion:null,commands};
}
const data={steals,scholar:scholar.id,scholar_placement:placements.find(r=>r.ResourceLabel===scholar.id&&r.TimeZoneSpawnType.endsWith('ALL'))?.id||placements.find(r=>r.ResourceLabel===scholar.id).id,ambush_level:battle.AssassinateNeedLevel,ambush_before:ambush.BeforeEventLabel,ambush_success:ambush.SuccessEventLabel[0],chests:chests.map(c=>({id:c.id,item:c.HaveItemLabel,count:c.HaveItemCnt})),boat_npc:boat.id,boat_purchase};
fs.writeFileSync(path.join(__dirname,'../data/galdera-flex.json'),JSON.stringify(data,null,2)+'\n');
fs.writeFileSync(path.join(__dirname,'../data/source/galdera-flex.json'),JSON.stringify({sources,steals,scholar,battle,chests,boat,boat_purchase,scripts,placements:placements.filter(r=>[scholar.id,boat.id,...Object.values(steals).map(x=>x.npc)].includes(r.ResourceLabel)||chests.some(c=>c.id===r.id))},null,2)+'\n');
fs.writeFileSync(path.join(__dirname,'../data/town-loading.json'),JSON.stringify(world,null,2)+'\n');
console.log(JSON.stringify(data));

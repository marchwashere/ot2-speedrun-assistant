'use strict';
const fs=require('node:fs'),path=require('node:path'),assert=require('node:assert/strict'),s=require('./source.cjs');
const sources=[];function table(n){const t=s.table(n);sources.push({source:t.source,sha256:t.sha256});return t.rows;}
const npcs=table('Character/Database/NPCData'),purchases=table('Character/Database/NPCPurchaseData'),items=table('Shop/Database/PurchaseItemTable');
const shopkeeper=npcs.find(r=>r.id==='NPC_Twn_Fst_2_1_A_TALK_0400'),mother=npcs.find(r=>r.id==='NPC_SS_TF21_0200_0300');assert(shopkeeper&&mother);
const shopPurchase=purchases.find(r=>r.id===shopkeeper.FCmd_Purchase_ID),motherPurchase=purchases.find(r=>r.id===mother.FCmd_Purchase_ID);
assert.equal(shopPurchase.BegSuccessTalkID,'None');assert.equal(shopPurchase.StealSuccessTalkID,'None');assert.equal(motherPurchase.BegSuccessTalkID,'None');assert.equal(motherPurchase.StealSuccessTalkID,'None');
const sharp=items.find(r=>r.id===shopPurchase.ShopID+'_02'),serum=items.find(r=>r.id===shopPurchase.ShopID+'_03'),lychee=items.find(r=>r.id===motherPurchase.ShopID+'_02');
const magicMerchant=npcs.find(r=>r.id==='NPC_Twn_Fst_2_1_A_TALK_1800');
const magicPurchase=purchases.find(r=>r.id===magicMerchant.FCmd_Purchase_ID);
const magic=items.find(r=>r.id.startsWith(magicPurchase.ShopID+'_')&&r.ItemLabel==='ITM_CSM_0610');
assert.equal(magic.ProperBeg,20);assert.equal(magicPurchase.BegSuccessTalkID,'None');
const params=table('GameParam/GameParamDefineTable');
const over=Number(params.find(r=>r.id==='STEAL_PROBABILITY_LEVEL_OVER').Init);assert.equal(over,11);
assert.equal(Number(params.find(r=>r.id==='STEAL_PROBABILITY_OVER').Init),100);
assert.equal(sharp.ItemLabel,'ITM_CSM_0670');assert.equal(serum.ItemLabel,'ITM_MRL_COM_0010');assert.equal(lychee.ItemLabel,'ITM_CSM_0230');
assert.equal(sharp.ProperBeg,26);assert.equal(serum.ProperBeg,26);assert.equal(lychee.ProperBeg,20);
const guild=table('Shop/Database/GuildData').find(r=>r.id==='GUILD_007');assert.equal(guild.LicenseItem,'ITM_EQP_JOB_0005');
assert.equal(guild.JobLicenseData[0].NeedMoney,0);assert.equal(guild.JobLicenseData[0].NeedItemLabel,'None');
const job=table('Character/Database/JobData').find(r=>r.id==='eDANCER');assert.equal(job.JobID,5);
const eventRows=table('Event/Database/EventList'),worldPath=path.join(__dirname,'../data/town-loading.json'),world=JSON.parse(fs.readFileSync(worldPath));
for(const name of [guild.FistTaskEvent,guild.CompleteEvent[0]]){
 const r=eventRows.find(r=>r.id===name);assert.equal(r.MapID,'None');assert.equal(r.EncountReset,false);
 const raw=fs.readFileSync(path.join(s.assets.CONTENT,'Event/json',name));sources.push({source:'/Game/Event/json/'+name,sha256:s.assets.hash(raw)});
 const commands=Object.entries(JSON.parse(raw)).map(([index,c])=>({index:Number(index),cmd:c.cmd,target:c.target,text:c.text,opt:c.opt}));
 assert(commands.every(c=>[40,70,10000].includes(c.cmd)));
 world.events[name]={map_id:0,reset_counter:false,completion:null,commands};
}
fs.writeFileSync(worldPath,JSON.stringify(world,null,2)+'\n');
assert.equal(lychee.ProperSteal,12);
const runtime={shopkeeper:{npc:shopkeeper.id,items:[sharp,serum].map((r,i)=>({item:r.ItemLabel,name:['Sharp Nut (L)','Diffusing Serum'][i],proper_steal:r.ProperSteal,guaranteed_level:r.ProperSteal+over}))},dancer_job:job.JobID,guild_resource:'SYS_GUILD_DAN',guild_events:[guild.FistTaskEvent,guild.CompleteEvent[0]]};
runtime.magic_merchant={npc:magicMerchant.id,item:magic.ItemLabel,required_level:magic.ProperBeg};
fs.writeFileSync(path.join(__dirname,'../data/galdera-wellgrove-supplies.json'),JSON.stringify(runtime,null,2)+'\n');
fs.writeFileSync(path.join(__dirname,'../data/source/galdera-wellgrove-supplies.json'),JSON.stringify({sources,shopkeeper,mother,shopPurchase,motherPurchase,sharp,serum,lychee,magicMerchant,magicPurchase,magic,guild,job},null,2)+'\n');
console.log(JSON.stringify(runtime));

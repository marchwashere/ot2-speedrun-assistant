'use strict';
const fs=require('fs'),path=require('path'),assert=require('assert/strict'),s=require('./source.cjs');
const {blueprint}=require('../../OT2BattleEngineRust/tools/audit-pirro.cjs');
const sources=[];function table(n){const t=s.table(n);sources.push({source:t.source,sha256:t.sha256});return t.rows;}
const one=(rs,id)=>{const r=rs.find(x=>x.id===id);assert(r,id);return r;};
const npcs=table('Character/Database/NPCData'),battles=table('Character/Database/NPCBattleData'),purchases=table('Character/Database/NPCPurchaseData');
const guard=one(npcs,'NPC_Twn_Wld_3_1_A_TALK_1600'),battle=one(battles,guard.FCmd_Battle_ID);
assert.equal(battle.AssassinateNeedLevel,30);assert(battle.CanSwoon);assert(battle.SwooningFlags.every(x=>x===0));assert.equal(battle.SwoonRecTimeZoneChange,false);
const shopkeeper=one(npcs,'NPC_Twn_Wld_3_1_A_TALK_0500'),purchase=one(purchases,shopkeeper.FCmd_Purchase_ID);
assert.equal(purchase.StealSuccessTalkID,'None');
const jam=one(table('Shop/Database/PurchaseItemTable'),purchase.ShopID+'_02');assert.equal(jam.ItemLabel,'ITM_CSM_0101');assert.equal(jam.ProperSteal,35);
const params=table('GameParam/GameParamDefineTable'),over=Number(one(params,'STEAL_PROBABILITY_LEVEL_OVER').Init);assert.equal(over,11);
const talent=one(table('Character/Database/SPActionMerchantDataDB'),'SPACT_MERCHANT_014');
assert.equal(talent.SPEffectType,'ESPACT_MERCHANT_TYPE::SPACT_MER_SELL_ITEM_HIGH');assert.deepEqual(talent.EffectIntParam.slice(0,2),[25,25]);
const commands=table('FieldCommand/Database/FieldCommandData'),ambush=one(commands,'FIELD_COMMAND_ASSASSINATE');assert.deepEqual(ambush.BranchEventWeight,[.5,.5]);
const itemDb=table('Item/Database/ItemDB'),special=table('Item/Database/SpecialItemDB');
const flagMap=require('../../OT2BattleEngine/tools/enums.cjs').load('/Script/Majesty.EKSBitFlag');
const flag=n=>{const f=flagMap.get('EKSBitFlag::'+n);assert(Number.isInteger(f));return f;};
const weapons=['01','04'].map((suffix,i)=>{const rust=one(itemDb,'ITM_TRE_WPM_'+suffix),sp=one(special,rust.SpecialItemLabel),weapon=one(itemDb,sp.RelativeItemLabel);return {rust:rust.id,item:weapon.id,sort:rust.BackpackPriority,sell_price:weapon.SellPrice,flag:flag('WEAPONMASTER_WEAPON_'+['SWD','AXE'][i]+'_FIRST_GET')};});
assert(weapons.every(w=>w.sell_price===100000));
const job=one(table('Character/Database/JobData'),'eWEAPON_MASTER');assert.equal(job.JobID,8);
const abilities=table('Ability/Database/AbilitySetData');
for(const w of weapons){const a=abilities.find(r=>JSON.stringify(r).includes(w.item));assert(a,'weapon ability '+w.item);w.ability=a.id;}
const placements=table('Placement/Database/PlacementData').filter(r=>[guard.id,shopkeeper.id,'SYS_GUILD_WPM'].includes(r.ResourceLabel));
assert(placements.filter(r=>r.ResourceLabel===guard.id).every(r=>!Object.entries(r).some(([k,v])=>k.startsWith('EventType_')&&/ASSASSINATE/.test(v))));
const eventRows=table('Event/Database/EventList'),wp=path.join(__dirname,'../data/town-loading.json'),world=JSON.parse(fs.readFileSync(wp));
const scripts={};
for(const name of ['SYS_WPM_GUILD_0010',...ambush.BeforeEventLabel,ambush.SuccessEventLabel[0]]){
 const row=one(eventRows,name);assert.equal(row.MapID,'None');assert.equal(row.EncountReset,false);
 const raw=fs.readFileSync(path.join(s.assets.CONTENT,'Event/json',name));sources.push({source:'/Game/Event/json/'+name,sha256:s.assets.hash(raw)});
 const cs=Object.entries(JSON.parse(raw)).map(([index,c])=>({index:Number(index),cmd:c.cmd,target:c.target,text:c.text,opt:c.opt}));
 scripts[name]=cs;world.events[name]={map_id:0,reset_counter:false,completion:null,commands:cs};
}
const bps=[];for(const [asset,names]of [
 ['ExtraJobEvent/WeaponMasterEvent/BP/BP_WeaponMasterEventFlowV2',['UpdateFlow','ApplyResult','OnClosedJobDialog','OnClosedAbilityDialog','OnClosedSupportAbilityDialog']],
 ['ExtraJobEvent/WeaponMasterEvent/BP/BP_WeaponMasterUtil',['ConvertRustWeaponToLegendWeapon','AcquisitionWeaponMasterJob','AcquisitionAbility','TryAcquisitionSupportAbility']],
 ['FieldCommand/BPv2/Flow/BP_FCv2_FieldCommandFlow_Assassinate',['UpdateFlow']],
 ['FieldCommand/BP/LibFieldCommand',['Swoon NPC']],
 ['UserInterface/Shop/BP/ShopMenuWidget',['CompleteTrade']],
 ]){const b=blueprint(asset,names);bps.push(b);sources.push({source:'/Game/'+asset,sha256:b.sha256});}
const serumStock=one(table('Shop/Database/PurchaseItemTable'),'NPC_Twn_Wld_3_1_A_SHOP02_08');
assert.equal(serumStock.ItemLabel,'ITM_MRL_COM_0010');assert.equal(serumStock.PossibleItemLabel,'None');
const serumItem=one(itemDb,serumStock.ItemLabel);assert.equal(serumItem.BuyPrice,3000);
const runtime={guard:guard.id,ambush_level:battle.AssassinateNeedLevel,before_events:ambush.BeforeEventLabel,success_event:ambush.SuccessEventLabel[0],
 shopkeeper:shopkeeper.id,jam:jam.ItemLabel,proper_steal:jam.ProperSteal,steal_level:jam.ProperSteal+over,weapons,job,
 first_talk_flag:flag('WEAPONMASTER_NPC_FIRST_TALK'),license_flag:flag('WEAPONMASTER_FIRST_JOB_LICENSE_GET')};
fs.writeFileSync(wp,JSON.stringify(world,null,2)+'\n');
fs.writeFileSync(path.join(__dirname,'../data/galdera-gravell.json'),JSON.stringify(runtime,null,2)+'\n');
fs.writeFileSync(path.join(__dirname,'../data/source/galdera-gravell.json'),JSON.stringify({sources,serumStock,serumItem,guard,battle,shopkeeper,purchase,jam,talent,placements,scripts,blueprints:bps,
 native_sales:'ShopMenuWidget.CompleteTrade -> ExecMerchantSpecialAction_WithoutNpc type1 -> 140e78f00 -> 140e78a30: SPACT_MERCHANT_014 sell-high range25..25 consumes one rand per sale even though range is constant; ceil(base*25/100) added. SetTimerForNextTick in Armsmaster notifications does not use K2_SetTimerDelegate variance lottery.'},null,2)+'\n');
console.log(JSON.stringify({ambush:runtime.ambush_level,steal:runtime.steal_level,weapons}));

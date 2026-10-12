'use strict';
const fs=require('fs'),path=require('path'),assert=require('assert/strict'),s=require('./source.cjs');
const {initialization}=require('../../OT2BattleEngineRust/tools/enemy-initialization.cjs');
const root=path.join(__dirname,'../data'),p=path.join(root,'town-loading.json'),data=JSON.parse(fs.readFileSync(p));
const events=s.table('Event/Database/EventList');
const names=['SC_SP_Tutorial_0200','SC_SP_Tutorial_0300','SC_FC13_Tutorial_0310','SC_FC13_Tutorial_0400'];
for(const name of names){const row=events.rows.find(r=>r.id===name);assert(row);const raw=JSON.parse(fs.readFileSync(path.join(s.assets.CONTENT,'Event/json',name)));data.events[name]={map_id:row.MapID==='None'?0:Number(row.MapID),reset_counter:row.EncountReset,completion:null,commands:Object.entries(raw).map(([index,c])=>({index:Number(index),cmd:c.cmd,target:c.target,text:c.text,opt:c.opt}))};}
fs.writeFileSync(p,JSON.stringify(data,null,2)+'\n');
const core=path.join(__dirname,'../../OT2BattleEngineRust/data');
const r=s.table('Character/Database/EnemyDB').rows.find(r=>r.id==='ENE_EVE_SUB_OTR_100'),q=r.Param;
const escape=JSON.parse(fs.readFileSync(path.join(core,'field_escape.json'))),battle=JSON.parse(fs.readFileSync(path.join(core,'field_break.json'))),rewards=JSON.parse(fs.readFileSync(path.join(core,'field_rare_rewards.json')));
escape.enemies[r.id]={speed:q.AGI,bp_base:q.BP,weakness_count:[...r.WeaponResist,...r.AttributeResist].filter(x=>x.endsWith('::eWEAK')).length,initialization:initialization(r)};
const e=structuredClone(battle.enemies.ENE_NML_FLD_CTY_010);e.id=r.id;e.name='Brigand';e.stats={max_hp:q.HP,max_sp:q.MP,speed:q.AGI};e.resources={hp:q.HP,sp:q.MP,bp:r.FirstBP,latent:0};
Object.assign(e.combat,{use_cat_damage_type:r.UseCatDamageType,attack:q.ATK,defense:q.DEF,magic:q.MATK,magic_defense:q.MDEF,accuracy:q.ACC,evasion:q.EVA,critical:q.CON,growth_attack:Math.round((r.DamageRatio-.5)*128),growth_magic:Math.round((r.DamageRatio-.5)*128),shields:q.SP,enemy_bp_base:q.BP,enemy_initialization:initialization(r),weapon_weaknesses:r.WeaponResist.flatMap((v,i)=>v.endsWith('::eWEAK')?[['sword','lance','dagger','axe','bow','staff'][i]]:[]),element_weaknesses:r.AttributeResist.flatMap((v,i)=>v.endsWith('::eWEAK')?[['none','fire','ice','lightning','wind','light','dark'][i]]:[])});battle.enemies[r.id]=e;
rewards[r.id]={exp:r.Exp,jp:r.JobPoint,money:r.Money,drop_item:r.HaveItemID,drop_rate:r.DropProbability};
for(const [name,value]of [['field_escape.json',escape],['field_break.json',battle],['field_rare_rewards.json',rewards]])fs.writeFileSync(path.join(core,name),JSON.stringify(value,null,2)+'\n');
fs.writeFileSync(path.join(root,'source/travelers-bag.json'),JSON.stringify({enemy:r,events:Object.fromEntries(names.map(n=>[n,data.events[n]]))},null,2)+'\n');
console.log('Imported Traveler’s Bag scripts and Brigand battle data');

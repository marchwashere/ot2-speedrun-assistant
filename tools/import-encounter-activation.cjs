'use strict';
const fs=require('fs'),path=require('path'),assert=require('assert/strict');
const mapping=require('../../outputs/encounter-zones/zone-data.json');
const all=mapping.rows.flatMap(r=>r.placements.map(p=>({...p,map:r.map,zone:r.trigger})));
const root=path.resolve(__dirname,'../..'),vm=require('vm'),{pkg}=require('../../tools/trace_damage_exact');
const src=fs.readFileSync(path.join(root,'tools/export_ability_data.js'),'utf8').split('const ctx = loadPackage(ABILITY_ASSET);')[0];
const sb={require:require('module').createRequire(path.join(root,'tools/export_ability_data.js')),__dirname:path.join(root,'tools'),Buffer};vm.createContext(sb);vm.runInContext(src+'\nthis.factory=makeReader;',sb);
function rawBase(p,field){const full=path.join(root,p.package),pack=pkg(full);pack.b=Buffer.concat([fs.readFileSync(full),fs.readFileSync(full.replace('.umap','.uexp'))]);const ex=pack.exports.find(e=>e.name===p.actor);return sb.factory(pack).parseProperties(ex.serialOffset,ex.serialOffset+ex.serialSize).values[field];}
const records=[];
for(const p of all.filter(p=>p.replacementFlag)){
 const m=/^ReplaceEncountData(_LND|_WTR)?\.(\d+)\./.exec(p.field);assert(m,p.field);
 const field='EncountTriggerLabel'+(m[1]||'');
 let base=all.find(b=>b.map===p.map&&b.package===p.package&&b.actor===p.actor&&b.field===field&&!b.replacementFlag);
 if(!base){const zone=rawBase(p,field);assert(zone,'Missing base '+p.actor);base={zone};}
 const key=[p.map,p.package,p.actor,field].join('|');
 let r=records.find(r=>r.key===key);if(!r){r={key,map:p.map,actor:p.actor,base:base.zone,start_flags:p.startFlags.filter(Boolean),end_flags:p.endFlags.filter(Boolean),replacements:[]};records.push(r);}
 const replacement={index:Number(m[2]),flag:p.replacementFlag,zone:p.zone};
 if(!r.replacements.some(x=>x.index===replacement.index))r.replacements.push(replacement);
 else assert.deepEqual(r.replacements.find(x=>x.index===replacement.index),replacement);
}
for(const r of records)r.replacements.sort((a,b)=>a.index-b.index);
records.sort((a,b)=>a.key.localeCompare(b.key));
fs.writeFileSync(path.join(__dirname,'../data/encounter-activation.json'),JSON.stringify(records,null,2)+'\n');
console.log(records.length+' replacement-bearing trigger/terrain bindings across '+new Set(records.map(r=>r.map)).size+' maps');

const {blueprint}=require('../../OT2BattleEngineRust/tools/audit-pirro.cjs');
fs.writeFileSync(path.join(__dirname,'../data/source/encounter-activation-audit.json'),JSON.stringify({trigger:blueprint('Battle/BP/EncountTrigger',['ExecuteUbergraph_EncountTrigger','CheckTriggerActive','SetEncountVolume']),canoe:blueprint('Environment/BP/BP_Canoedock',['ChangeToWalk','ChangeToCanoe'])},null,2)+'\n');

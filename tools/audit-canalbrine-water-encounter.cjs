'use strict';
// Source evidence only. No fixed draw total or runtime encounter special case.
const fs=require('node:fs'),path=require('node:path'),assert=require('node:assert/strict');
const s=require('./source.cjs');
const {blueprint}=require('../../OT2BattleEngineRust/tools/audit-pirro.cjs');
const {initialization}=require('../../OT2BattleEngineRust/tools/enemy-initialization.cjs');
const {PE}=require('../../OT2BattleEngine/tools/pe.cjs');
const sources=[];
function table(name) {const t=s.table(name);sources.push({source:t.source,sha256:t.sha256});return t.rows;}
function one(rows,id) {const r=rows.find(r=>r.id===id);assert(r,id);return r;}
const level=one(table('Level/Database/LevelTable'),'Fld_Sea_1_2');
const trigger=one(table('Battle/Database/EncountTriggerData'),'ENT_FLD_SEA_1_2_WTR');
const volumeRows=table('Battle/Database/EncountVolumeData');
const volumes=[trigger.DayTimeEncountVolumeName,trigger.NightEncountVolumeName].map(id=>one(volumeRows,id));
assert.deepEqual(volumes.map(r=>r.UseMapIndex),[1,1]);
assert(volumes.every(r=>r.WaterSide&&r.Area==='EAreaID::SEA'));
assert.deepEqual(volumes[0].EncounterList,volumes[1].EncounterList);
assert(volumes.every(r=>!r.RareCatlinParam.RareEnemyFlag&&r.RareOctlinParam.RareEnemyFlag));
assert.deepEqual(volumes[0].RareOctlinParam,{RareEnemyFlag:true,RareEnemyRate:4,RareEnemyCnt_Low:1,RareEnemyCnt_Middle:0,RareEnemyCnt_High:0});
const battleMap=level.BattleMapNameList[volumes[0].UseMapIndex];
assert.equal(battleMap,'Btl_Fld_Sea_River_A');
const encounterRows=table('Battle/Database/BattleEncounterData');
const tiers=volumes[0].EncounterList.filter(r=>r.EncounterName!=='None');
assert.deepEqual(tiers.map(r=>[r.ProgressBorder,r.DisplayLevel]),[[0,5],[3,8],[5,11]]);
const encounters=tiers.map(r=>one(encounterRows,r.EncounterName));
assert(encounters.every(r=>r.FirstEncount===0));
const groupRows=table('Battle/Database/EnemyGroupData');
const groups=[...new Set(encounters.flatMap(r=>r.Group.filter(g=>g.Appearance>0).map(g=>g.GroupID)))].map(id=>one(groupRows,id));
assert.equal(groups.length,12);
assert(groups.every(r=>r.EscapeEnable&&!r.BossBattle&&r.TalkPlayerID==='EPlayableCharacterID::eNONE'&&r.BattleStartEvent.asset.text==='None'));
const params=table('GameParam/GameParamDefineTable');
const rareParam=one(params,'RARE_ENEMY_OCTOPUS_SEA');
assert.equal(rareParam.Min,'ENE_NML_WTR_550');
const enemyRows=table('Character/Database/EnemyDB');
const enemies=[...new Set([...groups.flatMap(g=>g.EnemyID).filter(id=>id!=='None'),rareParam.Min])].map(id=>one(enemyRows,id));
assert.equal(enemies.length,6);
assert(enemies.every(r=>!r.NPCFlag&&r.EnemyAI.asset.text.endsWith('/BattleAI_NormalEnemy.BattleAI_NormalEnemy_C')));
const initializationProfiles=Object.fromEntries(enemies.map(r=>[r.id,initialization(r)]));
assert(Object.values(initializationProfiles).every(p=>p.initial_idle_frames>0&&!p.sync_to_parent&&!p.sync_children));
const blueprints=[];
for(const [asset,functions] of [
  ['Environment/BP/BP_Canoedock',null],
  ['Trigger/BP/TutorialTrigger',null],
  ['Battle/BP/EncountManagerBP',['SetEncountVolume','SetEncounter','SetForceStopEncountStep','SettingBattleParam','JudgeBattleSituation']],
  ['Library/LibBattle',['LotteryEncountData']],
  ['Library/LibRareEnemy',['GetRareEnemyLabel','GetRareOctlinLabelByArea']],
  ['Battle/BP/BattleManagerBP',['SetBattleStartType','CreatePlayerCharacter','CreateEnemyCharacter','CreateRareEnemyReplaceList']],
  ['Battle/BP/BattlePlayerBP',['Initialize','ExecuteUbergraph_BattlePlayerBP']],
  ['Battle/BP/BattleEnemyBP',['InitializeEnemy']],
  ['Character/BP/Type/KSPlayerCharacter',['SetUpMoveSpeedData']],
]) {
  const b=blueprint(asset,functions);blueprints.push(b);sources.push({source:'/Game/'+asset,sha256:b.sha256});
}
const dock=blueprints[0],tutorialTrigger=blueprints[1],manager=blueprints[2];
assert(dock.functions.ChangeToCanoe.includes('SetCharacterMoveMode\n    09121 ByteConst 5'));
assert(dock.functions.ChangeToCanoe.includes('LocalVirtualFunction GetEncountTriggerWTR'));
assert(!Object.values(dock.functions).some(v=>/Random|ResetEncountStep/.test(v)));
assert(!/Random|ResetEncountStep/.test(manager.functions.SetEncountVolume));
assert(!/Random|ResetEncountStep/.test(manager.functions.SetEncounter));
assert.equal(tutorialTrigger.defaults[0].TutorialType,'ETUTORIAL_TYPE::eACTION_CANOE');
const tutorial=table('Tutorial/Database/TutorialFlagPart').find(r=>r.ID===6);
assert.deepEqual([tutorial.TutorialListFlag,tutorial.TutorialOpenedFlag],[1005,1305]);
const environmentFile=path.join(__dirname,'../data/source/canalbrine-bridge-loading-audit.json');
const environmentBytes=fs.readFileSync(environmentFile),environment=JSON.parse(environmentBytes);
assert.equal(environment.runtime_random_functions.length,0);
assert.equal(environment.embedded_random_functions.length,0);
const riverScripts=environment.embedded_scripts.filter(r=>r.level===battleMap);
assert.equal(riverScripts.length,2);
sources.push({source:'data/source/canalbrine-bridge-loading-audit.json',sha256:s.assets.hash(environmentBytes)});
// PlayCanoePitching is a small native omitted by the text decompiler. Verify its
// real registration and both tail jumps directly in the read-only retail image.
const exe=process.env.OT2_EXE||'C:/Program Files (x86)/Steam/steamapps/common/Octopath_Traveler2/Octopath_Traveler2/Binaries/Win64/Octopath_Traveler2-Win64-Shipping.exe';
const pe=new PE(exe),registration=0x143de42a0;
assert.equal(pe.text(pe.ptr(registration)),'PlayCanoePitching');
assert.equal(pe.ptr(registration+8),0x141032ce0);
function code(va,length) {return pe.bytes.subarray(pe.offset(va,length),pe.offset(va,length)+length);}
const wrapper=code(0x141032ce0,26),pitch=code(0x140e66970,93);
assert.equal(wrapper.toString('hex'),'488b42204533c04885c0410f95c04c03c04c894220e9763ce3ff');
assert.equal(0x141032ce0+26+wrapper.readInt32LE(22),0x140e66970);
assert.equal(pitch.toString('hex'),'488b81680800004885c07450488b91b80400004885d27444f20f10801c010000f20f11812c0800008b8024010000898134080000f20f10821c010000f20f11811c0900008b82240100008981240900004881c188070000e904a81202c3');
assert.equal(0x140e66970+92+pitch.readInt32LE(88),0x142f911d0);
const nativeFile='Octopath_Traveler2-Win64-Shipping.exe.c';
const nativeText=fs.readFileSync(path.join(s.root,nativeFile),'utf8'),native={};
for(const name of ['FUN_140e66820','FUN_140e69840','FUN_142f911d0','FUN_142f95610',
  'FUN_14103f1f0','FUN_140e69ea0','FUN_140f05f40','FUN_140f12cc0',
  'FUN_1410347e0','FUN_140e6a6e0','FUN_14102ea10']) {
  const m=new RegExp('^[A-Za-z_][^\\r\\n]* '+name+'\\(','m').exec(nativeText);assert(m,name);
  const end=nativeText.indexOf('\n}',m.index);assert(end>m.index);
  const body=nativeText.slice(m.index,end+2);
  native[name]={source:nativeFile,function:name,sha256:s.assets.hash(Buffer.from(body)),body};
}
const audit={
  scope:'Daytime Canalbrine Bridge first water encounter after Hikari recruitment and Throné Scholar setup. Source-only evidence; carried party, progression, RNG and enemy knowledge remain inputs to the shared encounter and battle cores.',
  sources,level,trigger,volumes,tiers,encounters,groups,enemies,rare_parameter:rareParam,
  enemy_initialization:initializationProfiles,blueprints,tutorial,river_level_scripts:riverScripts,native,
  native_canoe_pitching:{registration:'0x143de42a0',exec:'0x141032ce0',exec_bytes:wrapper.toString('hex'),
    implementation:'0x140e66970',implementation_bytes:pitch.toString('hex'),timeline_start:'0x142f911d0',
    interpretation:'The exec wrapper advances the VM argument cursor then tail-calls the implementation. The implementation checks two component pointers, copies their transforms, and starts the member timeline at offset0x788. Timeline start calls SetPlaybackPosition(0) then sets playback flags; there is no random start-time choice. This is distinct from the decorative BP_EnvObjMD_Canoe_A actor.'},
  interpretation:{
    boarding:'BP_Canoedock.ChangeToCanoe sets movement mode5 and the dock water encounter volume. SetCharacterMoveMode exec14103f1f0 -> e69ea0 writes manager mode and saved mode through f05f40/f12cc0. Actor SetMoveMode exec1410347e0 -> e6a6e0 clamps the mode, writes its mode/change flags, calls SetUpMoveSpeedData14102ea10, and sets the canoe-mode boolean. These audited branches contain no CRT, resource spawning or reset. SetEncountVolume/SetEncounter select data deterministically and do not reset the counter. The dock pauses two encounter-step updates; that is not two CRT draws. Canoe tutorial6 is a once-only queue27 popup with list/open flags1005/1305; dismissing it uses the existing CRT-neutral tutorial path.',
    map:'UseMapIndex1 selects Btl_Fld_Sea_River_A deterministically. Both embedded river functions are empty startup forwarding/return. The complete Bridge environment audit found no additional runtime environmental CRT callback.',
    formation:'At progress5 the active row is ENC_FLD_SEA_1_3_WTR_DAY_030, danger11. Its FirstEncount0 means one weighted group draw even on the first encounter. Day/night share the same formation table. All groups allow Escape and have no forced battle-start event or talk actor.',
    selection_draws:'Weighted group selection, battle situation, one timing draw per active player, then the single enabled Octopuff-family gate at4%. No Cait-family draw. A successful gate performs the shared size-priority replacement-slot draw for low Harbor Octopuff ENE_NML_WTR_550. The normal enemy-initialization, turn-order and voice draws then follow. No extra river-map selection or canoe-pitching draw is inserted.',
    enemies:'All five ordinary water enemy types and Harbor Octopuff are ordinary non-NPC actors with a nonempty initial idle flipbook and no parent/child synchronization. Use each actual initialization profile rather than a water-specific offset.',
    player_display:'WaterSideMode only adds canoe visibility, fixed transforms, fixed timeline pitching and particle playback on this startup path. Particle RNG is outside CRT and excluded. Player timing and initial BP follow the normal shared Initialize path.',
  },
};
const output=path.join(__dirname,'../data/source/canalbrine-water-encounter-audit.json');
fs.writeFileSync(output,JSON.stringify(audit,null,2)+'\n');
console.log(JSON.stringify({output,tiers:tiers.length,groups:groups.length,enemies:enemies.length,battle_map:battleMap,
  progress5_encounter:encounters[2].id,first:encounters[2].FirstEncount,water_startup_extra_crt:0,sources:sources.length}));

'use strict';
// Run from the workspace root. Native registration/vtable addresses are from
// the audited retail executable; validate the C implementations and BP chain.
const fs=require('fs'),assert=require('node:assert/strict');
const source=fs.readFileSync('Octopath_Traveler2-Win64-Shipping.exe.c','utf8');
function native(n){const p=source.indexOf(' FUN_'+n+'(');assert(p>=0,n);const end=source.indexOf('\n}',p);return source.slice(p,end+2);}
function bp(file,name){const text=fs.readFileSync('tools/speedrun_audit/'+file+'.txt','utf8');const p=text.indexOf('# '+name+' script=');assert(p>=0,name);const end=text.indexOf('\n# ',p+1);return text.slice(p,end<0?undefined:end);}
const force=native('140e920f0'),load=native('140e8dd70');
assert(force.includes('(param_1 + 0x59d) = 1'));
assert(load.includes("if (*(char *)(param_1 + 0x59d) == '\\0')"));
assert(load.includes('iVar3 = FUN_140ece1a0()'));
assert(load.includes('if (iVar2 == iVar3)'));
assert(load.includes('return 0;'));
assert(load.includes('return 1;'));
assert(bp('UserInterface__BarMenu__BP__BarMenuStorySelect','OnClosedConfirmDialog').includes('.PlayEventForce('));
assert(bp('Event__BP__EventManagerBP','EventLoadLevel').includes('RequestMapChangeForEvent('));
assert(bp('Level__BP__LevelFunction','RequestMapChangeForEvent').includes('RequestLoadFieldLevel('));
assert(bp('Level__BP__BPC_MJLevelManager','RequestLoadFieldLevel').includes('.Proc_OnFinishLoadFieldLevel()'));
assert(bp('Level__BP__BPC_LevelManager_Others','Proc_OnFinishLoadFieldLevel_Impl').includes('.ResetEncountVolume()'));
assert(bp('Level__BP__BPC_LevelManager_Others','Proc_OnFieldLevelShown_Last_Impl').includes('ResetEncountOnChangeLevel()'));
assert(bp('Level__BP__BPC_LevelManager_Others','ResetEncountOnChangeLevel').includes('.ResetEncountVolume()'));
assert(bp('Battle__BP__EncountManagerBP','ResetEncountVolume').includes('ResetEncountStep()'));
console.log('Verified: chapter selection -> PlayEventForce -> forced same-map request -> both level lifecycle counter resets.');

'use strict';
// Run from workspace root. Keep load, visibility and actor RNG distinct.
const fs=require('fs'),assert=require('node:assert/strict');
const source=fs.readFileSync('Octopath_Traveler2-Win64-Shipping.exe.c','utf8');
function native(name){
  const re=new RegExp('^[^\\r\\n]*\\bFUN_'+name+'\\([^;]*?\\r?\\n\\r?\\n\\{[\\s\\S]*?\\r?\\n\\}', 'm');
  const m=source.match(re);assert(m,name);return m[0];
}
const load=native('140ed98a0');
assert(load.includes('FUN_140ed6bf0(lVar6,*(undefined8 *)(param_1 + 0x468),1,0)'));
assert(load.includes('FUN_140ed6bf0(lVar6,*(undefined8 *)(param_1 + 0x468),3,0)'));
assert(load.includes('local_68 = CONCAT31((int3)uVar2,1)'));
assert(load.includes('*(undefined8 **)(lVar8 + 0x80) = puVar4'));
const queue=native('140ed4030');
assert(queue.includes("if (cVar3 == '\\x01') {\n      cVar3 = FUN_140ed22a0();".replace(/\n/g,'\r\n')));
assert(queue.includes('cVar3 = FUN_140ed8640();'));
assert(native('140ed22a0').includes('(**(code **)(*plVar3 + 0x270))(plVar3,1)'));
const show=native('140ed8640');
assert(show.includes('*(longlong *)(lVar4 + 0x128) != 0'));
assert(show.includes('FUN_142c78250(lVar4,'));
assert(native('142c78250').includes('param_2 << 3'));
const transition=native('140eda7e0');
assert(transition.includes('FUN_140ed6bf0(uVar7,uVar1,uVar11,0)'));
assert(transition.includes('uVar11 = 3;'));
const lamp=fs.readFileSync('tools/rng_conning_lamp.txt','utf8');
assert(lamp.includes('ExecuteUbergraph_BP_Lamp_I_Sea(235)'));
assert(lamp.includes('RandomFloatInRange(0, 3)'));
const flag=fs.readFileSync('tools/rng_traces/Environment__Resource__Object__EnvObj_Batlefield__BP__BP_EnvObjMD_Batlefield_A_Flag_A_TwoTone_B.txt','utf8');
assert(flag.includes('RandomFloatInRange(0.8999999761581421, 1.100000023841858)'));
assert(!flag.includes('JumpIfNot'));
assert(!lamp.includes('JumpIfNot'));
console.log('Verified separate field load/show, battle preload, visibility dispatch, battle transition and unconditional flag/lamp RNG callbacks.');

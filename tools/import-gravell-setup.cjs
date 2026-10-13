'use strict';
const fs=require('fs'),path=require('path'),s=require('./source.cjs');
const jobs=s.table('Character/Database/JobData').rows.filter(x=>['eINVENTOR','eMERCHANT','eDANCER','eFENCER'].includes(x.id));
const sets=s.table('Ability/Database/AbilitySetData').rows.filter(x=>jobs.some(j=>j.JobCommandAbility.some(a=>a.AbilityName===x.id)));
const supports=s.table('Ability/Database/SupportAbilityData').rows.filter(x=>jobs.some(j=>j.JobSupportAbility.some(a=>a.AbilityName===x.id)));
const abilities=s.table('Ability/Database/AbilityData').rows.filter(x=>/^(ABI_DAN_MAKA_|ABI_DAN_0[67]0_|ABI_INV_020_|ABI_MER_0[57]0_|PC_ABI_MER_YOHEI)/.test(x.id));
fs.writeFileSync(path.join(__dirname,'../data/gravell-setup.json'),JSON.stringify({jobs,sets,supports,abilities},null,2)+'\n');

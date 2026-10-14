'use strict';
const fs=require('node:fs'),vm=require('node:vm'),path=require('node:path');
const elements=new Map();
function element(id){if(!elements.has(id))elements.set(id,{textContent:'',innerHTML:'',value:'0',dataset:{},style:{},classList:{toggle(){},add(){},remove(){}},addEventListener(){},setAttribute(){},getAttribute(){return 'false';},querySelectorAll(){return [];},focus(){}});return elements.get(id);}
const context=vm.createContext({console,URLSearchParams,TextDecoder,Event,setTimeout,
 localStorage:{getItem(){return null;},setItem(){}},
 document:{getElementById:element,querySelectorAll(){return [];},querySelector:element,addEventListener(){}},
 window:{addEventListener(){},dispatchEvent(){},scrollTo(){}},fetch:async()=>({ok:true,json:async()=>({})})});
for(const name of ['viewer.js','voice.js'])vm.runInContext(fs.readFileSync(path.join(__dirname,'../viewer',name),'utf8'),context);
for(const file of process.argv.slice(2)){
 context.snapshot=JSON.parse(fs.readFileSync(file,'utf8'));
 vm.runInContext('activeRoute="galdera";result=snapshot;render();',context);
 console.log('PASS: renders '+file);
}

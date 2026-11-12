const fs=require('fs');
const [localFile,fullFile,outFile]=process.argv.slice(2);
const read=p=>fs.readFileSync(p,'utf8').trim().split(/\r?\n/).filter(Boolean).map(JSON.parse);
function summary(rows,key,time){
  const groups={};for(const r of rows){(groups[r.strategy]??=[]).push(r);}
  const stats={};
  for(const [strategy,a]of Object.entries(groups)){
    const values=a.map(r=>r[key]).sort((a,b)=>a-b), times=a.map(r=>r[time]).sort((a,b)=>a-b);
    const mean=a=>a.reduce((s,x)=>s+x,0)/a.length;
    const median=a=>a.length%2?a[(a.length-1)/2]:(a[a.length/2-1]+a[a.length/2])/2;
    stats[strategy]={count:a.length,mean:mean(values),median:median(values),p95:values[Math.ceil(values.length*.95)-1],max:values.at(-1),mean_seconds:mean(times),median_seconds:median(times),max_seconds:times.at(-1)};
    for(const k of ['winter_cost','harbor_cost','cropdale_cost'])stats[strategy][k]=mean(a.map(r=>r[k]));
  }
  const pairs=new Map();for(const r of rows){if(!pairs.has(r.seed))pairs.set(r.seed,{});pairs.get(r.seed)[r.strategy]=r;}
  let wins=0,ties=0,losses=0;const changes=[];
  for(const p of pairs.values())if(p.harbor_required&&p.cropdale_fallback){const change=p.cropdale_fallback[key]-p.harbor_required[key];change<0?wins++:change>0?losses++:ties++;changes.push({seed:p.harbor_required.hex,old:p.harbor_required[key],new:p.cropdale_fallback[key],change});}
  return {stats,wins,ties,losses,changes:changes.sort((a,b)=>a.change-b.change)};
}
const result={method:{checkpoint:'Western Winterbloom Snows entry; same retail party fixture, prior rare already killed, Evasive Maneuvers active',sampler:'SplitMix64 upper 32 bits, seed 20261112; common ambushed opening excluded',cost:'Existing weighted input equivalents; fixed earlier common prefix omitted',timing:'Sequential optimized native process, lazy tables warmed, alternating strategy order; excludes network/browser and compilation'},local:summary(read(localFile),'local_cost','local_seconds'),full:fullFile?summary(read(fullFile),'through_exp100_cost','full_seconds'):null};
result.method.scope='Local cost: Western Winterbloom escapes + Harbor escape + Eastern Cropdale escape. Full cost: same checkpoint through selected EXP100 victory, including Oresrush, coasts, steals, time switches and battle controls. Full timing extends only the predicted selected route, not both UI override alternatives.';
result.method.party='Retail fixture: Throné Lv14 (1170 EXP), Osvald Lv14 (1107 EXP), Evasive Maneuvers already active. This does not sample the alternative of obtaining the first rare in Western Winterbloom.';
if(outFile)fs.writeFileSync(outFile,JSON.stringify(result,null,2));
console.log(JSON.stringify({...result,local:{...result.local,changes:result.local.changes.slice(0,3)},full:result.full&&{...result.full,changes:result.full.changes}},null,2));

'use strict';
(() => {
  let catalog, prediction, requestVersion = 0;
  const routeMode = () => $('field-route').value === 'throne_after_opening';
  const esc = escapeHtml;
  const option = (value,label) => `<option value="${esc(value)}">${esc(label)}</option>`;
  const fieldError = message => { $('field-error').textContent=message; $('field-error').hidden=false; };
  const integer = id => {
    const value=$(id).value.trim();
    if(!/^\d+$/.test(value)) throw Error('Enter a nonnegative whole number for '+$(id).closest('label').firstChild.textContent.trim()+'.');
    const n=Number(value); if(!Number.isSafeInteger(n)) throw Error('Modifier is too large.'); return n;
  };
  const count = id => { const value=$(id).value.trim(); if(!/^\d+$/.test(value)||BigInt(value)>18446744073709551615n)throw Error('CRT advances must be between 0 and 18446744073709551615.'); return BigInt(value).toString(); };
  window.addEventListener('screen-changed', event=>{if(event.detail==='field'&&!catalog)loadCatalog();});
  async function loadCatalog() {
    try {
      const response=await fetch('/api/encounters/catalog');
      if(!response.ok) throw Error('Unable to load encounter catalog.');
      catalog=await response.json(); renderMaps('Dng_Cty_3_1');
      $('field-status').textContent=`${catalog.coverage.maps} maps · ${catalog.coverage.volumes} encounter volumes. ${catalog.coverage.unresolved_zones} zones have unresolved map links.`;
      if(routeMode()) { applyBattleFrame(); if(result?.post_battle_field) predict(); }
    } catch(e) {fieldError(e.message);}
  }
  function renderMaps(preferred=$('field-map').value) {
    const query=$('field-search').value.trim().toLowerCase();
    const maps=catalog.maps.filter(m=>(m.name+' '+m.id).toLowerCase().includes(query)).sort((a,b)=>a.name.localeCompare(b.name)||a.id.localeCompare(b.id));
    $('field-map').innerHTML=maps.map(m=>option(m.id,`${m.name} · ${m.id}`)).join('');
    if(maps.some(m=>m.id===preferred)) $('field-map').value=preferred;
    renderZones();
  }
  function renderZones() {
    const map=catalog.maps.find(m=>m.id===$('field-map').value), previous=$('field-zone').value;
    const zones=$('field-all-zones').checked?Object.keys(catalog.zones).map(id=>({id,mapping:'manual'})):(map?.zones||[]);
    $('field-zone').innerHTML=zones.map(z=>option(z.id,z.id+(z.mapping==='inferred'?' · inferred':''))).join('');
    if(zones.some(z=>z.id===previous)) $('field-zone').value=previous;
    describeZone();
  }
  function describeZone() {
    const map=catalog.maps.find(m=>m.id===$('field-map').value), zone=catalog.zones[$('field-zone').value];
    const period=zone?.periods[$('field-period').value], volume=catalog.volumes[period?.volume];
    if(!map) { $('field-zone-info').textContent='No matching map.'; return; }
    const c=map.counter, link=map.zones.find(z=>z.id===zone?.id);
    const flags=[...(zone?.flags||[]),...(volume?.flags||[])].filter(Boolean);
    $('field-zone-info').textContent=`Counter: ${c.min} + B[0…${Math.max(0,c.b-1)}] + A[0…${Math.max(0,c.a-1)}]; first-encounter ratio ${c.first}. `+
      (volume ? `${volume.water?'Water':'Land'} · decrement ${period.decrement} before modifiers · ${volume.id}. `:'No encounter volume for this selection. Counter-reset-only prediction is available. ')+
      (link?.mapping==='inferred'?'Map link inferred; placement not recovered. ':'')+(flags.length?`Story flags: ${[...new Set(flags)].join(', ')}. Selected zone must already be active. `:'')+
      (link?.conditions?.length?'Placement has additional story conditions. ':'');
  }
  function boundaryChanged() {
    const b=$('field-boundary').value;
    $('field-gap-label').hidden=!['walk','current_counter'].includes(b); $('field-current-label').hidden=b!=='current_counter'; $('field-predict').textContent=b==='reset'?'Predict counter reset':'Predict encounter';
  }
  function readFieldSetup() {
    const levels=$('field-levels').value.trim().split(/[\s,]+/);
    if(levels.some(x=>!/^\d+$/.test(x)))throw Error('Enter comma-separated party levels.');
    if(routeMode()&&!result?.post_battle_field) throw Error('Run a successful opening battle first.');
    return {seed:validateSeed($('field-seed').value.trim()),advances:count('field-advances'),profile:routeMode()?'throne_after_opening':null,options:{
      map:$('field-map').value,zone:$('field-zone').value,period:$('field-period').value,
      boundary:$('field-boundary').value,first_encounter:$('field-first').checked,
      progress:integer('field-progress'),party_levels:levels.map(Number),dash:$('field-dash').checked,
      cait_bonus:integer('field-cait'),octopuff_bonus:integer('field-octo'),rare_support_bonus:integer('field-rare'),
      preemptive_bonus:integer('field-preempt'),surprise_reduction:integer('field-surprise'),
      ability_percent:integer('field-ability'),equipment_percent:integer('field-equipment'),
      intervening_advances:['walk','current_counter'].includes($('field-boundary').value)?count('field-gap'):'0',
      current_counter:$('field-boundary').value==='current_counter'?integer('field-current'):null,allow_unmapped_zone:$('field-all-zones').checked
    }};
  }
  const fixedFields=['seed','advances','boundary','current','search','map','zone','period','progress','levels','first','all-zones','cait','octo','rare','preempt','surprise','ability','equipment'];
  function setRouteMode() {
    for(const key of fixedFields) $('field-'+key).disabled=routeMode();
    $('field-frame').textContent=routeMode()?'Waiting for a completed opening battle.':'Custom area: supply the active zone, global progress points, party and counter for this frame.';
  }
  function applyBattleFrame() {
    setRouteMode();
    if(!catalog||!routeMode())return;
    invalidate();
    const frame=result?.post_battle_field;
    if(!frame)return;
    const o=frame.options;
    $('field-seed').value=hex(frame.seed); $('field-advances').value=frame.advances;
    $('field-boundary').value=o.boundary; $('field-current').value=o.current_counter;
    $('field-search').value=''; $('field-all-zones').checked=false; renderMaps(o.map);
    $('field-zone').value=o.zone; $('field-period').value=o.period;
    $('field-progress').value=o.progress; $('field-levels').value=o.party_levels.join(',');
    $('field-first').checked=o.first_encounter; $('field-gap').value='0';
    for(const [key,value] of Object.entries({cait:o.cait_bonus,octo:o.octopuff_bonus,rare:o.rare_support_bonus,preempt:o.preemptive_bonus,surprise:o.surprise_reduction,ability:o.ability_percent,equipment:o.equipment_percent}))$('field-'+key).value=value;
    boundaryChanged();describeZone();
    $('field-frame').textContent=`Underground Waterway · Night · main progress 0 · first encounter active · ${frame.profile.party.join(', ')} (level 1). ${frame.profile.boundary_label}: advance ${frame.advances}, ${hex(frame.starting_state)}, counter ${o.current_counter}. Valid until the next sewer story event; flag 25304 is off. No additional initial reset.`;
    $('field-error').hidden=true;
  }
  function invalidate() {
    ++requestVersion; prediction=null; $('field-result').hidden=true; $('field-predict').disabled=false;
  }
  function render(p) {
    const routeSummary=p.reset_reach?`<section class="panel overview route-checks">${result?.post_battle_field?.starting_state===p.setup.starting_state?battleAssessmentHtml():''}${resetReachHtml(p.reset_reach)}</section>`:'';
    if(p.kind==='reset_reached') {
      $('field-result').innerHTML=`${routeSummary}<section class="panel overview"><div class="result-heading"><h2>Next cutscene reached · no encounter predicted</h2><button type="button" id="field-export">Export JSON ↓</button></div><p>${p.counter_before_reset} counter points remain before the reset.</p><p class="route-help"><strong>${esc(p.total_advances)} advances since initial seed</strong> · <code>${hex(p.final_state)}</code> · ${esc(p.draws)} CRT calls consumed.</p><p class="muted small">${esc(p.endpoint)}. Walking/running itself consumes no CRT. No encounter rolls or cutscene reset rolls have been applied.</p></section>`;
      finishRender(p);return;
    }
    const encounter=p.group!==null, walk=['walk','current_counter'].includes(p.setup.options.boundary);
    const formula=c=>c.supplied?'Existing counter; no initial reset':`(${c.minimum} + B ${c.roll_b} + A ${c.roll_a})${c.ratio!==1?` × ${c.ratio}`:''}`;
    $('field-result').innerHTML=`${routeSummary}<div class="result-heading"><h2>${esc(p.map_name)}</h2><button type="button" id="field-export">Export JSON ↓</button></div>
      <div class="metrics"><div class="panel prediction"><div class="metric-label">${walk?'INITIAL COUNTER':'RESET COUNTER'}</div><div class="seed-value">${p.counter.value}</div><p>${formula(p.counter)}</p></div>
      <div class="panel metric"><div class="metric-label">${walk?'COUNTER UPDATES UNTIL ENCOUNTER':'CRT CALLS CONSUMED'}</div><strong>${walk?p.counter_updates:p.draws}</strong><p>${walk?`${p.decrement} deducted per update · movement updates, not animation footsteps`:esc(p.endpoint)}</p></div>
      <div class="panel metric"><div class="metric-label">ADVANCES SINCE INITIAL SEED</div><strong>${esc(p.total_advances)}</strong><p><code>${hex(p.final_state)}</code></p></div></div>
      ${encounter?`<section class="panel overview field-formation"><h2>${esc(p.situation)} encounter${p.rare_family?' · '+esc(p.rare_family)+' gate passed':''}</h2><div class="field-enemies">${p.enemies.map(e=>`<div><strong>${esc(e.enemy.name)}</strong><span>Level ${e.enemy.level} · group slot ${e.slot+1}</span><code>${esc(e.enemy.id)}</code>${e.enemy.id!==e.original_id?`<small>Replaces ${esc(e.original_id)}</small>`:''}</div>`).join('')}</div><p class="muted small">${esc(p.group)} · ${p.first_override?'Fixed first encounter':`group roll ${p.group_roll}`} · situation roll ${p.situation_roll}. ${p.trigger_reset?`Counter reset at trigger: ${p.trigger_reset.value} (${formula(p.trigger_reset)}).`:''}</p><p class="muted small">Table: ${esc(p.encounter_table)} · danger display ${p.encounter_level}. Slots use the data-table order.</p></section>`:''}
      <p class="accuracy-note">${esc(p.endpoint)}. This is not the command-menu or post-battle CRT state.</p>
      ${p.warnings.map(w=>`<p class="muted small">${esc(w)}</p>`).join('')}
      <section class="panel overview"><h2>Encounter RNG checkpoints</h2><div class="table-scroll"><table><thead><tr><th>Checkpoint</th><th>Advances since initial seed</th><th>CRT (hex)</th><th>Prediction draws</th></tr></thead><tbody>${p.checkpoints.map(c=>`<tr><td>${esc(c.label)}</td><td>${esc(c.total_advances)}</td><td><code>${hex(c.state)}</code></td><td>${esc(c.draws)}</td></tr>`).join('')}</tbody></table></div></section>`;
    finishRender(p);
  }
  function finishRender(p) {
    $('field-result').hidden=false;
    $('field-export').addEventListener('click',()=>{
      const url=URL.createObjectURL(new Blob([JSON.stringify(p,null,2)],{type:'application/json'}));
      const a=document.createElement('a');a.href=url;a.download=`ot2-encounter-${p.setup.options.map}-${p.setup.advances}.json`;a.click();URL.revokeObjectURL(url);
    });
  }
  $('field-search').addEventListener('input',()=>catalog&&renderMaps());
  $('field-map').addEventListener('change',renderZones); $('field-all-zones').addEventListener('change',renderZones);
  $('field-zone').addEventListener('change',describeZone); $('field-period').addEventListener('change',describeZone);
  $('field-boundary').addEventListener('change',boundaryChanged);
  $('field-use-battle').addEventListener('click',()=>{ $('field-route').value='throne_after_opening';applyBattleFrame();predict(); });
  $('field-route').addEventListener('change',()=>{invalidate();setRouteMode();if(routeMode()){applyBattleFrame();predict();}});
  window.addEventListener('battle-simulated',()=>{if(catalog&&routeMode()){applyBattleFrame();predict();}});
  $('field-form').addEventListener('input',()=>{invalidate();$('field-status').textContent='Settings changed. Predict again to update the result.';});
  async function predict() {
    if(stopped||!catalog)return;
    const version=++requestVersion;
    $('field-error').hidden=true;$('field-predict').disabled=true;
    try {
      const setup=readFieldSetup(); $('field-seed').value=hex(setup.seed);
      const response=await fetch('/api/encounters/predict?request='+encodeURIComponent(JSON.stringify(setup)));
      const data=await response.json();if(version!==requestVersion)return;if(!response.ok)throw Error(data.error||'Prediction failed.');
      prediction=data;render(data);$('field-status').textContent=`Predicted from advance ${setup.advances} (${hex(data.setup.starting_state)}); ${data.draws} CRT calls consumed.`;
    } catch(e) {if(version===requestVersion){invalidate();fieldError(e.message);$('field-status').textContent='No prediction available.';}}
    finally {if(version===requestVersion)$('field-predict').disabled=false;}
  }
  $('field-form').addEventListener('submit',event=>{event.preventDefault();predict();});
  setRouteMode();
})();

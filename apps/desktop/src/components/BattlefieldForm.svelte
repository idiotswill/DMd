<script lang="ts">
  import { untrack } from 'svelte';
  import { newId, type CharacterView } from '../table-api';
  import type { BattlefieldSetup } from '../tactical-api';
  let { characters, disabled = false, onPrepare }: { characters: CharacterView[]; disabled?: boolean; onPrepare: (setup: BattlefieldSetup) => void } = $props();
  let name = $state('Encounter'); let width = $state(50); let depth = $state(50);
  let light = $state<'Bright'|'Dim'|'Darkness'>('Bright');
  let reason = $state('The host established the terrain and starting positions from the current scene.');
  let positions = $state(untrack(() => characters.map((character, index) => ({ character, included: true, x: 5 + index * 5, y: 5, height: 6 }))));
  let regions = $state<{ kind: 'wall'|'difficult'; x: number; y: number; width: number; depth: number; height: number }[]>([]);
  let error = $state('');
  function submit(event: SubmitEvent) {
    event.preventDefault(); error = '';
    const selected = positions.filter(p => p.included);
    if (!selected.length || selected.some(p => !p.character.equipment?.prepared)) { error = "Prepare the equipment of every selected character first."; return; }
    if (selected.some(p => p.x < 0 || p.y < 0 || p.x + 5 > width || p.y + 5 > depth)) { error = 'Place each character inside the map.'; return; }
    if (regions.some(r => r.x < 0 || r.y < 0 || r.x + r.width > width || r.y + r.depth > depth)) { error = 'Place each terrain region inside the map.'; return; }
    const volume = (r: typeof regions[number]) => ({ min: { x: r.x*2, y: r.y*2, z: 0 }, max: { x: (r.x+r.width)*2, y: (r.y+r.depth)*2, z: r.height*2 } });
    onPrepare({ encounter_id: newId(), scene_id: newId(), location_id: newId(), name,
      battlefield: { bounds: { min: {x:0,y:0,z:0}, max: {x:width*2,y:depth*2,z:80} }, floor_z:0,floor_surface:'ground',ambient_light:light,
        obstacles: regions.filter(r=>r.kind==='wall').map((r,index)=>({id:`wall-${index}`,volume:volume(r),blocks_movement:true,blocks_sight:true,observable:true,cover:'Total'})),
        terrain: regions.filter(r=>r.kind==='difficult').map((r,index)=>({id:`terrain-${index}`,volume:volume(r),difficult:true,observable:true,water:false,climbable:false,burrowable:false,supports_top:false,surface:null,obscuration:'None',magical_darkness:false})), lights:[] },
      characters: selected.map(p=>({character_id:p.character.character_id,position:{x:p.x*2,y:p.y*2,z:0},height:p.height*2,allies:[],enemies:[]})),
      geometry_ruling:{basis:'GmAdjudication',reason} });
  }
</script>
<form onsubmit={submit}><fieldset {disabled}><legend>Prepare an encounter map</legend>
  <p>Place the current scene in feet. Characters keep their saved equipment and source movement speeds.</p>
  <label>Location name<input required maxlength="200" bind:value={name} /></label>
  <div class="form-grid"><label>Width (feet)<input type="number" min="10" max="250" step="5" required bind:value={width} /></label><label>Depth (feet)<input type="number" min="10" max="250" step="5" required bind:value={depth} /></label><label>Light<select bind:value={light}><option>Bright</option><option>Dim</option><option>Darkness</option></select></label></div>
  <h3>Starting positions</h3>
  {#each positions as position}<div class="form-grid"><label><input type="checkbox" bind:checked={position.included} />{position.character.name}{position.character.equipment?.prepared ? '' : ' — equipment needs preparation'}</label><label>{position.character.name}: east (feet)<input type="number" min="0" max={width-5} step="5" required bind:value={position.x} /></label><label>{position.character.name}: south (feet)<input type="number" min="0" max={depth-5} step="5" required bind:value={position.y} /></label><label>{position.character.name}: height (feet)<input type="number" min="1" max="10" step="0.5" required bind:value={position.height} /></label></div>{/each}
  <h3>Terrain</h3>
  {#each regions as region,index}<fieldset><legend>Region {index+1}</legend><div class="form-grid"><label>Kind<select bind:value={region.kind}><option value="wall">Opaque solid obstacle</option><option value="difficult">Difficult ground</option></select></label><label>East (feet)<input type="number" min="0" step="5" required bind:value={region.x} /></label><label>South (feet)<input type="number" min="0" step="5" required bind:value={region.y} /></label><label>Width (feet)<input type="number" min="5" step="5" required bind:value={region.width} /></label><label>Depth (feet)<input type="number" min="5" step="5" required bind:value={region.depth} /></label><label>Height (feet)<input type="number" min="0.5" max="40" step="0.5" required bind:value={region.height} /></label></div><button type="button" class="secondary" onclick={()=>regions=regions.filter((_,i)=>i!==index)}>Remove region {index+1}</button></fieldset>{/each}
  <button type="button" class="secondary" onclick={()=>regions=[...regions,{kind:'wall',x:20,y:20,width:5,depth:5,height:10}]}>Add terrain region</button>
  <label>Geometry and placement ruling<textarea required maxlength="4000" bind:value={reason}></textarea></label>
  {#if error}<p role="alert">{error}</p>{/if}<button type="submit">Prepare encounter map</button>
</fieldset></form>

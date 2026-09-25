<script lang="ts">
  import { untrack } from 'svelte';
  import { newId, type CharacterView, type CreatureView } from '../table-api';
  import type { BattlefieldSetup } from '../tactical-api';
  let { characters, creatures = [], ownedSourceActors = [], disabled = false, onPrepare }: { characters: CharacterView[]; creatures?: CreatureView[]; ownedSourceActors?: string[]; disabled?: boolean; onPrepare: (setup: BattlefieldSetup) => void } = $props();
  let name = $state('Encounter'); let width = $state(50); let depth = $state(50);
  let areaPolicy = $state(false);
  let light = $state<'Bright'|'Dim'|'Darkness'>('Bright');
  let reason = $state('The host established the terrain and starting positions from the current scene.');
  const footprint = (size:string) => ({Tiny:2.5,Small:5,Medium:5,Large:10,Huge:15,Gargantuan:20}[size] ?? 5);
  let positions = $state(untrack(() => [
    ...characters.map((character, index) => ({ actor:character.entity_id, characterId:character.character_id as string|null, name:character.name, publicLabel:character.name, prepared:!!character.equipment?.prepared, space:footprint(character.profile?.size??'Medium'), included:true, x:5+index*5, y:5, height:6, team:'Party' })),
    ...creatures.map((creature,index) => ({ actor:creature.actor, characterId:null, name:creature.name, publicLabel:'Creature', prepared:true, space:footprint(creature.size), included:creature.hp>0, x:25, y:5+index*15, height:6, team:'Opposition' }))
  ]));
  let regions = $state<{ kind: 'wall'|'difficult'; x: number; y: number; width: number; depth: number; height: number }[]>([]);
  let error = $state('');
  function submit(event: SubmitEvent) {
    event.preventDefault(); error = '';
    const selected = positions.filter(p => p.included);
    if (!selected.some(p=>p.characterId || ownedSourceActors.includes(p.actor)) || selected.some(p => !p.prepared)) { error = "Select an attending player's actor and prepare every selected character's equipment first."; return; }
    if (selected.some(p => p.x < 0 || p.y < 0 || p.x + p.space > width || p.y + p.space > depth)) { error = 'Place every participant inside the map.'; return; }
    if (regions.some(r => r.x < 0 || r.y < 0 || r.x + r.width > width || r.y + r.depth > depth)) { error = 'Place each terrain region inside the map.'; return; }
    const volume = (r: typeof regions[number]) => ({ min: { x: r.x*2, y: r.y*2, z: 0 }, max: { x: (r.x+r.width)*2, y: (r.y+r.depth)*2, z: r.height*2 } });
    const placement = (p:typeof selected[number]) => ({position:{x:p.x*2,y:p.y*2,z:0},height:p.height*2,
      allies:selected.filter(other=>other.actor!==p.actor&&p.team.trim()&&other.team.trim()===p.team.trim()).map(other=>other.actor),
      enemies:selected.filter(other=>p.team.trim()&&other.team.trim()&&other.team.trim()!==p.team.trim()).map(other=>other.actor)});
    onPrepare({ encounter_id: newId(), scene_id: newId(), location_id: newId(), name,
      battlefield: { bounds: { min: {x:0,y:0,z:0}, max: {x:width*2,y:depth*2,z:80} }, floor_z:0,floor_surface:'ground',ambient_light:light,
        obstacles: regions.filter(r=>r.kind==='wall').map((r,index)=>({id:`wall-${index}`,volume:volume(r),blocks_movement:true,blocks_sight:true,observable:true,cover:'Total'})),
        terrain: regions.filter(r=>r.kind==='difficult').map((r,index)=>({id:`terrain-${index}`,volume:volume(r),difficult:true,observable:true,water:false,climbable:false,burrowable:false,supports_top:false,surface:null,obscuration:'None',magical_darkness:false})), lights:[] },
      characters: selected.filter(p=>p.characterId).map(p=>({character_id:p.characterId!,...placement(p)})),
      creatures: selected.filter(p=>!p.characterId).map(p=>({actor:p.actor,public_label:p.publicLabel,...placement(p)})),
      area_grid_policy: areaPolicy ? 'OccupiedCellCentersV1' : null,
      geometry_ruling:{basis:'GmAdjudication',reason} });
  }
</script>
<form onsubmit={submit}><fieldset {disabled}><legend>Prepare an encounter map</legend>
  <p>Place the current scene in feet. Characters keep their saved equipment and source movement speeds.</p>
  <label>Location name<input required maxlength="200" bind:value={name} /></label>
  <div class="form-grid"><label>Width (feet)<input type="number" min="10" max="250" step="5" required bind:value={width} /></label><label>Depth (feet)<input type="number" min="10" max="250" step="5" required bind:value={depth} /></label><label>Light<select bind:value={light}><option>Bright</option><option>Dim</option><option>Darkness</option></select></label></div>
  <fieldset><legend>Area targeting on this map</legend>
    <label><input type="checkbox" bind:checked={areaPolicy} />Use occupied-space sampling for area abilities</label>
    <p>For each creature, test the center of every occupied part of a 5-foot square and height band. Small portions use their own midpoint, rounded down to the nearest half foot. One reachable point inside the shape includes the creature; a completely blocked creature is excluded.</p>
    <p>Solid terrain blocking at least half or three quarters of those points grants half or three-quarters cover. Authored cover and intervening creatures use the greatest cover benefit. Areas follow clear paths from the chosen origin and do not spread around corners. Leave this unchecked to keep area abilities unavailable on this map.</p>
  </fieldset>
  <h3>Starting positions</h3>
  <p>Matching team names are allies; different named teams are enemies. Leave a team blank for neutral participants.</p>
  {#each positions as position}<div class="form-grid"><label><input type="checkbox" bind:checked={position.included} />{position.name}{position.prepared ? '' : ' — equipment needs preparation'}</label><label>{position.name}: east (feet)<input type="number" min="0" max={width-position.space} step="0.5" required disabled={!position.included} bind:value={position.x} /></label><label>{position.name}: south (feet)<input type="number" min="0" max={depth-position.space} step="0.5" required disabled={!position.included} bind:value={position.y} /></label><label>{position.name}: height (feet)<input type="number" min="0.5" max="40" step="0.5" required disabled={!position.included} bind:value={position.height} /></label>{#if !position.characterId}<label>{position.name}: visible description<input required maxlength="200" disabled={!position.included} bind:value={position.publicLabel} /></label>{/if}<label>{position.name}: team<input maxlength="100" disabled={!position.included} bind:value={position.team} /></label></div>{/each}
  <h3>Terrain</h3>
  {#each regions as region,index}<fieldset><legend>Region {index+1}</legend><div class="form-grid"><label>Kind<select bind:value={region.kind}><option value="wall">Opaque solid obstacle</option><option value="difficult">Difficult ground</option></select></label><label>East (feet)<input type="number" min="0" step="5" required bind:value={region.x} /></label><label>South (feet)<input type="number" min="0" step="5" required bind:value={region.y} /></label><label>Width (feet)<input type="number" min="5" step="5" required bind:value={region.width} /></label><label>Depth (feet)<input type="number" min="5" step="5" required bind:value={region.depth} /></label><label>Height (feet)<input type="number" min="0.5" max="40" step="0.5" required bind:value={region.height} /></label></div><button type="button" class="secondary" onclick={()=>regions=regions.filter((_,i)=>i!==index)}>Remove region {index+1}</button></fieldset>{/each}
  <button type="button" class="secondary" onclick={()=>regions=[...regions,{kind:'wall',x:20,y:20,width:5,depth:5,height:10}]}>Add terrain region</button>
  <label>Geometry and placement ruling<textarea required maxlength="4000" bind:value={reason}></textarea></label>
  {#if error}<p role="alert">{error}</p>{/if}<button type="submit">Prepare encounter map</button>
</fieldset></form>

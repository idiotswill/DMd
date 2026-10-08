<script lang="ts">
  import { untrack } from 'svelte';
  import { newId, type CharacterView, type CreatureView } from '../table-api';
  import type { BattlefieldSetup } from '../tactical-api';
  let { characters, creatures = [], ownedSourceActors = [], requiredActors = [], disabled = false, onPrepare }: { characters: CharacterView[]; creatures?: CreatureView[]; ownedSourceActors?: string[]; requiredActors?: string[]; disabled?: boolean; onPrepare: (setup: BattlefieldSetup) => void } = $props();
  let name = $state('Encounter'); let width = $state(50); let depth = $state(50);
  let areaPolicy = $state(false);
  let light = $state<'Bright'|'Dim'|'Darkness'>('Bright');
  let reason = $state('The host established the terrain and starting positions from the current scene.');
  const footprint = (size:string) => ({Tiny:2.5,Small:5,Medium:5,Large:10,Huge:15,Gargantuan:20}[size] ?? 5);
  let positions = $state(untrack(() => [
    ...characters.map((character, index) => ({ actor:character.entity_id, characterId:character.character_id as string|null, name:character.name, publicLabel:character.name, prepared:!!character.equipment?.prepared, space:footprint(character.profile?.size??'Medium'), included:true, x:5+index*5, y:5, z:0, height:6, team:'Party' })),
    ...creatures.map((creature,index) => ({ actor:creature.actor, characterId:null, name:creature.name, publicLabel:'Creature', prepared:true, space:footprint(creature.size), included:creature.hp>0||requiredActors.includes(creature.actor), x:25, y:5+index*15, z:0, height:6, team:'Opposition' }))
  ]));
  let regions = $state<{ kind: 'wall'|'difficult'|'water'|'platform'; x: number; y: number; z: number; width: number; depth: number; height: number }[]>([]);
  let error = $state('');
  const bounded = (value:number, min:number, max:number, step=0.5) => Number.isFinite(value) && value>=min && value<=max && Number.isSafeInteger(value/step);
  function submit(event: SubmitEvent) {
    event.preventDefault(); error = '';
    if (disabled) return;
    const selected = positions.filter(p => p.included);
    if (!bounded(width,10,250,5) || !bounded(depth,10,250,5)) { error='Use map dimensions from 10 to 250 feet in 5-foot increments.'; return; }
    if (selected.length>100) { error='A battlefield supports at most 100 participants.'; return; }
    if (requiredActors.some(actor=>!selected.some(position=>position.actor===actor))) { error='Include every actor with a lasting consequence and ensure their controllers are attending this session.'; return; }
    if (!selected.some(p=>p.characterId || ownedSourceActors.includes(p.actor)) || selected.some(p => !p.prepared)) { error = "Select an attending player's actor and prepare every selected character's equipment first."; return; }
    if (selected.some(p => !bounded(p.x,0,width-p.space) || !bounded(p.y,0,depth-p.space) || !bounded(p.height,0.5,40) || !bounded(p.z,0,40-p.height))) { error = 'Place every participant inside the map, including their height, using half-foot increments.'; return; }
    if (regions.filter(r=>r.kind==='wall').length>512 || regions.filter(r=>r.kind!=='wall').length>512) { error='Use at most 512 solid obstacles and 512 terrain regions.'; return; }
    if (regions.some(r => !bounded(r.width,0.5,width) || !bounded(r.depth,0.5,depth) || !bounded(r.x,0,width-r.width) || !bounded(r.y,0,depth-r.depth) || !bounded(r.height,0.5,40) || !bounded(r.z,0,40-r.height))) { error = 'Place each terrain region inside the map, including its top, using half-foot increments.'; return; }
    const volume = (r: typeof regions[number]) => ({ min: { x: r.x*2, y: r.y*2, z: r.z*2 }, max: { x: (r.x+r.width)*2, y: (r.y+r.depth)*2, z: (r.z+r.height)*2 } });
    const placement = (p:typeof selected[number]) => ({position:{x:p.x*2,y:p.y*2,z:p.z*2},height:p.height*2,
      allies:selected.filter(other=>other.actor!==p.actor&&p.team.trim()&&other.team.trim()===p.team.trim()).map(other=>other.actor),
      enemies:selected.filter(other=>p.team.trim()&&other.team.trim()&&other.team.trim()!==p.team.trim()).map(other=>other.actor)});
    onPrepare({ encounter_id: newId(), scene_id: newId(), location_id: newId(), name,
      battlefield: { bounds: { min: {x:0,y:0,z:0}, max: {x:width*2,y:depth*2,z:80} }, floor_z:0,floor_surface:'ground',ambient_light:light,
        obstacles: regions.filter(r=>r.kind==='wall').map((r,index)=>({id:`wall-${index}`,volume:volume(r),blocks_movement:true,blocks_sight:true,observable:true,cover:'Total'})),
        terrain: regions.filter(r=>r.kind!=='wall').map((r,index)=>({id:`terrain-${index}`,volume:volume(r),difficult:r.kind==='difficult',observable:true,water:r.kind==='water',climbable:false,burrowable:false,supports_top:r.kind==='platform',surface:r.kind==='difficult'?null:`terrain-${index}`,obscuration:'None',magical_darkness:false})), lights:[] },
      characters: selected.filter(p=>p.characterId).map(p=>({character_id:p.characterId!,...placement(p)})),
      creatures: selected.filter(p=>!p.characterId).map(p=>({actor:p.actor,public_label:p.publicLabel,...placement(p)})),
      area_grid_policy: areaPolicy ? 'OccupiedCellCentersV1' : null,
      geometry_ruling:{basis:'GmAdjudication',reason} });
  }
</script>
<form onsubmit={submit}><fieldset {disabled}><legend>Prepare battlefield</legend>
  <p>Place the current scene in feet. Elevation is measured from the map floor at 0 feet; the ceiling is 40 feet. Characters keep their saved equipment and source movement speeds.</p>
  <label>Location name<input required maxlength="200" bind:value={name} /></label>
  <div class="form-grid"><label>Width (feet)<input type="number" min="10" max="250" step="5" required bind:value={width} /></label><label>Depth (feet)<input type="number" min="10" max="250" step="5" required bind:value={depth} /></label><label>Light<select bind:value={light}><option>Bright</option><option>Dim</option><option>Darkness</option></select></label></div>
  <fieldset><legend>Area targeting on this map</legend>
    <label><input type="checkbox" bind:checked={areaPolicy} />Use occupied-space sampling for area abilities</label>
    <p>For each creature, test the center of every occupied part of a 5-foot square and height band. Small portions use their own midpoint, rounded down to the nearest half foot. One reachable point inside the shape includes the creature; a completely blocked creature is excluded.</p>
    <p>Solid terrain blocking at least half or three quarters of those points grants half or three-quarters cover. Authored cover and intervening creatures use the greatest cover benefit. Areas follow clear paths from the chosen origin and do not spread around corners. Leave this unchecked to keep area abilities unavailable on this map.</p>
  </fieldset>
  <h3>Starting positions</h3>
  <p>Matching team names are allies; different named teams are enemies. Leave a team blank for neutral participants.</p>
  <p>Elevation locates the bottom of a creature's occupied space. Height is its vertical extent. The rules validate support, movement and any resulting fall.</p>
  {#each positions as position}<div class="form-grid"><label><input type="checkbox" bind:checked={position.included} />{position.name}{position.prepared ? '' : ' — equipment needs preparation'}</label><label>{position.name}: east (feet)<input type="number" min="0" max={width-position.space} step="0.5" required disabled={!position.included} bind:value={position.x} /></label><label>{position.name}: south (feet)<input type="number" min="0" max={depth-position.space} step="0.5" required disabled={!position.included} bind:value={position.y} /></label><label>{position.name}: elevation (feet)<input type="number" min="0" max={40-position.height} step="0.5" required disabled={!position.included} bind:value={position.z} /></label><label>{position.name}: height (feet)<input type="number" min="0.5" max="40" step="0.5" required disabled={!position.included} bind:value={position.height} /></label>{#if !position.characterId}<label>{position.name}: visible description<input required maxlength="200" disabled={!position.included} bind:value={position.publicLabel} /></label>{/if}<label>{position.name}: team<input maxlength="100" disabled={!position.included} bind:value={position.team} /></label></div>{/each}
  <h3>Terrain</h3>
  <p>Water fills its volume. A platform supports creatures on its dry top and leaves space beneath open; use an opaque solid obstacle for a solid ledge. Separate platforms can leave a gap. Region height is measured from its bottom elevation.</p>
  {#each regions as region,index}<fieldset><legend>Region {index+1}</legend><div class="form-grid"><label>Kind<select bind:value={region.kind}><option value="wall">Opaque solid obstacle</option><option value="difficult">Difficult ground</option><option value="water">Water</option><option value="platform">Dry platform or bridge</option></select></label><label>East (feet)<input type="number" min="0" max={width-region.width} step="0.5" required bind:value={region.x} /></label><label>South (feet)<input type="number" min="0" max={depth-region.depth} step="0.5" required bind:value={region.y} /></label><label>Width (feet)<input type="number" min="0.5" max={width} step="0.5" required bind:value={region.width} /></label><label>Depth (feet)<input type="number" min="0.5" max={depth} step="0.5" required bind:value={region.depth} /></label><label>Bottom elevation (feet)<input type="number" min="0" max={40-region.height} step="0.5" required bind:value={region.z} /></label><label>Height (feet)<input type="number" min="0.5" max="40" step="0.5" required bind:value={region.height} /></label></div><p>Top elevation: {region.z+region.height} feet.</p><button type="button" class="secondary" onclick={()=>regions=regions.filter((_,i)=>i!==index)}>Remove region {index+1}</button></fieldset>{/each}
  <button type="button" class="secondary" disabled={regions.length>=1024} onclick={()=>regions=[...regions,{kind:'wall',x:20,y:20,z:0,width:5,depth:5,height:10}]}>Add terrain region</button>
  <label>Geometry and placement ruling<textarea required maxlength="4000" bind:value={reason}></textarea></label>
  {#if error}<p role="alert">{error}</p>{/if}<button type="submit">Prepare battlefield</button>
</fieldset></form>

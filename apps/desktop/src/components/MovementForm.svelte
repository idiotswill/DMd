<script lang="ts">
  import type { MovementOptions, TacticalAction } from '../tactical-api';
  import { proposeMovement } from '../movement-intent';
  let {options,groundDrag=[],disabled=false,onAction}:{groundDrag?:{key:string;actor:string;label:string}[];options:MovementOptions;disabled?:boolean;onAction:(action:TacticalAction)=>void}=$props();
  let selectedDrag=$state('');
  $effect(()=>{if(selectedDrag && !groundDrag.some(offer=>offer.key===selectedDrag))selectedDrag='';});
  let route=$state('');const proposal=$derived(proposeMovement(route,options));
  const destination=$derived(proposal.path?.at(-1)?.destination);
  function submit(event:SubmitEvent){event.preventDefault();if(proposal.path)onAction(selectedDrag?{MoveGrappled:{option:selectedDrag,path:proposal.path}}:options.self_only_required?{MoveSelfOnly:{path:proposal.path}}:{Move:{path:proposal.path}});}
</script>
<form onsubmit={submit}><fieldset {disabled}><legend>Move</legend>
  <p>Describe each part of your route. North is toward the top of the map.</p>
  {#if groundDrag.length}<label>Move with<select bind:value={selectedDrag}>
    <option value="">Only my creature</option>
    {#each groundDrag as offer (offer.key)}<option value={offer.key}>{offer.label}</option>{/each}
  </select></label>{/if}
  {#if selectedDrag}<p>Drag one held creature along the ground. Use walking or crawling on a continuously supported dry route. Moving costs 1 extra foot per foot unless the creature is Tiny or at least two sizes smaller.</p>{/if}
  <label>Movement route<input bind:value={route} maxlength="512" placeholder="walk 10 feet north, then 5 feet east" required/></label>
  <p>Available movement: {options.modes.join(', ')}.</p>
  {#if route.trim() && proposal.error}<p role="status">{proposal.error}</p>{/if}
  {#if destination}<p>Destination: {destination.x/2}, {destination.y/2} feet; height {destination.z/2} feet. Terrain and reactions may stop your movement along the route.</p>{/if}
  {#if options.self_only_required && !selectedDrag}<p>Held creatures stay where they are. Moving out of reach ends the grip.</p>{/if}
  <button disabled={!proposal.path}>{selectedDrag?'Drag the selected creature along this route':options.self_only_required?'Move only my creature along this route':'Follow this route'}</button>
</fieldset></form>

<script lang="ts">
  import type { MovementOptions, TacticalAction } from '../tactical-api';
  import { proposeMovement } from '../movement-intent';
  let {options,disabled=false,onAction}:{options:MovementOptions;disabled?:boolean;onAction:(action:TacticalAction)=>void}=$props();
  let route=$state('');const proposal=$derived(proposeMovement(route,options));
  const destination=$derived(proposal.path?.at(-1)?.destination);
  function submit(event:SubmitEvent){event.preventDefault();if(proposal.path)onAction({Move:{path:proposal.path}});}
</script>
<form onsubmit={submit}><fieldset {disabled}><legend>Move</legend>
  <p>Describe each part of your route. North is toward the top of the map.</p>
  <label>Movement route<input bind:value={route} maxlength="512" placeholder="walk 10 feet north, then 5 feet east" required/></label>
  <p>Available movement: {options.modes.join(', ')}.</p>
  {#if route.trim() && proposal.error}<p role="status">{proposal.error}</p>{/if}
  {#if destination}<p>Destination: {destination.x/2}, {destination.y/2} feet; height {destination.z/2} feet. Terrain and reactions may stop your movement along the route.</p>{/if}
  <button disabled={!proposal.path}>Follow this route</button>
</fieldset></form>

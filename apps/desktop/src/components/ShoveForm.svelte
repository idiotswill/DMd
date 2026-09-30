<script lang="ts">
  import type { Id } from '../table-api';
  import type { TacticalAction } from '../tactical-api';
  let { targets, disabled=false, onAction }: { targets:{entity_id:Id;public_label:string}[];disabled?:boolean;onAction:(action:TacticalAction)=>void }=$props();
  let target=$state('');
</script>
<fieldset {disabled}><legend>Unarmed strike · Shove</legend>
  <p>Spend an Attack action attack. A creature within five feet and at most one size larger chooses a Strength or Dexterity save. On failure, choose Prone or a five-foot push away. This currently requires opposing creatures with at least one controlled by the host; other relationships need table consent.</p>
  <label>Shove target<select bind:value={target}><option value="">Choose a known creature</option>{#each targets as candidate}<option value={candidate.entity_id}>{candidate.public_label}</option>{/each}</select></label>
  <button disabled={!targets.some(candidate=>candidate.entity_id===target)} onclick={()=>onAction({Shove:{target}})}>Attempt Shove</button>
</fieldset>

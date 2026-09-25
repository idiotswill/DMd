<script lang="ts">
  import type { Id } from '../table-api';
  import type { TacticalAction } from '../tactical-api';
  let { targets, disabled=false, onAction }: {
    targets:{entity_id:Id;public_label:string}[]; disabled?:boolean; onAction:(action:TacticalAction)=>void;
  }=$props();
  let target=$state('');
</script>
<fieldset {disabled}><legend>Unarmed strike · damage</legend>
  <p>Use one attack from your Attack action to punch, kick or headbutt a creature within 5 feet. Add your Strength modifier and Proficiency Bonus to the attack roll. Damage is 1 plus your Strength modifier, to a minimum of 0, with no damage die. You can kick with both hands occupied.</p>
  <label>Unarmed strike target<select bind:value={target}><option value="">Choose a known creature</option>{#each targets as candidate}<option value={candidate.entity_id}>{candidate.public_label}</option>{/each}</select></label>
  <button disabled={!targets.some(candidate=>candidate.entity_id===target)} onclick={()=>onAction({UnarmedStrike:{target}})}>Make unarmed strike</button>
</fieldset>

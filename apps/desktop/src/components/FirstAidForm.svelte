<script lang="ts">
  import type { Id } from '../table-api';
  import type { TacticalAction } from '../tactical-api';
  let { targets, disabled=false, onAction }: {
    targets: { entity_id:Id; public_label:string }[];
    disabled?:boolean; onAction:(action:TacticalAction)=>void;
  }=$props();
  let target=$state('');
  let purpose=$state<'Stabilize'|'EndKnockout'>('Stabilize');
</script>
<fieldset {disabled}><legend>First aid</legend>
  <p>Spend your Action to attempt a DC 10 Wisdom (Medicine) check. At this table, first aid requires a located creature within 5 feet and an unobstructed contact path.</p>
  <label>Creature<select bind:value={target}><option value="">Choose a known creature</option>{#each targets as candidate}<option value={candidate.entity_id}>{candidate.public_label}</option>{/each}</select></label>
  <label>Care needed<select bind:value={purpose}><option value="Stabilize">Stabilize at zero hit points</option><option value="EndKnockout">Wake from a melee knockout</option></select></label>
  <p>First aid restores no hit points. Stabilizing leaves the creature unconscious; waking from a knockout does not remove other causes of unconsciousness.</p>
  <button disabled={!targets.some(candidate=>candidate.entity_id===target)} onclick={()=>onAction({FirstAid:{target,purpose}})}>Administer first aid</button>
</fieldset>

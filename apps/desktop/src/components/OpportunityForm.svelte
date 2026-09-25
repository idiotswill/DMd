<script lang="ts">
  import type {OpportunityView,TacticalAction} from '../tactical-api';
  import AttackForm from './AttackForm.svelte';
  let {opportunity,disabled=false,onAction}:{opportunity:OpportunityView;disabled?:boolean;onAction:(action:TacticalAction)=>void}=$props();
  function weapon(action:TacticalAction){if(typeof action==='object'&&'Attack' in action)onAction({OpportunityAttack:{choice:{Weapon:action.Attack.choice}}});}
</script>
<section><h3>Opportunity attack</h3>
  <p>{opportunity.target.label} is leaving your reach. You may spend your reaction before the creature moves away.</p>
  {#if opportunity.weapons?.weapons.length}<AttackForm options={opportunity.weapons} {disabled} opportunity onAction={weapon}/>{/if}
  <fieldset {disabled}><legend>Other reaction choices</legend>
    {#if opportunity.unarmed}<button onclick={()=>onAction({OpportunityAttack:{choice:{UnarmedDamage:{ability:'Strength'}}}})}>Unarmed Strike (Strength damage)</button>{/if}
    {#each opportunity.features as feature}<button onclick={()=>onAction({OpportunityAttack:{choice:{CreatureFeature:{feature_id:feature.feature_id,weapon:feature.weapon}}}})}>Use {feature.label}</button>{/each}
    <button class="secondary" onclick={()=>onAction('DeclineOpportunity')}>Let the creature pass</button>
  </fieldset>
</section>

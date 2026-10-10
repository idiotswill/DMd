<script lang="ts">
  import type { AttackEquipmentView, TacticalAction } from '../tactical-api';
  import { tacticalPromptIdentity } from '../tactical-focus';
  let { choice, disabled=false, onAction }: { choice:AttackEquipmentView; disabled?:boolean; onAction:(action:TacticalAction)=>void }=$props();
  let selected=$state('');
  let previousWork=$state('');
  const operation=$derived(choice.operations.find(option=>JSON.stringify(option.operation)===selected));
  $effect(()=>{
    const work=JSON.stringify([choice.actor,choice.key]);
    if(previousWork!==work) { previousWork=work; selected=''; }
    if(!operation) selected='';
  });
</script>
<fieldset {disabled} data-tactical-focus="prompt" data-tactical-focus-id={tacticalPromptIdentity({kind:'attack-equipment',actor:choice.actor,key:choice.key})} tabindex="-1">
  <legend>Equipment after your attack</legend>
  {#if choice.operations.length}
    <label>Equipment operation<select bind:value={selected}><option value="" disabled>Choose an operation</option>{#each choice.operations as option}<option value={JSON.stringify(option.operation)}>{option.label}</option>{/each}</select></label>
    <button disabled={!operation} onclick={()=>operation&&onAction({AttackEquipment:{handle:choice.key,choice:{Apply:operation.operation}}})}>Apply equipment choice</button>
  {:else}<p>No equipment operation is currently available.</p>{/if}
  {#if choice.may_decline}<button class="secondary" onclick={()=>onAction({AttackEquipment:{handle:choice.key,choice:'Decline'}})}>Decline equipment choice</button>{/if}
</fieldset>

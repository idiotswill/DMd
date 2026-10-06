<script lang="ts">
  import type { PhysicalSourceWeaponChoice, TacticalAction, WeaponGrip } from '../tactical-api';
  let {choices,disabled=false,onAction}:{choices:PhysicalSourceWeaponChoice[];disabled?:boolean;onAction:(action:TacticalAction)=>void}=$props();
  const choiceKey=(choice:PhysicalSourceWeaponChoice)=>JSON.stringify([choice.feature_id,choice.item]);
  const gripLabel=(grip:WeaponGrip)=>grip==='TwoHands'?'Both hands':`${grip.OneHand} hand`;
  const itemNumbers=new Map<string,number>();
  function itemNumber(id:string) { if(!itemNumbers.has(id))itemNumbers.set(id,itemNumbers.size+1);return itemNumbers.get(id); }
  let selection=$state('');
  let gripKey=$state('');
  const selected=$derived(choices.find(choice=>choiceKey(choice)===selection));
  const grip=$derived(selected?.grips.find(candidate=>JSON.stringify(candidate)===gripKey));
  $effect(()=>{
    if(selection&&!selected){selection='';gripKey='';}
    else if(gripKey&&!grip)gripKey='';
  });
  function choose(key:string) {
    selection=key;
    const choice=choices.find(candidate=>choiceKey(candidate)===key);
    gripKey=choice?.grips[0]?JSON.stringify(choice.grips[0]):'';
  }
  // Do not select a different Item, source action or grip when a refresh removes
  // an option. A new response requires another explicit legal selection.
  function submit(event:SubmitEvent) {
    event.preventDefault();
    if(disabled||!selected||!grip)return;
    onAction({OpportunityAttack:{choice:{CreatureWeapon:{feature_id:selected.feature_id,weapon:selected.item,grip}}}});
  }
</script>
<form onsubmit={submit}>
  <fieldset {disabled}><legend>Source weapon reaction</legend>
    <label>Reaction source weapon<select value={selection} onchange={(event)=>choose(event.currentTarget.value)} required>
      {#if !selected}<option value={selection} disabled>Choose a held source weapon</option>{/if}
      {#each choices as choice (choiceKey(choice))}<option value={choiceKey(choice)}>{choice.label} · {choice.weapon_name} · {itemNumber(choice.item)}</option>{/each}
    </select></label>
    {#if selected}<label>Reaction weapon grip<select bind:value={gripKey} required>
      {#if !grip}<option value={gripKey} disabled>Choose an available grip</option>{/if}
      {#each selected.grips as choice (JSON.stringify(choice))}<option value={JSON.stringify(choice)}>{gripLabel(choice)}</option>{/each}
    </select></label>{/if}
    <button disabled={!selected||!grip}>Use source weapon reaction</button>
  </fieldset>
</form>

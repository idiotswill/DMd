<script lang="ts">
  import { rawDice, signed, type RollRequest } from '../table-api';
  import type { SavageAttackerRoll } from '../tactical-api';
  let { request, disabled = false, onSubmit, savageOption = null, onSavage }: {
    request: RollRequest; disabled?: boolean; onSubmit: (faces: number[]) => void;
    savageOption?: { weapon_dice: number; heroic_inspiration: boolean } | null;
    onSavage?: (roll: SavageAttackerRoll) => void;
  } = $props();
  const sides = $derived(rawDice(request));
  let faces = $state<number[]>([]);
  let savage = $state(false); let second = $state<number[]>([]);
  let chosen = $state<''|'First'|'Second'>('');
  let inspiration = $state(false); let inspirationDie = $state(''); let replacement = $state<number>();
  let error = $state('');
  const rerollSides = $derived(sides[Number(inspirationDie.split(':')[1])]);
  const valid = (values: number[], dice: number[]) => values.length===dice.length&&dice.every((side,i)=>Number.isInteger(values[i])&&values[i]>=1&&values[i]<=side);
  function submit(event: SubmitEvent) {
    event.preventDefault();
    if (!valid(faces,sides)) { error = 'Enter one valid raw face for every requested die.'; return; }
    if(savage&&savageOption&&onSavage) {
      const count=savageOption.weapon_dice;
      if(!valid(second,sides.slice(0,count))||!chosen) { error='Report both weapon dice sets and choose which set to use.';return; }
      if(inspiration&&(!savageOption.heroic_inspiration||!inspirationDie||!Number.isInteger(replacement)||replacement!<1||replacement!>rerollSides)) { error='Select the die and report its valid Inspiration replacement.';return; }
      const raw=(values:number[])=>({request_id:request.id,source:'Physical' as const,dice:values.map((value,i)=>({sides:sides[i],value}))});
      error='';onSavage({weapon_dice:count,first:raw(faces),second:raw([...second,...faces.slice(count)]),chosen,
        inspiration:inspiration?{roll:inspirationDie.split(':')[0] as 'First'|'Second',die_index:Number(inspirationDie.split(':')[1]),replacement:{sides:rerollSides,value:replacement!}}:null});return;
    }
    error = ''; onSubmit([...faces]);
  }
</script>
<form onsubmit={submit}><fieldset {disabled}>
  <legend>Report physical dice</legend><p>{request.reason}</p>
  <p>{request.mode === 'Normal' ? 'Normal roll' : request.mode} · rules modifier {signed(request.modifier)}. Enter the faces exactly as rolled; do not add the modifier.</p>
  <div class="form-grid">{#each sides as side, i}<label>Die {i+1} · d{side}<input required type="number" min="1" max={side} step="1" bind:value={faces[i]} /></label>{/each}</div>
  {#if savageOption&&onSavage}
    <label><input type="checkbox" bind:checked={savage} /> Use Savage Attacker (once per turn)</label>
    {#if savage}
      <p>The dice above are the first set. Roll only the weapon dice again; additional damage dice are shared.</p>
      <div class="form-grid">{#each sides.slice(0,savageOption.weapon_dice) as side,i}<label>Second weapon die {i+1} · d{side}<input required type="number" min="1" max={side} step="1" bind:value={second[i]} /></label>{/each}</div>
      {#if savageOption.heroic_inspiration}
        <label><input type="checkbox" bind:checked={inspiration} /> Spend Heroic Inspiration to reroll one die</label>
        {#if inspiration}<label>Die to reroll<select required bind:value={inspirationDie}><option value="">Select a die</option>{#each sides as side,i}<option value={`First:${i}`}>{i<savageOption.weapon_dice?'First weapon':'Shared additional'} die {i+1} · d{side}</option>{#if i<savageOption.weapon_dice}<option value={`Second:${i}`}>Second weapon die {i+1} · d{side}</option>{/if}{/each}</select></label><label>Inspiration replacement<input required type="number" min="1" max={rerollSides} step="1" bind:value={replacement} /></label>{/if}
      {/if}
      <label>Damage set to use<select required bind:value={chosen}><option value="">Choose a set</option><option value="First">First weapon set</option><option value="Second">Second weapon set</option></select></label>
    {/if}
  {/if}
  {#if error}<p role="alert" class="error">{error}</p>{/if}<button type="submit">Report these faces</button>
</fieldset></form>

<script lang="ts">
  import type { ShieldOptions, TacticalAction, Hand } from '../tactical-api';
  let { options, disabled=false, onAction }: { options:ShieldOptions;disabled?:boolean;onAction:(action:TacticalAction)=>void }=$props();
  let selected=$state(''); let hand=$state<Hand>('Left');
  const shield=$derived(options.shields.find(choice=>choice.item===selected));
  $effect(()=>{
    if(!options.shields.some(choice=>choice.item===selected)) selected=options.shields[0]?.item??'';
    if(shield&&!shield.hands.includes(hand)) hand=shield.hands[0];
  });
</script>
<fieldset {disabled}><legend>Shield preparation · action</legend>
  {#if options.donned}
    <p>Remove and put away the donned shield using your Action. It stays carried.</p>
    <button onclick={()=>onAction('DoffShield')}>Doff shield</button>
  {:else if shield}
    <p>Don a carried shield using your Action. Only shield training grants its armor benefit.</p>
    <label>Carried shield<select bind:value={selected}>{#each options.shields as choice,index}<option value={choice.item}>Shield {index+1}</option>{/each}</select></label>
    <label>Shield hand<select bind:value={hand}>{#each shield.hands as choice}<option value={choice}>{choice} hand</option>{/each}</select></label>
    <button disabled={!shield.hands.includes(hand)} onclick={()=>onAction({DonShield:{shield:selected,hand}})}>Don shield</button>
  {/if}
</fieldset>

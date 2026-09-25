<script lang="ts">
  import type { CastingOptions, TacticalAction } from '../tactical-api';
  let { options, disabled=false, onAction }: {
    options:CastingOptions; disabled?:boolean; onAction:(action:TacticalAction)=>void;
  }=$props();
  let chosen=$state('');
  let targets=$state<string[]>([]);
  const selected=$derived(options.variants.find(variant=>JSON.stringify(variant.choice)===chosen));
  function choose(value:string) {
    chosen=value;
    const variant=options.variants.find(candidate=>JSON.stringify(candidate.choice)===value);
    targets=Array(variant?.minimum_targets??0).fill('');
  }
  $effect(()=>{
    if(!selected) { if(chosen) chosen=''; if(targets.length) targets=[]; return; }
    const updated=targets.slice(0,selected.maximum_targets).map(target=>selected.targets.some(candidate=>candidate.actor===target)?target:'');
    while(updated.length<selected.minimum_targets) updated.push('');
    if(JSON.stringify(updated)!==JSON.stringify(targets)) targets=updated;
  });
  const valid=$derived(!!selected && selected.choice.actor===options.actor
    && targets.length>=selected.minimum_targets && targets.length<=selected.maximum_targets
    && targets.every(target=>selected.targets.some(candidate=>candidate.actor===target))
    && (selected.repeated_targets || new Set(targets).size===targets.length));
  function submit(event:SubmitEvent) {
    event.preventDefault();
    if(disabled || !valid || !selected) return;
    onAction({CastSpell:{choice:selected.choice,targets:{Entities:[...targets]}}});
  }
</script>

<form onsubmit={submit}>
  <fieldset {disabled}><legend>Cast a spell</legend>
    {#if options.variants.length}
      <label>Spell and resource<select value={chosen} onchange={(event)=>choose(event.currentTarget.value)} required>
        <option value="" disabled>Choose a spell</option>
        {#each options.variants as variant,index}<option value={JSON.stringify(variant.choice)}>{variant.label} · {index+1}</option>{/each}
      </select></label>
      {#if selected}
        {#if selected.concentration}<p>This spell requires concentration and replaces the caster's existing concentration.</p>{/if}
        <div class="form-grid">
          {#each targets as target,index}
            <label>Spell target {index+1}<select value={target} onchange={(event)=>{targets=targets.map((value,at)=>at===index?event.currentTarget.value:value)}} required>
              <option value="" disabled>Choose a target</option>
              {#each selected.targets as candidate}<option value={candidate.actor} disabled={!selected.repeated_targets&&targets.some((value,at)=>at!==index&&value===candidate.actor)}>{candidate.label}</option>{/each}
            </select></label>
          {/each}
        </div>
        {#if selected.minimum_targets!==selected.maximum_targets}
          <button type="button" class="secondary" disabled={targets.length>=selected.maximum_targets} onclick={()=>targets=[...targets,'']}>Add spell target</button>
          <button type="button" class="secondary" disabled={targets.length<=selected.minimum_targets} onclick={()=>targets=targets.slice(0,-1)}>Remove last spell target</button>
        {/if}
        {#if selected.maximum_targets>1}<p>Targets resolve in the chosen order.{selected.repeated_targets?' You may choose the same target more than once.':''}</p>{/if}
      {/if}
      <p>Actions, resources, components and targeting are checked again when you cast. Physical dice are requested from their controller.</p>
      <button disabled={!valid}>Cast spell</button>
    {/if}
    {#each options.unavailable as reason}<p>{reason}</p>{/each}
  </fieldset>
</form>

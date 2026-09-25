<script lang="ts">
  import { untrack } from 'svelte';
  import type { Id } from '../table-api';
  import type { AreaOptions, TacticalAction } from '../tactical-api';
  let { options, host, player, disabled=false, onAction }: {
    options:AreaOptions;host:boolean;player:Id|null;disabled?:boolean;onAction:(action:TacticalAction)=>void;
  }=$props();
  let feature=$state(''); let consent=$state(false); let includeOrigin=$state(false); let error=$state('');
  let origin=$state({x:0,y:0,z:0}); let toward=$state({x:0,y:0,z:0});
  let fingerprint=$state('');
  $effect(()=>{
    const next=JSON.stringify([options,host,player]);
    if(next!==untrack(()=>fingerprint)) {
      fingerprint=next; feature=''; consent=false; includeOrigin=false; error='';
      const {min,max}=options.source_space;
      origin={x:max.x/2,y:Math.floor((min.y+max.y)/2)/2,z:Math.floor((min.z+max.z)/2)/2};
      toward={...origin,x:origin.x+5};
    }
  });
  const allowed=$derived(options.controller===null ? host : !host&&player===options.controller);
  function submit(event:SubmitEvent) {
    event.preventDefault();error='';
    if(!allowed || !options.variants.some(v=>v.feature_id===feature)) {error='Choose an available ability in its controller’s channel.';return;}
    if(options.controller!==null&&!consent){error='Choose whether to let the host order this ability’s consequences before using it.';return;}
    const values=[...Object.values(origin),...Object.values(toward)];
    if(values.some(value=>!Number.isFinite(value)||!Number.isInteger(value*2)||Math.abs(value)>100000)) {error='Use coordinates in half-foot increments within the map coordinate limits.';return;}
    if(origin.x===toward.x&&origin.y===toward.y&&origin.z===toward.z){error='Choose a direction away from the origin.';return;}
    const point=(p:typeof origin)=>({x:p.x*2,y:p.y*2,z:p.z*2});
    onAction({CreatureArea:{feature_id:feature,aim:{origin:point(origin),toward:point(toward),include_origin:includeOrigin},ordering:options.controller===null?'Host':'DelegateToHost'}});
  }
  function cancel(){feature='';consent=false;error='';}
</script>
<form onsubmit={submit}><fieldset {disabled}><legend>Area ability</legend>
  {#each options.unavailable as reason}<p>{reason}</p>{/each}
  {#if options.variants.length}
    <p>Aim from your creature’s space using map coordinates in feet. The ability determines its size and affects every creature reached, including allies. Target lists and damage previews are not provided.</p>
    {#if !allowed}<p>Use the source creature’s controller channel to choose this action. The host cannot consent on a player’s behalf.</p>{/if}
    <label>Ability<select required bind:value={feature} disabled={!allowed}><option value="">Choose an ability</option>{#each options.variants as variant}<option value={variant.feature_id}>{variant.label} · {variant.length_feet}-foot cone</option>{/each}</select></label>
    <div class="form-grid">
      <label>Origin east (feet)<input type="number" required step="0.5" min={options.source_space.min.x/2} max={options.source_space.max.x/2} bind:value={origin.x} disabled={!allowed}/></label>
      <label>Origin south (feet)<input type="number" required step="0.5" min={options.source_space.min.y/2} max={options.source_space.max.y/2} bind:value={origin.y} disabled={!allowed}/></label>
      <label>Origin elevation (feet)<input type="number" required step="0.5" min={options.source_space.min.z/2} max={options.source_space.max.z/2} bind:value={origin.z} disabled={!allowed}/></label>
      <label>Aim toward east (feet)<input type="number" required step="0.5" bind:value={toward.x} disabled={!allowed}/></label>
      <label>Aim toward south (feet)<input type="number" required step="0.5" bind:value={toward.y} disabled={!allowed}/></label>
      <label>Aim toward elevation (feet)<input type="number" required step="0.5" bind:value={toward.z} disabled={!allowed}/></label>
    </div>
    <label><input type="checkbox" bind:checked={includeOrigin} disabled={!allowed}/>Include the cone’s origin point</label>
    {#if options.controller!==null}
      <label><input type="checkbox" bind:checked={consent} disabled={!allowed}/>Let the host order all simultaneous consequences of this use</label>
      <p>This permission applies only to this ability and its consequences. It avoids revealing hidden creatures through ordering choices. Each player still decides and rolls their own saves and optional responses. If you do not consent, cancel without spending the ability.</p>
    {/if}
    {#if error}<p role="alert">{error}</p>{/if}
    <button type="submit" disabled={!allowed||!feature||(options.controller!==null&&!consent)}>Use area ability</button>
    <button type="button" class="secondary" onclick={cancel}>Cancel area choice</button>
  {/if}
</fieldset></form>

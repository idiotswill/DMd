<script lang="ts">
  import type { Id } from '../table-api';
  import type { HitDecision, HitView, ReactionUnlistedOrder, TacticalAction } from '../tactical-api';
  let { hit, actor, host, playerControlledSources=[], disabled=false, onAction }: {
    hit:HitView; actor:Id|null; host:boolean; playerControlledSources?:Id[]; disabled?:boolean; onAction:(action:TacticalAction)=>void;
  }=$props();
  let ranked=$state<Id[]>([]);
  let unlisted=$state<ReactionUnlistedOrder|''>('');
  let chosen=$state('');
  const mayOrder=$derived(!!hit.order && (host || actor===hit.order.actor));
  const mayRespond=$derived(!!hit.response && (host ? !playerControlledSources.includes(hit.response.actor) : actor===hit.response.actor));
  const selected=$derived(hit.response?.shield.find(choice=>JSON.stringify(choice)===chosen));
  $effect(()=>{
    const valid=ranked.filter(id=>hit.order?.participants.some(candidate=>candidate.actor===id));
    if(valid.length!==ranked.length) ranked=valid;
    if(chosen && !selected) chosen='';
  });
  function send(handle:Id, decision:HitDecision) {
    if(!disabled) onAction({HitResponse:{handle,decision}});
  }
  function move(index:number, step:number) {
    const next=[...ranked], to=index+step;
    if(to<0 || to>=next.length) return;
    [next[index],next[to]]=[next[to],next[index]]; ranked=next;
  }
</script>

<section aria-label="Hit response">
  {#if hit.order && mayOrder}
    <fieldset {disabled}><legend>Order responses to this hit</legend>
      <p>Choose an order for this hit. You can rank known participants and choose where everyone else belongs.</p>
      {#each hit.order.participants as participant}
        <label><input type="checkbox" value={participant.actor} bind:group={ranked}/>{participant.label}</label>
      {/each}
      {#if ranked.length}<ol>{#each ranked as id,index}<li>
        {hit.order.participants.find(participant=>participant.actor===id)?.label}
        <button type="button" class="secondary" aria-label={`Move ranked participant ${index+1} earlier`} disabled={index===0} onclick={()=>move(index,-1)}>Earlier</button>
        <button type="button" class="secondary" aria-label={`Move ranked participant ${index+1} later`} disabled={index===ranked.length-1} onclick={()=>move(index,1)}>Later</button>
      </li>{/each}</ol>{/if}
      <label>Other participants<select bind:value={unlisted}>
        <option value="" disabled>Choose their order</option>
        <option value="BeforeForward">Before the ranked list, in initiative order</option>
        <option value="BeforeReverse">Before the ranked list, in reverse initiative order</option>
        <option value="AfterForward">After the ranked list, in initiative order</option>
        <option value="AfterReverse">After the ranked list, in reverse initiative order</option>
      </select></label>
      <button disabled={!unlisted} onclick={()=>{if(hit.order&&unlisted)send(hit.order.key,{Order:{instruction:{ranked:[...ranked],unlisted}}});}}>Use this order</button>
      {#if hit.delegate}<button class="secondary" onclick={()=>{if(hit.delegate)send(hit.delegate,'Delegate');}}>Let the host order this hit</button>{/if}
    </fieldset>
  {/if}
  {#if hit.response && mayRespond}
    <fieldset {disabled}><legend>{hit.response.selected?'Resolve your Shield response':'Respond to this hit'}</legend>
      {#if hit.response.selected}
        <p>Cast Shield now to gain +5 AC, including against this attack, until the start of your next turn. A natural 20 still hits.</p>
        <label>Shield resource<select bind:value={chosen}><option value="" disabled>Choose a resource</option>
          {#each hit.response.shield as choice,index}<option value={JSON.stringify(choice)}>{typeof choice.resource==='object'?`Spell slot level ${choice.resource.Slot.level}`:'Source spellcasting use'} · {index+1}</option>{/each}
        </select></label>
        <button disabled={!selected} onclick={()=>{if(hit.response&&selected)send(hit.response.key,{Cast:{choice:selected}});}}>Cast Shield · spend Reaction</button>
        <button class="secondary" onclick={()=>{if(hit.response)send(hit.response.key,'Decline');}}>Continue without casting</button>
      {:else}
        {#if hit.response.shield.length}<p>You can offer Shield. No Reaction or spell resource is spent until you choose to cast it.</p>
          <button onclick={()=>{if(hit.response)send(hit.response.key,{Respond:{accept:true}});}}>Offer Shield response</button>
        {/if}
        <button class="secondary" onclick={()=>{if(hit.response)send(hit.response.key,{Respond:{accept:false}});}}>Continue without Shield</button>
      {/if}
    </fieldset>
  {/if}
  {#if !mayOrder && !mayRespond}<p>Waiting for this hit to resolve.</p>{/if}
</section>

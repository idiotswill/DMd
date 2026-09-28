<script lang="ts">
  import type { Id } from '../table-api';
  import type { MissileDecision, MissileView, ReactionUnlistedOrder, TacticalAction } from '../tactical-api';
  let { missile, actor, host, participants=[], playerControlledSources=[], disabled=false, onAction }: {
    missile:MissileView; actor:Id|null; host:boolean; participants?:{entity_id:Id;public_label:string}[];
    playerControlledSources?:Id[]; disabled?:boolean; onAction:(action:TacticalAction)=>void;
  }=$props();
  let ranked=$state<Id[]>([]);
  let unlisted=$state<ReactionUnlistedOrder|''>('');
  let chosen=$state<Record<Id,string>>({});
  const mayOrder=$derived(!!missile.order && (host || actor===missile.order.actor));
  const responses=$derived(missile.responses.filter(response=>host
    ? !playerControlledSources.includes(response.actor) : actor===response.actor));
  $effect(()=>{
    const valid=ranked.filter(id=>missile.order?.participants.some(candidate=>candidate.actor===id));
    if(valid.length!==ranked.length) ranked=valid;
  });
  function send(handle:Id, decision:MissileDecision) {
    if(!disabled) onAction({MissileResponse:{handle,decision}});
  }
  function move(index:number, step:number) {
    const next=[...ranked], to=index+step;
    if(to<0 || to>=next.length) return;
    [next[index],next[to]]=[next[to],next[index]]; ranked=next;
  }
</script>

<section aria-label="Magic Missile responses">
  {#if missile.order && mayOrder}
    <fieldset {disabled}><legend>Order responses to Magic Missile</legend>
      <p>Choose the order for this targeting event. Rank known participants and choose where everyone else belongs. Dart impacts will be ordered separately.</p>
      {#each missile.order.participants as participant}
        <label><input type="checkbox" value={participant.actor} bind:group={ranked}/>{participant.label}</label>
      {/each}
      {#if ranked.length}<ol>{#each ranked as id,index}<li>
        {missile.order.participants.find(participant=>participant.actor===id)?.label}
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
      <button disabled={!unlisted} onclick={()=>{if(missile.order&&unlisted)send(missile.order.key,{Order:{instruction:{ranked:[...ranked],unlisted}}});}}>Use this response order</button>
      {#if missile.delegate}<button class="secondary" onclick={()=>{if(missile.delegate)send(missile.delegate,'Delegate');}}>Let the host order these responses</button>{/if}
    </fieldset>
  {/if}
  {#each responses as response (response.key)}
    {@const selected=response.shield.find(choice=>JSON.stringify(choice)===chosen[response.key])}
    <fieldset {disabled}><legend>{participants.find(participant=>participant.entity_id===response.actor)?.public_label??'Your creature'} · Magic Missile response</legend>
      {#if response.selected}
        <p>You are targeted by Magic Missile. Casting Shield now prevents its damage to you until the start of your next turn. Each dart's physical face is still recorded.</p>
        <label>Shield resource<select value={chosen[response.key]??''} onchange={event=>chosen={...chosen,[response.key]:event.currentTarget.value}}>
          <option value="" disabled>Choose a resource</option>
          {#each response.shield as choice,index}<option value={JSON.stringify(choice)}>{typeof choice.resource==='object'?`Spell slot level ${choice.resource.Slot.level}`:'Source spellcasting use'} · {index+1}</option>{/each}
        </select></label>
        <button disabled={!selected} onclick={()=>{if(selected)send(response.key,{Cast:{choice:selected}});}}>Cast Shield · spend Reaction</button>
        <button class="secondary" onclick={()=>send(response.key,'Decline')}>Continue without casting</button>
      {:else}
        {#if response.shield.length}<p>You can offer Shield. No Reaction or spell resource is spent until you choose to cast it.</p>
          <button onclick={()=>send(response.key,{Respond:{accept:true}})}>Offer Shield response</button>
        {/if}
        <button class="secondary" onclick={()=>send(response.key,{Respond:{accept:false}})}>Continue without Shield</button>
      {/if}
    </fieldset>
  {/each}
  {#if !mayOrder && !responses.length}<p>Waiting for Magic Missile responses.</p>{/if}
</section>

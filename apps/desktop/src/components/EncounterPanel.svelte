<script lang="ts">
  import { newId, type CharacterView, type Id } from '../table-api';
  import type { TacticalAction, TacticalView } from '../tactical-api';
  import TacticalMap from './TacticalMap.svelte';
  import AttackForm from './AttackForm.svelte';
  import CastingForm from './CastingForm.svelte';
  import AreaForm from './AreaForm.svelte';
  import MovementForm from './MovementForm.svelte';
  import OpportunityForm from './OpportunityForm.svelte';
  import LiquidLandingForm from './LiquidLandingForm.svelte';
  import ShieldForm from './ShieldForm.svelte';
  let { tactical, characters, host, actor, player, disabled=false, pendingRoll=false, onAction }: {
    tactical:TacticalView;characters:CharacterView[];host:boolean;actor:Id|null;player:Id|null;disabled?:boolean;pendingRoll?:boolean;onAction:(action:TacticalAction)=>void;
  }=$props();
  let surprised=$state<Id[]>([]);
  let tieOrder=$state<Record<string,Id[]>>({});
  function reorder(total:number, actors:Id[], index:number, step:number) {
    const order=[...(tieOrder[total] ?? actors)];const to=index+step;
    if(to<0||to>=order.length)return;[order[index],order[to]]=[order[to],order[index]];
    tieOrder={...tieOrder,[total]:order};
  }
  const name=(id:Id)=>characters.find(character=>character.entity_id===id)?.name ?? tactical.participants.find(p=>p.entity_id===id)?.public_label ?? 'Combatant';
  function begin(){
    const combatants=tactical.combatant_sources.map(source=>({actor:source.actor,source:source.source,surprised:surprised.includes(source.actor)}));
    const grouped=new Map<string,Id[]>();
    for(const combatant of combatants){
      const preview=tactical.combatant_sources.find(source=>source.actor===combatant.actor)!;
      const key=combatant.source==='Character'?combatant.actor:`${combatant.source.Creature.definition_id}:${combatant.surprised}:${preview.initiative_modifier}:${combatant.surprised?preview.surprised_mode:preview.normal_mode}`;
      grouped.set(key,[...(grouped.get(key)??[]),combatant.actor]);
    }
    onAction({Begin:{combatants,groups:[...grouped.values()].map(actors=>({actors,request_id:newId()}))}});
  }
</script>
<section class="panel"><h2>Encounter{tactical.round ? ` · round ${tactical.round}` : ''}</h2>
  <TacticalMap {tactical} {characters}/>
  {#if tactical.initiative.length}<ol aria-label="Known initiative order">{#each tactical.initiative as entry}<li><strong>{entry.actor===tactical.active_actor?'Current turn: ':''}{entry.label}</strong>{entry.total===null?'':` · ${entry.total}`}</li>{/each}</ol>{/if}
  {#if host && tactical.phase==='setup'}<fieldset disabled={disabled||pendingRoll}><legend>Begin initiative</legend><p>Mark creatures surprised by combat starting. The rules apply their initiative disadvantage.</p>{#each tactical.participants as participant}<label><input type="checkbox" value={participant.entity_id} bind:group={surprised}/>{participant.public_label} is surprised</label>{/each}<button onclick={begin}>Roll initiative</button></fieldset>{/if}
  {#each tactical.ties as tie}<fieldset {disabled}><legend>Initiative tie at {tie.total}</legend><ol>{#each tieOrder[tie.total] ?? tie.proposed_order ?? tie.actors as tied,index}<li>{name(tied)} <button type="button" class="secondary" aria-label={`Move ${name(tied)} earlier`} onclick={()=>reorder(tie.total,tie.proposed_order??tie.actors,index,-1)}>Earlier</button><button type="button" class="secondary" aria-label={`Move ${name(tied)} later`} onclick={()=>reorder(tie.total,tie.proposed_order??tie.actors,index,1)}>Later</button></li>{/each}</ol><button onclick={()=>onAction({ProposeInitiativeTie:{order:tieOrder[tie.total]??tie.proposed_order??tie.actors}})}>Propose this order</button>{#if !host && tie.proposed_order}<button disabled={!!player&&tie.accepted_by.includes(player)} onclick={()=>onAction({AcceptInitiativeTie:{total:tie.total}})}>Agree to the proposed order</button>{/if}</fieldset>{/each}
  {#if tactical.phase==='active' && tactical.budget && (host || actor===tactical.active_actor)}
    <p>Movement used: {tactical.budget.movement_spent/2} feet. Action: {tactical.budget.action_spent?'spent':'available'}. Bonus action: {tactical.budget.bonus_action_spent?'spent':'available'}. Reaction: {tactical.budget.reaction_available?'available':'spent'}.</p>
    <fieldset disabled={disabled||pendingRoll||!!tactical.continuation}><legend>Current turn</legend><div class="actions"><button disabled={tactical.budget.action_spent} onclick={()=>onAction({Dash:{speed:'Speed'}})}>Dash</button><button disabled={tactical.budget.action_spent} onclick={()=>onAction('Disengage')}>Disengage</button><button disabled={tactical.budget.action_spent} onclick={()=>onAction('Dodge')}>Dodge</button><button onclick={()=>onAction('StandProne')}>Stand up</button><button class="secondary" onclick={()=>onAction('EndTurn')}>End turn</button></div></fieldset>
  {/if}
  {#if tactical.area_options && (host || actor===tactical.area_options.actor)}
    {#key `${host}:${player}:${actor}:${tactical.area_options.actor}`}<AreaForm options={tactical.area_options} {host} {player} disabled={disabled||pendingRoll||!!tactical.continuation} {onAction}/>{/key}
  {/if}
  {#if tactical.casting_options && (host || actor===tactical.casting_options.actor)}
    {#key `${host}:${player}:${actor}:${tactical.casting_options.actor}`}<CastingForm options={tactical.casting_options} disabled={disabled||pendingRoll||!!tactical.continuation} {onAction}/>{/key}
  {/if}
  {#if tactical.shield_options && (host || actor===tactical.shield_options.actor)}
    {#key `${host}:${player}:${actor}:${tactical.shield_options.actor}`}<ShieldForm options={tactical.shield_options} disabled={disabled||pendingRoll||!!tactical.continuation} {onAction}/>{/key}
  {/if}
  {#if tactical.attack_options && (host || actor===tactical.attack_options.actor) && tactical.attack_options.weapons.some(weapon=>weapon.purposes.length>0)}
    {#key `${host}:${player}:${actor}:${tactical.attack_options.actor}`}<AttackForm options={tactical.attack_options} disabled={disabled||pendingRoll||!!tactical.continuation} {onAction}/>{/key}
  {/if}
  {#if tactical.movement_options && (host || actor===tactical.movement_options.actor)}
    {#key `${host}:${player}:${actor}:${tactical.movement_options.actor}:${JSON.stringify(tactical.movement_options.position)}`}<MovementForm options={tactical.movement_options} disabled={disabled||pendingRoll||!!tactical.continuation} {onAction}/>{/key}
  {/if}
  {#if tactical.opportunity && (host || actor===tactical.opportunity.actor)}
    {#key `${host}:${player}:${actor}:${tactical.opportunity.actor}:${tactical.opportunity.target.actor}`}<OpportunityForm opportunity={tactical.opportunity} disabled={disabled||pendingRoll} {onAction}/>{/key}
  {/if}
  {#if tactical.liquid_landing && (host || actor===tactical.liquid_landing.actor)}
    <LiquidLandingForm disabled={disabled||pendingRoll} {onAction}/>
  {/if}
  {#if tactical.attack_decision && (host || actor===tactical.attack_decision.actor)}
    <fieldset disabled={disabled||pendingRoll}><legend>{tactical.attack_decision.kind==='Knockout'?'Melee damage choice':'Graze mastery'}</legend>
      {#if tactical.attack_decision.kind==='Knockout'}
        <p>This melee attack can knock the creature out. Choose how to resolve the damage.</p>
        <button onclick={()=>onAction({ChooseAttackKnockout:{choice:'KnockOut'}})}>Knock out</button><button class="secondary" onclick={()=>onAction({ChooseAttackKnockout:{choice:'NormalDamage'}})}>Apply normal damage</button>
      {:else}
        <p>The attack missed. You may apply Graze damage.</p>
        <button onclick={()=>onAction({ChooseAttackMastery:{choice:'Graze'}})}>Use Graze</button><button class="secondary" onclick={()=>onAction({ChooseAttackMastery:{choice:'Decline'}})}>Decline Graze</button>
      {/if}
    </fieldset>
  {/if}
  {#if tactical.continuation && (host || actor===tactical.continuation.actor)}
    {#if tactical.continuation.choices.length}
      <fieldset disabled={disabled||pendingRoll}><legend>Choose which consequence happens next</legend>
        <p>{tactical.continuation.host_adjudication?'The host has authority to order these consequences. Individual saves and optional responses stay with their controllers.':'These consequences occur at the same time. Choose their order for this turn.'}</p>
        {#each tactical.continuation.choices as choice,index}<button onclick={()=>onAction({ChooseTurnWork:{occurrence:choice.occurrence}})}>{choice.label} · {index+1}</button>{/each}
      </fieldset>
    {:else}<p>Resolve the pending consequence before continuing the turn.</p>{/if}
  {/if}
  {#if tactical.may_fail_save && (host || actor===tactical.may_fail_save)}
    <fieldset {disabled}><legend>Saving throw choice</legend>
      <p>You may choose to fail this saving throw before reporting dice. This resolves it as a failure.</p>
      <button class="secondary" onclick={()=>onAction('VoluntarilyFailSave')}>Choose to fail this save</button>
    </fieldset>
  {/if}
  {#if tactical.legendary_resistance && (host || actor===tactical.legendary_resistance)}
    <fieldset {disabled}><legend>Legendary Resistance</legend><p>This saving throw failed. Spend a remaining use to succeed instead, or keep the failure.</p>
      <button onclick={()=>onAction('UseLegendaryResistance')}>Use Legendary Resistance</button><button class="secondary" onclick={()=>onAction('DeclineLegendaryResistance')}>Keep the failed save</button>
    </fieldset>
  {/if}
  {#if tactical.legendary_action && (host || actor===tactical.legendary_action)}
    <fieldset {disabled}><legend>Legendary Action opportunity</legend><p>{name(tactical.legendary_action)} may act after this turn.</p><button class="secondary" onclick={()=>onAction('DeclineLegendaryAction')}>Pass this opportunity</button></fieldset>
  {/if}
</section>

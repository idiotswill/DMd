<script lang="ts">
  import type { CreatureController, Player, SourceAdoption, SourceControlOptions, SourceControlView } from '../table-api';
  let { control, options, players, disabled, onReview, onEnable, onAssign }: {
    control?: SourceControlView; options: SourceControlOptions | null; players: Player[]; disabled: boolean;
    onReview: () => void; onEnable: (adopted: SourceAdoption[]) => void;
    onAssign: (actor: string, controller: CreatureController) => void;
  } = $props();
  let selected = $state<Record<string,string>>({});
  function current(controller: CreatureController): string { return typeof controller === 'string' ? controller : controller.Player; }
  function assign(actor: string, controller: CreatureController) {
    const value = selected[actor] ?? current(controller);
    onAssign(actor, value === 'Host' || value === 'Autonomous' ? value : {Player:value});
  }
</script>
<section class="panel">
  <h2>Source creature control</h2>
  {#if !control}
    <p>Let attending players select and control assigned creatures. Existing characters keep their usual controls.</p>
    {#if options}
      {#if options.adopted.length}<p>Existing assignments will become available to their owners:</p><ul>{#each options.adopted as entry}<li>{options.actors.find(actor=>actor.actor===entry.actor)?.name} — {players.find(player=>player.id===entry.player_id)?.display_name}</li>{/each}</ul>{/if}
      {#if !options.settled}<p>Finish pending rolls and decisions, and release or abandon held actions first.</p>{/if}
      <button disabled={disabled || !options.settled} onclick={()=>onEnable(options!.adopted)}>Enable source creature control</button>
    {:else}<button {disabled} onclick={onReview}>Review source creature control</button>{/if}
  {:else}
    <p>Choose who controls each creature. Finish pending or held actions before transferring control.</p>
    {#each control.actors as actor (actor.actor)}
      <form onsubmit={(event)=>{event.preventDefault();assign(actor.actor,actor.controller);}}>
        <fieldset {disabled}><legend>{actor.name}</legend>
          <label>Controller for {actor.name}<select value={selected[actor.actor] ?? current(actor.controller)} onchange={event=>selected[actor.actor]=event.currentTarget.value}>
            <option value="Autonomous">Autonomous</option><option value="Host">Host</option>
            {#each players as player}<option value={player.id}>{player.display_name}</option>{/each}
          </select></label>
          <button type="submit">Assign control of {actor.name}</button>
        </fieldset>
      </form>
    {:else}<p>Prepare a source creature to assign its control.</p>{/each}
  {/if}
</section>

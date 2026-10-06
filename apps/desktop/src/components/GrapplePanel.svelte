<script lang="ts">
  import type { TableView } from '../table-api';
  let { view, host, actor, disabled, onEnable, onEnableTransport, onChoice }: {
    view: TableView; host: boolean; actor: string|null; disabled: boolean;
    onEnableTransport?: ()=>void;onEnable: ()=>void; onChoice: (handle: string)=>void;
  } = $props();
  const choices = $derived(view.grapple?.choices.filter(choice=>host || choice.actor===actor) ?? []);
  function actorName(id: string) {
    return view.characters.find(character=>character.entity_id===id)?.name
      ?? view.source_control?.actors.find(source=>source.actor===id)?.name
      ?? view.creature_setup?.creatures.find(source=>source.actor===id)?.name
      ?? 'Controlled creature';
  }
</script>

{#if view.grapple && choices.length}
  <fieldset disabled={disabled} tabindex="-1" data-tactical-focus="grapple" data-tactical-focus-id={actor??'host'}>
    <legend>Grapple</legend>
    <p>Choose your free hand, resist a hold, escape, or release a grip. Use the movement controls to choose self-only movement or an available ground drag.</p>
    <div class="choices">
      {#each choices as choice (choice.key)}
        <button type="button" onclick={()=>onChoice(choice.key)}>{host ? `${actorName(choice.actor)}: ` : ''}{choice.label}</button>
      {/each}
    </div>
  </fieldset>
{:else if !view.grapple && host && view.tactical?.execution==='EncounterReleaseV1' && view.tactical.phase==='active'}
  <fieldset disabled={disabled||!!view.pending||!!view.roll||!!view.tactical.continuation||!!view.tactical.ready?.length}>
    <legend>Grapple</legend>
    <p>Enable Grapple for this table at a settled turn. Each creature's controller chooses its hand, save, and escape.</p>
    <button type="button" onclick={onEnable}>Enable Grapple</button>
  </fieldset>
{/if}

{#if view.grapple?.version===3 && host && onEnableTransport && view.tactical?.phase==='active'}
  <fieldset disabled={disabled||!!view.pending||!!view.roll||!!view.tactical.continuation||!!view.tactical.ready?.length}>
    <legend>Ground drag</legend>
    <p>Enable movement with one held creature along a dry, supported route.</p>
    <button type="button" onclick={onEnableTransport}>Enable ground drag</button>
  </fieldset>
{/if}

<style>
  .choices { display:flex; flex-wrap:wrap; gap:.5rem; }
  fieldset { margin-block:1rem; }
</style>

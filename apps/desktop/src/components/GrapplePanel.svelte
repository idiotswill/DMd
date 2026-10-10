<script lang="ts">
  import { tableApi, type GrappleOptionsRequest, type LocalChannel, type TableView } from '../table-api';
  let { view, host, actor, channel, disabled, onEnable, onEnableTransport, onChoice }: {
    view: TableView; host: boolean; actor: string|null; channel: LocalChannel|null; disabled: boolean;
    onEnableTransport?: ()=>void;onEnable: ()=>void; onChoice: (handle: string)=>void;
  } = $props();
  const choices = $derived(view.grapple?.choices.filter(choice=>host || choice.actor===actor) ?? []);
  const request = $derived<GrappleOptionsRequest|null>(channel ? {
    version: 1, campaign_id: view.campaign_id, channel, revision: view.revision,
  } : null);
  const identity = $derived(JSON.stringify([request, host, actor, choices,
    view.active_session?.session_id, view.tactical?.encounter_id, view.tactical?.round]));
  let allowed = $state<string[]>([]);
  let readyIdentity = $state<string|null>(null);
  let loading = $state(false);
  let error = $state('');
  let generation = 0;
  const available = $derived(readyIdentity === identity ? choices.filter(choice => allowed.includes(choice.key)) : []);
  function channelKey(value: LocalChannel): string {
    return value === 'Host' ? 'Host' : 'Player' in value
      ? JSON.stringify(['Player', value.Player.player_id, value.Player.character_id])
      : JSON.stringify(['SourceCreature', value.SourceCreature.player_id, value.SourceCreature.actor]);
  }
  async function load(value: GrappleOptionsRequest, observed: string, ticket: number, offered: string[]) {
    const current = () => generation === ticket && !disabled && identity === observed;
    try {
      const answer = await tableApi.grappleOptions(value);
      if (!current()) return;
      if (answer.version !== 1 || answer.campaign_id !== value.campaign_id || answer.revision !== value.revision
        || channelKey(answer.channel) !== channelKey(value.channel) || !Array.isArray(answer.handles)
        || new Set(answer.handles).size !== answer.handles.length
        || answer.handles.some(handle => !offered.includes(handle))) throw new Error('Mismatched Grapple options. Refresh the table.');
      allowed = answer.handles;
      readyIdentity = observed;
    } catch (reason) {
      if (current()) error = reason instanceof Error ? reason.message : 'Grapple choices could not be loaded. Refresh the table or retry.';
    } finally {
      if (current()) loading = false;
    }
  }
  $effect(() => {
    const value = request, observed = identity, blocked = disabled, offered = choices.map(choice => choice.key);
    const ticket = ++generation;
    allowed = []; readyIdentity = null; error = '';
    loading = !!value && !blocked && offered.length > 0;
    if (value && !blocked && offered.length > 0) void load(value, observed, ticket, offered);
    return () => { generation += 1; };
  });
  function retry() {
    if (disabled || loading || !request) return;
    allowed = []; readyIdentity = null; error = ''; loading = true;
    void load(request, identity, ++generation, choices.map(choice => choice.key));
  }
  function choose(handle: string) {
    if (disabled || readyIdentity !== identity || !available.some(choice => choice.key === handle)) return;
    onChoice(handle);
  }
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
    {#if loading}<p role="status">Checking current Grapple choices...</p>{/if}
    {#if error}<p role="alert">{error}</p><button type="button" onclick={retry}>Retry Grapple choices</button>{/if}
    {#if readyIdentity === identity && !available.length}<p>No Grapple choices are available for this actor right now.</p>{/if}
    <div class="choices">
      {#each available as choice (choice.key)}
        <button type="button" onclick={()=>choose(choice.key)}>{host ? `${actorName(choice.actor)}: ` : ''}{choice.label}</button>
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
    <legend>Dragging and Inspiration</legend>
    <p>Enable movement with one held creature along a dry, supported route and Host-awarded Heroic Inspiration. Each award is a separate Host ruling.</p>
    <button type="button" onclick={onEnableTransport}>Enable dragging and Inspiration</button>
  </fieldset>
{/if}

<style>
  .choices { display:flex; flex-wrap:wrap; gap:.5rem; }
  fieldset { margin-block:1rem; }
</style>

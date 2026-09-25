<script lang="ts">
  import { onMount, tick } from 'svelte';
  import ContractForm from './components/ContractForm.svelte';
  import CharacterForm from './components/CharacterForm.svelte';
  import CharacterSheet from './components/CharacterSheet.svelte';
  import SessionForm from './components/SessionForm.svelte';
  import SituationForm from './components/SituationForm.svelte';
  import RollForm from './components/RollForm.svelte';
  import BattlefieldForm from './components/BattlefieldForm.svelte';
  import CreatureForm from './components/CreatureForm.svelte';
  import SourceControlForm from './components/SourceControlForm.svelte';
  import EncounterPanel from './components/EncounterPanel.svelte';
  import { rawDice, channelPlayer, type LocalChannel, type SourceControlOptions, type SourceAdoption } from './table-api';
  import type { SavageAttackerRoll } from './tactical-api';
  import { clearRequest, loadRequest, loadSelection, newId, requestLabel, saveRequest, saveSelection, tableApi, type CreationOptions, type CreatureOption, type RequestContext, type Situation, type TableAction, type TableContract, type TableView, type UnconfirmedRequest } from './table-api';

  let campaigns = $state<{ id: string; name: string }[]>([]);
  let defaults = $state<TableContract | null>(null);
  let view = $state<TableView | null>(null);
  let savageOption = $state<{ weapon_dice: number; heroic_inspiration: boolean } | null>(null);
  let creatureCatalog = $state<CreatureOption[] | null>(null);
  let sourceControlOptions = $state<SourceControlOptions | null>(null);
  let sourceControlLoading = $state(false);
  let options = $state<CreationOptions | null>(null);
  let hostSituation = $state<Situation | null>(null);
  let campaignId = $state(''); let playerId = $state('');
  let sourceActorId = $state('');
  let campaignName = $state(''); let playerName = $state(''); let text = $state('');
  let page = $state<'play' | 'setup'>('play'); let creating = $state(false);
  let busy = $state(true); let error = $state(''); let message = $state('');
  let retry = $state<UnconfirmedRequest | null>(null); let unreadableRetry = $state(false);
  let errorElement = $state<HTMLDivElement>();
  let refreshGeneration = 0;
  const host = $derived(!playerId);
  const locked = $derived(busy || retry !== null || unreadableRetry);
  const binding = $derived(view?.active_session?.participants.find(p => p.player_id === playerId && p.attendance === 'Present' && p.character_id));
  const currentCharacter = $derived(view?.characters.find(c => c.character_id === binding?.character_id));
  const sourceActors = $derived(view?.source_control?.actors.filter(actor=>typeof actor.controller==='object'&&actor.controller.Player===playerId) ?? []);
  const sourceActor = $derived(sourceActors.find(actor=>actor.actor===sourceActorId));
  const selectedActor = $derived(sourceActor?.actor ?? (!sourceActorId ? currentCharacter?.entity_id : undefined));
  const attending = $derived(view?.active_session?.participants.some(p=>p.player_id===playerId&&p.attendance==='Present') ?? false);
  const canSpeak = $derived(Boolean(!sourceActorId && binding && currentCharacter));
  const ownsPending = $derived(view?.pending?.player_id === playerId);
  function hostMayRoll(current: TableView): boolean {
    return !!current.roll && !current.characters.some(character=>character.entity_id===current.roll?.roller)
      && (current.roll.visibility==='Secret' || !current.source_control?.actors.some(actor=>actor.actor===current.roll?.roller&&typeof actor.controller==='object'));
  }

  function explain(reason: unknown): string {
    if (typeof reason === 'string') return reason;
    if (reason && typeof reason === 'object' && 'message' in reason && typeof reason.message === 'string') return reason.message;
    return 'The application could not complete this request. Refresh the table or retry the saved request.';
  }
  async function showError(reason: unknown) { error = explain(reason); await tick(); errorElement?.focus(); }
  function rememberSelection() { saveSelection({ campaignId: campaignId || null, playerId: playerId || null, ...(sourceActorId ? {sourceActorId} : {}) }); }
  async function refresh() {
    const generation = ++refreshGeneration;
    const target = campaignId; const selectedPlayer = playerId;
    view = null; options = null; hostSituation = null; savageOption = null; creatureCatalog = null; sourceControlOptions = null; sourceControlLoading=false;
    if (!target) return;
    const [next, choices, situation] = await Promise.all([tableApi.view(target, selectedPlayer ? { Player: selectedPlayer } : 'Host'), tableApi.options(target), selectedPlayer ? Promise.resolve(null) : tableApi.situation(target)]);
    if (generation !== refreshGeneration || campaignId !== target || playerId !== selectedPlayer) return;
    view = next; options = choices; hostSituation = situation ?? null;
    if (!retry && sourceActorId && !next.source_control?.actors.some(actor=>actor.actor===sourceActorId&&typeof actor.controller==='object'&&actor.controller.Player===selectedPlayer)) sourceActorId='';
    rememberSelection();
    if (!selectedPlayer && next.creature_setup && !next.tactical) void loadCreatureCatalog(target, next.revision, generation);
    const attending = next.active_session?.participants.find(p=>p.player_id===selectedPlayer&&p.attendance==='Present'&&p.character_id);
    const selected = next.characters.find(c=>c.character_id===attending?.character_id);
    if (next.roll && next.roll_channel === 'Tactical'
      && ((!selectedPlayer && hostMayRoll(next))
        || selected?.entity_id===next.roll.roller && !sourceActorId
        || next.source_control?.actors.some(actor=>actor.actor===sourceActorId&&actor.actor===next.roll?.roller&&typeof actor.controller==='object'&&actor.controller.Player===selectedPlayer))) {
      const answer = await tableApi.rollOptions({campaign_id:target,revision:next.revision,roll_id:next.roll.id,
        channel:selectedPlayer ? sourceActorId ? {SourceCreature:{player_id:selectedPlayer,actor:sourceActorId}} : {Player:{player_id:selectedPlayer,character_id:attending!.character_id!}} : 'Host'});
      if(generation===refreshGeneration&&campaignId===target&&playerId===selectedPlayer&&view?.revision===next.revision) savageOption=answer.savage_attacker;
    }
  }
  async function loadCreatureCatalog(target: string, revision: string, generation: number) {
    const current = () => generation === refreshGeneration && campaignId === target && !playerId && view?.revision === revision;
    try {
      const catalog = await tableApi.creatureOptions({campaign_id:target,channel:'Host',revision});
      if (current()) creatureCatalog = catalog;
    } catch (reason) { if (current()) await showError(reason); }
  }
  async function selectCampaign(id: string) {
    busy = true; error = ''; message = ''; campaignId = id; playerId = ''; sourceActorId=''; page = 'play'; creating = false;
    try { await refresh(); } catch (reason) { await showError(reason); } finally { busy = false; }
  }
  async function selectPlayer(id: string) {
    busy = true; error = ''; message = ''; playerId = id; sourceActorId=''; page = 'play'; text = ''; view = null;
    try { await refresh(); busy = false; await tick(); document.getElementById('table-channel')?.focus(); } catch (reason) { await showError(reason); } finally { busy = false; }
  }
  async function refreshAll() {
    busy = true; error = '';
    try { campaigns = await tableApi.list(); await refresh(); } catch (reason) { await showError(reason); } finally { busy = false; }
  }
  async function selectActor(id: string) {
    if (locked) return;
    sourceActorId=id; text=''; message=''; await refreshAll();
  }
  async function reviewSourceControl() {
    if (!view || !host || locked) return;
    const target=campaignId;const revision=view.revision;const generation=refreshGeneration;
    const current=()=>generation===refreshGeneration&&campaignId===target&&!playerId&&view?.revision===revision;
    sourceControlLoading=true; error='';
    try { const result=await tableApi.sourceControlOptions({campaign_id:target,channel:'Host',revision}); if(current())sourceControlOptions=result; }
    catch(reason) { if(current())await showError(reason); } finally { if(current())sourceControlLoading=false; }
  }
  async function enableSourceControl(adopted: SourceAdoption[]) {
    try { await send({kind:'action',request:{...context(),version:2,action:{EnableSourceActorAccess:{adopted}}}}); }
    catch(reason) { await showError(reason); }
  }
  async function initialize() {
    busy = true;
    try {
      try { retry = loadRequest(); } catch (reason) { unreadableRetry = true; await showError(reason); }
      const saved = loadSelection(); campaignId = saved.campaignId ?? ''; playerId = saved.playerId ?? ''; sourceActorId=saved.sourceActorId??'';
      if (retry) {
        campaignId = retry.kind === 'create' ? retry.request.id : retry.request.campaign_id;
        playerId = retry.kind !== 'create' ? channelPlayer(retry.request.channel) ?? '' : '';
        sourceActorId = retry.kind !== 'create' && typeof retry.request.channel==='object' && 'SourceCreature' in retry.request.channel ? retry.request.channel.SourceCreature.actor : '';
      }
      [campaigns, defaults] = await Promise.all([tableApi.list(), tableApi.defaults()]);
      if (campaignId && campaigns.some(c => c.id === campaignId)) await refresh();
      else if (!retry) campaignId = '';
    } catch (reason) { await showError(reason); } finally { busy = false; }
  }
  onMount(initialize);

  function context(sessionId?: string | null): RequestContext {
    if (!view) throw new Error('Open the saved campaign first.');
    if (playerId && (!attending || (sourceActorId ? !sourceActor : !binding?.character_id))) throw new Error('Select an attending player and their controlled actor.');
    const channel:LocalChannel = playerId ? sourceActor ? {SourceCreature:{player_id:playerId,actor:sourceActor.actor}} : {Player:{player_id:playerId,character_id:binding!.character_id!}} : 'Host';
    return { command_id: newId(), campaign_id: view.campaign_id, version: view.source_control ? 2 : 1, revision: view.revision,
      session_id: sessionId === undefined ? view.active_session?.session_id ?? null : sessionId,
      channel };
  }
  async function send(request: UnconfirmedRequest, existing = false) {
    if (busy || (!existing && (retry || unreadableRetry))) return;
    busy = true; error = ''; message = '';
    try {
      if (!existing) { saveRequest(request); retry = request; }
      if (request.kind === 'create') {
        const result = await tableApi.create(request.request);
        campaignId = result.campaign_id; playerId = ''; sourceActorId=''; creating = false; page = 'setup';
        message = 'Campaign created. Add players and their characters, then start a session.';
      } else if (request.kind === 'action') {
        const result = await tableApi.action(request.request); message = result.outcome.message;
        if (typeof request.request.action === 'object' && 'AddPlayer' in request.request.action) playerName = '';
      } else {
        const result = await tableApi.text(request.request);
        message = 'Accepted' in result ? result.Accepted.outcome.message : result.Observed.answer; text = '';
      }
      clearRequest(); retry = null; campaigns = await tableApi.list(); await refresh();
    } catch (reason) {
      // Only a typed host rejection proves the mutation was not accepted. Transport failures
      // keep the complete original request for exact retry, including after closing the app.
      if (reason && typeof reason === 'object' && 'retryable' in reason && reason.retryable === false) { clearRequest(); retry = null; }
      await showError(reason);
    } finally { busy = false; }
  }
  async function act(action: TableAction, sessionId?: string | null) {
    try { await send({ kind: 'action', request: { ...context(sessionId), action } }); } catch (reason) { await showError(reason); }
  }
  function reportFaces(faces: number[]) {
    if (!view?.roll) return;
    if (view.roll_channel === 'Tactical') {
      const sides=rawDice(view.roll);
      return act({ Tactical: { action: { SubmitRoll: { result: {
        request_id: view.roll.id, source: 'Physical',
        dice: faces.map((value,index) => ({ sides:sides[index], value }))
      } } } } });
    }
    return act({SubmitPhysical:{request_id:view.roll.id,faces}});
  }
  function reportSavage(roll: SavageAttackerRoll) {
    return act({Tactical:{action:{SubmitSavageAttacker:{roll}}}});
  }
  async function speak(event: SubmitEvent) {
    event.preventDefault();
    try { await send({ kind: 'text', request: { ...context(), text } }); } catch (reason) { await showError(reason); }
  }
  async function releaseRetry() {
    busy = true; error = '';
    try {
      campaigns = await tableApi.list();
      if (campaignId && campaigns.some(c => c.id === campaignId)) await refresh();
      clearRequest(); retry = null; unreadableRetry = false;
      message = 'The saved table has been refreshed. The local retry was released; accepted history was not changed.';
    } catch (reason) { await showError(reason); } finally { busy = false; }
  }
</script>

<div class="table-app" aria-busy={busy}>
  <div class="campaign-bar">
    <label>Campaign<select value={campaignId} onchange={(event) => selectCampaign(event.currentTarget.value)} disabled={busy || retry !== null}><option value="">Choose a campaign</option>{#each campaigns as campaign}<option value={campaign.id}>{campaign.name}</option>{/each}</select></label>
    <button class="secondary" disabled={locked} onclick={() => { creating = !creating; }}>New campaign</button>
    <button class="secondary" disabled={busy} onclick={refreshAll}>Refresh saved table</button>
  </div>
  {#if error}<div bind:this={errorElement} tabindex="-1" role="alert" class="error"><h2>Unable to complete the request</h2><p>{error}</p>{#if !retry && !unreadableRetry}<button disabled={busy} onclick={refreshAll}>Try refreshing</button>{/if}</div>{/if}
  {#if message}<p class="notice preserve" role="status" aria-live="polite">{message}</p>{/if}
  {#if retry || unreadableRetry}<section class="retry" aria-labelledby="retry-title"><h2 id="retry-title">A request is awaiting confirmation</h2><p>DMd kept the original request. Retry it to recover its saved result without duplicating the action. Review the refreshed table before releasing the retry.</p>{#if retry}<p>{requestLabel(retry)}</p>{/if}<div class="actions">{#if retry}<button disabled={busy} onclick={() => retry && send(retry, true)}>Retry original request</button>{/if}<button disabled={busy} class="secondary" onclick={releaseRetry}>Refresh and release local retry</button></div></section>{/if}
  {#if creating && defaults}<section class="panel"><h1>Create a campaign</h1><label>Campaign name<input required maxlength="200" bind:value={campaignName} /></label>{#key defaults}<ContractForm value={defaults} disabled={locked || !campaignName.trim()} submitLabel="Create campaign with this agreement" onSave={(contract) => send({ kind: 'create', request: { id: newId(), name: campaignName, contract } })} />{/key}</section>{/if}
  {#if view}
    <section class="table-heading"><div><p class="eyebrow">{view.active_session ? 'Session in progress' : 'Between sessions'}</p><h1>{view.name}</h1><p>{view.active_session?.display_name ?? 'Set up your table, then begin play.'}</p></div>
      <label>Local viewing and input channel<select id="table-channel" value={playerId} onchange={(event) => selectPlayer(event.currentTarget.value)} disabled={busy || retry !== null}><option value="">Host — setup and adjudication</option>{#each view.players as player}<option value={player.id}>{player.display_name}</option>{/each}</select></label>
    </section>
    <p class="muted">This shared computer trusts the selected local channel. Player text cannot change who is speaking or controlling a character.</p>
    {#if !host && sourceActors.length}<label>Controlled actor<select value={sourceActorId} onchange={event=>selectActor(event.currentTarget.value)} disabled={locked}><option value="">{currentCharacter?.name ?? 'Select a creature'}</option>{#each sourceActors as actor}<option value={actor.actor}>{actor.name}</option>{/each}</select></label>{/if}
    <nav class="actions" aria-label="Campaign sections"><button class:secondary={page !== 'play'} onclick={() => { page = 'play'; }}>Table and sheets</button>{#if host}<button class:secondary={page !== 'setup'} onclick={() => { page = 'setup'; }}>Setup and host controls</button>{/if}</nav>
    {#if host && hostSituation?.challenges.length}<details class="panel"><summary>Current host check context</summary><p class="muted">These difficulties and unrevealed consequences are host-only.</p>{#each hostSituation.challenges as challenge}<article><h3>{challenge.title}</h3><p>{challenge.description}</p><p>{challenge.kind.Check.ability}{challenge.kind.Check.skill ? ` (${challenge.kind.Check.skill})` : ''} · DC {challenge.dc} · {challenge.resolution ? 'Resolved' : 'Unresolved'}</p><p>On success: {challenge.success}</p><p>On failure: {challenge.failure}</p></article>{/each}</details>{/if}
    {#if page === 'setup' && host}
      <SourceControlForm control={view.source_control} options={sourceControlOptions} players={view.players} disabled={locked||sourceControlLoading||!!view.pending||!!view.roll} onReview={reviewSourceControl} onEnable={enableSourceControl} onAssign={(actor,controller)=>act({SetSourceCreatureController:{actor,controller}})}/>
      <section class="panel"><h2>Table agreement</h2>{#if view.active_session}<p>End the session to update the agreement.</p><details><summary>Read saved agreement</summary><dl>{#each Object.entries(view.contract).filter(([,value]) => typeof value === 'string') as [key,value]}<dt>{key.replaceAll('_',' ')}</dt><dd>{value}</dd>{/each}</dl></details>{:else}{#key view.revision}<ContractForm value={view.contract} disabled={locked} mechanicsLocked={view.characters.length > 0} onSave={(contract) => act({ UpdateContract: { contract } })} />{/key}{/if}</section>
      {#if !view.active_session}
        <section class="panel"><h2>Players and characters</h2><form onsubmit={(event) => { event.preventDefault(); act({ AddPlayer: { id: newId(), name: playerName } }); }}><fieldset disabled={locked}><legend>Add a player</legend><label>Player name<input required maxlength="200" bind:value={playerName} /></label><button type="submit">Add player</button></fieldset></form>
        {#if view.players.length}<ul>{#each view.players as player}<li>{player.display_name}</li>{/each}</ul>{/if}
        {#if options && view.players.length}{#key view.revision}<details><summary>Create a character</summary><CharacterForm {options} players={view.players} disabled={locked} onCreate={(player_id,input) => act({ CreateCharacter: { player_id, input, character_id: newId(), entity_id: newId() } })} /></details>{/key}{/if}</section>
        {#if view.players.length}<section class="panel">{#key view.revision}<SessionForm players={view.players} characters={view.characters} sourceActors={view.source_control?.actors??[]} disabled={locked} onStart={(name,participants) => { const id = newId(); act({ StartSession: { id, name, participants } }, id); }} />{/key}</section>{/if}
      {:else}<section class="panel"><h2>Current session</h2><ul>{#each view.active_session.participants as participant}<li>{view.players.find(p=>p.id===participant.player_id)?.display_name}: {participant.attendance} · {view.characters.find(c=>c.character_id===participant.character_id)?.name ?? 'No character'}</li>{/each}</ul><button disabled={locked || !!view.pending || !!view.roll} onclick={() => act('EndSession')}>End and save session</button><p class="muted">Finish or withdraw pending work before ending the session. Closing the app preserves pending work for later.</p></section>{/if}
      <section class="panel"><SituationForm disabled={locked || !!view.roll} onSave={(situation) => act({ SetSituation: { situation } })} /></section>
      {#if !view.tactical && view.creature_setup}<section class="panel">{#if !view.characters.length}<p>Create a player character before preparing creatures.</p>{/if}{#if creatureCatalog !== null}<CreatureForm setup={{...view.creature_setup,catalog:creatureCatalog}} disabled={locked||!!view.pending||!!view.roll||!view.characters.length} onCreate={(creation)=>act({CreateCreature:{creation}})}/>{:else}<p>Loading creature sources...</p>{/if}</section>{/if}
      {#if view.active_session && !view.tactical}<section class="panel">{#key view.revision}<BattlefieldForm characters={view.characters.filter(character=>view?.active_session?.participants.some(p=>p.character_id===character.character_id&&p.attendance==='Present'))} creatures={view.creature_setup?.creatures??[]} ownedSourceActors={(view.source_control?.actors??[]).filter(actor=>typeof actor.controller==="object"&&view?.active_session?.participants.some(p=>p.player_id===(typeof actor.controller==="object"?actor.controller.Player:null)&&p.attendance==="Present")).map(actor=>actor.actor)} disabled={locked||!!view.pending||!!view.roll} onPrepare={(setup)=>act({PrepareBattlefield:{setup}})}/>{/key}</section>{/if}
    {:else}
      <section class="panel"><h2>{view.situation_title || 'The current situation'}</h2><p class="preserve">{view.situation_description || 'The host has not established a situation yet.'}</p>
        {#if view.pending}<div class="pending"><h3>Uncommitted declaration</h3><p class="preserve">{view.pending.text}</p>{#if typeof view.pending.intent === 'object' && 'Unresolved' in view.pending.intent}<p>{view.pending.intent.Unresolved.question}</p>{:else}<p>The proposed action is understood and awaits the host's roll request.</p>{/if}
          {#if host}<button disabled={locked || (typeof view.pending.intent === 'object' && 'Unresolved' in view.pending.intent)} onclick={() => view?.pending && act({ Adjudicate: { pending_id: view.pending.id, revision: view.pending.revision, request_id: newId() } })}>Request the supported roll</button>{:else if ownsPending}<div class="actions"><button class="secondary" disabled={locked} onclick={() => { text = 'Actually '; document.getElementById('table-text')?.focus(); }}>Correct this declaration</button><button class="secondary" disabled={locked} onclick={() => view?.pending && act({ CancelDecision: { pending_id: view.pending.id, revision: view.pending.revision } })}>Withdraw declaration</button></div>{/if}
        </div>{/if}
        {#if view.roll}{#if (!host && attending && selectedActor === view.roll.roller) || (host && !!view.tactical && hostMayRoll(view))}{#key `${host}:${playerId}:${sourceActorId}:${view.roll.id}`}<RollForm request={view.roll} {savageOption} disabled={locked} onSubmit={reportFaces} onSavage={reportSavage} />{/key}{:else}<p>A physical roll is pending. Select the attending player's channel to report their dice.</p>{/if}{/if}
        {#if !host && sourceActor}<p>Controlling {sourceActor.name}. Use the encounter controls and physical dice requests for this creature.</p>{:else if !host && canSpeak}<form onsubmit={speak}><fieldset disabled={locked}><legend>Talk at the table</legend><label for="table-text">Your declaration, question or correction</label><textarea id="table-text" required maxlength="8000" rows="3" bind:value={text}></textarea><p class="muted">Use ordinary words. Questions do not take actions. Begin a correction with “Actually”. Unclear actions wait for clarification.</p><button type="submit">Send to the table</button></fieldset></form>{:else if host}<p>Select an attending player's channel to speak or report dice. The host requests supported rolls and establishes scene context.</p>{:else}<p>This player is not currently present with a bound character. The host can set attendance when starting the next session.</p>{/if}
      </section>
      {#if view.tactical}{#key view.tactical.encounter_id}<EncounterPanel tactical={view.tactical} characters={view.characters} {host} actor={selectedActor??null} playerControlledSources={(view.source_control?.actors??[]).filter(actor=>typeof actor.controller==="object").map(actor=>actor.actor)} player={playerId||null} disabled={locked||!!view.pending} pendingRoll={!!view.roll} onAction={(action)=>act({Tactical:{action}})}/>{/key}{/if}
      <section class="panel"><h2>Character sheets</h2>{#each view.characters as character (character.character_id)}<CharacterSheet {character} disabled={locked || !!view.pending || !!view.roll} onPrepare={host && character.equipment && !character.equipment.prepared ? () => act({ PrepareEquipment: { character_id: character.character_id, item_ids: Array.from({ length: character.equipment!.initial_item_count }, () => newId()) } }) : undefined} />{:else}<p>Create your first character in host setup.</p>{/each}</section>
      <section class="panel"><h2>Player-safe recap</h2>{#if view.recap.length}<ol>{#each view.recap as entry}<li class="preserve">{entry}</li>{/each}</ol>{:else}<p>No accepted outcomes yet. Proposed actions and questions do not enter the recap.</p>{/if}</section>
      <section class="panel"><h2>Table transcript</h2><div class="transcript" role="log" aria-label="Saved table transcript">{#each view.transcript as entry (entry.id)}<article><p><strong>{entry.speaker}</strong> <span class="tag">{entry.kind}</span></p><p class="preserve">{entry.text}</p></article>{:else}<p>Your accepted table activity and conversation appear here.</p>{/each}</div></section>
    {/if}
  {:else if !busy && !creating}<section class="panel"><h1>Welcome to your table</h1><p>{campaigns.length ? 'Choose a saved campaign above, or create a new one.' : 'Create a campaign to establish your table agreement, add players and make characters.'}</p></section>{/if}
  {#if busy}<p role="status">Loading the saved table…</p>{/if}
</div>

<style>
  .table-app { max-width: 1180px; margin: auto; padding: 1rem; }
  .campaign-bar, .table-heading { display:flex; flex-wrap:wrap; align-items:end; justify-content:space-between; gap:1rem; }
  .table-app :global(.panel), .retry { background:#fffdf7; border:1px solid #cbd5d0; border-radius:12px; padding:1.4rem; margin:1.2rem 0; }
  .table-app :global(label) { display:flex; flex-direction:column; gap:.35rem; margin:.6rem 0; font-weight:600; }
  .table-app :global(input:not([type=checkbox])), .table-app :global(select), .table-app :global(textarea) { width:100%; min-height:2.6rem; padding:.5rem .65rem; border:1px solid #839b98; border-radius:5px; background:white; color:#20363a; font-weight:400; }
  .table-app :global(textarea) { resize:vertical; }
  .table-app :global(fieldset) { border:1px solid #ccd4ce; border-radius:7px; padding:1rem; margin:.8rem 0; min-width:0; }
  .table-app :global(legend) { padding:0 .4rem; font-weight:700; }
  .table-app :global(.form-grid), .table-app :global(.equipment-grid), .table-app :global(.sheet-columns) { display:grid; grid-template-columns:repeat(auto-fit,minmax(min(240px,100%),1fr)); gap:.5rem 1rem; }
  .table-app :global(.ability-grid) { display:grid; grid-template-columns:repeat(auto-fit,minmax(130px,1fr)); gap:.7rem; }
  .table-app :global(.ability-grid > div) { border:1px solid #d5ddd6; padding:.65rem; border-radius:6px; }
  .table-app :global(.check) { flex-direction:row; align-items:center; gap:.65rem; }
  .table-app :global(input[type=checkbox]) { width:1.2rem; height:1.2rem; }
  .table-app :global(.actions) { display:flex; flex-wrap:wrap; gap:.7rem; margin:.8rem 0; }
  .table-app :global(.secondary) { color:#294f54; background:#f6f5ed; border:1px solid #6c8582; }
  .table-app :global(.error) { border:2px solid #a02c22; background:#fff2eb; padding:1rem; color:#752219; margin:.9rem 0; }
  .notice { border-left:4px solid #47785e; padding:1rem; background:#e8f3e9; }
  .retry { border:2px solid #9b6b19; background:#fff8df; }
  .table-app :global(.muted), .table-app :global(small) { color:#526366; font-weight:400; }
  .table-app :global(.preserve) { white-space:pre-wrap; overflow-wrap:anywhere; }
  .table-app :global(details) { margin:1rem 0; border-top:1px solid #d4ddd4; padding-top:.6rem; }
  .table-app :global(summary) { cursor:pointer; font-weight:700; padding:.4rem 0; }
  .table-app :global(.stats) { display:grid; grid-template-columns:repeat(auto-fit,minmax(115px,1fr)); gap:.7rem; }
  .table-app :global(.stats > div) { background:#eff3eb; border-radius:7px; padding:.8rem; }
  .table-app :global(dd) { margin:0 0 .6rem; }.table-app :global(.stats dd) { font-size:1.35rem; font-weight:700; margin:0; }
  .table-app :global(dt) { font-weight:600; }.table-app :global(.character-sheet) { border-top:1px solid #ccd5ce; padding:.75rem 0; }
  .pending { background:#edf3ec; padding:1rem; border-radius:8px; margin:1rem 0; }
  .transcript { max-height:34rem; overflow:auto; }.transcript article { border-bottom:1px solid #d3dbd5; padding:.7rem 0; }
  .tag { font-size:.8rem; margin-left:.7rem; color:#526366; text-transform:capitalize; }
  @media(max-width:540px) { .table-app { padding:.5rem; }.table-app :global(.panel) { padding:.85rem; }.campaign-bar { align-items:stretch; flex-direction:column; } }
</style>

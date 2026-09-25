<script lang="ts">
  import type { CharacterView, ControlledSourceActor, Participant, Player } from '../table-api';
  let { players, characters, sourceActors = [], disabled = false, onStart }: { players: Player[]; characters: CharacterView[]; sourceActors?: ControlledSourceActor[]; disabled?: boolean; onStart: (name: string, participants: Participant[]) => void } = $props();
  let name = $state('');
  let bindings = $state<Record<string, string>>({});
  let attending = $state<Record<string, boolean>>({});
  let error = $state('');
  function start(event: SubmitEvent) {
    event.preventDefault();
    const participants = players.map(player => ({ player_id: player.id, character_id: bindings[player.id] || null, attendance: attending[player.id] ? 'Present' as const : 'Absent' as const }));
    if (!participants.some(p => p.attendance === 'Present' && (p.character_id || sourceActors.some(actor=>typeof actor.controller==='object'&&actor.controller.Player===p.player_id)))) { error = 'Select at least one attending player and their controlled actor.'; return; }
    error = ''; onStart(name, participants);
  }
</script>
<form onsubmit={start}>
  <fieldset {disabled}><legend>Start a session</legend>
    <label>Session name<input required maxlength="200" bind:value={name} /></label>
    <p>Choose who is here and the character each player controls. Absent players cannot take voluntary actions.</p>
    {#each players as player}<div class="participant-row">
      <label class="check"><input type="checkbox" bind:checked={attending[player.id]} />{player.display_name} is present</label>
      <label>{player.display_name}'s character<select bind:value={bindings[player.id]}><option value="">No character this session</option>{#each characters.filter(c => c.player_id === player.id) as character}<option value={character.character_id}>{character.name}</option>{/each}</select></label>
      {#if sourceActors.some(actor=>typeof actor.controller==='object'&&actor.controller.Player===player.id)}<p>{player.display_name} can also select their assigned creatures while present.</p>{/if}
    </div>{/each}
    {#if error}<p role="alert" class="error">{error}</p>{/if}
    <button type="submit" disabled={!players.length}>Start session</button>
  </fieldset>
</form>

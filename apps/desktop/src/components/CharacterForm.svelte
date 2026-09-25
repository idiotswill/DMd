<script lang="ts">
  import { untrack } from 'svelte';
  import { ABILITIES, SKILLS, label, money, type CharacterInput, type CreationOptions, type Player, type Skill } from '../table-api';
  let { options, players, disabled = false, onCreate }: { options: CreationOptions; players: Player[]; disabled?: boolean; onCreate: (playerId: string, input: CharacterInput) => void } = $props();
  let playerId = $state(untrack(() => players[0]?.id ?? ''));
  let name = $state(''); let pronouns = $state(''); let description = $state(''); let backstory = $state('');
  let alignment = $state(untrack(() => options.alignments[0] ?? ''));
  let scores = $state([15, 14, 13, 12, 10, 8]); let boosts = $state([2, 1, 0, 0, 0, 0]);
  let fighter = $state<Skill[]>(['Perception', 'Survival']); let human = $state<Skill>('Insight');
  let skilled = $state<Skill[]>(['Acrobatics', 'Stealth', 'Investigation']);
  let size = $state<'Small' | 'Medium'>('Medium'); let style = $state<'Defense' | 'Archery'>('Defense');
  let gaming = $state<CharacterInput['gaming_set']>('Dice');
  let languages = $state(untrack(() => [options.standard_languages[0] ?? '', options.standard_languages[1] ?? '']));
  let quantities = $state<Record<string, number>>({}); let armor = $state(false); let shield = $state(false);
  let masteries = $state(['club', 'dagger', 'shortbow']);
  let error = $state('');
  const spent = $derived(options.catalog.items.reduce((total, item) => total + item.unit_cost_cp * (quantities[item.id] || 0), 0));
  const remaining = $derived(options.catalog.starting_money_cp - spent);
  const distinctSkills = $derived(new Set(['Athletics', 'Intimidation', ...fighter, human, ...skilled]).size === 8);
  function submit(event: SubmitEvent) {
    event.preventDefault(); error = '';
    if ([...scores].sort((a,b)=>a-b).join(',') !== '8,10,12,13,14,15') { error = 'Assign each standard-array value exactly once.'; return; }
    if (!['0,1,2', '1,1,1'].includes([...boosts.slice(0,3)].sort().join(','))) { error = 'Choose +2 and +1 to different physical abilities, or +1 to all three.'; return; }
    if (!distinctSkills) { error = 'Choose eight different skills, including the Soldier grants of Athletics and Intimidation.'; return; }
    if (languages[0] === languages[1]) { error = 'Choose two different languages.'; return; }
    if (new Set(masteries).size !== 3 || masteries.some(id => !options.fighter_masteries.includes(id))) { error = 'Choose three different weapon masteries.'; return; }
    if (remaining < 0) { error = 'Your purchases exceed the starting gold. Reduce the quantities.'; return; }
    onCreate(playerId, { name, pronouns, description, backstory, alignment, ability_scores: [...scores], background_boosts: [...boosts], fighter_skills: [...fighter], human_skill: human, skilled_skills: [...skilled], size, languages: [...languages], fighting_style: style, gaming_set: gaming,
      purchases: options.catalog.items.filter(i => (quantities[i.id] || 0) > 0).map(i => ({ item_id: i.id, quantity: quantities[i.id] })),
      worn_armor: armor && quantities['leather-armor'] > 0 ? 'leather-armor' : null, shield: shield && quantities.shield > 0,
      masteries: [...masteries] });
  }
</script>

<form onsubmit={submit}>
  <fieldset {disabled}>
    <legend>Create a supported character</legend>
    <p><strong>Human · Fighter level 1 · Soldier · Skilled.</strong> Choose your abilities, proficiencies, style and starting equipment below. More character options are not available yet.</p>
    <div class="form-grid">
      <label>Controlling player<select required bind:value={playerId}><option value="" disabled>Choose a player</option>{#each players as player}<option value={player.id}>{player.display_name}</option>{/each}</select></label>
      <label>Character name<input required maxlength="100" bind:value={name} /></label>
      <label>Pronouns (optional)<input maxlength="100" bind:value={pronouns} /></label>
      <label>Alignment<select bind:value={alignment}>{#each options.alignments as option}<option>{option}</option>{/each}</select></label>
      <label>Size<select bind:value={size}><option>Small</option><option>Medium</option></select></label>
      <label>Appearance and description (optional)<textarea maxlength="2000" bind:value={description}></textarea></label>
    </div>
    <label>Backstory and aspirations (optional)<textarea rows="3" maxlength="8000" bind:value={backstory}></textarea></label>
    <p class="muted">Backstory is your proposal. It does not automatically establish people, possessions or facts in the campaign world.</p>
    <h3>Abilities</h3><p>Use each score once: 15, 14, 13, 12, 10, 8. Soldier grants +2/+1 to different physical abilities, or +1 to each.</p>
    <div class="ability-grid">{#each ABILITIES as ability, i}
      <div><label>{ability} score<select bind:value={scores[i]}>{#each [15,14,13,12,10,8] as score}<option value={score}>{score}</option>{/each}</select></label>
      {#if i < 3}<label>{ability} Soldier increase<select bind:value={boosts[i]}>{#each [0,1,2] as boost}<option value={boost}>+{boost}</option>{/each}</select></label>{/if}<p>Final: <strong>{scores[i]+boosts[i]}</strong></p></div>
    {/each}</div>
    <h3>Skills and languages</h3><p>Soldier grants Athletics and Intimidation. Each further choice must be a different skill.</p>
    <div class="form-grid">
      {#each [0,1] as i}<label>Fighter skill {i+1}<select bind:value={fighter[i]}>{#each options.fighter_skills.filter(s => !['Athletics','Intimidation'].includes(s)) as skill}<option value={skill}>{label(skill)}</option>{/each}</select></label>{/each}
      <label>Human skill<select bind:value={human}>{#each SKILLS as skill}<option value={skill}>{label(skill)}</option>{/each}</select></label>
      {#each [0,1,2] as i}<label>Skilled feat skill {i+1}<select bind:value={skilled[i]}>{#each SKILLS as skill}<option value={skill}>{label(skill)}</option>{/each}</select></label>{/each}
      {#each [0,1] as i}<label>Additional language {i+1}<select bind:value={languages[i]}>{#each options.standard_languages as language}<option value={language}>{label(language)}</option>{/each}</select></label>{/each}
      <label>Gaming set proficiency<select bind:value={gaming}><option value="Dice">Dice</option><option value="Dragonchess">Dragonchess</option><option value="PlayingCards">Playing cards</option><option value="ThreeDragonAnte">Three-Dragon Ante</option></select></label>
      <label>Fighting Style<select bind:value={style}><option value="Defense">Defense — +1 AC while wearing armor</option><option value="Archery">Archery — +2 to ranged weapon attack rolls</option></select></label>
    </div>
    <p>Common is included. Choose three Simple or Martial weapons for your mastery training; you do not need to own them.</p>
    <div class="form-grid">{#each [0,1,2] as i}<label>Weapon mastery {i+1}<select bind:value={masteries[i]}>{#each options.fighter_masteries as weapon}<option value={weapon}>{label(weapon)}</option>{/each}</select></label>{/each}</div>
    <h3>Starting equipment</h3>
    <p>Starting gold: {money(options.catalog.starting_money_cp)}. Choose source-priced purchases; unspent gold stays on the sheet.</p>
    <div class="equipment-grid">{#each options.catalog.items as item}<label>{item.name} · {money(item.unit_cost_cp * item.purchase_multiple)}{item.purchase_multiple > 1 ? ` per ${item.purchase_multiple}` : ' each'}<input aria-label={`${item.name} quantity`} type="number" min="0" max="1000" step={item.purchase_multiple} bind:value={quantities[item.id]} placeholder="0" /></label>{/each}</div>
    <p aria-live="polite"><strong>Remaining: {money(remaining)}</strong></p>
    <label class="check"><input type="checkbox" bind:checked={armor} disabled={!quantities['leather-armor']} />Wear purchased Leather Armor</label>
    <label class="check"><input type="checkbox" bind:checked={shield} disabled={!quantities.shield} />Equip purchased Shield</label>
    <p class="muted">Buying an item does not execute its use. Weapon properties, ammunition and hand requirements are not yet resolved by this table interface.</p>
    {#if error}<p role="alert" class="error">{error}</p>{/if}
    <button type="submit" disabled={!playerId}>Create character</button>
  </fieldset>
</form>

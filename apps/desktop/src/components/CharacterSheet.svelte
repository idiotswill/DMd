<script lang="ts">
  import { ABILITIES, label, money, signed, type CharacterView } from '../table-api';
  let { character, onPrepare, disabled = false }: { character: CharacterView; onPrepare?: () => void; disabled?: boolean } = $props();
  const profile = $derived(character.profile);
  const sheet = $derived(character.sheet?.Character);
</script>

<article class="character-sheet" aria-label={`${character.name} character sheet`}>
  <h3>{character.name}</h3>
  {#if profile && sheet}
    <p>{profile.pronouns}{profile.pronouns ? ' · ' : ''}{label(profile.species_id)} {label(profile.class_id)} {profile.level} · {label(profile.background_id)} · {profile.alignment}</p>
    <dl class="stats"><div><dt>Hit points</dt><dd>{sheet.hp} / {sheet.max_hp}</dd></div><div><dt>Temporary HP</dt><dd>{sheet.temporary_hp}</dd></div><div><dt>Armor class</dt><dd>{sheet.armor_class}</dd></div><div><dt>Proficiency</dt><dd>{signed(sheet.proficiency_bonus)}</dd></div><div><dt>Speed</dt><dd>{profile.speed_feet} ft</dd></div><div><dt>Second Wind</dt><dd>{character.second_wind_remaining ?? '—'} / 2</dd></div></dl>
    <div class="ability-grid">{#each ABILITIES as ability, i}<div><strong>{ability}</strong><p>{character.details?.ability_scores[i] ?? profile.base_ability_scores[i]+profile.background_boosts[i]} <span class="muted">({signed(sheet.ability_modifiers[i])})</span></p></div>{/each}</div>
    {#if character.details}
      <p><strong>Hit dice:</strong> {character.details.hit_dice.remaining} / {character.details.hit_dice.maximum} d{character.details.hit_dice.sides}. <strong>Heroic Inspiration:</strong> {character.details.heroic_inspiration ? 'Available' : 'Not held'}.</p>
      <p><strong>Conditions:</strong> {character.details.conditions.length ? character.details.conditions.map(label).join(', ') : 'None'}. <strong>Exhaustion:</strong> {character.details.exhaustion}.</p>
      {#if character.details.death.dead}<p class="error">This character has died.</p>{:else if sheet.hp === 0}<p>Death saves: {character.details.death.successes} successes, {character.details.death.failures} failures. {character.details.death.stable ? 'Stable.' : 'Not stable.'}</p>{/if}
      <details><summary>Saving throws and all skills</summary><div class="sheet-columns"><div><h4>Saving throws</h4><ul>{#each character.details.saving_throws as save}<li>{save.ability} {signed(save.modifier)}{save.proficient ? ' · proficient' : ''}</li>{/each}</ul></div><div><h4>Skills</h4><ul>{#each character.details.skills as skill}<li>{label(skill.skill)} {signed(skill.modifier)} · {skill.ability}{skill.proficiency ? ` · ${skill.proficiency.toLowerCase()}` : ''}</li>{/each}</ul></div></div></details>
    {/if}
    <p><strong>Size:</strong> {profile.size}. <strong>Experience:</strong> {profile.experience_points}.</p>
    <p><strong>Proficient skills:</strong> {['Athletics','Intimidation',...profile.fighter_skills,profile.human_skill,...profile.skilled_skills].map(label).join(', ')}.</p>
    <p><strong>Languages:</strong> {profile.languages.map(label).join(', ')}.</p>
    <p><strong>Fighting Style:</strong> {profile.fighting_style}. <strong>Armor training:</strong> {profile.armor_training.map(label).join(', ')}.</p>
    <p><strong>Weapon proficiencies:</strong> {profile.weapon_proficiencies.map(label).join(', ')}. <strong>Gaming set proficiency:</strong> {profile.tool_proficiencies.map(label).join(', ')}.</p>
    <details><summary>Equipment and features</summary>
      <p><strong>Money:</strong> {money(profile.money_cp)}.</p>
      {#if character.equipment?.prepared}
        <p><strong>Worn armor:</strong> {character.equipment.items.find(item => item.id === character.equipment?.worn_armor)?.name ?? 'None'}. <strong>Shield:</strong> {character.equipment.shield ? 'Equipped' : 'Not equipped'}.</p>
        {#if character.equipment.items.length}<ul>{#each character.equipment.items as item}<li>{item.quantity} × {item.name}</li>{/each}</ul>{:else}<p>No equipment currently carried.</p>{/if}
      {:else}
        <p><strong>Starting equipment:</strong></p>
        {#if profile.equipment.length}<ul>{#each profile.equipment as item}<li>{item.quantity} × {item.display_name}</li>{/each}</ul>{:else}<p>No starting equipment purchased.</p>{/if}
        {#if onPrepare}<button type="button" {disabled} onclick={onPrepare}>Prepare {character.name}'s equipment</button>{/if}
      {/if}
      <p><strong>Mastery grants:</strong> {profile.masteries.map(label).join(', ')}. Mastery effects and full weapon-property resolution are not available in this table interface yet.</p>
      <ul><li>Second Wind: report a d10; Fighter level is added by the rules engine. Limited uses are saved.</li><li>Human Resourceful: grants Heroic Inspiration after a completed long rest. Rest controls are not available in this table interface yet.</li><li>Savage Attacker: recorded on your sheet. Weapon attack and damage selection controls are not available in this table interface yet.</li><li>Skillful and Skilled: the selected skill proficiencies are included in supported checks.</li></ul>
    </details>
    <details><summary>Description and backstory</summary><p>{profile.description || 'No description supplied.'}</p><p class="preserve">{profile.backstory || 'No backstory supplied.'}</p><p class="muted">Player-authored background; not automatically accepted world facts.</p></details>
  {:else}<p class="muted">Private sheet details are visible to this character's controller and the local host.</p>{/if}
</article>

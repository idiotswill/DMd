<script lang="ts">
  import { untrack } from 'svelte';
  import { newId, label, type CreatureCreation, type CreatureSetupView, type CreatureSize } from '../table-api';
  let { setup, disabled = false, onCreate }: { setup: CreatureSetupView; disabled?: boolean; onCreate: (creation: CreatureCreation) => void } = $props();
  let definition = $state(untrack(() => setup.catalog[0]?.definition_id ?? ''));
  const selected = $derived(setup.catalog.find(source => source.definition_id === definition));
  let name = $state(''); let size = $state<CreatureSize>(untrack(() => setup.catalog[0]?.sizes[0] ?? 'Medium'));
  let languages = $state(''); let ammunition = $state(20); let error = $state('');
  function changeSource(event: Event) {
    const source = setup.catalog.find(option => option.definition_id === (event.currentTarget as HTMLSelectElement).value);
    if (source) { definition = source.definition_id; size = source.sizes[0]; }
    languages = ''; error = '';
  }
  function submit(event: SubmitEvent) {
    event.preventDefault(); error = '';
    if (!selected) return;
    const additional_languages = languages.split(',').map(language => language.trim()).filter(Boolean);
    if (additional_languages.length !== selected.additional_languages || new Set(additional_languages).size !== additional_languages.length) {
      error = `Choose ${selected.additional_languages} distinct additional languages.`; return;
    }
    onCreate({ entity_id: newId(), name: name.trim() || selected.name, definition_id: selected.definition_id, size, additional_languages,
      ammunition_units: selected.ammunition_required ? ammunition : 0, item_ids: Array.from({length:selected.item_count}, newId) });
  }
</script>
<form onsubmit={submit}><fieldset {disabled}><legend>Prepare a creature</legend>
  <p>Preparation saves creature statistics and equipment. Only the host can see this setup. Encounter play is not available in this build yet.</p>
  <label>Creature source<select bind:value={definition} onchange={changeSource}>{#each setup.catalog as source}<option value={source.definition_id}>{source.name}</option>{/each}</select></label>
  {#if selected}
    <label>Creature name<input maxlength="200" placeholder={selected.name} bind:value={name} /></label>
    <label>Creature size<select bind:value={size}>{#each selected.sizes as option}<option>{option}</option>{/each}</select></label>
    {#if selected.additional_languages}<label>Additional languages ({selected.additional_languages}, comma separated)<input required bind:value={languages} /></label>{/if}
    {#if selected.ammunition_required}<label>Starting ammunition per type<input type="number" required min="1" max="1000" step="1" bind:value={ammunition} /></label><p>This finite supply is saved with the creature and is spent during play.</p>{/if}
    <p>Source abilities: {selected.abilities.join(', ')}.</p>
    {#if selected.omitted_features.length}<p>This creature definition does not yet include: {selected.omitted_features.map(label).join(', ')}.</p>{/if}
    {#if error}<p role="alert">{error}</p>{/if}
    <button type="submit">Prepare creature</button>
  {/if}
</fieldset></form>
{#if setup.creatures.length}<h3>Prepared creatures</h3><ul>{#each setup.creatures as creature}<li>{creature.name} · {creature.size} · {creature.hp}/{creature.max_hp} HP</li>{/each}</ul>{/if}

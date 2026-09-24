<script lang="ts">
  import { ABILITIES, SKILLS, label, newId, type Ability, type Skill, type Situation } from '../table-api';
  let { disabled = false, onSave }: { disabled?: boolean; onSave: (situation: Situation) => void } = $props();
  interface DraftChallenge { id: string; title: string; description: string; phrases: string; ability: Ability; skill: Skill | ''; dc: number; success: string; failure: string }
  let title = $state(''); let description = $state('');
  let challenges = $state<DraftChallenge[]>([]);
  let error = $state('');
  function addChallenge() { challenges.push({ id: newId(), title: '', description: '', phrases: '', ability: 'Strength', skill: '', dc: 10, success: '', failure: '' }); }
  function save(event: SubmitEvent) {
    event.preventDefault(); error = '';
    const prepared = challenges.map(c => ({ id: c.id, title: c.title, description: c.description, phrases: c.phrases.split(',').map(p => p.trim()).filter(Boolean), kind: { Check: { ability: c.ability, skill: c.skill || null } }, dc: c.dc, success: c.success, failure: c.failure, resolution: null }));
    if (prepared.some(c => !c.phrases.length || c.phrases.length > 20 || c.phrases.some(p => p.length > 200))) { error = 'Each check needs 1–20 short action phrases, separated by commas.'; return; }
    onSave({ title, description, challenges: prepared });
  }
</script>
<form onsubmit={save}><fieldset {disabled}>
  <legend>Establish a situation</legend>
  <p>The local host establishes the scene and any ability checks. Players describe their actions in ordinary words; unsupported actions remain open for clarification.</p>
  <label>Situation title<input required maxlength="200" bind:value={title} /></label>
  <label>What everyone can see and know<textarea required rows="4" maxlength="16000" bind:value={description}></textarea></label>
  <p class="muted">Only the public description is shown to players. Check difficulty and unrevealed consequences stay in host controls.</p>
  {#each challenges as challenge, index (challenge.id)}
    <fieldset><legend>Ability check {index+1}</legend>
      <div class="form-grid">
        <label>Check title<input required maxlength="200" bind:value={challenge.title} /></label>
        <label>Relevant context<textarea required maxlength="4000" bind:value={challenge.description}></textarea></label>
        <label>Action phrases, separated by commas<input required maxlength="4000" bind:value={challenge.phrases} /><small>Use short ordinary phrases players might say.</small></label>
        <label>Ability<select bind:value={challenge.ability}>{#each ABILITIES as ability}<option>{ability}</option>{/each}</select></label>
        <label>Applicable skill<select bind:value={challenge.skill}><option value="">No skill</option>{#each SKILLS as skill}<option value={skill}>{label(skill)}</option>{/each}</select></label>
        <label>Difficulty class<input required type="number" min="0" max="30" step="1" bind:value={challenge.dc} /></label>
        <label>Consequence on success<textarea required maxlength="4000" bind:value={challenge.success}></textarea></label>
        <label>Consequence on failure<textarea required maxlength="4000" bind:value={challenge.failure}></textarea></label>
      </div>
      <button type="button" class="secondary" onclick={() => { challenges = challenges.filter(c => c.id !== challenge.id); }}>Remove check {index+1}</button>
    </fieldset>
  {/each}
  <div class="actions"><button type="button" class="secondary" onclick={addChallenge} disabled={challenges.length >= 100}>Add ability check</button><button type="submit">Establish new situation</button></div>
  <p class="muted">This replaces the current situation. Previous accepted outcomes remain in the saved transcript. Consequence text records the check result; it does not automatically move characters or apply damage.</p>
  {#if error}<p role="alert" class="error">{error}</p>{/if}
</fieldset></form>

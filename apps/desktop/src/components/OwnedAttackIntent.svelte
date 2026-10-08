<script lang="ts">
  import type { Ability } from '../table-api';
  import { distinguishableAttackTargets, type OwnedAttackContext } from '../owned-attack-context';
  import { emptyAttackChoices, gripKey, gripLabel, proposeOwnedAttack, type AttackIntentDraft } from '../owned-attack-intent';
  let { context, onConfirm }: { context: OwnedAttackContext; onConfirm: (key: string, draft: AttackIntentDraft) => void } = $props();
  let text = $state('');
  let reviewed = $state(false);
  let choices = $state(emptyAttackChoices());
  const proposal = $derived(proposeOwnedAttack(text, context.options, choices));
  const distinguishable = $derived(distinguishableAttackTargets(context, proposal.targets));
  function heldLabel(item: string) {
    const hands = context.options.hands.hands.map((slot, index) => typeof slot === 'object' && slot.Item === item ? index === 0 ? 'left' : 'right' : '').filter(Boolean);
    return hands.length === 2 ? 'both hands' : `${hands[0]} hand`;
  }
  function edit(value: string) { text = value; reviewed = false; choices = emptyAttackChoices(); }
  function review(event: SubmitEvent) { event.preventDefault(); choices = emptyAttackChoices(); reviewed = true; }
  function confirm() {
    // Re-read the current offers/choices; a previously displayed command is not retained.
    const current = proposeOwnedAttack(text, context.options, choices);
    if (reviewed && current.choice && distinguishableAttackTargets(context, current.targets)) {
      onConfirm(context.key, { text, choices: { ...choices } });
    }
  }
</script>

<section class="panel" data-tactical-focus="attack-intent" tabindex="-1">
  <h2>Describe a melee attack</h2>
  <form onsubmit={review}>
    <label>Attack declaration<input value={text} oninput={(event) => edit(event.currentTarget.value)} maxlength="512" placeholder="I attack the guard with my sword" required/></label>
    <p>Describe one attack with a weapon you already hold. Review the target and weapon before attacking.</p>
    <button type="submit">Review attack declaration</button>
  </form>
  {#if reviewed}
    {#if proposal.error}<p role="status">{proposal.error}</p>
    {:else if !distinguishable}
      <p role="status">These target names cannot be distinguished from your current view. Clarify which creature you mean before attacking; nothing has been sent.</p>
    {:else}
      <div class="form-grid">
        <label>Attack declaration target<select value={proposal.target} onchange={(event) => choices = { ...emptyAttackChoices(), target: event.currentTarget.value }}>
          <option value="" disabled>Choose the intended located creature</option>
          {#each proposal.targets as target (target.actor)}<option value={target.actor}>{target.label}{context.targetDetails.find(detail => detail.actor === target.actor)?.position ? ` · ${context.targetDetails.find(detail => detail.actor === target.actor)?.position}` : ''}</option>{/each}
        </select></label>
        {#if proposal.target}
          <label>Attack declaration weapon<select value={proposal.weapon} onchange={(event) => choices = { ...emptyAttackChoices(), target: proposal.target, weapon: event.currentTarget.value }}>
            <option value="" disabled>Choose the held weapon</option>
            {#each proposal.weapons as weapon (weapon.item)}<option value={weapon.item}>{weapon.name} · held in {heldLabel(weapon.item)}</option>{/each}
          </select></label>
        {/if}
        {#if proposal.weapon}
          <label>Attack declaration ability<select value={proposal.ability} onchange={(event) => choices = { ...choices, ability: event.currentTarget.value as Ability }}>
            <option value="" disabled>Choose the attack ability</option>
            {#each proposal.abilities as ability}<option value={ability}>{ability}</option>{/each}
          </select></label>
          <label>Attack declaration grip<select value={proposal.grip} onchange={(event) => choices = { ...choices, grip: event.currentTarget.value }}>
            <option value="" disabled>Choose the grip</option>
            {#each proposal.grips as grip (gripKey(grip))}<option value={gripKey(grip)}>{gripLabel(grip)}</option>{/each}
          </select></label>
        {/if}
      </div>
      {#if proposal.choice}
        <p>Attack {proposal.targets.find(target => target.actor === proposal.target)?.label} with {proposal.weapons.find(weapon => weapon.item === proposal.weapon)?.name}: melee, {proposal.ability}, {gripLabel(proposal.choice.grip)}, ordinary Attack action.</p>
        <p>The rules check the current attack before requesting physical dice.</p>
      {:else}<p>Choose the missing details before attacking. Nothing has been sent.</p>{/if}
      <button type="button" disabled={!proposal.choice} onclick={confirm}>Attack as described</button>
    {/if}
  {/if}
</section>

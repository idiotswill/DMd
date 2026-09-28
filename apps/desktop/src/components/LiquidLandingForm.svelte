<script lang="ts">
  import type { TacticalAction } from '../tactical-api';
  import type { Id } from '../table-api';
  import { tacticalPromptIdentity } from '../tactical-focus';
  let { actor, disabled = false, onAction }: {
    actor: Id; disabled?: boolean; onAction: (action: TacticalAction) => void;
  } = $props();
</script>

<fieldset data-tactical-focus="prompt" data-tactical-focus-id={tacticalPromptIdentity({kind:'liquid-landing',actor})} tabindex="-1" {disabled}>
  <legend>Landing in liquid</legend>
  <p>You may spend your Reaction to make a DC 15 Strength (Athletics) or Dexterity (Acrobatics) check. A success halves the falling damage.</p>
  <button onclick={() => onAction({ ChooseLiquidLanding: { choice: 'Athletics' } })}>Use Athletics</button>
  <button onclick={() => onAction({ ChooseLiquidLanding: { choice: 'Acrobatics' } })}>Use Acrobatics</button>
  <button class="secondary" onclick={() => onAction({ ChooseLiquidLanding: { choice: null } })}>Decline landing Reaction</button>
</fieldset>

<script lang="ts">
  import type { TacticalAction } from '../tactical-api';
  let { disabled = false, onAction }: {
    disabled?: boolean; onAction: (action: TacticalAction) => void;
  } = $props();
</script>

<fieldset {disabled}>
  <legend>Landing in liquid</legend>
  <p>You may spend your Reaction to make a DC 15 Strength (Athletics) or Dexterity (Acrobatics) check. A success halves the falling damage.</p>
  <button onclick={() => onAction({ ChooseLiquidLanding: { choice: 'Athletics' } })}>Use Athletics</button>
  <button onclick={() => onAction({ ChooseLiquidLanding: { choice: 'Acrobatics' } })}>Use Acrobatics</button>
  <button class="secondary" onclick={() => onAction({ ChooseLiquidLanding: { choice: null } })}>Decline landing Reaction</button>
</fieldset>

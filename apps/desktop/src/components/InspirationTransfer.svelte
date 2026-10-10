<script lang="ts">
  import type { TableView } from '../table-api';
  let { view, host, disabled = false, canChoose = true, onChoice }: {
    view: TableView; host: boolean; disabled?: boolean; canChoose?: boolean; onChoice: (handle: string) => void;
  } = $props();
  const transfer = $derived(view.inspiration_transfer);
  const name = $derived(view.characters.find(c=>c.character_id===transfer?.character_id)?.name ?? 'The character');
</script>

{#if transfer}
  <fieldset {disabled}>
    <legend>Extra Heroic Inspiration</legend>
    {#if host}
      <p>{name}'s player must choose whether to give away the extra Inspiration. Select their attending player channel to continue.</p>
    {:else if canChoose}
      <p>Your original Inspiration stays. Give the extra to an eligible character, or decline it. The extra is lost if you decline.</p>
      {#each transfer.choices as choice (choice.key)}
        <button type="button" onclick={()=>onChoice(choice.key)}>{choice.label}</button>
      {/each}
    {:else}
      <p>Select your player character to choose what happens to the extra Inspiration.</p>
    {/if}
  </fieldset>
{/if}

<script lang="ts">
  import type { PhysicalView, PhysicalFactInput, PhysicalItemInput } from '../table-api';
  let { physical, host, disabled = false, onEnable, onAccept }: {
    physical?: PhysicalView; host: boolean; disabled?: boolean;
    onEnable?: () => void; onAccept?: (handle: string, input: PhysicalFactInput) => void;
  } = $props();
  let selected = $state('');
  const control = $derived(physical?.controls.find(c=>c.key===selected));
  let pounds = $state(''); let reason = $state(''); let name = $state('');
  let unknown = $state(false); let complete = $state(false); let separate = $state(false);
  let choice = $state('catalog'); let condition = $state('');
  let cp = $state(0); let sp = $state(0); let ep = $state(0); let gp = $state(0); let pp = $state(0);
  function select(event: Event) { const next=physical?.controls.find(c=>c.key===(event.currentTarget as HTMLSelectElement).value); pounds=''; reason=''; name=''; unknown=false; complete=false; separate=false; condition=''; choice=next?.source ? 'catalog' : next?.unit ? 'unit' : 'unknown'; cp=0;sp=0;ep=0;gp=0;pp=0; }
  function submit(event: SubmitEvent) {
    event.preventDefault();
    if (!control || disabled || !onAccept) return;
    const entered = pounds.trim(); const explanation = reason.trim();
    let input: PhysicalFactInput;
    if (control.kind==='body') input={Body:{pounds:unknown?null:entered,reason:explanation}};
    else if (control.kind==='coverage') input={Coverage:{complete,reason:explanation}};
    else if (control.kind==='personal') input={PersonalItem:{name:name.trim(),pounds:entered,separate_from_listed:separate,reason:explanation}};
    else if (control.kind==='currency') input={Currency:{coins:{cp,sp,ep,gp,pp},reason:explanation}};
    else {
      let item: PhysicalItemInput = 'Unknown';
      if (choice==='catalog') item={Catalog:{condition:condition||null}};
      if (choice==='unit') item={Unit:{pounds:entered,condition:condition||null}};
      if (choice==='gross') item={Gross:{pounds:entered}};
      if (choice==='nonstandard') item={NonstandardUnit:{description:name.trim(),pounds:entered}};
      input={Item:{choice:item,reason:explanation}};
    }
    onAccept(control.key,input);
  }
</script>

{#if physical || host}
  <section class="panel" aria-label="Physical facts and load">
    <h2>Physical facts and load</h2>
    {#if !physical}
      <p>Record missing body and equipment weights, actual coins and additional apparel. Printed weights remain source values. This does not enable carrying.</p>
      <button disabled={disabled || !onEnable} onclick={()=>onEnable?.()}>Enable physical facts</button>
    {:else}
      {#each physical.actors as actor (actor.actor)}
        <article>
          <h3>{actor.name}</h3>
          <p>Unladen body: {actor.body_pounds === null ? 'unknown' : `${actor.body_pounds} lb`}</p>
          {#if actor.load}
            <p>{actor.load.complete ? 'Equipment load' : 'Known equipment subtotal'}: {actor.load.known_pounds} lb</p>
            {#if actor.load.items.length}<details><summary>Physical equipment</summary><ul>{#each actor.load.items as item (item.item)}<li>{item.name} · {item.quantity} units · {item.pounds === null ? 'unknown weight' : `${item.pounds} lb`}<br/>{item.basis}</li>{/each}</ul></details>{/if}
            {#if actor.load.body_and_load_pounds !== null}<p>Body and equipment together: {actor.load.body_and_load_pounds} lb</p>{/if}
            {#if !actor.load.complete}<ul>{#each actor.load.unresolved as reason}<li>{reason}</li>{/each}</ul>{/if}
          {:else}<p>Load details are private.</p>{/if}
        </article>
      {/each}
      {#if host}
        {#if !physical.controls.length}<p>Finish the encounter and pending work before editing physical facts.</p>{:else}
          <form onsubmit={submit}>
            <label>Physical fact<select bind:value={selected} onchange={select} disabled={disabled}><option value="">Choose a subject</option>{#each physical.controls as option (option.key)}<option value={option.key}>{option.label}</option>{/each}</select></label>
            {#if control}
              {#if control.source}<p>{control.source}</p>{/if}
              {#if control.kind==='item'}
                <label>Mass treatment<select bind:value={choice} disabled={disabled}>
                  {#if control.source}<option value="catalog">Use the printed value or condition</option>{/if}
                  {#if control.unit}<option value="unit">Record an unquantified unit's weight</option>{/if}
                  {#if control.gross}<option value="gross">Record this object's current total weight</option>{/if}
                  <option value="nonstandard">Describe a nonstandard physical object and its unit weight</option>
                  <option value="unknown">Mark unresolved</option>
                </select></label>
                {#if control.conditions.length && (choice==='catalog' || choice==='unit')}<label>Actual form or condition<select required bind:value={condition} disabled={disabled}><option value="">Choose the actual condition</option>{#each control.conditions as option}<option value={option}>{option}</option>{/each}</select></label>{/if}
              {/if}
              {#if control.kind==='body'}<label><input type="checkbox" bind:checked={unknown} disabled={disabled}/> Weight is unknown</label><p>Exclude clothing, equipment and carried objects.</p>{/if}
              {#if (control.kind==='body' && !unknown) || control.kind==='personal' || (control.kind==='item' && (choice==='unit' || choice==='gross' || choice==='nonstandard'))}
                <label>Pounds<input required type="text" inputmode="decimal" maxlength="27" pattern={'(0|[1-9][0-9]*)(\\.[0-9]{1,6})?'} bind:value={pounds} disabled={disabled}/></label>
              {/if}
              {#if control.kind==='item' && choice==='nonstandard'}<label>Physical difference from the ordinary catalog object<input required maxlength="200" bind:value={name} disabled={disabled}/></label><p>This records this object's physical weight. Its printed source entry and combat properties stay unchanged.</p>{/if}
              {#if control.kind==='personal'}
                <label>Additional apparel or payload<input required maxlength="200" bind:value={name} disabled={disabled}/></label>
                <label><input required type="checkbox" bind:checked={separate} disabled={disabled}/> This is a separate previously unlisted object, excluding the body, listed equipment, contents and coins.</label>
              {/if}
              {#if control.kind==='coverage'}<label><input type="checkbox" bind:checked={complete} disabled={disabled}/> All additional apparel and payload is listed as physical items; nothing untracked remains.</label><p>Leave unchecked to record that coverage is still unresolved.</p>{/if}
              {#if control.kind==='currency'}
                <p>Describe the actual coins representing the existing {control.wallet_cp} CP wallet value. This records or corrects physical facts; it does not create additional wealth or perform an exchange.</p>
                <label>CP coins<input type="number" min="0" max="4294967295" step="1" required bind:value={cp} disabled={disabled}/></label>
                <label>SP coins<input type="number" min="0" max="4294967295" step="1" required bind:value={sp} disabled={disabled}/></label>
                <label>EP coins<input type="number" min="0" max="4294967295" step="1" required bind:value={ep} disabled={disabled}/></label>
                <label>GP coins<input type="number" min="0" max="4294967295" step="1" required bind:value={gp} disabled={disabled}/></label>
                <label>PP coins<input type="number" min="0" max="4294967295" step="1" required bind:value={pp} disabled={disabled}/></label>
              {/if}
              <label>Physical description or correction reason<textarea required maxlength="1000" bind:value={reason} disabled={disabled}></textarea></label>
              <button disabled={disabled}>Record physical fact</button>
            {/if}
          </form>
        {/if}
      {/if}
    {/if}
  </section>
{/if}

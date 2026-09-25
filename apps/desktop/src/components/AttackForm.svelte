<script lang="ts">
  import type { Ability } from '../table-api';
  import type { AttackOptions, TacticalAction, WeaponAttackPurpose, WeaponDelivery, WeaponGrip, WeaponUseChoice } from '../tactical-api';
  let { options, disabled=false, opportunity=false, onAction }: {
    options:AttackOptions; disabled?:boolean; opportunity?:boolean; onAction:(action:TacticalAction)=>void;
  }=$props();
  let item=$state(''); let target=$state(''); let ammunition=$state('');
  let delivery=$state<WeaponDelivery>('Melee'); let ability=$state<Ability>('Strength');
  let gripIndex=$state(0); let equipment=$state('held'); let purposeKey=$state('');
  const available=$derived(options.weapons.filter(weapon=>weapon.purposes.length>0));
  const selected=$derived(available.find(weapon=>weapon.item===item));
  function chooseWeapon(id:string) {
    item=id; const weapon=options.weapons.find(candidate=>candidate.item===id);
    delivery=weapon?.deliveries[0]??'Melee'; ability=weapon?.abilities[0]??'Strength';
    gripIndex=0; ammunition=''; equipment='held'; purposeKey=JSON.stringify(weapon?.purposes[0]??null);
  }
  $effect(()=>{
    if(!available.some(weapon=>weapon.item===item)) chooseWeapon(available[0]?.item??'');
    if(!options.targets.some(candidate=>candidate.actor===target)) target='';
    if(ammunition && !selected?.ammunition.some(stack=>stack.id===ammunition)) ammunition='';
    if(!selected?.purposes.some(purpose=>JSON.stringify(purpose)===purposeKey)) purposeKey=JSON.stringify(selected?.purposes[0]??null);
  });
  const gripLabel=(grip:WeaponGrip)=>grip==='TwoHands'?'Both hands':`${grip.OneHand} hand`;
  const handLabel=(hand:'Free'|{Item:string})=>hand==='Free'?'free':options.weapons.find(weapon=>weapon.item===hand.Item)?.name??'occupied';
  const purposeLabel=(purpose:WeaponAttackPurpose)=>purpose==='Normal'?(opportunity?'Opportunity attack · reaction':'Attack action'):'LightBonus' in purpose?'Light extra attack · bonus action':'Nick' in purpose?'Nick extra attack · Attack action':'Cleave extra attack';
  function submit(event:SubmitEvent) {
    event.preventDefault();
    const purpose=selected?.purposes.find(choice=>JSON.stringify(choice)===purposeKey);
    if(!selected || !purpose || !target || !selected.grips[gripIndex] || (selected.ammunition_required&&!ammunition)) return;
    let change:WeaponUseChoice['equipment_change']=null;
    if(!opportunity&&(equipment==='draw-left'||equipment==='draw-right')) change={timing:'BeforeAttack',operation:{Equip:{item,hand:equipment==='draw-left'?'Left':'Right'}}};
    if(!opportunity&&equipment==='stow-after') change={timing:'AfterAttack',operation:{Unequip:{item}}};
    onAction({Attack:{choice:{weapon:item,target,delivery,ability,grip:selected.grips[gripIndex],purpose,ammunition:selected.ammunition_required?ammunition:null,equipment_change:change}}});
  }
</script>
<form onsubmit={submit}>
  <fieldset {disabled}><legend>{opportunity?'Held weapon reaction':'Weapon attack'}</legend>
    <p>Left hand: {handLabel(options.hands.hands[0])}. Right hand: {handLabel(options.hands.hands[1])}.</p>
    {#if available.length && options.targets.length}
      <div class="form-grid">
        <label>Weapon<select value={item} onchange={(event)=>chooseWeapon(event.currentTarget.value)} required>{#each available as weapon,index}<option value={weapon.item}>{weapon.name} · {index+1}</option>{/each}</select></label>
        <label>Target<select bind:value={target} required><option value="" disabled>Choose a located creature</option>{#each options.targets as candidate}<option value={candidate.actor}>{candidate.label}</option>{/each}</select></label>
        {#if selected}
          <label>Attack opportunity<select bind:value={purposeKey}>{#each selected.purposes as purpose}<option value={JSON.stringify(purpose)}>{purposeLabel(purpose)}</option>{/each}</select></label>
          <label>Attack method<select bind:value={delivery}>{#each selected.deliveries as method}<option value={method}>{method==='Shot'?'Shoot':method==='Thrown'?'Throw':'Melee'}</option>{/each}</select></label>
          <label>Attack ability<select bind:value={ability}>{#each selected.abilities as choice}<option value={choice}>{choice}</option>{/each}</select></label>
          <label>Weapon grip<select bind:value={gripIndex}>{#each selected.grips as grip,index}<option value={index}>{gripLabel(grip)}</option>{/each}</select></label>
          {#if !opportunity}<label>Ready or put away weapon<select bind:value={equipment}><option value="held">Use the weapon as held</option><option value="draw-left">Ready in left hand before attacking</option><option value="draw-right">Ready in right hand before attacking</option><option value="stow-after">Put away after attacking</option></select></label>{/if}
          {#if selected.ammunition_required}<label>Ammunition<select bind:value={ammunition} required><option value="" disabled>Choose a carried stack</option>{#each selected.ammunition as stack,index}<option value={stack.id}>{stack.name} · {stack.quantity} remaining · stack {index+1}</option>{/each}</select></label>{/if}
        {/if}
      </div>
      <p>The rules check hands, range and current circumstances before requesting dice.</p>
      <button disabled={!selected || !selected.purposes.length || !target || (selected.ammunition_required&&!ammunition)}>Attack</button>
    {:else if !available.length}<p>No physical weapon attack is currently available.</p>
    {:else}<p>This creature has no currently located target.</p>{/if}
  </fieldset>
</form>

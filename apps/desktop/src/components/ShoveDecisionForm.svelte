<script lang="ts">
  import type { ShoveDecision, ShoveView, TacticalAction } from '../tactical-api';
  import { tacticalPromptIdentity } from '../tactical-focus';
  let { shove, disabled=false, onAction }: { shove:ShoveView;disabled?:boolean;onAction:(action:TacticalAction)=>void }=$props();
  let horizontal=$state('');
  let vertical=$state('');
  const directions=[{value:'-1,-1',label:'Northwest'},{value:'0,-1',label:'North'},{value:'1,-1',label:'Northeast'},
    {value:'-1,0',label:'West'},{value:'0,0',label:'No horizontal change'},{value:'1,0',label:'East'},
    {value:'-1,1',label:'Southwest'},{value:'0,1',label:'South'},{value:'1,1',label:'Southeast'}];
  const valid=$derived(directions.some(d=>d.value===horizontal)&&['-1','0','1'].includes(vertical)&&!(horizontal==='0,0'&&vertical==='0'));
  function choose(decision:ShoveDecision){onAction({ShoveDecision:{handle:shove.key,decision}});}
  function push(){if(!valid||!shove.from)return;const [x,y]=horizontal.split(',').map(Number);choose({Outcome:{choice:{Push:{destination:{x:shove.from.x+x*10,y:shove.from.y+y*10,z:shove.from.z+Number(vertical)*10}}}}});}
</script>
<fieldset data-tactical-focus="prompt" data-tactical-focus-id={tacticalPromptIdentity({kind:'shove',key:shove.key,actor:shove.actor,stage:shove.stage})} tabindex="-1" {disabled}><legend>{shove.stage==='SaveChoice'?'Choose your Shove saving throw':shove.stage==='OutcomeChoice'?'Choose the Shove result':'Private host push review'}</legend>
  {#if shove.stage==='SaveChoice'}
    <p>Choose Strength or Dexterity. The rules determine any automatic failure and the dice needed. Choosing to fail a save remains a separate decision.</p>
    <button onclick={()=>choose({Save:{ability:'Strength'}})}>Strength saving throw</button>
    <button onclick={()=>choose({Save:{ability:'Dexterity'}})}>Dexterity saving throw</button>
  {:else if shove.stage==='OutcomeChoice'}
    <p>The save failed. Choose Prone or propose one five-foot step away. Every push goes to the host for the same geometry review.</p>
    <button onclick={()=>choose({Outcome:{choice:'Prone'}})}>Choose Prone</button>
    <label>Push direction<select bind:value={horizontal}><option value="">Choose a direction</option>{#each directions as direction}<option value={direction.value}>{direction.label}</option>{/each}</select></label>
    <label>Push elevation<select bind:value={vertical}><option value="">Choose elevation</option><option value="-1">Down five feet</option><option value="0">Keep elevation</option><option value="1">Up five feet</option></select></label>
    <button disabled={!valid} onclick={push}>Propose five-foot push</button>
  {:else}
    <p>Review the full swept body path from {shove.from?.x},{shove.from?.y},{shove.from?.z} to {shove.destination?.x},{shove.destination?.y},{shove.destination?.z} in half-foot map units. The rules require a five-foot grid step that increases distance from the shover. A blocked ruling requires a proven solid obstruction or battlefield boundary. Unsupported special geometry cannot be overridden.</p>
    <button onclick={()=>choose({RulePush:{ruling:'CommitExactPush'}})}>Commit exact push</button>
    <button onclick={()=>choose({RulePush:{ruling:'ConfirmBlockedNoMovement'}})}>Confirm blocked · no movement</button>
    <button class="secondary" onclick={()=>choose({RulePush:{ruling:'ReturnToShover'}})}>Return choice to shover</button>
    <p>No ruling refunds the attack, adds collision damage or substitutes Prone. Returning the choice preserves the failed save.</p>
  {/if}
</fieldset>

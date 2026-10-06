import {render,screen} from '@testing-library/svelte';
import userEvent from '@testing-library/user-event';
import {expect,it,vi} from 'vitest';
import type {AttackOptions,OpportunityView,PhysicalSourceWeaponChoice,TacticalView} from '../tactical-api';
import AttackForm from './AttackForm.svelte';
import OpportunityForm from './OpportunityForm.svelte';
import EncounterPanel from './EncounterPanel.svelte';

// Synthetic presentation DTOs only. These controls prove UI intent/selection,
// never admitted Ogre creation, finite inventory, legal reaction or accepted play.
const left=JSON.stringify({OneHand:'Left'});
const right=JSON.stringify({OneHand:'Right'});
const reactionKey=(feature:string,item:string)=>JSON.stringify([feature,item]);
function attacks():AttackOptions {
  return {actor:'ogre-ui-fixture',hands:{hands:['Free','Free']},targets:[{actor:'target',label:'Located creature'}],weapons:[
    {item:'club',name:'Greatclub',deliveries:['Melee'],abilities:['Strength'],grips:['TwoHands'],purposes:['Normal'],ammunition_required:false,ammunition:[],source_features:[{feature_id:'greatclub',label:'Greatclub',weapon:'club'}]},
    ...['javelin-a','javelin-b','javelin-c'].map(item=>({item,name:'Javelin',deliveries:['Melee','Thrown'] as const,abilities:['Strength'] as const,grips:[{OneHand:'Left' as const},{OneHand:'Right' as const}],purposes:['Normal' as const],ammunition_required:false,ammunition:[],source_features:[
      {feature_id:'javelin-melee',label:'Javelin (Melee)',weapon:item},
      {feature_id:'javelin-thrown',label:'Javelin (Thrown)',weapon:item},
    ]})).map(weapon=>({...weapon,deliveries:[...weapon.deliveries],abilities:[...weapon.abilities]})),
  ]};
}
function reactions():OpportunityView {
  return {actor:'ogre-ui-fixture',target:{actor:'target',label:'Departing creature'},weapons:null,unarmed:false,features:[],physical_source_weapons:[
    {feature_id:'greatclub',item:'club',label:'Greatclub',weapon_name:'Greatclub',grips:['TwoHands']},
    ...['javelin-a','javelin-b','javelin-c'].map(item=>({feature_id:'javelin-melee',item,label:'Javelin (Melee)',weapon_name:'Javelin',grips:[{OneHand:'Left' as const},{OneHand:'Right' as const}]})),
  ]};
}
const ownTurnCases=[['club','greatclub'],...['javelin-a','javelin-b','javelin-c'].flatMap(item=>[[item,'javelin-melee'],[item,'javelin-thrown']])];
it.each(ownTurnCases)('emits only the offered physical source intent for %s / %s',async(item,feature)=>{
  const user=userEvent.setup();const onAction=vi.fn();
  render(AttackForm,{options:attacks(),onAction});
  await user.selectOptions(screen.getByLabelText('Weapon'),item);
  await user.selectOptions(screen.getByLabelText('Target'),'target');
  await user.selectOptions(screen.getByLabelText('Source action'),feature);
  expect(screen.queryByLabelText('Attack method')).toBeNull();
  expect(screen.queryByLabelText('Attack ability')).toBeNull();
  expect(screen.queryByLabelText('Ammunition')).toBeNull();
  await user.click(screen.getByRole('button',{name:/^Attack$/}));
  expect(onAction).toHaveBeenCalledExactlyOnceWith({CreatureWeaponAttack:{feature_id:feature,choice:{weapon:item,target:'target',grip:item==='club'?'TwoHands':{OneHand:'Left'},ammunition:null,equipment_change:null}}});
});

it('keeps Item, grip and source form across reorder, then requires explicit replacement after removal',async()=>{
  const user=userEvent.setup();const onAction=vi.fn();const original=attacks();
  const component=render(AttackForm,{options:original,onAction});
  await user.selectOptions(screen.getByLabelText('Weapon'),'javelin-b');
  await user.selectOptions(screen.getByLabelText('Target'),'target');
  await user.selectOptions(screen.getByLabelText('Source action'),'javelin-thrown');
  await user.selectOptions(screen.getByLabelText('Weapon grip'),right);
  const label=(screen.getByLabelText('Weapon') as HTMLSelectElement).selectedOptions[0].textContent;
  const reordered={...original,weapons:[...original.weapons].reverse().map(weapon=>({...weapon,grips:[...weapon.grips].reverse(),source_features:[...(weapon.source_features??[])].reverse()}))};
  await component.rerender({options:reordered});
  expect((screen.getByLabelText('Weapon') as HTMLSelectElement).selectedOptions[0].textContent).toBe(label);
  await user.click(screen.getByRole('button',{name:/^Attack$/}));
  expect(onAction.mock.calls[0][0]).toEqual({CreatureWeaponAttack:{feature_id:'javelin-thrown',choice:{weapon:'javelin-b',target:'target',grip:{OneHand:'Right'},ammunition:null,equipment_change:null}}});
  const noRight={...reordered,weapons:reordered.weapons.map(weapon=>({...weapon,grips:weapon.grips.filter(grip=>JSON.stringify(grip)!==right)}))};
  await component.rerender({options:noRight});
  expect((screen.getByRole('button',{name:/^Attack$/}) as HTMLButtonElement).disabled).toBe(true);
  await component.rerender({options:reordered});
  expect((screen.getByRole('button',{name:/^Attack$/}) as HTMLButtonElement).disabled).toBe(true);
  await user.selectOptions(screen.getByLabelText('Weapon grip'),left);
  const noThrown={...reordered,weapons:reordered.weapons.map(weapon=>({...weapon,source_features:weapon.source_features.filter(source=>source.feature_id!=='javelin-thrown')}))};
  await component.rerender({options:noThrown});
  await user.click(screen.getByRole('button',{name:/^Attack$/}));
  expect(onAction).toHaveBeenCalledTimes(1);
  await component.rerender({options:reordered});
  expect((screen.getByRole('button',{name:/^Attack$/}) as HTMLButtonElement).disabled).toBe(true);
  await user.selectOptions(screen.getByLabelText('Source action'),'javelin-melee');
  await user.click(screen.getByRole('button',{name:/^Attack$/}));
  expect(onAction.mock.calls[1][0].CreatureWeaponAttack).toMatchObject({feature_id:'javelin-melee',choice:{weapon:'javelin-b',grip:{OneHand:'Left'}}});
  await component.rerender({options:{...reordered,weapons:reordered.weapons.filter(weapon=>weapon.item!=='javelin-b')}});
  expect((screen.getByLabelText('Weapon') as HTMLSelectElement).value).toBe('');
  await component.rerender({options:reordered});
  expect((screen.getByRole('button',{name:/^Attack$/}) as HTMLButtonElement).disabled).toBe(true);
  await user.selectOptions(screen.getByLabelText('Weapon'),'javelin-c');
  await user.click(screen.getByRole('button',{name:/^Attack$/}));
  expect(onAction.mock.calls[2][0].CreatureWeaponAttack.choice.weapon).toBe('javelin-c');
});

it('uses stable physical reaction tuples and grips without source mechanics or ordinary attack fields',async()=>{
  const user=userEvent.setup();const onAction=vi.fn();const original=reactions();
  const component=render(OpportunityForm,{opportunity:original,onAction});
  expect((screen.getByRole('button',{name:'Use source weapon reaction'}) as HTMLButtonElement).disabled).toBe(true);
  await user.selectOptions(screen.getByLabelText('Reaction source weapon'),reactionKey('javelin-melee','javelin-b'));
  await user.selectOptions(screen.getByLabelText('Reaction weapon grip'),right);
  const reordered={...original,physical_source_weapons:[...original.physical_source_weapons!].reverse().map(choice=>({...choice,grips:[...choice.grips].reverse()}))};
  await component.rerender({opportunity:reordered});
  await user.click(screen.getByRole('button',{name:'Use source weapon reaction'}));
  expect(onAction).toHaveBeenCalledExactlyOnceWith({OpportunityAttack:{choice:{CreatureWeapon:{feature_id:'javelin-melee',weapon:'javelin-b',grip:{OneHand:'Right'}}}}});
  await user.selectOptions(screen.getByLabelText('Reaction source weapon'),reactionKey('greatclub','club'));
  await user.click(screen.getByRole('button',{name:'Use source weapon reaction'}));
  expect(onAction.mock.calls[1][0]).toEqual({OpportunityAttack:{choice:{CreatureWeapon:{feature_id:'greatclub',weapon:'club',grip:'TwoHands'}}}});
  expect(screen.queryByLabelText('Target')).toBeNull();
  expect(screen.queryByLabelText('Ready or put away weapon')).toBeNull();
});

it('invalidates removed reaction choices and grips without silently substituting or reviving them',async()=>{
  const user=userEvent.setup();const onAction=vi.fn();const original=reactions();
  const component=render(OpportunityForm,{opportunity:original,onAction});
  await user.selectOptions(screen.getByLabelText('Reaction source weapon'),reactionKey('javelin-melee','javelin-b'));
  await user.selectOptions(screen.getByLabelText('Reaction weapon grip'),right);
  const without=(predicate:(choice:PhysicalSourceWeaponChoice)=>boolean)=>({...original,physical_source_weapons:original.physical_source_weapons!.filter(predicate)});
  await component.rerender({opportunity:without(choice=>choice.item!=='javelin-b')});
  await component.rerender({opportunity:original});
  expect((screen.getByRole('button',{name:'Use source weapon reaction'}) as HTMLButtonElement).disabled).toBe(true);
  await user.selectOptions(screen.getByLabelText('Reaction source weapon'),reactionKey('javelin-melee','javelin-b'));
  await user.selectOptions(screen.getByLabelText('Reaction weapon grip'),right);
  await component.rerender({opportunity:{...original,physical_source_weapons:original.physical_source_weapons!.map(choice=>({...choice,grips:choice.grips.filter(grip=>JSON.stringify(grip)!==right)}))}});
  await component.rerender({opportunity:original});
  expect((screen.getByRole('button',{name:'Use source weapon reaction'}) as HTMLButtonElement).disabled).toBe(true);
  await user.selectOptions(screen.getByLabelText('Reaction weapon grip'),left);
  await component.rerender({disabled:true});
  await user.click(screen.getByRole('button',{name:'Use source weapon reaction'}));
  expect(onAction).not.toHaveBeenCalled();
  await component.rerender({disabled:false,opportunity:{...original,physical_source_weapons:[]}});
  expect(screen.queryByRole('button',{name:'Use source weapon reaction'})).toBeNull();
  await user.click(screen.getByRole('button',{name:'Let the creature pass'}));
  expect(onAction).toHaveBeenCalledExactlyOnceWith('DeclineOpportunity');
});

it('keeps physical source reactions private to the current controller and resets them on a channel change',async()=>{
  const user=userEvent.setup();const onAction=vi.fn();
  const tactical:TacticalView={encounter_id:'encounter',round:1,active_actor:'target',phase:'active',execution:'EncounterReleaseV1',battlefield:null,participants:[],combatant_sources:[],observers:[],initiative:[],ties:[],continuation:null,may_fail_save:null,legendary_resistance:null,legendary_action:null,budget:null,opportunity:reactions()};
  const component=render(EncounterPanel,{tactical,characters:[],host:false,actor:'ogre-ui-fixture',player:'controller',playerControlledSources:['ogre-ui-fixture'],onAction});
  await user.selectOptions(screen.getByLabelText('Reaction source weapon'),reactionKey('greatclub','club'));
  await component.rerender({actor:'other',player:'observer'});
  expect(screen.queryByLabelText('Reaction source weapon')).toBeNull();
  expect(screen.queryByText('Departing creature')).toBeNull();
  await component.rerender({host:true,actor:null,player:null});
  expect(screen.queryByLabelText('Reaction source weapon')).toBeNull();
  await component.rerender({host:false,actor:'ogre-ui-fixture',player:'controller'});
  expect((screen.getByRole('button',{name:'Use source weapon reaction'}) as HTMLButtonElement).disabled).toBe(true);
  expect(onAction).not.toHaveBeenCalled();
});

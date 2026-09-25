import { render, screen } from '@testing-library/svelte';
import { expect, it, vi } from 'vitest';
import userEvent from '@testing-library/user-event';
import AttackForm from './AttackForm.svelte';
import EncounterPanel from './EncounterPanel.svelte';
import type { AttackOptions, TacticalView } from '../tactical-api';

function options():AttackOptions {
  return {actor:'actor',hands:{hands:['Free','Free']},targets:[{actor:'target',label:'Visible guard'}],weapons:[
    {item:'dagger',name:'Dagger',deliveries:['Melee','Thrown'],abilities:['Strength','Dexterity'],grips:[{OneHand:'Left'},{OneHand:'Right'}],purposes:['Normal'],ammunition_required:false,ammunition:[]},
    {item:'bow',name:'Shortbow',deliveries:['Shot'],abilities:['Dexterity'],grips:['TwoHands'],purposes:['Normal'],ammunition_required:true,ammunition:[{id:'stack',name:'Arrows',quantity:4}]},
  ]};
}

it('uses the canonical source action for a physical weapon while preserving ordinary looted use',async()=>{
  const user=userEvent.setup();const onAction=vi.fn();const initial=options();
  initial.weapons[1].source_features=[{feature_id:'shortbow',label:'Shortbow',weapon:'bow'}];
  render(AttackForm,{options:initial,onAction});
  await user.selectOptions(screen.getByLabelText('Weapon'),'bow');
  await user.selectOptions(screen.getByLabelText('Target'),'target');
  expect(screen.queryByLabelText('Attack ability')).toBeNull();
  expect(screen.queryByLabelText('Attack method')).toBeNull();
  await user.selectOptions(screen.getByLabelText('Ammunition'),'stack');
  await user.selectOptions(screen.getByLabelText('Ready or put away weapon'),'draw-left');
  await user.click(screen.getByRole('button',{name:/^Attack$/}));
  expect(onAction).toHaveBeenLastCalledWith({CreatureWeaponAttack:{feature_id:'shortbow',choice:{weapon:'bow',target:'target',grip:'TwoHands',ammunition:'stack',equipment_change:{timing:'BeforeAttack',operation:{Equip:{item:'bow',hand:'Left'}}}}}});
  await user.selectOptions(screen.getByLabelText('Source action'),'');
  expect(screen.getByLabelText('Attack ability')).toBeTruthy();
  await user.selectOptions(screen.getByLabelText('Ready or put away weapon'),'stow-after');
  await user.click(screen.getByRole('button',{name:/^Attack$/}));
  expect(onAction.mock.calls[1][0]).toEqual({Attack:{choice:{weapon:'bow',target:'target',delivery:'Shot',ability:'Dexterity',grip:'TwoHands',purpose:'Normal',ammunition:'stack',equipment_change:{timing:'AfterAttack',operation:{Unequip:{item:'bow'}}}}}});
});

it('preserves physical weapon, target, hand and ability choices without sending mechanics',async()=>{
  const user=userEvent.setup();const onAction=vi.fn();
  render(AttackForm,{options:options(),onAction});
  await user.selectOptions(screen.getByLabelText('Target'),'target');
  await user.selectOptions(screen.getByLabelText('Attack method'),'Thrown');
  await user.selectOptions(screen.getByLabelText('Attack ability'),'Dexterity');
  await user.selectOptions(screen.getByLabelText('Weapon grip'),'1');
  await user.selectOptions(screen.getByLabelText('Ready or put away weapon'),'draw-right');
  await user.click(screen.getByRole('button',{name:/^Attack$/}));
  expect(onAction).toHaveBeenCalledExactlyOnceWith({Attack:{choice:{weapon:'dagger',target:'target',delivery:'Thrown',ability:'Dexterity',grip:{OneHand:'Right'},purpose:'Normal',ammunition:null,equipment_change:{timing:'BeforeAttack',operation:{Equip:{item:'dagger',hand:'Right'}}}}}});
});

it('resets source-incompatible choices when switching weapons and clears vanished targets',async()=>{
  const user=userEvent.setup();const onAction=vi.fn();const initial=options();
  const component=render(AttackForm,{options:initial,onAction});
  await user.selectOptions(screen.getByLabelText('Target'),'target');
  await user.selectOptions(screen.getByLabelText('Attack method'),'Thrown');
  await user.selectOptions(screen.getByLabelText('Weapon'),'bow');
  expect((screen.getByLabelText('Attack method') as HTMLSelectElement).value).toBe('Shot');
  expect((screen.getByLabelText('Attack ability') as HTMLSelectElement).value).toBe('Dexterity');
  expect((screen.getByRole('button',{name:/^Attack$/}) as HTMLButtonElement).disabled).toBe(true);
  await user.selectOptions(screen.getByLabelText('Ammunition'),'stack');
  await user.click(screen.getByRole('button',{name:/^Attack$/}));
  expect(onAction.mock.calls[0][0].Attack.choice).toMatchObject({weapon:'bow',ammunition:'stack',grip:'TwoHands',delivery:'Shot'});
  await component.rerender({options:{...initial,targets:[]}});
  expect(screen.queryByText('Visible guard')).toBeNull();
  expect(screen.queryByRole('button',{name:/^Attack$/})).toBeNull();
});

it('removes a previous actor weapon form when the viewing channel changes',async()=>{
  const user=userEvent.setup();
  const tactical:TacticalView={encounter_id:'encounter',round:1,active_actor:'actor',phase:'active',execution:'ReactionsV1',battlefield:null,participants:[],combatant_sources:[],observers:[],initiative:[],ties:[],continuation:null,may_fail_save:null,legendary_resistance:null,legendary_action:null,
    budget:{movement_spent:0,attacks_remaining:0,action_spent:false,bonus_action_spent:false,reaction_available:true},attack_options:options()};
  const component=render(EncounterPanel,{tactical,characters:[],host:true,actor:null,player:null,onAction:vi.fn()});
  expect(screen.getByText('Weapon attack')).toBeTruthy();
  await user.selectOptions(screen.getByLabelText('Target'),'target');
  await user.selectOptions(screen.getByLabelText('Attack method'),'Thrown');
  await component.rerender({host:false,actor:'actor',player:'player'});
  expect((screen.getByLabelText('Target') as HTMLSelectElement).value).toBe('');
  expect((screen.getByLabelText('Attack method') as HTMLSelectElement).value).toBe('Melee');
  await component.rerender({host:false,actor:'other',player:'other-player'});
  expect(screen.queryByText('Weapon attack')).toBeNull();
  expect(screen.queryByText('Visible guard')).toBeNull();
});

it('sends owned knockout and Graze decisions and removes them on a channel change',async()=>{
  const user=userEvent.setup();const onAction=vi.fn();
  const tactical:TacticalView={encounter_id:'encounter',round:1,active_actor:'actor',phase:'active',execution:'ReactionsV1',battlefield:null,participants:[],combatant_sources:[],observers:[],initiative:[],ties:[],continuation:{actor:'actor',host_adjudication:false,choices:[]},may_fail_save:null,legendary_resistance:null,legendary_action:null,budget:null,attack_decision:{actor:'actor',kind:'Knockout'}};
  const component=render(EncounterPanel,{tactical,characters:[],host:false,actor:'actor',player:'player',onAction});
  await user.click(screen.getByRole('button',{name:'Knock out'}));
  expect(onAction).toHaveBeenLastCalledWith({ChooseAttackKnockout:{choice:'KnockOut'}});
  await user.click(screen.getByRole('button',{name:'Apply normal damage'}));
  expect(onAction).toHaveBeenLastCalledWith({ChooseAttackKnockout:{choice:'NormalDamage'}});
  await component.rerender({tactical:{...tactical,attack_decision:{actor:'actor',kind:'Graze'}}});
  expect(screen.queryByRole('button',{name:'Knock out'})).toBeNull();
  await user.click(screen.getByRole('button',{name:'Use Graze'}));
  expect(onAction).toHaveBeenLastCalledWith({ChooseAttackMastery:{choice:'Graze'}});
  await user.click(screen.getByRole('button',{name:'Decline Graze'}));
  expect(onAction).toHaveBeenLastCalledWith({ChooseAttackMastery:{choice:'Decline'}});
  await component.rerender({actor:'other',player:'other-player'});
  expect(screen.queryByText('Graze mastery')).toBeNull();
});

it('offers Light and Nick after the action is spent and removes an expended follow-up',async()=>{
  const user=userEvent.setup();const onAction=vi.fn();const initial=options();
  initial.weapons[0].purposes=[{LightBonus:{trigger:'accepted-attack'}},{Nick:{trigger:'accepted-attack'}}];
  initial.weapons[1].purposes=[];
  const tactical:TacticalView={encounter_id:'encounter',round:1,active_actor:'actor',phase:'active',execution:'ReactionsV1',battlefield:null,participants:[],combatant_sources:[],observers:[],initiative:[],ties:[],continuation:null,may_fail_save:null,legendary_resistance:null,legendary_action:null,
    budget:{movement_spent:0,attacks_remaining:0,action_spent:true,bonus_action_spent:false,reaction_available:true},attack_options:initial};
  const component=render(EncounterPanel,{tactical,characters:[],host:false,actor:'actor',player:'player',onAction});
  await user.selectOptions(screen.getByLabelText('Target'),'target');
  expect(screen.queryByRole('option',{name:'Shortbow · 2'})).toBeNull();
  await user.selectOptions(screen.getByLabelText('Attack opportunity'),JSON.stringify({Nick:{trigger:'accepted-attack'}}));
  await user.click(screen.getByRole('button',{name:/^Attack$/}));
  expect(onAction.mock.calls[0][0].Attack.choice.purpose).toEqual({Nick:{trigger:'accepted-attack'}});
  const changed={...initial,weapons:initial.weapons.map(weapon=>({...weapon,purposes:weapon.item==='dagger'?[{LightBonus:{trigger:'later-accepted-attack'}}]:[]}))};
  await component.rerender({tactical:{...tactical,attack_options:changed}});
  await user.click(screen.getByRole('button',{name:/^Attack$/}));
  expect(onAction.mock.calls[1][0].Attack.choice.purpose).toEqual({LightBonus:{trigger:'later-accepted-attack'}});
  await component.rerender({tactical:{...tactical,attack_options:{...initial,weapons:initial.weapons.map(weapon=>({...weapon,purposes:[]}))}}});
  expect(screen.queryByText('Weapon attack')).toBeNull();
});

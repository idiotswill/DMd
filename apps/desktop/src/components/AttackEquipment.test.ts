import { render, screen } from '@testing-library/svelte';
import userEvent from '@testing-library/user-event';
import { expect, it, vi } from 'vitest';
import AttackEquipmentForm from './AttackEquipmentForm.svelte';
import AttackForm from './AttackForm.svelte';
import type { AttackEquipmentView, AttackOptions } from '../tactical-api';

it('keeps Decline available with no legal Apply and binds it to the selected opaque work',async()=>{
  const user=userEvent.setup(); const onAction=vi.fn();
  render(AttackEquipmentForm,{choice:{key:'selected',actor:'retired-attacker',operations:[],may_decline:true},onAction});
  expect(screen.queryByRole('button',{name:'Apply equipment choice'})).toBeNull();
  await user.click(screen.getByRole('button',{name:'Decline equipment choice'}));
  expect(onAction).toHaveBeenCalledExactlyOnceWith({AttackEquipment:{handle:'selected',choice:'Decline'}});
});

it('preserves the operation identity through reorder and clears a vanished choice',async()=>{
  const user=userEvent.setup(); const onAction=vi.fn();
  const pickup={Pickup:{item:'loose-spear',hand:'Left' as const}};
  const choice:AttackEquipmentView={key:'work',actor:'actor',may_decline:true,operations:[
    {operation:pickup,label:'Pick up Spear · left hand'},
    {operation:{Unequip:{item:'held-dagger'}},label:'Put away Dagger'},
  ]};
  const component=render(AttackEquipmentForm,{choice,onAction});
  await user.selectOptions(screen.getByLabelText('Equipment operation'),JSON.stringify(pickup));
  await component.rerender({choice:{...choice,operations:[...choice.operations].reverse()}});
  await user.click(screen.getByRole('button',{name:'Apply equipment choice'}));
  expect(onAction).toHaveBeenLastCalledWith({AttackEquipment:{handle:'work',choice:{Apply:pickup}}});
  await component.rerender({choice:{...choice,key:'new-work'}});
  expect((screen.getByRole('button',{name:'Apply equipment choice'}) as HTMLButtonElement).disabled).toBe(true);
  await user.selectOptions(screen.getByLabelText('Equipment operation'),JSON.stringify(pickup));
  await component.rerender({choice:{...choice,operations:[choice.operations[1]]}});
  expect((screen.getByRole('button',{name:'Apply equipment choice'}) as HTMLButtonElement).disabled).toBe(true);
  await component.rerender({disabled:true});
  await user.click(screen.getByRole('button',{name:'Decline equipment choice'}));
  expect(onAction).toHaveBeenCalledTimes(1);
});

function options():AttackOptions {
  return {actor:'actor',hands:{hands:['Free','Free']},targets:[{actor:'target',label:'Visible target'}],
    equipment:{pickups:[{item:'different-spear',name:'Spear',hands:['Left']}]},weapons:[
      {item:'dagger',name:'Dagger',deliveries:['Thrown'],abilities:['Strength'],grips:[{OneHand:'Right'}],purposes:['Normal'],ammunition_required:false,ammunition:[]},
    ]};
}

it('picks a different Item before attacking and makes the later choice mutually exclusive',async()=>{
  const user=userEvent.setup();const onAction=vi.fn();
  render(AttackForm,{options:options(),onAction});
  await user.selectOptions(screen.getByLabelText('Target'),'target');
  await user.selectOptions(screen.getByLabelText('Ready or put away weapon'),'pickup');
  await user.selectOptions(screen.getByLabelText('Weapon to pick up'),'different-spear');
  await user.selectOptions(screen.getByLabelText('Pickup hand'),'Left');
  await user.click(screen.getByRole('button',{name:'Attack'}));
  expect(onAction.mock.calls[0][0].Attack.choice).toMatchObject({weapon:'dagger',equipment_change:{timing:'BeforeAttack',operation:{Pickup:{item:'different-spear',hand:'Left'}}}});
  expect(onAction.mock.calls[0][0].Attack.choice.after_equipment).toBeUndefined();
  await user.selectOptions(screen.getByLabelText('Ready or put away weapon'),'choose-after');
  expect(screen.queryByLabelText('Weapon to pick up')).toBeNull();
  await user.click(screen.getByRole('button',{name:'Attack'}));
  expect(onAction.mock.calls[1][0].Attack.choice).toMatchObject({equipment_change:null,after_equipment:'Choose'});
});

it('uses the same explicit after intent for a printed source and never offers it on reactions',async()=>{
  const user=userEvent.setup();const onAction=vi.fn();const source=options();
  source.weapons[0].source_features=[{feature_id:'fixture-printed-throw',label:'Throw',weapon:'dagger'}];
  const component=render(AttackForm,{options:source,onAction});
  await user.selectOptions(screen.getByLabelText('Target'),'target');
  await user.selectOptions(screen.getByLabelText('Ready or put away weapon'),'choose-after');
  await user.click(screen.getByRole('button',{name:'Attack'}));
  expect(onAction.mock.calls[0][0].CreatureWeaponAttack.choice).toMatchObject({equipment_change:null,after_equipment:'Choose'});
  await component.rerender({opportunity:true});
  expect(screen.queryByLabelText('Ready or put away weapon')).toBeNull();
  expect(screen.queryByLabelText('Weapon to pick up')).toBeNull();
});

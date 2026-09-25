import { render, screen } from '@testing-library/svelte';
import { expect, it, vi } from 'vitest';
import userEvent from '@testing-library/user-event';
import ShieldForm from './ShieldForm.svelte';
import EncounterPanel from './EncounterPanel.svelte';
import type { ShieldOptions, TacticalView } from '../tactical-api';

it('sends the real shield and legal hand without changing equipment optimistically', async()=>{
  const user=userEvent.setup();const onAction=vi.fn();
  const options:ShieldOptions={actor:'actor',donned:null,shields:[{item:'shield-a',hands:['Left','Right']},{item:'shield-b',hands:['Right']}]};
  const component=render(ShieldForm,{options,onAction});
  await user.selectOptions(screen.getByLabelText('Carried shield'),'shield-b');
  expect((screen.getByLabelText('Shield hand') as HTMLSelectElement).value).toBe('Right');
  await user.click(screen.getByRole('button',{name:'Don shield'}));
  expect(onAction).toHaveBeenLastCalledWith({DonShield:{shield:'shield-b',hand:'Right'}});
  expect(screen.queryByRole('button',{name:'Doff shield'})).toBeNull();
  await component.rerender({options:{actor:'actor',donned:'shield-b',shields:[]}});
  expect(screen.queryByLabelText('Carried shield')).toBeNull();
  await user.click(screen.getByRole('button',{name:'Doff shield'}));
  expect(onAction).toHaveBeenLastCalledWith('DoffShield');
  expect(screen.getByRole('button',{name:'Doff shield'})).toBeTruthy();
});

it('blocks shield action submission while delivery is pending and removes another actor controls',async()=>{
  const user=userEvent.setup();const onAction=vi.fn();
  const tactical:TacticalView={encounter_id:'encounter',round:1,active_actor:'actor',phase:'active',battlefield:null,participants:[],combatant_sources:[],observers:[],initiative:[],ties:[],continuation:null,may_fail_save:null,legendary_resistance:null,legendary_action:null,
    budget:{movement_spent:0,attacks_remaining:0,action_spent:false,bonus_action_spent:false,reaction_available:true},shield_options:{actor:'actor',donned:'actual-shield',shields:[]}};
  const component=render(EncounterPanel,{tactical,characters:[],host:false,actor:'actor',player:'owner',disabled:true,onAction});
  await user.click(screen.getByRole('button',{name:'Doff shield'}));
  expect(onAction).not.toHaveBeenCalled();
  await component.rerender({disabled:false,pendingRoll:true});
  await user.click(screen.getByRole('button',{name:'Doff shield'}));
  expect(onAction).not.toHaveBeenCalled();
  await component.rerender({pendingRoll:false});
  await user.click(screen.getByRole('button',{name:'Doff shield'}));
  expect(onAction).toHaveBeenCalledExactlyOnceWith('DoffShield');
  await component.rerender({actor:'other',player:'other-player'});
  expect(screen.queryByRole('button',{name:'Doff shield'})).toBeNull();
});

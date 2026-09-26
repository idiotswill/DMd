import { render, screen } from '@testing-library/svelte';
import userEvent from '@testing-library/user-event';
import { expect, it, vi } from 'vitest';
import AftermathForm from './AftermathForm.svelte';
import EncounterPanel from './EncounterPanel.svelte';
import type { TacticalView } from '../tactical-api';

function tactical():TacticalView {
  return {encounter_id:'encounter',phase:'active',execution:'ShieldHitV1',round:1,active_actor:'pc',battlefield:null,participants:[],combatant_sources:[],observers:[],initiative:[],ties:[],continuation:null,may_fail_save:null,legendary_resistance:null,legendary_action:null,budget:null};
}

it('requires explicit cadence and ruling before submitting the typed conclusion', async()=>{
  const user=userEvent.setup();const onAction=vi.fn();
  const component=render(AftermathForm,{onAction});
  const button=screen.getByRole('button',{name:'Conclude hostilities and retain timing'});
  const policy=screen.getByRole('checkbox');
  expect((policy as HTMLInputElement).checked).toBe(false);
  expect((button as HTMLButtonElement).disabled).toBe(true);
  await user.type(screen.getByLabelText('Private host timing ruling'),'  Keep the existing timing.  ');
  expect((button as HTMLButtonElement).disabled).toBe(true);
  await user.click(policy);
  await user.click(button);
  expect(onAction).toHaveBeenCalledExactlyOnceWith({ConcludeHostilities:{cadence:'ContinueExistingOrder',ruling:'Keep the existing timing.'}});
  await component.rerender({disabled:true});
  await user.click(button);
  expect(onAction).toHaveBeenCalledTimes(1);
});

it('offers conclusion only to the host and never for legacy timing', async()=>{
  const component=render(EncounterPanel,{tactical:tactical(),characters:[],host:false,actor:'pc',player:'player',onAction:vi.fn()});
  expect(screen.queryByRole('button',{name:'Conclude hostilities and retain timing'})).toBeNull();
  await component.rerender({host:true,actor:null,player:null});
  expect(screen.getByRole('button',{name:'Conclude hostilities and retain timing'})).toBeTruthy();
  await component.rerender({tactical:{...tactical(),execution:null}});
  expect(screen.queryByRole('button',{name:'Conclude hostilities and retain timing'})).toBeNull();
  await component.rerender({tactical:{...tactical(),execution:'ReactionsV1'}});
  expect(screen.queryByRole('button',{name:'Conclude hostilities and retain timing'})).toBeNull();
});

it('keeps the private ruling host-only while explaining preserved timing to players', async()=>{
  const source={...tactical(),aftermath:{cadence:'ContinueExistingOrder' as const,host_ruling:'Private adjudication detail',may_pause_session:true}};
  const component=render(EncounterPanel,{tactical:source,characters:[],host:true,actor:null,player:null,onAction:vi.fn()});
  expect(screen.getByText('Private adjudication detail')).toBeTruthy();
  expect(screen.queryByRole('button',{name:'Conclude hostilities and retain timing'})).toBeNull();
  await component.rerender({host:false,actor:'pc',player:'player'});
  expect(screen.queryByText('Private adjudication detail')).toBeNull();
  expect(screen.getByText(/Hostilities concluded/)).toBeTruthy();
});

it('blocks conclusion while the Shield response or its physical damage is pending', async()=>{
  const user=userEvent.setup();const onAction=vi.fn();
  const component=render(EncounterPanel,{tactical:tactical(),characters:[],host:true,actor:null,player:null,onAction});
  await user.type(screen.getByLabelText('Private host timing ruling'),'Retain ongoing timing.');
  await user.click(screen.getByRole('checkbox'));
  const button=screen.getByRole('button',{name:'Conclude hostilities and retain timing'});
  await component.rerender({tactical:{...tactical(),hit:{order:null,delegate:null,response:null}}});
  expect((button.closest('fieldset') as HTMLFieldSetElement).disabled).toBe(true);
  await user.click(button);
  expect(onAction).not.toHaveBeenCalled();
  await component.rerender({tactical:tactical(),pendingRoll:true});
  expect((button.closest('fieldset') as HTMLFieldSetElement).disabled).toBe(true);
  await user.click(button);
  expect(onAction).not.toHaveBeenCalled();
});

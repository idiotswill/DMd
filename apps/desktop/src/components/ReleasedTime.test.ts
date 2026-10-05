import { render, screen } from '@testing-library/svelte';
import userEvent from '@testing-library/user-event';
import { expect, it, vi } from 'vitest';
import EncounterPanel from './EncounterPanel.svelte';
import type { TacticalView } from '../tactical-api';

const finished=():TacticalView=>({encounter_id:'encounter',phase:'finished',execution:'ReleasedTimeV1',round:null,active_actor:null,battlefield:null,participants:[],observers:[],initiative:[],ties:[],budget:null,continuation:null,may_fail_save:null,legendary_resistance:null,legendary_action:null,combatant_sources:[],released_time:{may_advance:true,blocker:null,may_pause_session:false,interval:null}});

it('requires a positive interval and explicit private Host ruling',async()=>{
  const user=userEvent.setup();const onAction=vi.fn();
  const component=render(EncounterPanel,{tactical:finished(),characters:[],host:true,actor:null,player:null,onAction});
  expect((screen.getByRole('button',{name:'Advance elapsed time'}) as HTMLButtonElement).disabled).toBe(true);
  await user.clear(screen.getByLabelText('Elapsed seconds'));
  await user.type(screen.getByLabelText('Elapsed seconds'),'61');
  await user.type(screen.getByLabelText('Private Host timing ruling'),'  Wait beside the courtyard.  ');
  await user.click(screen.getByRole('button',{name:'Advance elapsed time'}));
  expect(onAction).toHaveBeenCalledExactlyOnceWith({AdvanceReleasedTime:{seconds:61,ordering:'HostSelect',ruling:'Wait beside the courtyard.'}});
  await component.rerender({disabled:true});
  expect(screen.getByLabelText('Elapsed seconds').closest('fieldset')?.disabled).toBe(true);
  await component.rerender({disabled:false,tactical:{...finished(),released_time:{may_advance:false,blocker:'Retained dependency needs resolution.',may_pause_session:false,interval:null}}});
  expect(screen.getByText('Retained dependency needs resolution.')).toBeTruthy();
  expect(screen.getByLabelText('Elapsed seconds').closest('fieldset')?.disabled).toBe(true);
  await component.rerender({host:false,actor:'player-actor',player:'player'});
  expect(screen.queryByLabelText('Elapsed seconds')).toBeNull();
  expect(screen.queryByLabelText('Private Host timing ruling')).toBeNull();
});

it('keeps the fixed target and submits only an opaque ordering handle',async()=>{
  const user=userEvent.setup();const onAction=vi.fn();
  const tactical:TacticalView={...finished(),released_time:{may_advance:false,blocker:null,may_pause_session:true,interval:{started_at:28799,progress_at:28800,target_at:28860,ruling:'Private elapsed ruling.',choices:[{handle:'opaque-one',label:'Resolve simultaneous deadline 1'},{handle:'opaque-two',label:'Resolve simultaneous deadline 2'}]}}};
  const component=render(EncounterPanel,{tactical,characters:[],host:true,actor:null,player:null,onAction});
  expect(screen.queryByLabelText('Elapsed seconds')).toBeNull();
  expect(screen.queryByRole('button',{name:'End turn'})).toBeNull();
  expect(screen.getByText(/World time 28800 of 28860/)).toBeTruthy();
  await user.click(screen.getByRole('button',{name:'Resolve simultaneous deadline 2'}));
  expect(onAction).toHaveBeenCalledExactlyOnceWith({ChooseTurnWork:{handle:'opaque-two'}});
  await component.rerender({disabled:true});
  expect(screen.getByRole('button',{name:'Resolve simultaneous deadline 1'}).closest('fieldset')?.disabled).toBe(true);
  await component.rerender({disabled:false,host:false,actor:'player-actor',player:'player'});
  expect(screen.queryByText(/Private elapsed ruling/)).toBeNull();
  expect(screen.queryByRole('button',{name:/Resolve simultaneous deadline/})).toBeNull();
});

it('requires a separate current-session upgrade from a saved flow-5 encounter',async()=>{
  const user=userEvent.setup();const onAction=vi.fn();
  const component=render(EncounterPanel,{tactical:{...finished(),execution:'EncounterReleaseV1',released_time:undefined},characters:[],host:true,actor:null,player:null,disabled:true,administrativeDisabled:false,onAction});
  expect((screen.getByRole('button',{name:'Continue saved encounter'}) as HTMLButtonElement).disabled).toBe(true);
  await component.rerender({disabled:false});
  await user.click(screen.getByRole('button',{name:'Continue saved encounter'}));
  expect(onAction).toHaveBeenCalledExactlyOnceWith({UpgradeExecutionTo:{execution:'ReleasedTimeV1'}});
});

it('keeps ordinary saved flow-5 turns playable before the elapsed-time upgrade',()=>{
  const tactical:TacticalView={...finished(),phase:'active',execution:'EncounterReleaseV1',active_actor:'actor',released_time:undefined,budget:{action_spent:false,bonus_action_spent:false,reaction_available:true,movement_spent:0,attacks_remaining:0}};
  render(EncounterPanel,{tactical,characters:[],host:false,actor:'actor',player:'player',onAction:vi.fn()});
  expect(screen.getByRole('button',{name:'Dodge'})).toBeTruthy();
  expect(screen.queryByLabelText('Elapsed seconds')).toBeNull();
});

import { render, screen } from '@testing-library/svelte';
import userEvent from '@testing-library/user-event';
import { expect, it, vi } from 'vitest';
import AreaForm from './AreaForm.svelte';
import BattlefieldForm from './BattlefieldForm.svelte';
import type { AreaOptions, TacticalAction } from '../tactical-api';
import { loadRequest, saveRequest, type CharacterView } from '../table-api';

const options=(controller:string|null='owner'):AreaOptions=>({
  actor:'source',controller,source_space:{min:{x:10,y:20,z:0},max:{x:30,y:40,z:12}},
  variants:[{feature_id:'fire-breath',label:'Fire Breath',length_feet:15}],unavailable:[],
});

it('requires a fresh explicit delegation and preserves exact aim and consent for retry',async()=>{
  const user=userEvent.setup();const onAction=vi.fn();
  render(AreaForm,{options:options(),host:false,player:'owner',onAction});
  const consent=screen.getByRole('checkbox',{name:/Let the host order/}) as HTMLInputElement;
  expect(consent.checked).toBe(false);
  await user.selectOptions(screen.getByLabelText('Ability'),'fire-breath');
  const button=screen.getByRole('button',{name:'Use area ability'}) as HTMLButtonElement;
  expect(button.disabled).toBe(true);
  await user.click(consent);
  await user.click(button);
  const action:TacticalAction={CreatureArea:{feature_id:'fire-breath',aim:{origin:{x:30,y:30,z:6},toward:{x:40,y:30,z:6},include_origin:false},ordering:'DelegateToHost'}};
  expect(onAction).toHaveBeenCalledExactlyOnceWith(action);
  const retained={kind:'action' as const,request:{command_id:'stable-command',campaign_id:'campaign',expected_event_sequence:18,session_id:'session',channel:{Player:{player_id:'owner',character_id:'character'}},action:{Tactical:{action}}}};
  saveRequest(retained);
  expect(loadRequest()).toEqual(retained);
});

it('host and other players cannot assert a player source consent',async()=>{
  const user=userEvent.setup();const onAction=vi.fn();
  const view=render(AreaForm,{options:options(),host:true,player:null,onAction});
  expect((screen.getByLabelText('Ability') as HTMLSelectElement).disabled).toBe(true);
  const consent=screen.getByRole('checkbox',{name:/Let the host order/}) as HTMLInputElement;
  expect(consent.disabled).toBe(true);
  await user.click(consent);
  await user.click(screen.getByRole('button',{name:'Use area ability'}));
  await view.rerender({host:false,player:'another'});
  expect((screen.getByLabelText('Ability') as HTMLSelectElement).disabled).toBe(true);
  expect(onAction).not.toHaveBeenCalled();
});

it('cancel and replacement remove consent while uncertain requests lock the complete form',async()=>{
  const user=userEvent.setup();const onAction=vi.fn();
  const view=render(AreaForm,{options:options(),host:false,player:'owner',onAction});
  await user.selectOptions(screen.getByLabelText('Ability'),'fire-breath');
  await user.click(screen.getByRole('checkbox',{name:/Let the host order/}));
  await view.rerender({disabled:true});
  await user.click(screen.getByRole('button',{name:'Use area ability'}));
  expect(onAction).not.toHaveBeenCalled();
  await view.rerender({disabled:false});
  await user.click(screen.getByRole('button',{name:'Cancel area choice'}));
  expect((screen.getByRole('checkbox',{name:/Let the host order/}) as HTMLInputElement).checked).toBe(false);
  await user.selectOptions(screen.getByLabelText('Ability'),'fire-breath');
  await user.click(screen.getByRole('checkbox',{name:/Let the host order/}));
  await view.rerender({host:true,player:null});
  await view.rerender({host:false,player:'owner'});
  expect((screen.getByRole('checkbox',{name:/Let the host order/}) as HTMLInputElement).checked).toBe(false);
  await user.selectOptions(screen.getByLabelText('Ability'),'fire-breath');
  await user.click(screen.getByRole('checkbox',{name:/Let the host order/}));
  const changed=options();changed.actor='new-source';
  await view.rerender({options:changed});
  expect((screen.getByLabelText('Ability') as HTMLSelectElement).value).toBe('');
  expect((screen.getByRole('checkbox',{name:/Let the host order/}) as HTMLInputElement).checked).toBe(false);
});

it('host-controlled sources need no player delegation and never choose target identifiers',async()=>{
  const user=userEvent.setup();const onAction=vi.fn();
  render(AreaForm,{options:options(null),host:true,player:null,onAction});
  expect(screen.queryByRole('checkbox',{name:/Let the host order/})).toBeNull();
  await user.selectOptions(screen.getByLabelText('Ability'),'fire-breath');
  await user.click(screen.getByRole('checkbox',{name:/Include the cone/}));
  await user.click(screen.getByRole('button',{name:'Use area ability'}));
  expect(onAction.mock.calls[0][0].CreatureArea.ordering).toBe('Host');
  expect(onAction.mock.calls[0][0].CreatureArea.aim.include_origin).toBe(true);
  expect(Object.keys(onAction.mock.calls[0][0].CreatureArea).sort()).toEqual(['aim','feature_id','ordering']);
});

it('map area adjudication is an explicit unselected host choice',async()=>{
  const user=userEvent.setup();const onPrepare=vi.fn();
  const character:CharacterView={character_id:'pc',player_id:'player',entity_id:'actor',name:'Hero',profile:null,sheet:null,details:null,second_wind_remaining:null,equipment:{prepared:true,initial_item_count:0,items:[],worn_armor:null,shield:null,hands:{hands:['Free','Free']}}};
  render(BattlefieldForm,{characters:[character],onPrepare});
  const policy=screen.getByRole('checkbox',{name:/Use occupied-space sampling/}) as HTMLInputElement;
  expect(policy.checked).toBe(false);
  await user.click(screen.getByRole('button',{name:'Prepare encounter map'}));
  expect(onPrepare.mock.calls[0][0].area_grid_policy).toBeNull();
  await user.click(policy);
  await user.click(screen.getByRole('button',{name:'Prepare encounter map'}));
  expect(onPrepare.mock.calls[1][0].area_grid_policy).toBe('OccupiedCellCentersV1');
  expect(screen.getByText(/Small portions use their own midpoint/)).toBeTruthy();
});

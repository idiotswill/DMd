import { render, screen } from '@testing-library/svelte';
import { expect, it, vi } from 'vitest';
import userEvent from '@testing-library/user-event';
import TacticalMap from './TacticalMap.svelte';
import EncounterPanel from './EncounterPanel.svelte';
import type { TacticalView } from '../tactical-api';

it('replaces host map truth when the selected viewer changes', async () => {
  const host:TacticalView={encounter_id:'encounter',round:null,active_actor:null,phase:'setup',
    battlefield:{bounds:{min:{x:0,y:0,z:0},max:{x:100,y:100,z:40}},floor_z:0,floor_surface:'stone',ambient_light:'Darkness',terrain:[],obstacles:[],lights:[]},
    participants:[{entity_id:'secret-actor',public_label:'Unseen sentinel',position:{x:70,y:80,z:0},size:'Medium'}],observers:[],initiative:[],ties:[],budget:null,continuation:null,may_fail_save:null};
  const component=render(TacticalMap,{tactical:host,characters:[]});
  expect(screen.getAllByText(/Unseen sentinel: 35 feet east/).length).toBeGreaterThan(0);
  const player:TacticalView={...host,battlefield:null,participants:[],observers:[{observer:'player-actor',position:{x:10,y:10,z:0},contacts:[],cells:[]}]};
  await component.rerender({tactical:player,characters:[]});
  expect(screen.queryByText(/Unseen sentinel/)).toBeNull();
  expect(screen.getByRole('img',{name:'Your known surroundings'})).toBeTruthy();
  expect(screen.queryByRole('img',{name:'Host encounter map'})).toBeNull();
});

it('keeps ordinary turns blocked and sends the retained consequence identity', async () => {
  const user=userEvent.setup();const onAction=vi.fn();
  const tactical:TacticalView={encounter_id:'encounter',round:2,active_actor:'actor',phase:'active',battlefield:null,participants:[],observers:[],initiative:[],ties:[],
    budget:{movement_spent:0,attacks_remaining:0,action_spent:false,bonus_action_spent:false,reaction_available:true},
    continuation:{actor:'actor',choices:[{occurrence:19,label:'Death saving throw'},{occurrence:27,label:'Concurrent consequence'}]},may_fail_save:null};
  const component=render(EncounterPanel,{tactical,characters:[],host:false,actor:'actor',player:'player',onAction});
  expect(screen.getByRole('button',{name:'End turn'}).closest('fieldset')?.disabled).toBe(true);
  await user.click(screen.getByRole('button',{name:'Concurrent consequence · 2'}));
  expect(onAction).toHaveBeenLastCalledWith({ChooseTurnWork:{occurrence:27}});
  await component.rerender({tactical:{...tactical,continuation:{actor:'actor',choices:[]},may_fail_save:'actor'},pendingRoll:true});
  expect(screen.queryByRole('button',{name:'Concurrent consequence · 2'})).toBeNull();
  await user.click(screen.getByRole('button',{name:'Choose to fail this save'}));
  expect(onAction).toHaveBeenLastCalledWith('VoluntarilyFailSave');
  await component.rerender({actor:'other',player:'other-player',tactical:{...tactical,budget:null,continuation:null,may_fail_save:null}});
  expect(screen.queryByText('Saving throw choice')).toBeNull();
  expect(screen.queryByText('Choose which consequence happens next')).toBeNull();
});

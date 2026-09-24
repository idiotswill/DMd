import { render, screen } from '@testing-library/svelte';
import { expect, it } from 'vitest';
import TacticalMap from './TacticalMap.svelte';
import type { TacticalView } from '../tactical-api';

it('replaces host map truth when the selected viewer changes', async () => {
  const host:TacticalView={encounter_id:'encounter',round:null,active_actor:null,phase:'setup',
    battlefield:{bounds:{min:{x:0,y:0,z:0},max:{x:100,y:100,z:40}},floor_z:0,floor_surface:'stone',ambient_light:'Darkness',terrain:[],obstacles:[],lights:[]},
    participants:[{entity_id:'secret-actor',public_label:'Unseen sentinel',position:{x:70,y:80,z:0},size:'Medium'}],observers:[],initiative:[],ties:[],budget:null};
  const component=render(TacticalMap,{tactical:host,characters:[]});
  expect(screen.getAllByText(/Unseen sentinel: 35 feet east/).length).toBeGreaterThan(0);
  const player:TacticalView={...host,battlefield:null,participants:[],observers:[{observer:'player-actor',position:{x:10,y:10,z:0},contacts:[],cells:[]}]};
  await component.rerender({tactical:player,characters:[]});
  expect(screen.queryByText(/Unseen sentinel/)).toBeNull();
  expect(screen.getByRole('img',{name:'Your known surroundings'})).toBeTruthy();
  expect(screen.queryByRole('img',{name:'Host encounter map'})).toBeNull();
});

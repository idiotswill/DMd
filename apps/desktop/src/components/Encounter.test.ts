import { render, screen } from '@testing-library/svelte';
import { expect, it, vi } from 'vitest';
import userEvent from '@testing-library/user-event';
import TacticalMap from './TacticalMap.svelte';
import EncounterPanel from './EncounterPanel.svelte';
import type { TacticalView } from '../tactical-api';
import type { CharacterView } from '../table-api';

it('uses an owned Second Wind bonus action and blocks another or pending turn', async () => {
  const user=userEvent.setup();const onAction=vi.fn();
  const tactical:TacticalView={encounter_id:'encounter',round:2,active_actor:'actor',phase:'active',battlefield:null,participants:[],observers:[],initiative:[],ties:[],
    budget:{movement_spent:0,attacks_remaining:0,action_spent:true,bonus_action_spent:false,reaction_available:true},
    continuation:null,may_fail_save:null,legendary_resistance:null,legendary_action:null,combatant_sources:[]};
  const character:CharacterView={character_id:'pc',player_id:'player',entity_id:'actor',name:'Fighter',profile:null,sheet:null,details:null,second_wind_remaining:2};
  const component=render(EncounterPanel,{tactical,characters:[character],host:false,actor:'actor',player:'player',onAction});
  await user.click(screen.getByRole('button',{name:'Second Wind · 2 uses'}));
  expect(onAction).toHaveBeenCalledExactlyOnceWith('SecondWind');
  await component.rerender({tactical:{...tactical,budget:{...tactical.budget!,bonus_action_spent:true}},characters:[{...character,second_wind_remaining:1}]});
  expect((screen.getByRole('button',{name:'Second Wind · 1 uses'}) as HTMLButtonElement).disabled).toBe(true);
  await component.rerender({tactical,pendingRoll:true});
  expect(screen.getByRole('button',{name:'Second Wind · 1 uses'}).closest('fieldset')?.disabled).toBe(true);
  await component.rerender({actor:'other',player:'other-player',pendingRoll:false});
  expect(screen.queryByRole('button',{name:/Second Wind/})).toBeNull();
  await component.rerender({actor:'actor',player:'player',characters:[{...character,second_wind_remaining:0}]});
  expect((screen.getByRole('button',{name:'Second Wind · 0 uses'}) as HTMLButtonElement).disabled).toBe(true);
});

it('replaces host map truth when the selected viewer changes', async () => {
  const host:TacticalView={encounter_id:'encounter',round:null,active_actor:null,phase:'setup',
    battlefield:{bounds:{min:{x:0,y:0,z:0},max:{x:100,y:100,z:40}},floor_z:0,floor_surface:'stone',ambient_light:'Darkness',terrain:[],obstacles:[],lights:[]},
    participants:[{entity_id:'secret-actor',public_label:'Unseen sentinel',position:{x:70,y:80,z:0},size:'Medium'}],observers:[],initiative:[],ties:[],budget:null,continuation:null,may_fail_save:null,legendary_resistance:null,legendary_action:null,combatant_sources:[]};
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
    continuation:{actor:'actor',host_adjudication:false,choices:[{handle:'opaque-death',label:'Death saving throw'},{handle:'opaque-consequence',label:'Concurrent consequence'}]},may_fail_save:null,legendary_resistance:null,legendary_action:null,combatant_sources:[]};
  const component=render(EncounterPanel,{tactical,characters:[],host:false,actor:'actor',player:'player',onAction});
  expect(screen.getByRole('button',{name:'End turn'}).closest('fieldset')?.disabled).toBe(true);
  await user.click(screen.getByRole('button',{name:'Concurrent consequence · 2'}));
  expect(onAction).toHaveBeenLastCalledWith({ChooseTurnWork:{handle:'opaque-consequence'}});
  await component.rerender({tactical:{...tactical,continuation:{actor:'actor',host_adjudication:false,choices:[]},may_fail_save:'actor'},pendingRoll:true});
  expect(screen.queryByRole('button',{name:'Concurrent consequence · 2'})).toBeNull();
  await user.click(screen.getByRole('button',{name:'Choose to fail this save'}));
  expect(onAction).toHaveBeenLastCalledWith('VoluntarilyFailSave');
  await component.rerender({actor:'other',player:'other-player',tactical:{...tactical,budget:null,continuation:null,may_fail_save:null,legendary_resistance:null,legendary_action:null,combatant_sources:[]}});
  expect(screen.queryByText('Saving throw choice')).toBeNull();
  expect(screen.queryByText('Choose which consequence happens next')).toBeNull();
});

it('shares identical creature initiative while separating current roll circumstances', async () => {
  const user=userEvent.setup();const onAction=vi.fn();
  const source={Creature:{definition_id:'goblin-warrior'}};
  const tactical:TacticalView={encounter_id:'encounter',round:null,active_actor:null,phase:'setup',battlefield:null,
    participants:['a','b','c'].map(entity_id=>({entity_id,public_label:entity_id,position:{x:0,y:0,z:0},size:'Small'})),
    combatant_sources:[{actor:'a',source,initiative_modifier:2,normal_mode:'Normal',surprised_mode:'Disadvantage'},{actor:'b',source,initiative_modifier:2,normal_mode:'Normal',surprised_mode:'Disadvantage'},{actor:'c',source,initiative_modifier:2,normal_mode:'Disadvantage',surprised_mode:'Disadvantage'}],
    observers:[],initiative:[],ties:[],budget:null,continuation:null,may_fail_save:null,legendary_resistance:null,legendary_action:null};
  render(EncounterPanel,{tactical,characters:[],host:true,actor:null,player:null,onAction});
  await user.click(screen.getByRole('button',{name:'Roll initiative'}));
  expect(onAction.mock.calls[0][0].Begin.groups.map((group:{actors:string[]})=>group.actors)).toEqual([['a','b'],['c']]);
  expect(Object.keys(onAction.mock.calls[0][0].Begin.combatants[0]).sort()).toEqual(['actor','source','surprised']);
});

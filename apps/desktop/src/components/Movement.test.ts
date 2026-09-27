import {render,screen} from '@testing-library/svelte';
import {expect,it,vi} from 'vitest';
import userEvent from '@testing-library/user-event';
import {proposeMovement} from '../movement-intent';
import type {MovementOptions,TacticalView} from '../tactical-api';
import EncounterPanel from './EncounterPanel.svelte';

const options:MovementOptions={actor:'actor',position:{x:20,y:30,z:0},grid_units:10,modes:['Walk','Crawl','Swim','Jump']};

it('turns explicit route legs into adjacent proposals without inventing source permissions',()=>{
  expect(proposeMovement('walk 10 feet north, then swim 5 feet east.',options)).toEqual({error:null,path:[
    {destination:{x:20,y:20,z:0},mode:'Walk'},
    {destination:{x:20,y:10,z:0},mode:'Walk'},
    {destination:{x:30,y:10,z:0},mode:'Swim'},
  ]});
  for(const input of ['fly 5 feet up','teleport 20 feet east','walk 5 feet north and attack','walk 2.5 feet north','walk 0 feet east','walk 99999999999 feet east'])expect(proposeMovement(input,options).path).toBeNull();
  expect(proposeMovement('crawl 2.5 feet northeast',{...options,grid_units:5})).toEqual({error:null,path:[{destination:{x:25,y:25,z:0},mode:'Crawl'}]});
});

it('keeps movement available after the action and clears a stale route when position or viewer changes',async()=>{
  const user=userEvent.setup();const onAction=vi.fn();
  const tactical:TacticalView={encounter_id:'encounter',round:1,active_actor:'actor',phase:'active',execution:'ShieldHitV1',battlefield:null,participants:[],combatant_sources:[],observers:[],initiative:[],ties:[],continuation:null,may_fail_save:null,legendary_resistance:null,legendary_action:null,
    budget:{movement_spent:0,attacks_remaining:0,action_spent:true,bonus_action_spent:true,reaction_available:true},movement_options:options};
  const component=render(EncounterPanel,{tactical,characters:[],host:false,actor:'actor',player:'player',onAction});
  await user.type(screen.getByLabelText('Movement route'),'walk 10 feet north');
  await user.click(screen.getByRole('button',{name:'Follow this route'}));
  expect(onAction).toHaveBeenCalledExactlyOnceWith({Move:{path:[{destination:{x:20,y:20,z:0},mode:'Walk'},{destination:{x:20,y:10,z:0},mode:'Walk'}]}});
  await component.rerender({tactical:{...tactical,movement_options:{...options,position:{x:20,y:10,z:0}}}});
  expect((screen.getByLabelText('Movement route') as HTMLInputElement).value).toBe('');
  expect((screen.getByRole('button',{name:'Follow this route'}) as HTMLButtonElement).disabled).toBe(true);
  await component.rerender({actor:'other',player:'other-player'});
  expect(screen.queryByLabelText('Movement route')).toBeNull();
});

it('discards the host drafted route when the same actor is viewed through a player channel',async()=>{
  const user=userEvent.setup();
  const tactical:TacticalView={encounter_id:'encounter',round:1,active_actor:'actor',phase:'active',execution:'ShieldHitV1',battlefield:null,participants:[],combatant_sources:[],observers:[],initiative:[],ties:[],continuation:null,may_fail_save:null,legendary_resistance:null,legendary_action:null,budget:null,movement_options:options};
  const component=render(EncounterPanel,{tactical,characters:[],host:true,actor:null,player:null,onAction:vi.fn()});
  await user.type(screen.getByLabelText('Movement route'),'walk 10 feet north');
  await component.rerender({host:false,actor:'actor',player:'player'});
  expect((screen.getByLabelText('Movement route') as HTMLInputElement).value).toBe('');
  expect((screen.getByRole('button',{name:'Follow this route'}) as HTMLButtonElement).disabled).toBe(true);
});

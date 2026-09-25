import { render, screen } from '@testing-library/svelte';
import userEvent from '@testing-library/user-event';
import { expect, it, vi } from 'vitest';
import CreatureForm from './CreatureForm.svelte';
import BattlefieldForm from './BattlefieldForm.svelte';
import { loadRequest, saveRequest, type CharacterView, type CreatureCreation } from '../table-api';

it('retains the selected source, finite ammunition and physical identities for retry', async () => {
  const user=userEvent.setup(); const onCreate=vi.fn();
  render(CreatureForm,{setup:{catalog:[{definition_id:'goblin-warrior',name:'Goblin Warrior',sizes:['Small'],additional_languages:0,ammunition_required:true,item_count:5,omitted_features:[],abilities:['Scimitar','Shortbow']}],creatures:[]},onCreate});
  await user.type(screen.getByLabelText('Creature name'),'The secret captain');
  await user.clear(screen.getByLabelText('Starting ammunition per type'));
  await user.type(screen.getByLabelText('Starting ammunition per type'),'7');
  await user.click(screen.getByRole('button',{name:'Prepare creature'}));
  const creation=onCreate.mock.calls[0][0] as CreatureCreation;
  expect(creation).toMatchObject({name:'The secret captain',definition_id:'goblin-warrior',size:'Small',ammunition_units:7});
  expect(new Set(creation.item_ids).size).toBe(5);
  const retained={kind:'action' as const,request:{command_id:'command',campaign_id:'campaign',session_id:'session',expected_event_sequence:8,channel:'Host' as const,action:{CreateCreature:{creation}}}};
  saveRequest(retained);
  expect(loadRequest()).toEqual(retained);
});

it('uses an explicit visible descriptor without copying the private creature name', async () => {
  const user=userEvent.setup(); const onPrepare=vi.fn();
  const character:CharacterView={character_id:'pc',player_id:'player',entity_id:'pc-actor',name:'Hero',profile:null,sheet:null,details:null,second_wind_remaining:null,equipment:{prepared:true,initial_item_count:0,items:[],worn_armor:null,shield:null,hands:{hands:['Free','Free']}}};
  render(BattlefieldForm,{characters:[character],creatures:[{actor:'npc',name:'The secret captain',definition_id:'goblin-warrior',size:'Small',hp:10,max_hp:10}],onPrepare});
  const descriptor=screen.getByLabelText('The secret captain: visible description');
  expect((descriptor as HTMLInputElement).value).toBe('Creature');
  await user.clear(descriptor); await user.type(descriptor,'Small armored figure');
  await user.click(screen.getByRole('button',{name:'Prepare encounter map'}));
  const setup=onPrepare.mock.calls[0][0];
  expect(setup.creatures[0]).toMatchObject({actor:'npc',public_label:'Small armored figure',enemies:['pc-actor']});
  expect(JSON.stringify(setup)).not.toContain('The secret captain');
  expect(setup.characters[0].enemies).toEqual(['npc']);
  await user.clear(screen.getByLabelText('The secret captain: visible description'));
  const south=screen.getByLabelText('The secret captain: south (feet)');
  await user.clear(south);await user.type(south,'200');
  await user.click(screen.getByRole('checkbox',{name:'The secret captain'}));
  expect((descriptor as HTMLInputElement).disabled).toBe(true);
  expect((south as HTMLInputElement).disabled).toBe(true);
  await user.click(screen.getByRole('button',{name:'Prepare encounter map'}));
  expect(onPrepare).toHaveBeenCalledTimes(2);
  expect(onPrepare.mock.calls[1][0].creatures).toEqual([]);
});

it('changing source resets size and removes an inapplicable ammunition grant', async () => {
  const user=userEvent.setup();const onCreate=vi.fn();
  render(CreatureForm,{setup:{catalog:[{definition_id:'goblin-warrior',name:'Goblin Warrior',sizes:['Small'],additional_languages:0,ammunition_required:true,item_count:5,omitted_features:[],abilities:[]},{definition_id:'wolf',name:'Wolf',sizes:['Medium'],additional_languages:0,ammunition_required:false,item_count:0,omitted_features:[],abilities:['Bite']}],creatures:[]},onCreate});
  await user.selectOptions(screen.getByLabelText('Creature source'),'wolf');
  expect((screen.getByLabelText('Creature size') as HTMLSelectElement).value).toBe('Medium');
  expect(screen.queryByLabelText('Starting ammunition per type')).toBeNull();
  await user.click(screen.getByRole('button',{name:'Prepare creature'}));
  expect(onCreate.mock.calls[0][0]).toMatchObject({definition_id:'wolf',size:'Medium',ammunition_units:0,item_ids:[]});
});

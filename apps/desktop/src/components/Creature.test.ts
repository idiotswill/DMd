import { render, screen } from '@testing-library/svelte';
import userEvent from '@testing-library/user-event';
import { expect, it, vi } from 'vitest';
import CreatureForm from './CreatureForm.svelte';
import { loadRequest, saveRequest, type CreatureCreation } from '../table-api';

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

it('changing source resets size and removes an inapplicable ammunition grant', async () => {
  const user=userEvent.setup();const onCreate=vi.fn();
  render(CreatureForm,{setup:{catalog:[{definition_id:'goblin-warrior',name:'Goblin Warrior',sizes:['Small'],additional_languages:0,ammunition_required:true,item_count:5,omitted_features:[],abilities:[]},{definition_id:'wolf',name:'Wolf',sizes:['Medium'],additional_languages:0,ammunition_required:false,item_count:0,omitted_features:[],abilities:['Bite']}],creatures:[]},onCreate});
  await user.selectOptions(screen.getByLabelText('Creature source'),'wolf');
  expect((screen.getByLabelText('Creature size') as HTMLSelectElement).value).toBe('Medium');
  expect(screen.queryByLabelText('Starting ammunition per type')).toBeNull();
  await user.click(screen.getByRole('button',{name:'Prepare creature'}));
  expect(onCreate.mock.calls[0][0]).toMatchObject({definition_id:'wolf',size:'Medium',ammunition_units:0,item_ids:[]});
});

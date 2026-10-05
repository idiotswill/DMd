import { render, screen, waitFor } from '@testing-library/svelte';
import userEvent from '@testing-library/user-event';
import { beforeEach, expect, it, vi } from 'vitest';
import TableApp from './TableApp.svelte';
import CharacterForm from './components/CharacterForm.svelte';
import { contract, emptyView, options } from './components/table-fixtures.test-support';
import { REQUEST_KEY, SELECTION_KEY, tableApi, type CreationOptions } from './table-api';
import physicalCatalog from '../../../content/srd-5.2.1/character-creation-physical-v1.json';

vi.mock('./table-api', async original => ({ ...await original<typeof import('./table-api')>(), tableApi: { defaults:vi.fn(), list:vi.fn(), create:vi.fn(), view:vi.fn(), options:vi.fn(), situation:vi.fn(), action:vi.fn(), text:vi.fn(), rollOptions:vi.fn(), creatureOptions:vi.fn(), sourceControlOptions:vi.fn() } }));
const currentOptions: CreationOptions = { ...options, catalog: physicalCatalog, fighter_masteries: [...options.fighter_masteries, 'greatsword', 'glaive'] };

beforeEach(() => {
  localStorage.clear(); vi.resetAllMocks();
  const view = emptyView();
  view.players = [{id:'player', campaign_id:'campaign', display_name:'Sam'}];
  vi.mocked(tableApi.defaults).mockResolvedValue(structuredClone(contract));
  vi.mocked(tableApi.list).mockResolvedValue([{id:'campaign',name:'Saved campaign'}]);
  vi.mocked(tableApi.view).mockResolvedValue(view);
  vi.mocked(tableApi.options).mockResolvedValue(structuredClone(currentOptions));
  vi.mocked(tableApi.situation).mockResolvedValue({title:'',description:'',challenges:[]});
  localStorage.setItem(SELECTION_KEY, JSON.stringify({campaignId:'campaign',playerId:null}));
});

it('purchases actual catalog weapons and retains the exact offered creation source through uncertain restart', async () => {
  const user = userEvent.setup();
  vi.mocked(tableApi.action).mockRejectedValueOnce({message:'Delivery uncertain.',retryable:true})
    .mockImplementationOnce(async request => ({command_id:request.command_id,revision:'accepted',outcome:{message:'Character created.'}}));
  const mounted = render(TableApp);
  await user.click(await screen.findByRole('button',{name:'Setup and host controls'}));
  await user.click(screen.getByText('Create a character'));
  await user.type(screen.getByLabelText('Character name'),'River');
  expect(screen.getByText('Greatsword · 50.00 GP each')).toBeTruthy();
  expect(screen.getByText('Glaive · 20.00 GP each')).toBeTruthy();
  await user.type(screen.getByRole('spinbutton',{name:'Greatsword quantity'}),'1');
  await user.type(screen.getByRole('spinbutton',{name:'Glaive quantity'}),'1');
  await user.selectOptions(screen.getByLabelText('Weapon mastery 1'),'greatsword');
  await user.selectOptions(screen.getByLabelText('Weapon mastery 2'),'glaive');
  expect(screen.getByText('Remaining: 135.00 GP')).toBeTruthy();
  expect(screen.queryByLabelText(/fingerprint/i)).toBeNull();
  await user.click(screen.getByRole('button',{name:'Create character'}));
  await screen.findByText('Delivery uncertain.');
  const saved = JSON.parse(localStorage.getItem(REQUEST_KEY)!);
  expect(saved.request.action.CreateCharacterFromSource.source).toEqual(currentOptions.source);
  expect(saved.request.action.CreateCharacterFromSource.input.purchases).toEqual([{item_id:'glaive',quantity:1},{item_id:'greatsword',quantity:1}]);
  expect(saved.request.action.CreateCharacterFromSource.input.masteries).toEqual(['greatsword','glaive','shortbow']);
  expect(saved.request).toMatchObject({channel:'Host',session_id:null,revision:'visible-revision'});
  expect(saved.request.action.CreateCharacterFromSource.input).not.toHaveProperty('money_cp');
  expect(saved.request.action.CreateCharacterFromSource.input).not.toHaveProperty('creation_source');
  mounted.unmount();
  vi.mocked(tableApi.options).mockResolvedValue({...currentOptions,source:{...currentOptions.source,definition_fingerprint:'ffffffffffffffff'}});
  vi.mocked(tableApi.view).mockResolvedValue({...emptyView(),revision:'later-table'});
  render(TableApp);
  await waitFor(() => expect(screen.getByRole('button',{name:'Retry original request'}).hasAttribute('disabled')).toBe(false));
  await user.click(screen.getByRole('button',{name:'Retry original request'}));
  await waitFor(() => expect(localStorage.getItem(REQUEST_KEY)).toBeNull());
  expect(tableApi.action).toHaveBeenNthCalledWith(1,saved.request);
  expect(tableApi.action).toHaveBeenNthCalledWith(2,saved.request);
});

it('rejects purchases above the source budget before submitting current creation', async () => {
  const user = userEvent.setup(), onCreate = vi.fn();
  render(CharacterForm,{options:currentOptions,players:[{id:'player',campaign_id:'campaign',display_name:'Sam'}],onCreate});
  await user.type(screen.getByLabelText('Character name'),'River');
  await user.type(screen.getByRole('spinbutton',{name:'Greatsword quantity'}),'5');
  await user.click(screen.getByRole('button',{name:'Create character'}));
  expect(screen.getByRole('alert').textContent).toContain('exceed the starting gold');
  expect(onCreate).not.toHaveBeenCalled();
  expect(tableApi.action).not.toHaveBeenCalled();
});

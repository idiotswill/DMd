import { render, fireEvent, cleanup } from '@testing-library/svelte';
import { afterEach, beforeEach, expect, it, vi } from 'vitest';
import GrapplePanel from './GrapplePanel.svelte';
import { emptyView } from './table-fixtures.test-support';
import { tableApi, type LocalChannel } from '../table-api';
vi.mock('../table-api', async original => { const actual = await original<typeof import('../table-api')>(); return { ...actual, tableApi: { ...actual.tableApi, grappleOptions: vi.fn() } }; });
const ownChannel: LocalChannel = { Player: { player_id: 'player', character_id: 'own' } };
beforeEach(() => {
  vi.mocked(tableApi.grappleOptions).mockReset();
  vi.mocked(tableApi.grappleOptions).mockImplementation(async request => ({ ...request, handles: [typeof request.channel === 'object' && 'Player' in request.channel && request.channel.Player.character_id === 'other' ? 'other-opaque' : 'own-opaque'] }));
});
afterEach(cleanup);

it('shows only the selected creature choices and returns the offered handle',async()=>{
  const view=emptyView();view.grapple={version:3,choices:[{key:'own-opaque',actor:'own',label:'Release held creature'},{key:'other-opaque',actor:'other',label:'Private other choice'}]};
  const onChoice=vi.fn();const mounted=render(GrapplePanel,{view,host:false,actor:'own',channel:ownChannel,disabled:false,onEnable:vi.fn(),onChoice});
  await mounted.findByRole('button',{name:'Release held creature'});
  expect(mounted.queryByText('Private other choice')).toBeNull();
  expect(mounted.container.textContent).not.toContain('own-opaque');
  await fireEvent.click(mounted.getByRole('button',{name:'Release held creature'}));expect(onChoice).toHaveBeenCalledWith('own-opaque');
  await mounted.rerender({view,host:false,actor:'other',channel:{Player:{player_id:'player',character_id:'other'}},disabled:false,onEnable:vi.fn(),onChoice});
  expect(mounted.queryByRole('button',{name:'Release held creature'})).toBeNull();
});

it('keeps unresolved requests locked and removes expired controls on refresh',async()=>{
  const view=emptyView();view.grapple={version:3,choices:[{key:'opaque',actor:'own',label:'Resist Grapple with Dexterity'}]};
  const onChoice=vi.fn();const props={view,host:false,actor:'own',channel:ownChannel,disabled:true,onEnable:vi.fn(),onChoice};
  const mounted=render(GrapplePanel,props);
  expect(mounted.getByRole('group').hasAttribute('disabled')).toBe(true);
  await mounted.rerender({...props,view:{...view,grapple:{version:3,choices:[]}},disabled:false});
  expect(mounted.queryByRole('button')).toBeNull();expect(onChoice).not.toHaveBeenCalled();
});

it('offers explicit activation only to the host at a settled current encounter',async()=>{
  const view=emptyView();view.tactical={execution:'EncounterReleaseV1',encounter_id:'encounter',round:1,active_actor:'own',phase:'active',battlefield:null,participants:[],combatant_sources:[],observers:[],initiative:[],ties:[],continuation:null,may_fail_save:null,legendary_resistance:null,legendary_action:null,budget:null};
  const onEnable=vi.fn();const props={view,host:true,actor:null,channel:'Host' as const,disabled:false,onEnable,onChoice:vi.fn()};
  const mounted=render(GrapplePanel,props);
  await fireEvent.click(mounted.getByRole('button',{name:'Enable Grapple'}));expect(onEnable).toHaveBeenCalledOnce();
  await mounted.rerender({...props,view:{...view,tactical:{...view.tactical,continuation:{actor:'own',host_adjudication:false,choices:[]}}}});
  expect(mounted.getByRole('group').hasAttribute('disabled')).toBe(true);
  await mounted.rerender({...props,view:{...view,tactical:{...view.tactical,ready:[{actor:'own',action:'Attack',may_abandon:true}]}}});
  expect(mounted.getByRole('group').hasAttribute('disabled')).toBe(true);
  await mounted.rerender({...props,host:false});expect(mounted.queryByRole('button',{name:'Enable Grapple'})).toBeNull();
  await mounted.rerender({...props,view:{...view,grapple:{version:3,choices:[]}}});
  expect(mounted.queryByRole('button',{name:'Enable Grapple'})).toBeNull();
});

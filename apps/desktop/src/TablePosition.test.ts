import { render, screen, waitFor } from '@testing-library/svelte';
import userEvent from '@testing-library/user-event';
import { afterEach, beforeEach, it, expect, vi } from 'vitest';
import TableApp from './TableApp.svelte';
import { contract, emptyView, options } from './components/table-fixtures.test-support';
import { REQUEST_KEY, SELECTION_KEY, tableApi } from './table-api';

vi.mock('./table-api', async original => ({ ...await original<typeof import('./table-api')>(), tableApi:{defaults:vi.fn(),list:vi.fn(),create:vi.fn(),view:vi.fn(),options:vi.fn(),situation:vi.fn(),action:vi.fn(),text:vi.fn(),rollOptions:vi.fn(),creatureOptions:vi.fn(),sourceControlOptions:vi.fn()} }));
beforeEach(()=>{
  localStorage.clear();vi.resetAllMocks();
  vi.mocked(tableApi.defaults).mockResolvedValue(structuredClone(contract));
  vi.mocked(tableApi.list).mockResolvedValue([{id:'campaign',name:'Saved campaign'}]);
  vi.mocked(tableApi.options).mockResolvedValue(options);
  vi.mocked(tableApi.situation).mockResolvedValue({title:'',description:'',challenges:[]});
  vi.mocked(tableApi.rollOptions).mockResolvedValue({savage_attacker:null});
  vi.mocked(tableApi.creatureOptions).mockResolvedValue([]);
  localStorage.setItem(SELECTION_KEY,JSON.stringify({campaignId:'campaign',playerId:null}));
});
afterEach(()=>vi.unstubAllGlobals());

function consequenceView() {
  const view=emptyView();
  view.active_session={session_id:'session',display_name:'Courtyard',started_at_world:0,participants:[]};
  view.tactical={encounter_id:'encounter',phase:'active',execution:'ShieldMissileV1',round:1,active_actor:'hag',battlefield:null,participants:[],observers:[],initiative:[],ties:[],budget:null,
    continuation:{actor:'hag',host_adjudication:false,choices:[{handle:'first',label:'Dart 1'},{handle:'second',label:'Dart 2'}]},
    may_fail_save:null,legendary_resistance:null,legendary_action:null,combatant_sources:[]};
  return view;
}

it('keeps successive accepted consequences reachable with fresh controls and their original handles',async()=>{
  const user=userEvent.setup();let view=consequenceView();
  vi.mocked(tableApi.view).mockImplementation(async()=>structuredClone(view));
  vi.mocked(tableApi.action).mockImplementation(async request=>{
    view={...view,revision:`revision-${request.command_id}`,tactical:{...view.tactical!,continuation:{actor:'hag',host_adjudication:false,choices:[{handle:'third',label:'Dart 3'}]}}};
    return {command_id:request.command_id,revision:view.revision,outcome:{message:'Encounter action recorded.'}};
  });
  render(TableApp);
  await waitFor(()=>expect(screen.getByRole('button',{name:'Dart 2 · 2'}).closest('fieldset')?.disabled).toBe(false));
  vi.stubGlobal('scrollY',1420);
  const oldGroup=screen.getByRole('group',{name:'Choose which consequence happens next'});
  await user.click(screen.getByRole('button',{name:'Dart 2 · 2'}));
  await waitFor(()=>expect(window.scrollTo).toHaveBeenCalledWith({left:0,top:1420,behavior:'instant'}));
  const freshGroup=screen.getByRole('group',{name:'Choose which consequence happens next'});
  expect(freshGroup).not.toBe(oldGroup);expect(document.activeElement).toBe(freshGroup);
  expect(screen.queryByRole('button',{name:'Dart 2 · 2'})).toBeNull();
  expect(vi.mocked(tableApi.action).mock.calls[0][0].action).toEqual({Tactical:{action:{ChooseTurnWork:{handle:'second'}}}});
  vi.stubGlobal('scrollY',1500);
  await user.click(screen.getByRole('button',{name:'Dart 3 · 1'}));
  await waitFor(()=>expect(window.scrollTo).toHaveBeenLastCalledWith({left:0,top:1500,behavior:'instant'}));
  expect(vi.mocked(tableApi.action).mock.calls[1][0].action).toEqual({Tactical:{action:{ChooseTurnWork:{handle:'third'}}}});
  expect(localStorage.getItem(REQUEST_KEY)).toBeNull();
});

it('clears previous physical faces while keeping the next roll reachable and using its new request',async()=>{
  const user=userEvent.setup();let view=consequenceView();view.tactical!.continuation=null;
  view.roll_channel='Tactical';view.roll={id:'roll-one',roller:'hag',dice:[{count:1,sides:4}],modifier:1,mode:'Normal',visibility:'Secret',reason:'Spell damage'};
  vi.mocked(tableApi.view).mockImplementation(async()=>structuredClone(view));
  vi.mocked(tableApi.action).mockImplementation(async request=>{
    view={...view,revision:'next-roll',roll:{...view.roll!,id:'roll-two'}};
    return {command_id:request.command_id,revision:view.revision,outcome:{message:'Encounter action recorded.'}};
  });
  render(TableApp);
  await waitFor(()=>expect(screen.getByLabelText('Die 1 · d4').closest('fieldset')?.disabled).toBe(false));
  vi.stubGlobal('scrollY',680);
  await user.type(screen.getByLabelText('Die 1 · d4'),'4');
  await user.click(screen.getByRole('button',{name:'Report these faces'}));
  await waitFor(()=>expect(window.scrollTo).toHaveBeenCalled());
  expect((screen.getByLabelText('Die 1 · d4') as HTMLInputElement).value).toBe('');
  expect(document.activeElement).toBe(screen.getByRole('group',{name:'Report physical dice'}));
  expect(window.scrollTo).toHaveBeenLastCalledWith({left:0,top:680,behavior:'instant'});
  await user.type(screen.getByLabelText('Die 1 · d4'),'2');
  await user.click(screen.getByRole('button',{name:'Report these faces'}));
  await waitFor(()=>expect(tableApi.action).toHaveBeenCalledTimes(2));
  expect(vi.mocked(tableApi.action).mock.calls.map(([request])=>request.action)).toEqual([
    {Tactical:{action:{SubmitRoll:{result:{request_id:'roll-one',source:'Physical',dice:[{sides:4,value:4}]}}}}},
    {Tactical:{action:{SubmitRoll:{result:{request_id:'roll-two',source:'Physical',dice:[{sides:4,value:2}]}}}}},
  ]);
});

it('keeps uncertain delivery on its error and exact saved request without restoring position',async()=>{
  const user=userEvent.setup();vi.mocked(tableApi.view).mockResolvedValue(consequenceView());
  vi.mocked(tableApi.action).mockRejectedValue({message:'Delivery uncertain.',retryable:true});
  render(TableApp);
  await waitFor(()=>expect(screen.getByRole('button',{name:'Dart 1 · 1'}).closest('fieldset')?.disabled).toBe(false));
  await user.click(screen.getByRole('button',{name:'Dart 1 · 1'}));
  await screen.findByText('Delivery uncertain.');
  expect(document.activeElement).toBe(screen.getByRole('alert'));
  expect(window.scrollTo).not.toHaveBeenCalled();
  expect(JSON.parse(localStorage.getItem(REQUEST_KEY)!).request).toEqual(vi.mocked(tableApi.action).mock.calls[0][0]);
  expect(screen.getByRole('button',{name:'Dart 2 · 2'}).closest('fieldset')?.disabled).toBe(true);
});

it.each(['consequence','turn'] as const)('brings a new physical roll into view after the %s decision',async decision=>{
  const user=userEvent.setup();let view=consequenceView();
  if(decision==='turn') {
    view.tactical!.continuation=null;
    view.tactical!.budget={movement_spent:0,attacks_remaining:0,action_spent:true,bonus_action_spent:false,reaction_available:true};
  }
  vi.mocked(tableApi.view).mockImplementation(async()=>structuredClone(view));
  vi.mocked(tableApi.action).mockImplementation(async request=>{
    view={...view,revision:'child-roll',tactical:{...view.tactical!,continuation:null},roll_channel:'Tactical',
      roll:{id:'concentration-child',roller:'hag',dice:[{count:1,sides:20}],modifier:3,mode:'Normal',visibility:'Secret',reason:'Concentration save'}};
    return {command_id:request.command_id,revision:view.revision,outcome:{message:'Encounter action recorded.'}};
  });
  render(TableApp);
  const actionName=decision==='turn'?'End turn':'Dart 1 · 1';
  await waitFor(()=>expect(screen.getByRole('button',{name:actionName}).closest('fieldset')?.disabled).toBe(false));
  vi.stubGlobal('scrollY',1800);
  await user.click(screen.getByRole('button',{name:actionName}));
  await waitFor(()=>expect(document.activeElement).toBe(screen.getByRole('group',{name:'Report physical dice'})));
  expect(vi.mocked(tableApi.action).mock.calls[0][0].action).toEqual({Tactical:{action:decision==='turn'?'EndTurn':{ChooseTurnWork:{handle:'first'}}}});
  expect((screen.getByLabelText('Die 1 · d20') as HTMLInputElement).value).toBe('');
  const calls=vi.mocked(Element.prototype.scrollIntoView);
  expect(calls).toHaveBeenCalledWith({block:'nearest',behavior:'instant'});
  expect(calls.mock.contexts.at(-1)).toBe(document.activeElement);
  await user.type(screen.getByLabelText('Die 1 · d20'),'13');
  await user.click(screen.getByRole('button',{name:'Report these faces'}));
  await waitFor(()=>expect(tableApi.action).toHaveBeenCalledTimes(2));
  expect(vi.mocked(tableApi.action).mock.calls[1][0].action).toEqual({Tactical:{action:{SubmitRoll:{result:{request_id:'concentration-child',source:'Physical',dice:[{sides:20,value:13}]}}}}});
});

it('does not restore tactical focus after navigating to host setup during a request',async()=>{
  const user=userEvent.setup();vi.mocked(tableApi.view).mockResolvedValue(consequenceView());
  let finish!:()=>void;
  vi.mocked(tableApi.action).mockImplementation(request=>new Promise(resolve=>{finish=()=>resolve({command_id:request.command_id,revision:'accepted',outcome:{message:'Encounter action recorded.'}});}));
  render(TableApp);
  await waitFor(()=>expect(screen.getByRole('button',{name:'Dart 1 · 1'}).closest('fieldset')?.disabled).toBe(false));
  await user.click(screen.getByRole('button',{name:'Dart 1 · 1'}));
  await user.click(screen.getByRole('button',{name:'Setup and host controls'}));
  finish();await screen.findByText('Encounter action recorded.');
  await waitFor(()=>expect(screen.getByRole('button',{name:'Refresh saved table'}).hasAttribute('disabled')).toBe(false));
  expect(screen.getByRole('heading',{name:'Current session'})).toBeTruthy();
  expect(screen.queryByRole('group',{name:'Choose which consequence happens next'})).toBeNull();
  expect(window.scrollTo).not.toHaveBeenCalled();
});

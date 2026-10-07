import { render, screen, waitFor } from '@testing-library/svelte';
import userEvent from '@testing-library/user-event';
import { beforeEach, describe, it, expect, vi } from 'vitest';
import TableApp from './TableApp.svelte';
import { contract, emptyView, options } from './components/table-fixtures.test-support';
import { REQUEST_KEY, SELECTION_KEY, tableApi, type TableView } from './table-api';
vi.mock('./table-api', async (original) => ({ ...await original<typeof import('./table-api')>(), tableApi:{defaults:vi.fn(),list:vi.fn(),create:vi.fn(),view:vi.fn(),options:vi.fn(),situation:vi.fn(),action:vi.fn(),text:vi.fn(),rollDetails:vi.fn(),creatureOptions:vi.fn(),sourceControlOptions:vi.fn()} }));

function table(): TableView {
  return {...emptyView(),grapple:{version:4,choices:[],ground_drag:[]},
    players:[{id:'player',campaign_id:'campaign',display_name:'Sam'}],
    characters:[{character_id:'pc',entity_id:'actor',player_id:'player',name:'River',profile:{},sheet:null,details:{heroic_inspiration:false},second_wind_remaining:null}] as unknown as TableView['characters'],
    active_session:{session_id:'session',display_name:'Evening',started_at_world:0,participants:[{player_id:'player',character_id:'pc',attendance:'Present'}]}};
}
function dice(): TableView {
  return {...table(),roll_channel:'Tactical',roll:{id:'first-owned-roll',roller:'actor',dice:[{count:1,sides:20}],modifier:5,mode:'Normal',visibility:'Public',reason:'Resist the grip'}};
}
beforeEach(()=>{
  localStorage.clear();vi.resetAllMocks();
  vi.mocked(tableApi.defaults).mockResolvedValue(structuredClone(contract));
  vi.mocked(tableApi.list).mockResolvedValue([{id:'campaign',name:'Saved campaign'}]);
  vi.mocked(tableApi.options).mockResolvedValue(options);
  vi.mocked(tableApi.situation).mockResolvedValue({title:'',description:'',challenges:[]});
  vi.mocked(tableApi.rollDetails).mockResolvedValue({version:1,options:{savage_attacker:null,heroic_inspiration:true},display_reason:"Resist the grip"});
});

describe('ordinary Inspiration through the saved desktop request',()=>{
  it('retains the exact Host award across an uncertain reply and a later session after restart',async()=>{
    const user=userEvent.setup();const view=table();
    vi.mocked(tableApi.view).mockResolvedValue(view);
    vi.mocked(tableApi.action).mockRejectedValueOnce({message:'Award delivery uncertain.',retryable:true}).mockImplementationOnce(async request=>({command_id:request.command_id,revision:'accepted-award',outcome:{message:'Heroic Inspiration awarded.'}}));
    localStorage.setItem(SELECTION_KEY,JSON.stringify({campaignId:'campaign',playerId:null}));
    const mounted=render(TableApp);
    await waitFor(()=>expect(screen.getByLabelText('Recipient').matches(':disabled')).toBe(false));
    await user.selectOptions(screen.getByLabelText('Recipient'),'pc');
    await user.type(screen.getByLabelText('Reason for the award'),'For defending the retreat.');
    await user.click(screen.getByRole('button',{name:'Award Heroic Inspiration'}));
    await screen.findByText('Award delivery uncertain.');
    const saved=JSON.parse(localStorage.getItem(REQUEST_KEY)!);
    expect(saved.request).toMatchObject({version:4,channel:'Host',revision:view.revision,session_id:'session',action:{AwardHeroicInspiration:{character_id:'pc',reason:'For defending the retreat.'}}});
    expect(screen.getByLabelText('Recipient').matches(':disabled')).toBe(true);
    mounted.unmount();
    vi.mocked(tableApi.view).mockResolvedValue({...view,revision:'later',active_session:{...view.active_session!,session_id:'later-session'}});
    localStorage.setItem(SELECTION_KEY,JSON.stringify({campaignId:'campaign',playerId:'player'}));
    render(TableApp);
    await waitFor(()=>expect(screen.getByRole('button',{name:'Retry original request'}).matches(':disabled')).toBe(false));
    await user.click(screen.getByRole('button',{name:'Retry original request'}));
    await waitFor(()=>expect(localStorage.getItem(REQUEST_KEY)).toBeNull());
    expect(tableApi.action).toHaveBeenCalledTimes(2);
    expect(tableApi.action).toHaveBeenNthCalledWith(1,saved.request);
    expect(tableApi.action).toHaveBeenNthCalledWith(2,saved.request);
  });
  it('retains original high dice and lower replacement with the old player and capability on retry',async()=>{
    const user=userEvent.setup();const view=dice();
    vi.mocked(tableApi.view).mockResolvedValue(view);
    vi.mocked(tableApi.action).mockRejectedValueOnce({message:'Dice delivery uncertain.',retryable:true}).mockImplementationOnce(async request=>({command_id:request.command_id,revision:'accepted-dice',outcome:{message:'Encounter action recorded.'}}));
    localStorage.setItem(SELECTION_KEY,JSON.stringify({campaignId:'campaign',playerId:'player'}));
    const mounted=render(TableApp);
    await waitFor(()=>expect(screen.getByLabelText('Spend Heroic Inspiration to reroll one die').matches(':disabled')).toBe(false));
    await user.type(screen.getByLabelText('Die 1 · d20'),'20');
    await user.click(screen.getByLabelText('Spend Heroic Inspiration to reroll one die'));
    await user.selectOptions(screen.getByLabelText('Die to reroll'),'0');
    await user.type(screen.getByLabelText('Inspiration replacement'),'1');
    await user.click(screen.getByRole('button',{name:'Report these faces'}));
    await screen.findByText('Dice delivery uncertain.');
    const saved=JSON.parse(localStorage.getItem(REQUEST_KEY)!);
    expect(saved.request).toMatchObject({version:4,channel:{Player:{player_id:'player',character_id:'pc'}},revision:view.revision,session_id:'session',action:{Tactical:{action:{SubmitRollWithInspiration:{result:{request_id:'first-owned-roll',source:'Physical',dice:[{sides:20,value:20}]},die_index:0,replacement:{sides:20,value:1}}}}}});
    mounted.unmount();
    vi.mocked(tableApi.view).mockResolvedValue({...table(),revision:'later',active_session:{...view.active_session!,session_id:'later-session'}});
    localStorage.setItem(SELECTION_KEY,JSON.stringify({campaignId:'campaign',playerId:null}));
    render(TableApp);
    await waitFor(()=>expect(screen.getByRole('button',{name:'Retry original request'}).matches(':disabled')).toBe(false));
    await user.click(screen.getByRole('button',{name:'Retry original request'}));
    await waitFor(()=>expect(localStorage.getItem(REQUEST_KEY)).toBeNull());
    expect(tableApi.action).toHaveBeenCalledTimes(2);
    expect(tableApi.action).toHaveBeenNthCalledWith(1,saved.request);
    expect(tableApi.action).toHaveBeenNthCalledWith(2,saved.request);
  });
  it('resets unsubmitted original and replacement drafts on player switching and a new owned request',async()=>{
    const user=userEvent.setup();let view=dice();
    vi.mocked(tableApi.view).mockImplementation(async()=>structuredClone(view));
    localStorage.setItem(SELECTION_KEY,JSON.stringify({campaignId:'campaign',playerId:'player'}));
    render(TableApp);
    await waitFor(()=>expect(screen.getByLabelText('Spend Heroic Inspiration to reroll one die').matches(':disabled')).toBe(false));
    await user.type(screen.getByLabelText('Die 1 · d20'),'20');
    await user.click(screen.getByLabelText('Spend Heroic Inspiration to reroll one die'));
    await user.selectOptions(screen.getByLabelText('Die to reroll'),'0');
    await user.type(screen.getByLabelText('Inspiration replacement'),'1');
    await user.selectOptions(screen.getByLabelText('Local viewing and input channel'),'');
    await waitFor(()=>expect(screen.queryByLabelText('Inspiration replacement')).toBeNull());
    await waitFor(()=>expect(screen.getByLabelText('Local viewing and input channel').matches(':disabled')).toBe(false));
    await user.selectOptions(screen.getByLabelText('Local viewing and input channel'),'player');
    await waitFor(()=>expect(screen.getByLabelText('Spend Heroic Inspiration to reroll one die').matches(':disabled')).toBe(false));
    expect((screen.getByLabelText('Die 1 · d20') as HTMLInputElement).value).toBe('');
    expect((screen.getByLabelText('Spend Heroic Inspiration to reroll one die') as HTMLInputElement).checked).toBe(false);
    await user.type(screen.getByLabelText('Die 1 · d20'),'3');
    await user.click(screen.getByLabelText('Spend Heroic Inspiration to reroll one die'));
    view={...view,revision:'next-request-view',roll:{...view.roll!,id:'second-owned-roll'}};
    await user.click(screen.getByRole('button',{name:'Refresh saved table'}));
    await waitFor(()=>expect(screen.getByLabelText('Spend Heroic Inspiration to reroll one die').matches(':disabled')).toBe(false));
    expect((screen.getByLabelText('Die 1 · d20') as HTMLInputElement).value).toBe('');
    expect((screen.getByLabelText('Spend Heroic Inspiration to reroll one die') as HTMLInputElement).checked).toBe(false);
    expect(screen.queryByLabelText('Inspiration replacement')).toBeNull();
    expect(tableApi.action).not.toHaveBeenCalled();expect(localStorage.getItem(REQUEST_KEY)).toBeNull();
    expect(tableApi.rollDetails).toHaveBeenLastCalledWith({version:1,campaign_id:'campaign',revision:'next-request-view',roll_id:'second-owned-roll',channel:{Player:{player_id:'player',character_id:'pc'}}});
  });
  it('clears unsubmitted award recipient and reason after leaving the Host channel',async()=>{
    const user=userEvent.setup();vi.mocked(tableApi.view).mockResolvedValue(table());
    localStorage.setItem(SELECTION_KEY,JSON.stringify({campaignId:'campaign',playerId:null}));
    render(TableApp);
    await waitFor(()=>expect(screen.getByLabelText('Recipient').matches(':disabled')).toBe(false));
    await user.selectOptions(screen.getByLabelText('Recipient'),'pc');
    await user.type(screen.getByLabelText('Reason for the award'),'Unsubmitted explanation');
    await user.selectOptions(screen.getByLabelText('Local viewing and input channel'),'player');
    await waitFor(()=>expect(screen.queryByLabelText('Recipient')).toBeNull());
    await waitFor(()=>expect(screen.getByLabelText('Local viewing and input channel').matches(':disabled')).toBe(false));
    await user.selectOptions(screen.getByLabelText('Local viewing and input channel'),'');
    await waitFor(()=>expect(screen.getByLabelText('Recipient').matches(':disabled')).toBe(false));
    expect((screen.getByLabelText('Recipient') as HTMLSelectElement).value).toBe('');
    expect((screen.getByLabelText('Reason for the award') as HTMLTextAreaElement).value).toBe('');
    expect(tableApi.action).not.toHaveBeenCalled();expect(localStorage.getItem(REQUEST_KEY)).toBeNull();
  });
});

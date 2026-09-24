import { render, screen, waitFor } from '@testing-library/svelte';
import userEvent from '@testing-library/user-event';
import { beforeEach, describe, it, expect, vi } from 'vitest';
import TableApp from './TableApp.svelte';
import { contract, emptyView, options } from './components/table-fixtures.test-support';
import { REQUEST_KEY, SELECTION_KEY, tableApi, type UnconfirmedRequest } from './table-api';
vi.mock('./table-api', async (original) => ({ ...await original<typeof import('./table-api')>(), tableApi:{defaults:vi.fn(),list:vi.fn(),create:vi.fn(),view:vi.fn(),options:vi.fn(),situation:vi.fn(),action:vi.fn(),text:vi.fn()} }));

beforeEach(() => { localStorage.clear(); vi.clearAllMocks(); vi.mocked(tableApi.defaults).mockResolvedValue(structuredClone(contract)); vi.mocked(tableApi.list).mockResolvedValue([{id:'campaign',name:'Saved campaign'}]); vi.mocked(tableApi.view).mockResolvedValue(emptyView()); vi.mocked(tableApi.options).mockResolvedValue(options); vi.mocked(tableApi.situation).mockResolvedValue({title:'',description:'',challenges:[]}); });
describe('durable UI retry',()=>{
  it('reports raw tactical dice through the retained tactical action envelope',async()=>{
    const user=userEvent.setup();const view=emptyView();
    view.players=[{id:'player',campaign_id:'campaign',display_name:'Sam'}];
    view.characters=[{character_id:'pc',entity_id:'actor',player_id:'player',name:'River',profile:null,sheet:null,details:null,second_wind_remaining:null}];
    view.active_session={session_id:'session',display_name:'Encounter',started_at_world:0,participants:[{player_id:'player',character_id:'pc',attendance:'Present'}]};
    view.tactical={encounter_id:'encounter',phase:'initiative',round:null,active_actor:null,battlefield:null,participants:[],observers:[],initiative:[],ties:[],budget:null};
    view.roll={id:'initiative-roll',roller:'actor',dice:[{sides:20,count:1}],modifier:2,mode:'Disadvantage',visibility:'Public',reason:'Initiative'};
    vi.mocked(tableApi.view).mockResolvedValue(view);
    localStorage.setItem(SELECTION_KEY,JSON.stringify({campaignId:'campaign',playerId:'player'}));
    vi.mocked(tableApi.action).mockRejectedValue('Connection interrupted');
    const mounted=render(TableApp);
    await waitFor(()=>expect(screen.getByRole('button',{name:'Report these faces'}).closest('fieldset')?.hasAttribute('disabled')).toBe(false));
    await user.type(screen.getByLabelText('Die 1 · d20'),'17');await user.type(screen.getByLabelText('Die 2 · d20'),'4');
    await user.click(screen.getByRole('button',{name:'Report these faces'}));
    await screen.findByRole('alert');
    const saved=JSON.parse(localStorage.getItem(REQUEST_KEY)!);
    expect(saved.request.action).toEqual({Tactical:{action:{SubmitRoll:{result:{request_id:'initiative-roll',source:'Physical',dice:[{sides:20,value:17},{sides:20,value:4}]}}}}});
    mounted.unmount();render(TableApp);
    await waitFor(()=>expect(screen.getByRole('button',{name:'Retry original request'}).hasAttribute('disabled')).toBe(false));
    await user.click(screen.getByRole('button',{name:'Retry original request'}));
    expect(tableApi.action).toHaveBeenLastCalledWith(saved.request);
  });
  it('retries equipment preparation after restart with the original command and every item identity',async()=>{
    const user=userEvent.setup();
    const request: UnconfirmedRequest={kind:'action',request:{command_id:'equipment-command',campaign_id:'campaign',expected_event_sequence:4,session_id:null,channel:'Host',action:{PrepareEquipment:{character_id:'character',item_ids:['first-weapon','second-weapon','ammo-stack']}}}};
    localStorage.setItem(REQUEST_KEY,JSON.stringify(request));
    vi.mocked(tableApi.action).mockRejectedValueOnce('Connection interrupted').mockImplementationOnce(async received => { expect(received).toEqual(request.request); return {command_id:'equipment-command',event_sequence:5,already_accepted:true,outcome:{message:'Equipment ready.',mechanics:null}}; });
    const first=render(TableApp);
    await waitFor(()=>expect(screen.getByRole('button',{name:'Retry original request'}).hasAttribute('disabled')).toBe(false));
    await user.click(screen.getByRole('button',{name:'Retry original request'}));
    await screen.findByRole('alert');
    expect(JSON.parse(localStorage.getItem(REQUEST_KEY)!)).toEqual(request);
    first.unmount();render(TableApp);
    await waitFor(()=>expect(screen.getByRole('button',{name:'Retry original request'}).hasAttribute('disabled')).toBe(false));
    await user.click(screen.getByRole('button',{name:'Retry original request'}));
    await waitFor(()=>expect(localStorage.getItem(REQUEST_KEY)).toBeNull());
    expect(tableApi.action).toHaveBeenCalledTimes(2);
    expect(tableApi.action).toHaveBeenNthCalledWith(1,request.request);
    expect(tableApi.action).toHaveBeenNthCalledWith(2,request.request);
  });
  it('retries exactly the saved nonce, head and channel after restart, then clears only on success',async()=>{
    const user=userEvent.setup();
    const request: UnconfirmedRequest={kind:'action',request:{command_id:'same-command',campaign_id:'campaign',expected_event_sequence:4,session_id:null,channel:'Host',action:{AddPlayer:{id:'same-player',name:'Sam'}}}};
    localStorage.setItem(REQUEST_KEY,JSON.stringify(request));
    vi.mocked(tableApi.action).mockImplementation(async received => { expect(received).toEqual(request.request); expect(JSON.parse(localStorage.getItem(REQUEST_KEY)!)).toEqual(request); return {command_id:'same-command',event_sequence:5,already_accepted:true,outcome:{message:'Sam joined.',mechanics:null}}; });
    render(TableApp);
    await waitFor(()=>expect(screen.getByRole('button',{name:'Retry original request'}).hasAttribute('disabled')).toBe(false));
    await user.click(screen.getByRole('button',{name:'Retry original request'}));
    await waitFor(()=>expect(localStorage.getItem(REQUEST_KEY)).toBeNull());
    expect(tableApi.action).toHaveBeenCalledOnce();
    expect(screen.getByText('Sam joined.')).toBeTruthy();
  });
  it('keeps an uncertain failed request and does not regenerate identity',async()=>{
    const user=userEvent.setup(); const request:UnconfirmedRequest={kind:'action',request:{command_id:'nonce',campaign_id:'campaign',expected_event_sequence:3,session_id:null,channel:'Host',action:{AddPlayer:{id:'player',name:'Sam'}}}};
    localStorage.setItem(REQUEST_KEY,JSON.stringify(request)); vi.mocked(tableApi.action).mockRejectedValue('Connection interrupted');
    render(TableApp); await waitFor(()=>expect(screen.getByRole('button',{name:'Retry original request'}).hasAttribute('disabled')).toBe(false));
    await user.click(screen.getByRole('button',{name:'Retry original request'}));
    await screen.findByRole('alert'); expect(JSON.parse(localStorage.getItem(REQUEST_KEY)!)).toEqual(request);
    expect(screen.getByRole('button',{name:'New campaign'}).hasAttribute('disabled')).toBe(true);
  });
  it('requests the selected player projection instead of reusing a host view',async()=>{
    const user=userEvent.setup(); const host=emptyView();host.players=[{id:'player',campaign_id:'campaign',display_name:'Sam'}];
    vi.mocked(tableApi.view).mockResolvedValue(host);localStorage.setItem(SELECTION_KEY,JSON.stringify({campaignId:'campaign',playerId:null}));
    render(TableApp); await screen.findByRole('combobox',{name:'Local viewing and input channel'});
    await waitFor(()=>expect(screen.getByRole('combobox',{name:'Local viewing and input channel'}).hasAttribute('disabled')).toBe(false));
    await user.selectOptions(screen.getByRole('combobox',{name:'Local viewing and input channel'}),'player');
    await waitFor(()=>expect(tableApi.view).toHaveBeenLastCalledWith('campaign',{Player:'player'}));
    expect(screen.queryByRole('button',{name:'Setup and host controls'})).toBeNull();
  });
  it('releases a confirmed host rejection so the player can correct input',async()=>{
    const user=userEvent.setup();const request:UnconfirmedRequest={kind:'action',request:{command_id:'rejected',campaign_id:'campaign',expected_event_sequence:19,session_id:null,channel:'Host',action:{AddPlayer:{id:'player',name:'Sam'}}}};
    localStorage.setItem(REQUEST_KEY,JSON.stringify(request));vi.mocked(tableApi.action).mockRejectedValue({code:'table_action',message:'Correct this input.',retryable:false});
    render(TableApp);await waitFor(()=>expect(screen.getByRole('button',{name:'Retry original request'}).hasAttribute('disabled')).toBe(false));
    await user.click(screen.getByRole('button',{name:'Retry original request'}));
    await screen.findByText('Correct this input.');expect(localStorage.getItem(REQUEST_KEY)).toBeNull();expect(screen.queryByText('A request is awaiting confirmation')).toBeNull();
  });
  it('contains malformed saved input without invoking it or losing campaign navigation',async()=>{
    localStorage.setItem(REQUEST_KEY,JSON.stringify({kind:'action',request:{command_id:'broken',campaign_id:'campaign'}}));
    render(TableApp);await screen.findByText('The saved retry is incomplete or incompatible. Review the saved campaign before clearing this request.');
    await waitFor(()=>expect(tableApi.list).toHaveBeenCalledOnce());
    expect(tableApi.action).not.toHaveBeenCalled();expect(screen.queryByRole('button',{name:'Retry original request'})).toBeNull();
    expect(screen.getByRole('button',{name:'Refresh and release local retry'})).toBeTruthy();
  });
  it('clears private transient answers before changing the selected player',async()=>{
    const user=userEvent.setup();const view=emptyView();
    view.players=[{id:'one',campaign_id:'campaign',display_name:'One'},{id:'two',campaign_id:'campaign',display_name:'Two'}];
    view.characters=[{character_id:'pc',player_id:'one',entity_id:'actor',name:'River',profile:null,sheet:null,details:null,second_wind_remaining:null}];
    view.active_session={session_id:'session',display_name:'Evening',started_at_world:0,participants:[{player_id:'one',character_id:'pc',attendance:'Present'}]};
    vi.mocked(tableApi.view).mockResolvedValue(view);localStorage.setItem(SELECTION_KEY,JSON.stringify({campaignId:'campaign',playerId:'one'}));
    vi.mocked(tableApi.text).mockResolvedValue({Observed:{meta:{id:'question',campaign_id:'campaign',session_id:'session',issuer:{Player:'one'},actor:{Entity:'actor'},expected_event_sequence:19},text:'What is my health?',answer:'Private answer for One'}});
    render(TableApp);await waitFor(()=>expect(screen.getByRole('button',{name:'Send to the table'}).closest('fieldset')?.hasAttribute('disabled')).toBe(false));
    await user.type(screen.getByLabelText('Your declaration, question or correction'),'What is my health?');await user.click(screen.getByRole('button',{name:'Send to the table'}));
    await screen.findByText('Private answer for One');await waitFor(()=>expect(screen.getByRole('combobox',{name:'Local viewing and input channel'}).hasAttribute('disabled')).toBe(false));
    await user.selectOptions(screen.getByRole('combobox',{name:'Local viewing and input channel'}),'two');
    await waitFor(()=>expect(tableApi.view).toHaveBeenLastCalledWith('campaign',{Player:'two'}));expect(screen.queryByText('Private answer for One')).toBeNull();
  });
});

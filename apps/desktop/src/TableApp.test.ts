import { render, screen, waitFor } from '@testing-library/svelte';
import userEvent from '@testing-library/user-event';
import { beforeEach, describe, it, expect, vi } from 'vitest';
import TableApp from './TableApp.svelte';
import { contract, emptyView, options } from './components/table-fixtures.test-support';
import { REQUEST_KEY, SELECTION_KEY, tableApi, type UnconfirmedRequest } from './table-api';
vi.mock('./table-api', async (original) => ({ ...await original<typeof import('./table-api')>(), tableApi:{defaults:vi.fn(),list:vi.fn(),create:vi.fn(),view:vi.fn(),options:vi.fn(),situation:vi.fn(),action:vi.fn(),text:vi.fn()} }));

beforeEach(() => { localStorage.clear(); vi.clearAllMocks(); vi.mocked(tableApi.defaults).mockResolvedValue(structuredClone(contract)); vi.mocked(tableApi.list).mockResolvedValue([{id:'campaign',name:'Saved campaign'}]); vi.mocked(tableApi.view).mockResolvedValue(emptyView()); vi.mocked(tableApi.options).mockResolvedValue(options); vi.mocked(tableApi.situation).mockResolvedValue({title:'',description:'',challenges:[]}); });
describe('durable UI retry',()=>{
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
});

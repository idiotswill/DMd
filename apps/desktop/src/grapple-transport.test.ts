import { beforeEach, describe, expect, it, vi } from 'vitest';
import { invoke } from '@tauri-apps/api/core';
import { loadRequest, saveRequest, tableApi, type RequestContext, type UnconfirmedRequest } from './table-api';
vi.mock('@tauri-apps/api/core',()=>({invoke:vi.fn()}));
const context:RequestContext={version:3,command_id:'retained-command',campaign_id:'campaign',session_id:'session',channel:{SourceCreature:{player_id:'owner',actor:'goblin'}},revision:'original-revision'};
beforeEach(()=>{vi.mocked(invoke).mockReset();localStorage.clear();});

describe('Grapple original opaque transport',()=>{
  it('retains the complete source choice after uncertain acknowledgement without adding canonical identity',async()=>{
    const saved:UnconfirmedRequest={kind:'action',request:{...context,action:{Tactical:{action:{GrappleChoice:{handle:'original-offer'}}}}}};
    saveRequest(saved);
    vi.mocked(invoke).mockRejectedValueOnce(new Error('lost reply')).mockResolvedValueOnce({Accepted:{command_id:context.command_id,revision:'accepted',outcome:{message:'Recorded'}}});
    await expect(tableApi.action(saved.request)).rejects.toThrow('lost reply');
    const retry=loadRequest();if(retry?.kind!=='action')throw new Error('retry absent');
    expect(retry).toEqual(saved);await tableApi.action(retry.request);
    expect(vi.mocked(invoke).mock.calls[0]).toEqual(vi.mocked(invoke).mock.calls[1]);
    expect(invoke).toHaveBeenLastCalledWith('desktop_submit_table',{request:{...context,input:{GrappleChoice:{handle:'original-offer'}}}});
    const wire=JSON.stringify(vi.mocked(invoke).mock.calls[0]);
    for(const privateKey of ['grip','occurrence','expected_event_sequence','before_change'])expect(wire).not.toContain(privateKey);
  });
  it('keeps activation as a version three host action and keeps old retries readable',async()=>{
    const activation:UnconfirmedRequest={kind:'action',request:{...context,channel:'Host',action:'EnableGrappleAccess'}};
    saveRequest(activation);expect(loadRequest()).toEqual(activation);
    vi.mocked(invoke).mockResolvedValue({Accepted:{command_id:context.command_id,revision:'accepted',outcome:{message:'Enabled'}}});
    await tableApi.action(activation.request);
    expect(invoke).toHaveBeenLastCalledWith('desktop_submit_table',{request:{...context,channel:'Host',input:{Action:'EnableGrappleAccess'}}});
    for(const version of [1,2] as const){const old:UnconfirmedRequest={kind:'action',request:{...context,version,channel:'Host',action:'EndSession'}};saveRequest(old);expect(loadRequest()).toEqual(old);}
  });
  it('retains source administration under version three after Grapple activation',()=>{
    const saved:UnconfirmedRequest={kind:'action',request:{...context,channel:'Host',action:{SetSourceCreatureController:{actor:'goblin',controller:{Player:'new-owner'}}}}};
    saveRequest(saved);expect(loadRequest()).toEqual(saved);
  });
});

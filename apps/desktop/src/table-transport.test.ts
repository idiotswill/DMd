import { beforeEach, describe, expect, it, vi } from 'vitest';
import { invoke } from '@tauri-apps/api/core';
import { loadRequest, REQUEST_KEY, saveRequest, tableApi, type RequestContext, type UnconfirmedRequest } from './table-api';
vi.mock('@tauri-apps/api/core',()=>({invoke:vi.fn()}));
const context:RequestContext={version:1,command_id:'same-command',campaign_id:'campaign',session_id:'session',channel:{Player:{player_id:'player',character_id:'character'}},revision:'original-visible-token'};
beforeEach(()=>{vi.mocked(invoke).mockReset();localStorage.clear();});

describe('durable opaque desktop transport',()=>{
  it('retains the whole original versioned envelope before uncertain delivery and exact retry',async()=>{
    const saved:UnconfirmedRequest={kind:'text',request:{...context,text:'How do I roll?'}};
    saveRequest(saved);
    vi.mocked(invoke).mockRejectedValueOnce(new Error('lost response')).mockResolvedValueOnce({Observed:{command_id:context.command_id,revision:'accepted-token',text:saved.request.text,answer:'Stored answer'}});
    await expect(tableApi.text(saved.request)).rejects.toThrow('lost response');
    expect(loadRequest()).toEqual(saved);
    const retry=loadRequest()!;
    if(retry.kind!=='text')throw new Error('wrong retained request');
    await tableApi.text(retry.request);
    expect(vi.mocked(invoke).mock.calls[0]).toEqual(vi.mocked(invoke).mock.calls[1]);
    expect(vi.mocked(invoke).mock.calls[0]).toEqual(['desktop_submit_table',{request:{...context,input:{Text:{text:'How do I roll?'}}}}]);
    expect(JSON.stringify(vi.mocked(invoke).mock.calls)).not.toContain('expected_event_sequence');
  });
  it('submits the offered work capability without exposing or deriving a canonical occurrence',async()=>{
    vi.mocked(invoke).mockResolvedValue({Accepted:{command_id:context.command_id,revision:'next',outcome:{message:'Recorded.'}}});
    await tableApi.action({...context,action:{Tactical:{action:{ChooseTurnWork:{handle:'opaque-choice'}}}}});
    expect(invoke).toHaveBeenCalledWith('desktop_submit_table',{request:{...context,input:{SelectWork:{handle:'opaque-choice'}}}});
    expect(JSON.stringify(vi.mocked(invoke).mock.calls)).not.toContain('occurrence');
  });
  it('recovers old numeric requests only through the legacy recovery endpoint without rewriting them',async()=>{
    const legacy:UnconfirmedRequest={kind:'action',request:{command_id:'old',campaign_id:'campaign',session_id:null,channel:'Host',expected_event_sequence:7,action:{AddPlayer:{id:'p',name:'Pat'}}}};
    saveRequest(legacy);
    const retry=loadRequest()!;
    if(retry.kind!=='action')throw new Error('wrong request');
    vi.mocked(invoke).mockResolvedValue({Accepted:{command_id:'old',outcome:{message:'Pat joined.'}}});
    await tableApi.action(retry.request);
    expect(invoke).toHaveBeenCalledWith('desktop_table_action',{request:legacy.request});
    expect(loadRequest()).toEqual(legacy);
  });
  it('contains mixed or unknown envelope versions instead of guessing a new context',()=>{
    for(const request of [{...context,version:2},{...context,expected_event_sequence:3},{...context,revision:''}]){
      localStorage.setItem(REQUEST_KEY,JSON.stringify({kind:'text',request:{...request,text:'Hello'}}));
      expect(()=>loadRequest()).toThrow('incomplete or incompatible');
      expect(invoke).not.toHaveBeenCalled();
    }
  });
});

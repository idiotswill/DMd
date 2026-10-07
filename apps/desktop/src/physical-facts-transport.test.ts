import { beforeEach, describe, expect, it, vi } from 'vitest';
import { invoke } from '@tauri-apps/api/core';
import { loadRequest, saveRequest, tableApi, REQUEST_KEY, type RequestContext, type PhysicalFactInput, type UnconfirmedRequest } from './table-api';
vi.mock('@tauri-apps/api/core', () => ({ invoke: vi.fn() }));
const context: RequestContext = { version:5, command_id:'original-fact',campaign_id:'campaign',session_id:null,channel:'Host',revision:'actual-host-view' };
beforeEach(() => { localStorage.clear(); vi.mocked(invoke).mockReset(); });
const inputs: PhysicalFactInput[] = [
  {Body:{pounds:'180.000001',reason:'Measured unladen body.'}},
  {Body:{pounds:null,reason:'Body mass is currently unknown.'}},
  {Item:{choice:{NonstandardUnit:{description:'Heavy custom pommel on this exact dagger.',pounds:'3.25'}},reason:'Physical classification; source properties unchanged.'}},
  {Item:{choice:{Catalog:{condition:'full'}},reason:'This exact waterskin is full.'}},
  {Currency:{coins:{cp:0,sp:7,ep:0,gp:171,pp:0},reason:'Actual denominations represent the existing wallet.'}},
  {PersonalItem:{name:'Separate cloak',pounds:'2.5',separate_from_listed:true,reason:'Additional untracked apparel.'}},
  {Coverage:{complete:true,reason:'All physical payload is listed.'}},
];
describe('physical fact durable transport',()=>{
  it.each(inputs)('retains exact decimal text, opaque control and original envelope for %o',async input=>{
    const saved:UnconfirmedRequest={kind:'action',request:{...context,action:{PhysicalFact:{handle:'actual-control',input}}}};
    saveRequest(saved);
    vi.mocked(invoke).mockRejectedValueOnce(new Error('lost acknowledgement')).mockResolvedValueOnce({Accepted:{command_id:context.command_id}});
    await expect(tableApi.action(saved.request)).rejects.toThrow('lost acknowledgement');
    const retry=loadRequest();if(retry?.kind!=='action')throw new Error('missing original request');
    expect(retry).toEqual(saved);
    await tableApi.action(retry.request);
    expect(vi.mocked(invoke).mock.calls[0]).toEqual(vi.mocked(invoke).mock.calls[1]);
    expect(invoke).toHaveBeenLastCalledWith('desktop_submit_table',{request:{...context,input:{PhysicalFact:{handle:'actual-control',input}}}});
    expect(JSON.stringify(vi.mocked(invoke).mock.calls)).not.toContain('expected_event_sequence');
  });
  it('uses the explicit activation input and keeps later real feature activations on v5',async()=>{
    const saved:UnconfirmedRequest={kind:'action',request:{...context,action:'EnablePhysicalFacts'}};
    saveRequest(saved);expect(loadRequest()).toEqual(saved);
    await tableApi.action(saved.request);
    expect(invoke).toHaveBeenLastCalledWith('desktop_submit_table',{request:{...context,input:'EnablePhysicalFacts'}});
    for(const action of ['EnableGrappleAccess','EnableGrappleTransport'] as const) {
      const request:UnconfirmedRequest={kind:'action',request:{...context,session_id:'session',action}};
      saveRequest(request);expect(loadRequest()).toEqual(request);
      await tableApi.action(request.request);
      expect(invoke).toHaveBeenLastCalledWith('desktop_submit_table',{request:{...context,session_id:'session',input:{Action:action}}});
    }
  });
  it.each([
    {version:4},
    {channel:{Player:{player_id:'owner',character_id:'pc'}}},
    {channel:{SourceCreature:{player_id:'owner',actor:'ogre'}}},
    {action:{PhysicalFact:{handle:'control',input:{Body:{pounds:180.25,reason:'Numeric pounds'}}}}},
    {action:{PhysicalFact:{handle:'control',input:{Body:{pounds:'1e3',reason:'Exponent pounds'}}}}},
    {action:{PhysicalFact:{handle:'control',input:{Body:{pounds:'0',reason:'Unknown disguised as zero'}}}}},
    {action:{PhysicalFact:{handle:'control',input:{Body:{pounds:'1.0000001',reason:'Would require rounding'}}}}},
    {action:{PhysicalFact:{handle:'control',input:{Item:{choice:{Catalog:{condition:null,mass:1}},reason:'Invented source amount'}}}}},
    {action:{PhysicalFact:{handle:'control',input:{Item:{choice:{NonstandardUnit:{description:'',pounds:'3'}},reason:'No physical classification'}}}}},
    {action:{PhysicalFact:{handle:'control',input:{Currency:{coins:{cp:0.5,sp:0,ep:0,gp:0,pp:0},reason:'Fractional coin'}}}}},
    {action:{PhysicalFact:{handle:'control',input:{Coverage:{complete:true,reason:'Coverage'}},actor:'foreign'}}},
  ])('keeps malformed or unauthorized original input for explicit recovery without invoking it',patch=>{
    const raw=JSON.stringify({kind:'action',request:{...context,action:{PhysicalFact:{handle:'actual-control',input:inputs[0]}},...patch}});
    localStorage.setItem(REQUEST_KEY,raw);
    expect(()=>loadRequest()).toThrow('The saved retry is incomplete or incompatible.');
    expect(localStorage.getItem(REQUEST_KEY)).toBe(raw);
    expect(invoke).not.toHaveBeenCalled();
  });
});

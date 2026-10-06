import {afterEach,beforeEach,expect,it,vi} from 'vitest';
import {cleanup,render,screen} from '@testing-library/svelte';
import userEvent from '@testing-library/user-event';
import {invoke} from '@tauri-apps/api/core';
import type {MovementOptions} from '../tactical-api';
import {loadRequest,saveRequest,tableApi,type RequestContext,type UnconfirmedRequest} from '../table-api';
import MovementForm from './MovementForm.svelte';
vi.mock('@tauri-apps/api/core',()=>({invoke:vi.fn()}));
afterEach(cleanup);
beforeEach(()=>{vi.mocked(invoke).mockReset();localStorage.clear();});
const options:MovementOptions={actor:'holder',position:{x:10,y:10,z:0},grid_units:10,modes:['Walk'],self_only_required:true};
const groundDrag=[{key:'opaque-drag',actor:'holder',label:'Drag creature with left hand'}];
const path=[{destination:{x:10,y:20,z:0},mode:'Walk' as const}];
it('selects one opaque ground drag explicitly and submits only its handle and route',async()=>{
  const user=userEvent.setup();const onAction=vi.fn();render(MovementForm,{options,groundDrag,onAction});
  await user.selectOptions(screen.getByLabelText('Move with'),'opaque-drag');
  await user.type(screen.getByLabelText('Movement route'),'walk 5 feet south');
  expect(onAction).not.toHaveBeenCalled();
  expect(screen.getByText(/1 extra foot per foot/)).toBeTruthy();
  expect(screen.queryByText('Held creatures stay where they are. Moving out of reach ends the grip.')).toBeNull();
  await user.click(screen.getByRole('button',{name:'Drag the selected creature along this route'}));
  expect(onAction).toHaveBeenCalledExactlyOnceWith({MoveGrappled:{option:'opaque-drag',path}});
});
it('drops a stale drag option on refresh and preserves the deliberate self-only control',async()=>{
  const user=userEvent.setup();const onAction=vi.fn();const mounted=render(MovementForm,{options,groundDrag,onAction});
  await user.selectOptions(screen.getByLabelText('Move with'),'opaque-drag');
  await mounted.rerender({options,groundDrag:[],onAction});
  expect(screen.queryByRole('button',{name:'Drag the selected creature along this route'})).toBeNull();
  expect(screen.getByRole('button',{name:'Move only my creature along this route'})).toBeTruthy();
  expect(onAction).not.toHaveBeenCalled();
});
it('locks both drag selection and submission while an earlier request is unresolved',async()=>{
  const user=userEvent.setup();const onAction=vi.fn();render(MovementForm,{options,groundDrag,onAction,disabled:true});
  await user.selectOptions(screen.getByLabelText('Move with'),'opaque-drag');
  expect((screen.getByLabelText('Move with') as HTMLSelectElement).value).toBe('');
  expect(onAction).not.toHaveBeenCalled();
});
it('retries the exact version4 opaque drag envelope after a lost response',async()=>{
  const context:RequestContext={version:4,command_id:'original-drag',campaign_id:'campaign',session_id:'session',channel:{Player:{player_id:'owner',character_id:'character'}},revision:'original-revision'};
  const saved:UnconfirmedRequest={kind:'action',request:{...context,action:{Tactical:{action:{MoveGrappled:{option:'opaque-drag',path}}}}}};
  saveRequest(saved);vi.mocked(invoke).mockRejectedValueOnce(new Error('lost reply')).mockResolvedValueOnce({Accepted:{command_id:context.command_id,revision:'accepted',outcome:{message:'Moved'}}});
  await expect(tableApi.action(saved.request)).rejects.toThrow('lost reply');
  const reopened=loadRequest();if(reopened?.kind!=='action')throw new Error('saved drag absent');
  expect(reopened).toEqual(saved);await tableApi.action(reopened.request);
  expect(vi.mocked(invoke).mock.calls[0]).toEqual(vi.mocked(invoke).mock.calls[1]);
  expect(invoke).toHaveBeenLastCalledWith('desktop_submit_table',{request:{...context,input:{MoveGrappled:{option:'opaque-drag',path}}}});
});
it('retains the explicit host upgrade as version4 through reload and transport',async()=>{
  const saved:UnconfirmedRequest={kind:'action',request:{version:4,command_id:'upgrade',campaign_id:'campaign',session_id:'session',channel:'Host',revision:'v3-revision',action:'EnableGrappleTransport'}};
  saveRequest(saved);expect(loadRequest()).toEqual(saved);
  vi.mocked(invoke).mockResolvedValueOnce({Accepted:{command_id:'upgrade',outcome:{message:'Enabled'}}});
  await tableApi.action(saved.request);
  expect(invoke).toHaveBeenCalledWith('desktop_submit_table',{request:{version:4,command_id:'upgrade',campaign_id:'campaign',session_id:'session',channel:'Host',revision:'v3-revision',input:{Action:'EnableGrappleTransport'}}});
});

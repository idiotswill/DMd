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
const path=[{destination:{x:10,y:20,z:0},mode:'Walk' as const}];

it('requires a deliberate self-only submission and explains that the held creature stays put',async()=>{
  const user=userEvent.setup();const onAction=vi.fn();
  render(MovementForm,{options,onAction});
  expect(screen.getByText('Held creatures stay where they are. Moving out of reach ends the grip.')).toBeTruthy();
  expect(screen.queryByRole('button',{name:'Follow this route'})).toBeNull();
  expect(screen.getByRole('button',{name:'Move only my creature along this route'}).hasAttribute('disabled')).toBe(true);
  await user.type(screen.getByLabelText('Movement route'),'walk 5 feet south');
  expect(onAction).not.toHaveBeenCalled();
  await user.click(screen.getByRole('button',{name:'Move only my creature along this route'}));
  expect(onAction).toHaveBeenCalledExactlyOnceWith({MoveSelfOnly:{path}});
});

it('keeps the original ordinary action when the optional field is absent',async()=>{
  const user=userEvent.setup();const onAction=vi.fn();
  const original:MovementOptions={actor:options.actor,position:options.position,grid_units:options.grid_units,modes:options.modes};
  render(MovementForm,{options:original,onAction});
  expect(screen.queryByText('Held creatures stay where they are. Moving out of reach ends the grip.')).toBeNull();
  await user.type(screen.getByLabelText('Movement route'),'walk 5 feet south');
  await user.click(screen.getByRole('button',{name:'Follow this route'}));
  expect(onAction).toHaveBeenCalledExactlyOnceWith({Move:{path}});
});

it('retries the exact saved self-only action after a lost reply and never translates an old Move',async()=>{
  const context:RequestContext={version:3,command_id:'original-choice',campaign_id:'campaign',session_id:'session',channel:{Player:{player_id:'owner',character_id:'character'}},revision:'original-revision'};
  const saved:UnconfirmedRequest={kind:'action',request:{...context,action:{Tactical:{action:{MoveSelfOnly:{path}}}}}};
  saveRequest(saved);
  vi.mocked(invoke).mockRejectedValueOnce(new Error('lost reply')).mockResolvedValueOnce({Accepted:{command_id:context.command_id,revision:'accepted',outcome:{message:'Moved'}}});
  await expect(tableApi.action(saved.request)).rejects.toThrow('lost reply');
  const reopened=loadRequest();if(reopened?.kind!=='action')throw new Error('saved action absent');
  expect(reopened).toEqual(saved);await tableApi.action(reopened.request);
  expect(vi.mocked(invoke).mock.calls[0]).toEqual(vi.mocked(invoke).mock.calls[1]);
  expect(invoke).toHaveBeenLastCalledWith('desktop_submit_table',{request:{...context,input:{Action:{Tactical:{action:{MoveSelfOnly:{path}}}}}}});
  const original:UnconfirmedRequest={kind:'action',request:{...context,version:2,action:{Tactical:{action:{Move:{path}}}}}};
  saveRequest(original);expect(loadRequest()).toEqual(original);
});

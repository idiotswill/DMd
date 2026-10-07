import { render, screen } from '@testing-library/svelte';
import userEvent from '@testing-library/user-event';
import { describe, it, expect, vi } from 'vitest';
import RollForm from './RollForm.svelte';
import InspirationAward from './InspirationAward.svelte';
import type { RollRequest, TableView } from '../table-api';

const request: RollRequest = {id:'owned-roll',roller:'pc',dice:[{count:1,sides:20}],modifier:5,mode:'Normal',visibility:'Public',reason:'Escape the grip'};

describe('ordinary Heroic Inspiration',()=>{
  it('preserves the original high face and submits the mandatory lower replacement',async()=>{
    const user=userEvent.setup();const normal=vi.fn();const inspired=vi.fn();
    render(RollForm,{request,heroicInspiration:true,onSubmit:normal,onInspiration:inspired});
    await user.type(screen.getByLabelText('Die 1 · d20'),'20');
    await user.click(screen.getByLabelText('Spend Heroic Inspiration to reroll one die'));
    await user.selectOptions(screen.getByLabelText('Die to reroll'),'0');
    await user.type(screen.getByLabelText('Inspiration replacement'),'1');
    await user.click(screen.getByRole('button',{name:'Report these faces'}));
    expect(normal).not.toHaveBeenCalled();
    expect(inspired).toHaveBeenCalledExactlyOnceWith([20],0,{sides:20,value:1});
  });
  it('selects one of two original dice and leaves the other physical face intact',async()=>{
    const user=userEvent.setup();const inspired=vi.fn();
    render(RollForm,{request:{...request,mode:'Disadvantage'},heroicInspiration:true,onSubmit:vi.fn(),onInspiration:inspired});
    await user.type(screen.getByLabelText('Die 1 · d20'),'18');
    await user.type(screen.getByLabelText('Die 2 · d20'),'2');
    await user.click(screen.getByLabelText('Spend Heroic Inspiration to reroll one die'));
    await user.selectOptions(screen.getByLabelText('Die to reroll'),'1');
    await user.type(screen.getByLabelText('Inspiration replacement'),'14');
    await user.click(screen.getByRole('button',{name:'Report these faces'}));
    expect(inspired).toHaveBeenCalledExactlyOnceWith([18,2],1,{sides:20,value:14});
  });
  it('keeps declined and unavailable Inspiration on the ordinary physical path',async()=>{
    const user=userEvent.setup();const normal=vi.fn();const inspired=vi.fn();
    const mounted=render(RollForm,{request,heroicInspiration:true,onSubmit:normal,onInspiration:inspired});
    await user.type(screen.getByLabelText('Die 1 · d20'),'7');
    await user.click(screen.getByRole('button',{name:'Report these faces'}));
    expect(normal).toHaveBeenCalledExactlyOnceWith([7]);expect(inspired).not.toHaveBeenCalled();
    await mounted.rerender({request,heroicInspiration:false,onSubmit:normal,onInspiration:inspired});
    expect(screen.queryByLabelText('Spend Heroic Inspiration to reroll one die')).toBeNull();
  });
  it('cannot submit an absent selection or out-of-range replacement',async()=>{
    const user=userEvent.setup();const inspired=vi.fn();const normal=vi.fn();
    render(RollForm,{request,heroicInspiration:true,onSubmit:normal,onInspiration:inspired});
    await user.type(screen.getByLabelText('Die 1 · d20'),'3');
    await user.click(screen.getByLabelText('Spend Heroic Inspiration to reroll one die'));
    await user.click(screen.getByRole('button',{name:'Report these faces'}));
    expect(inspired).not.toHaveBeenCalled();expect(normal).not.toHaveBeenCalled();
    await user.selectOptions(screen.getByLabelText('Die to reroll'),'0');
    await user.type(screen.getByLabelText('Inspiration replacement'),'21');
    await user.click(screen.getByRole('button',{name:'Report these faces'}));
    expect(inspired).not.toHaveBeenCalled();expect(normal).not.toHaveBeenCalled();
  });
});

const view = {grapple:{version:4,choices:[]},active_session:{session_id:'session'},characters:[
  {character_id:'pc',name:'Traveler',player_id:'player',profile:{},details:{heroic_inspiration:false}},
  {character_id:'other',name:'Inspired ally',player_id:'ally',profile:{},details:{heroic_inspiration:true}}
]} as unknown as TableView;

describe('Host Inspiration award',()=>{
  it('records the explicitly selected recipient and reason without automatic transfer',async()=>{
    const user=userEvent.setup();const award=vi.fn();
    render(InspirationAward,{view,host:true,onAward:award});
    expect((screen.getByRole('option',{name:'Inspired ally — already inspired'}) as HTMLOptionElement).disabled).toBe(true);
    await user.selectOptions(screen.getByLabelText('Recipient'),'pc');
    await user.type(screen.getByLabelText('Reason for the award'),'For protecting the retreating group.');
    await user.click(screen.getByRole('button',{name:'Award Heroic Inspiration'}));
    expect(award).toHaveBeenCalledExactlyOnceWith('pc','For protecting the retreating group.');
  });
  it('shows no award control to a player or before the shared feature activation',async()=>{
    const props={view,host:false,onAward:vi.fn()};const mounted=render(InspirationAward,props);
    expect(screen.queryByRole('button',{name:'Award Heroic Inspiration'})).toBeNull();
    await mounted.rerender({...props,host:true,view:{...view,grapple:{version:3,choices:[]}}});
    expect(screen.queryByRole('button',{name:'Award Heroic Inspiration'})).toBeNull();
  });
  it('locks during physical dice and refuses a blank explanation',async()=>{
    const user=userEvent.setup();const award=vi.fn();const props={view,host:true,onAward:award};
    const mounted=render(InspirationAward,props);
    await user.selectOptions(screen.getByLabelText('Recipient'),'pc');
    await user.type(screen.getByLabelText('Reason for the award'),'   ');
    await user.click(screen.getByRole('button',{name:'Award Heroic Inspiration'}));expect(award).not.toHaveBeenCalled();
    await mounted.rerender({...props,view:{...view,roll:request}});
    expect((screen.getByRole('button',{name:'Award Heroic Inspiration'}) as HTMLButtonElement).matches(':disabled')).toBe(true);
  });
});

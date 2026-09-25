import { render, screen } from '@testing-library/svelte';
import userEvent from '@testing-library/user-event';
import { describe, it, expect, vi } from 'vitest';
import RollForm from './RollForm.svelte';
import type { RollRequest } from '../table-api';

const request: RollRequest={id:'opaque-roll',roller:'actor',dice:[{count:2,sides:6},{count:1,sides:8}],modifier:3,mode:'Normal',visibility:'Public',reason:'Attack damage'};
describe('source Savage Attacker dice',()=>{
  it('retains both weapon sets, one shared extra die, explicit selection and Inspiration',async()=>{
    const user=userEvent.setup();const normal=vi.fn();const savage=vi.fn();
    render(RollForm,{request,savageOption:{weapon_dice:2,heroic_inspiration:true},onSubmit:normal,onSavage:savage});
    for(const [label,value] of [['Die 1 · d6','1'],['Die 2 · d6','2'],['Die 3 · d8','5']])await user.type(screen.getByLabelText(label),value);
    await user.click(screen.getByLabelText('Use Savage Attacker (once per turn)'));
    await user.type(screen.getByLabelText('Second weapon die 1 · d6'),'6');
    await user.type(screen.getByLabelText('Second weapon die 2 · d6'),'4');
    expect(screen.queryByLabelText('Second weapon die 3 · d8')).not.toBeInTheDocument();
    await user.click(screen.getByLabelText('Spend Heroic Inspiration to reroll one die'));
    await user.selectOptions(screen.getByLabelText('Die to reroll'),'First:2');
    await user.type(screen.getByLabelText('Inspiration replacement'),'8');
    await user.selectOptions(screen.getByLabelText('Damage set to use'),'Second');
    await user.click(screen.getByRole('button',{name:'Report these faces'}));
    expect(normal).not.toHaveBeenCalled();
    expect(savage).toHaveBeenCalledExactlyOnceWith({weapon_dice:2,
      first:{request_id:'opaque-roll',source:'Physical',dice:[{sides:6,value:1},{sides:6,value:2},{sides:8,value:5}]},
      second:{request_id:'opaque-roll',source:'Physical',dice:[{sides:6,value:6},{sides:6,value:4},{sides:8,value:5}]},
      chosen:'Second',inspiration:{roll:'First',die_index:2,replacement:{sides:8,value:8}}});
  });
  it('keeps the optional feature declined and does not invent an unavailable reroll',async()=>{
    const user=userEvent.setup();const normal=vi.fn();const savage=vi.fn();
    render(RollForm,{request:{...request,dice:[{count:1,sides:6}]},savageOption:{weapon_dice:1,heroic_inspiration:false},onSubmit:normal,onSavage:savage});
    await user.type(screen.getByLabelText('Die 1 · d6'),'3');
    await user.click(screen.getByLabelText('Use Savage Attacker (once per turn)'));
    expect(screen.queryByLabelText('Spend Heroic Inspiration to reroll one die')).not.toBeInTheDocument();
    await user.click(screen.getByLabelText('Use Savage Attacker (once per turn)'));
    await user.click(screen.getByRole('button',{name:'Report these faces'}));
    expect(normal).toHaveBeenCalledExactlyOnceWith([3]);expect(savage).not.toHaveBeenCalled();
  });
});

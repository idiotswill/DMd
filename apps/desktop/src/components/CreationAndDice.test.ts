import { render, screen } from '@testing-library/svelte';
import userEvent from '@testing-library/user-event';
import { describe, it, expect, vi } from 'vitest';
import CharacterForm from './CharacterForm.svelte';
import ContractForm from './ContractForm.svelte';
import RollForm from './RollForm.svelte';
import { contract, options } from './table-fixtures.test-support';

describe('source choices and physical input', () => {
  it('submits source choices and selected loadout without accepting derived totals', async () => {
    const user = userEvent.setup(); const onCreate = vi.fn();
    render(CharacterForm, { options, players:[{id:'player',campaign_id:'campaign',display_name:'Sam'}],onCreate });
    await user.type(screen.getByLabelText('Character name'),'River');
    await user.type(screen.getByRole('spinbutton',{name:'Leather Armor quantity'}),'1');
    await user.click(screen.getByLabelText('Wear purchased Leather Armor'));
    await user.selectOptions(screen.getByLabelText('Weapon mastery 1'), 'longsword');
    expect(screen.getByText('Remaining: 195.00 GP')).toBeTruthy();
    await user.click(screen.getByRole('button',{name:'Create character'}));
    expect(onCreate).toHaveBeenCalledOnce();
    const input = onCreate.mock.calls[0][1];
    expect(input.purchases).toEqual([{item_id:'leather-armor',quantity:1}]);
    expect(input.worn_armor).toBe('leather-armor');
    expect(input.masteries).toEqual(['longsword', 'dagger', 'shortbow']);
    expect(input).not.toHaveProperty('hp'); expect(input).not.toHaveProperty('armor_class');
  });
  it('keeps invalid repeated score assignment local until corrected', async () => {
    const user=userEvent.setup(); const onCreate=vi.fn();
    render(CharacterForm,{options,players:[{id:'player',campaign_id:'campaign',display_name:'Sam'}],onCreate});
    await user.type(screen.getByLabelText('Character name'),'River');
    await user.selectOptions(screen.getByLabelText('Dexterity score'),'15');
    await user.click(screen.getByRole('button',{name:'Create character'}));
    expect(screen.getByRole('alert').textContent).toContain('exactly once'); expect(onCreate).not.toHaveBeenCalled();
  });
  it('submits raw advantage faces without adding the displayed rules modifier', async () => {
    const user=userEvent.setup(); const onSubmit=vi.fn();
    render(RollForm,{request:{id:'roll',roller:'actor',dice:[{sides:20,count:1}],modifier:5,mode:'Advantage',visibility:'Public',reason:'Climb carefully'},onSubmit});
    await user.type(screen.getByLabelText('Die 1 · d20'),'3'); await user.type(screen.getByLabelText('Die 2 · d20'),'17');
    await user.click(screen.getByRole('button',{name:'Report these faces'}));
    expect(onSubmit).toHaveBeenCalledWith([3,17]);
  });
  it('retains every agreement field and does not silently enable the house rule', async () => {
    const user=userEvent.setup(); const onSave=vi.fn(); render(ContractForm,{value:contract,onSave});
    await user.clear(screen.getByLabelText('Content boundaries')); await user.type(screen.getByLabelText('Content boundaries'),'No spiders');
    await user.click(screen.getByRole('button',{name:'Save table agreement'}));
    expect(onSave).toHaveBeenCalledWith({...contract,content_boundaries:'No spiders'});
  });
});

import { render, screen } from '@testing-library/svelte';
import { expect, it, vi } from 'vitest';
import userEvent from '@testing-library/user-event';
import CastingForm from './CastingForm.svelte';
import EncounterPanel from './EncounterPanel.svelte';
import type { CastingOptions, TacticalView } from '../tactical-api';

function options():CastingOptions {
  return {actor:'caster',unavailable:['Fireball: this spell cannot be cast here yet.'],variants:[{
    choice:{actor:'caster',spell_id:'scorching-ray',grant:{CreatureFeature:{feature_id:'spellcasting'}},resource:'SourceFeature',material:'None',mode:'Immediate'},
    label:'Scorching Ray — source ability',concentration:false,minimum_targets:3,maximum_targets:3,repeated_targets:true,
    targets:[{actor:'guard',label:'Visible guard'},{actor:'archer',label:'Visible archer'}],
  },{
    choice:{actor:'caster',spell_id:'hold-person',grant:{CreatureFeature:{feature_id:'spellcasting'}},resource:'SourceFeature',material:{Material:{item:'actual-material'}},mode:'Immediate'},
    label:'Hold Person — source ability, specified material',concentration:true,minimum_targets:1,maximum_targets:2,repeated_targets:false,
    targets:[{actor:'guard',label:'Visible guard'},{actor:'archer',label:'Visible archer'}],
  }]};
}

it('sends actual source choices and explicit repeated ray order without derived mechanics',async()=>{
  const user=userEvent.setup();const onAction=vi.fn();const source=options();
  render(CastingForm,{options:source,onAction});
  expect((screen.getByRole('button',{name:'Cast spell'}) as HTMLButtonElement).disabled).toBe(true);
  await user.selectOptions(screen.getByLabelText('Spell and resource'),JSON.stringify(source.variants[0].choice));
  await user.selectOptions(screen.getByLabelText('Spell target 1'),'guard');
  await user.selectOptions(screen.getByLabelText('Spell target 2'),'archer');
  expect((screen.getByRole('button',{name:'Cast spell'}) as HTMLButtonElement).disabled).toBe(true);
  await user.selectOptions(screen.getByLabelText('Spell target 3'),'guard');
  await user.click(screen.getByRole('button',{name:'Cast spell'}));
  expect(onAction).toHaveBeenCalledExactlyOnceWith({CastSpell:{choice:source.variants[0].choice,targets:{Entities:['guard','archer','guard']}}});
  expect(screen.getByText(/Fireball:.*cannot be cast/)).toBeTruthy();
  expect(screen.queryByRole('option',{name:/Fireball/})).toBeNull();
});

it('retains material identity, permits distinct target count and clears replaced source choices',async()=>{
  const user=userEvent.setup();const onAction=vi.fn();const source=options();
  const component=render(CastingForm,{options:source,onAction});
  await user.selectOptions(screen.getByLabelText('Spell and resource'),JSON.stringify(source.variants[1].choice));
  await user.selectOptions(screen.getByLabelText('Spell target 1'),'guard');
  await user.click(screen.getByRole('button',{name:'Add spell target'}));
  const duplicate=screen.getByLabelText('Spell target 2').querySelector('option[value="guard"]') as HTMLOptionElement;
  expect(duplicate.disabled).toBe(true);
  await user.selectOptions(screen.getByLabelText('Spell target 2'),'archer');
  await user.click(screen.getByRole('button',{name:'Cast spell'}));
  expect(onAction.mock.calls[0][0]).toEqual({CastSpell:{choice:source.variants[1].choice,targets:{Entities:['guard','archer']}}});
  const replaced=structuredClone(source);
  replaced.variants[1].choice.material={Material:{item:'different-material'}};
  await component.rerender({options:replaced});
  expect((screen.getByLabelText('Spell and resource') as HTMLSelectElement).value).toBe('');
  expect(screen.queryByLabelText('Spell target 1')).toBeNull();
  expect((screen.getByRole('button',{name:'Cast spell'}) as HTMLButtonElement).disabled).toBe(true);
});

it('clears no-longer-perceived targets and locks uncertain requests',async()=>{
  const user=userEvent.setup();const onAction=vi.fn();const source=options();
  const component=render(CastingForm,{options:source,onAction});
  await user.selectOptions(screen.getByLabelText('Spell and resource'),JSON.stringify(source.variants[1].choice));
  await user.selectOptions(screen.getByLabelText('Spell target 1'),'guard');
  await component.rerender({disabled:true});
  await user.click(screen.getByRole('button',{name:'Cast spell'}));
  expect(onAction).not.toHaveBeenCalled();
  const changed=structuredClone(source);changed.variants[1].targets=changed.variants[1].targets.filter(target=>target.actor!=='guard');
  await component.rerender({options:changed,disabled:false});
  expect((screen.getByLabelText('Spell target 1') as HTMLSelectElement).value).toBe('');
  expect((screen.getByRole('button',{name:'Cast spell'}) as HTMLButtonElement).disabled).toBe(true);
});

it('resets drafts on controller change and hides another actor casting choices',async()=>{
  const user=userEvent.setup();const source=options();
  const tactical:TacticalView={encounter_id:'encounter',round:1,active_actor:'caster',phase:'active',execution:'ReactionsV1',battlefield:null,participants:[],combatant_sources:[],observers:[],initiative:[],ties:[],continuation:null,may_fail_save:null,legendary_resistance:null,legendary_action:null,budget:null,casting_options:source};
  const component=render(EncounterPanel,{tactical,characters:[],host:true,actor:null,player:null,onAction:vi.fn()});
  await user.selectOptions(screen.getByLabelText('Spell and resource'),JSON.stringify(source.variants[1].choice));
  await user.selectOptions(screen.getByLabelText('Spell target 1'),'guard');
  await component.rerender({host:false,actor:'caster',player:'owner'});
  expect((screen.getByLabelText('Spell and resource') as HTMLSelectElement).value).toBe('');
  await component.rerender({actor:'other',player:'other-owner'});
  expect(screen.queryByText('Cast a spell')).toBeNull();
  expect(screen.queryByText(/Fireball:.*cannot be cast/)).toBeNull();
});

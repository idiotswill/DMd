import { render, screen } from '@testing-library/svelte';
import { expect, it, vi } from 'vitest';
import userEvent from '@testing-library/user-event';
import ShoveDecisionForm from './ShoveDecisionForm.svelte';
import EncounterPanel from './EncounterPanel.svelte';
import type { ShoveView, TacticalView } from '../tactical-api';

const choice=(stage:ShoveView['stage'],key='choice'):ShoveView=>({key,actor:'actor',stage,from:{x:20,y:10,z:0},destination:null});
const base:TacticalView={encounter_id:'encounter',phase:'active',round:1,execution:'EncounterReleaseV1',active_actor:'actor',battlefield:null,
  participants:[],combatant_sources:[],observers:[],initiative:[],ties:[],budget:null,continuation:null,may_fail_save:null,legendary_resistance:null,legendary_action:null};

it('chooses only the saving ability without fabricating dice or voluntary failure',async()=>{
  const onAction=vi.fn();const user=userEvent.setup();
  render(ShoveDecisionForm,{shove:choice('SaveChoice'),onAction});
  await user.click(screen.getByRole('button',{name:'Dexterity saving throw'}));
  expect(onAction).toHaveBeenCalledExactlyOnceWith({ShoveDecision:{handle:'choice',decision:{Save:{ability:'Dexterity'}}}});
  expect(screen.queryByRole('spinbutton')).toBeNull();
  expect(screen.queryByRole('button',{name:'Choose Prone'})).toBeNull();
});

it('keeps all grid directions visible and requires an explicit fresh direction and elevation',async()=>{
  const onAction=vi.fn();const user=userEvent.setup();
  render(ShoveDecisionForm,{shove:choice('OutcomeChoice'),onAction});
  expect((screen.getByLabelText('Push direction') as HTMLSelectElement).value).toBe('');
  expect((screen.getByRole('button',{name:'Propose five-foot push'}) as HTMLButtonElement).disabled).toBe(true);
  expect(screen.getByLabelText('Push direction').querySelectorAll('option')).toHaveLength(10);
  await user.selectOptions(screen.getByLabelText('Push direction'),'1,1');
  await user.selectOptions(screen.getByLabelText('Push elevation'),'0');
  await user.click(screen.getByRole('button',{name:'Propose five-foot push'}));
  expect(onAction).toHaveBeenCalledExactlyOnceWith({ShoveDecision:{handle:'choice',decision:{Outcome:{choice:{Push:{destination:{x:30,y:20,z:0}}}}}}});
});

it('remounts the whole consequence draft on a new opaque choice or selected actor',async()=>{
  const onAction=vi.fn();const user=userEvent.setup();
  const component=render(EncounterPanel,{tactical:{...base,shove:choice('OutcomeChoice')},characters:[],host:false,actor:'actor',player:'player',onAction});
  await user.selectOptions(screen.getByLabelText('Push direction'),'1,0');
  await user.selectOptions(screen.getByLabelText('Push elevation'),'0');
  await component.rerender({tactical:{...base,shove:choice('OutcomeChoice','new-choice')}});
  expect((screen.getByLabelText('Push direction') as HTMLSelectElement).value).toBe('');
  expect((screen.getByLabelText('Push elevation') as HTMLSelectElement).value).toBe('');
  await component.rerender({actor:'someone-else'});
  expect(screen.queryByRole('button',{name:'Choose Prone'})).toBeNull();
});

it('exposes all closed geometry rulings only on the host stage and locks them for pending input',async()=>{
  const onAction=vi.fn();const user=userEvent.setup();
  const tactical={...base,shove:{...choice('PushReview'),destination:{x:30,y:10,z:0}}};
  const component=render(EncounterPanel,{tactical,characters:[],host:false,actor:'actor',player:'player',onAction});
  expect(screen.queryByRole('button',{name:'Commit exact push'})).toBeNull();
  await component.rerender({host:true,actor:null,player:null,playerControlledSources:['actor']});
  await user.click(screen.getByRole('button',{name:'Return choice to shover'}));
  expect(onAction).toHaveBeenCalledExactlyOnceWith({ShoveDecision:{handle:'choice',decision:{RulePush:{ruling:'ReturnToShover'}}}});
  expect(screen.getByRole('button',{name:'Confirm blocked · no movement'})).toBeTruthy();
  await component.rerender({disabled:true});
  expect(screen.getByRole('button',{name:'Commit exact push'}).closest('fieldset')?.disabled).toBe(true);
});

it('offers Shove from only the selected observer contacts and existing attack budget',async()=>{
  const user=userEvent.setup();const onAction=vi.fn();
  const contact=(entity_id:string,status:'Seen'|'Remembered')=>({entity_id,label:entity_id,position:{x:20,y:10,z:0},status,modality:'Sight'});
  const tactical:TacticalView={...base,budget:{movement_spent:0,attacks_remaining:0,action_spent:false,bonus_action_spent:false,reaction_available:true},
    observers:[{observer:'actor',position:null,contacts:[contact('seen','Seen'),contact('old','Remembered')],cells:[]},
      {observer:'other',position:null,contacts:[contact('private','Seen')],cells:[]}]};
  const component=render(EncounterPanel,{tactical,characters:[],host:false,actor:'actor',player:'player',onAction});
  const select=screen.getByLabelText('Shove target');
  expect(select.querySelector('option[value="old"]')).toBeNull();
  expect(select.querySelector('option[value="private"]')).toBeNull();
  await user.selectOptions(select,'seen');await user.click(screen.getByRole('button',{name:'Attempt Shove'}));
  expect(onAction).toHaveBeenCalledExactlyOnceWith({Shove:{target:'seen'}});
  await component.rerender({tactical:{...tactical,budget:{...tactical.budget!,action_spent:true}}});
  expect(screen.getByRole('button',{name:'Attempt Shove'}).closest('fieldset')?.disabled).toBe(true);
});

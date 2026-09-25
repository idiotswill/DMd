import { render, screen } from '@testing-library/svelte';
import userEvent from '@testing-library/user-event';
import { expect, it, vi } from 'vitest';
import HitResponsePanel from './HitResponsePanel.svelte';
import EncounterPanel from './EncounterPanel.svelte';
import type { HitView, SpellCastChoice, TacticalView } from '../tactical-api';

const choice:SpellCastChoice={actor:'mage',spell_id:'shield',grant:{CreatureFeature:{feature_id:'protective-magic'}},resource:'SourceFeature',material:'None',mode:'Immediate'};
const order:HitView={order:{key:'order-handle',actor:'turn-owner',participants:[{actor:'known',label:'Known participant'},{actor:'turn-owner',label:'Your character'}]},delegate:'delegate-handle',response:null};

it('requires an explicit total-order fallback and preserves the chosen ranked order and separate delegation',async()=>{
  const user=userEvent.setup(), onAction=vi.fn();
  render(HitResponsePanel,{hit:order,actor:'turn-owner',host:false,onAction});
  expect((screen.getByRole('button',{name:'Use this order'}) as HTMLButtonElement).disabled).toBe(true);
  await user.click(screen.getByLabelText('Known participant'));
  await user.click(screen.getByLabelText('Your character'));
  await user.click(screen.getByRole('button',{name:'Move ranked participant 2 earlier'}));
  await user.selectOptions(screen.getByLabelText('Other participants'),'BeforeReverse');
  await user.click(screen.getByRole('button',{name:'Use this order'}));
  expect(onAction).toHaveBeenLastCalledWith({HitResponse:{handle:'order-handle',decision:{Order:{instruction:{ranked:['turn-owner','known'],unlisted:'BeforeReverse'}}}}});
  await user.click(screen.getByRole('button',{name:'Let the host order this hit'}));
  expect(onAction).toHaveBeenLastCalledWith({HitResponse:{handle:'delegate-handle',decision:'Delegate'}});
  expect(screen.queryByRole('button',{name:'Offer Shield response'})).toBeNull();
});

it('keeps acknowledgment owned when no Shield exists and separates an offer from its selected paid cast',async()=>{
  const user=userEvent.setup(), onAction=vi.fn();
  const hit:HitView={order:null,delegate:null,response:{key:'intent-handle',actor:'mage',selected:false,shield:[]}};
  const component=render(HitResponsePanel,{hit,actor:'mage',host:false,onAction});
  expect(screen.queryByRole('button',{name:'Offer Shield response'})).toBeNull();
  await user.click(screen.getByRole('button',{name:'Continue without Shield'}));
  expect(onAction).toHaveBeenLastCalledWith({HitResponse:{handle:'intent-handle',decision:{Respond:{accept:false}}}});
  await component.rerender({hit:{...hit,response:{...hit.response!,shield:[choice]}}});
  await user.click(screen.getByRole('button',{name:'Offer Shield response'}));
  expect(onAction).toHaveBeenLastCalledWith({HitResponse:{handle:'intent-handle',decision:{Respond:{accept:true}}}});
  expect(screen.queryByRole('button',{name:'Cast Shield · spend Reaction'})).toBeNull();
  await component.rerender({hit:{...hit,response:{key:'selected-handle',actor:'mage',selected:true,shield:[choice]}}});
  expect((screen.getByRole('button',{name:'Cast Shield · spend Reaction'}) as HTMLButtonElement).disabled).toBe(true);
  await user.selectOptions(screen.getByLabelText('Shield resource'),JSON.stringify(choice));
  await user.click(screen.getByRole('button',{name:'Cast Shield · spend Reaction'}));
  expect(onAction).toHaveBeenLastCalledWith({HitResponse:{handle:'selected-handle',decision:{Cast:{choice}}}});
  await component.rerender({actor:'someone-else'});
  expect(screen.queryByLabelText('Shield resource')).toBeNull();
  expect(screen.getByText('Waiting for this hit to resolve.')).toBeTruthy();
});

it('resets form choices on actor or window replacement and blocks ordinary actions during a hit pause',async()=>{
  const user=userEvent.setup(), onAction=vi.fn();
  const tactical:TacticalView={encounter_id:'encounter',phase:'active',round:1,execution:'ShieldHitV1',active_actor:'mage',battlefield:null,participants:[],combatant_sources:[],observers:[],initiative:[],ties:[],continuation:null,may_fail_save:null,legendary_resistance:null,legendary_action:null,
    budget:{action_spent:false,bonus_action_spent:false,reaction_available:true,movement_spent:0,attacks_remaining:0},
    hit:{order:null,delegate:null,response:{key:'first-hit',actor:'mage',selected:true,shield:[choice]}}};
  const component=render(EncounterPanel,{tactical,characters:[],host:false,actor:'mage',player:'owner',onAction});
  expect(screen.getByRole('button',{name:'End turn'}).closest('fieldset')?.disabled).toBe(true);
  await user.selectOptions(screen.getByLabelText('Shield resource'),JSON.stringify(choice));
  await component.rerender({tactical:{...tactical,hit:{...tactical.hit!,response:{...tactical.hit!.response!,key:'second-hit'}}}});
  expect((screen.getByRole('button',{name:'Cast Shield · spend Reaction'}) as HTMLButtonElement).disabled).toBe(true);
  await component.rerender({actor:'other',player:'other-player'});
  expect(screen.queryByLabelText('Shield resource')).toBeNull();
  await component.rerender({actor:'mage',player:'owner',disabled:true});
  expect(screen.getByRole('button',{name:'Continue without casting'}).closest('fieldset')?.disabled).toBe(true);
});

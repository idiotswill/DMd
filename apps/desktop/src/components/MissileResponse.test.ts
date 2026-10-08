import { render, screen } from '@testing-library/svelte';
import userEvent from '@testing-library/user-event';
import { expect, it, vi } from 'vitest';
import MissileResponsePanel from './MissileResponsePanel.svelte';
import EncounterPanel from './EncounterPanel.svelte';
import type { MissileView, SpellCastChoice, TacticalView } from '../tactical-api';

const shield=(actor:string):SpellCastChoice=>({actor,spell_id:'shield',grant:{CreatureFeature:{feature_id:'protective-magic'}},resource:'SourceFeature',material:'None',mode:'Immediate'});
const response=(actor:string,selected=false)=>({key:`${actor}-${selected?'selected':'intent'}`,actor,selected,shield:[shield(actor)]});
const tactical=(missile:MissileView):TacticalView=>({encounter_id:'encounter',phase:'active',round:1,execution:'ReleasedTimeV1',active_actor:'mage-a',battlefield:null,participants:[],combatant_sources:[],observers:[],initiative:[],ties:[],continuation:null,may_fail_save:null,legendary_resistance:null,legendary_action:null,budget:{action_spent:false,bonus_action_spent:false,reaction_available:true,movement_spent:0,attacks_remaining:0},missile});

it('keeps acknowledgment explicit without Shield and requires separate ordering even with no offers',async()=>{
  const user=userEvent.setup(), onAction=vi.fn();
  render(MissileResponsePanel,{missile:{order:{key:'order',actor:'mage-a',participants:[{actor:'mage-b',label:'Blue cloak'},{actor:'mage-a',label:'Red cloak'}]},delegate:'delegate',responses:[{...response('mage-a'),shield:[]}]},actor:'mage-a',host:false,onAction});
  expect(screen.queryByRole('button',{name:'Offer Shield response'})).toBeNull();
  expect((screen.getByRole('button',{name:'Use this response order'}) as HTMLButtonElement).disabled).toBe(true);
  await user.click(screen.getByRole('button',{name:'Continue without Shield'}));
  expect(onAction).toHaveBeenLastCalledWith({MissileResponse:{handle:'mage-a-intent',decision:{Respond:{accept:false}}}});
  await user.click(screen.getByLabelText('Blue cloak'));
  await user.click(screen.getByLabelText('Red cloak'));
  await user.click(screen.getByRole('button',{name:'Move ranked participant 2 earlier'}));
  await user.selectOptions(screen.getByLabelText('Other participants'),'AfterReverse');
  await user.click(screen.getByRole('button',{name:'Use this response order'}));
  expect(onAction).toHaveBeenLastCalledWith({MissileResponse:{handle:'order',decision:{Order:{instruction:{ranked:['mage-a','mage-b'],unlisted:'AfterReverse'}}}}});
  await user.click(screen.getByRole('button',{name:'Let the host order these responses'}));
  expect(onAction).toHaveBeenLastCalledWith({MissileResponse:{handle:'delegate',decision:'Delegate'}});
});

it('selects only the current owned source and resets payment across actors, windows and changed options',async()=>{
  const user=userEvent.setup(),onAction=vi.fn();
  const missile:MissileView={order:null,delegate:null,responses:[response('mage-a',true),response('mage-b',true)]};
  const component=render(EncounterPanel,{tactical:tactical(missile),characters:[],host:false,actor:'mage-a',player:'owner',onAction});
  expect(screen.getAllByLabelText('Shield resource')).toHaveLength(1);
  expect(screen.getByRole('button',{name:'End turn'}).closest('fieldset')?.disabled).toBe(true);
  await user.selectOptions(screen.getByLabelText('Shield resource'),JSON.stringify(shield('mage-a')));
  await user.click(screen.getByRole('button',{name:'Cast Shield · spend Reaction'}));
  expect(onAction).toHaveBeenLastCalledWith({MissileResponse:{handle:'mage-a-selected',decision:{Cast:{choice:shield('mage-a')}}}});
  await component.rerender({actor:'mage-b'});
  expect((screen.getByRole('button',{name:'Cast Shield · spend Reaction'}) as HTMLButtonElement).disabled).toBe(true);
  await user.selectOptions(screen.getByLabelText('Shield resource'),JSON.stringify(shield('mage-b')));
  await component.rerender({tactical:tactical({...missile,responses:[{...response('mage-b',true),key:'new-window'}]})});
  expect((screen.getByRole('button',{name:'Cast Shield · spend Reaction'}) as HTMLButtonElement).disabled).toBe(true);
  await user.selectOptions(screen.getByLabelText('Shield resource'),JSON.stringify(shield('mage-b')));
  await component.rerender({tactical:tactical({...missile,responses:[{...response('mage-b',true),key:'new-window',shield:[]}]})});
  expect((screen.getByRole('button',{name:'Cast Shield · spend Reaction'}) as HTMLButtonElement).disabled).toBe(true);
  await component.rerender({disabled:true});
  expect(screen.getByRole('button',{name:'Continue without casting'}).closest('fieldset')?.disabled).toBe(true);
});

it('shows Host only its source cards while retaining independently issued delegated response order',async()=>{
  const onAction=vi.fn(),user=userEvent.setup();
  render(MissileResponsePanel,{missile:{order:{key:'delegated',actor:'caster',participants:[]},delegate:null,responses:[response('private',true),{...response('host-npc'),shield:[]}]},actor:null,host:true,playerControlledSources:['private','caster'],onAction});
  expect(screen.queryByLabelText('Shield resource')).toBeNull();
  expect(screen.getAllByRole('button',{name:'Continue without Shield'})).toHaveLength(1);
  await user.selectOptions(screen.getByLabelText('Other participants'),'BeforeForward');
  await user.click(screen.getByRole('button',{name:'Use this response order'}));
  expect(onAction).toHaveBeenLastCalledWith({MissileResponse:{handle:'delegated',decision:{Order:{instruction:{ranked:[],unlisted:'BeforeForward'}}}}});
});

it('retains meaningful distinct dart buttons and saved flow3 hit continuation',async()=>{
  const user=userEvent.setup(),onAction=vi.fn();
  const current=tactical({order:null,delegate:null,responses:[]});
  const component=render(EncounterPanel,{tactical:{...current,missile:null,continuation:{actor:'mage-a',host_adjudication:false,choices:[{handle:'dart-one',label:'Dart 1: Blue cloak'},{handle:'dart-two',label:'Dart 2: Blue cloak'}]}},characters:[],host:false,actor:'mage-a',player:'owner',onAction});
  await user.click(screen.getByRole('button',{name:'Dart 2: Blue cloak · 2'}));
  expect(onAction).toHaveBeenLastCalledWith({ChooseTurnWork:{handle:'dart-two'}});
  await component.rerender({tactical:{...current,execution:'ShieldHitV1',missile:null,hit:{order:null,delegate:null,response:response('mage-a',true)}}});
  expect(screen.queryByRole('button',{name:'End turn'})).toBeNull();
  await user.selectOptions(screen.getByLabelText('Shield resource'),JSON.stringify(shield('mage-a')));
  await user.click(screen.getByRole('button',{name:'Cast Shield · spend Reaction'}));
  expect(onAction).toHaveBeenLastCalledWith({HitResponse:{handle:'mage-a-selected',decision:{Cast:{choice:shield('mage-a')}}}});
});

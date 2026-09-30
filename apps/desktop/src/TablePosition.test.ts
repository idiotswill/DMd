import { render, screen, waitFor, within } from '@testing-library/svelte';
import userEvent from '@testing-library/user-event';
import { afterEach, beforeEach, it, expect, vi } from 'vitest';
import TableApp from './TableApp.svelte';
import { contract, emptyView, options } from './components/table-fixtures.test-support';
import { REQUEST_KEY, SELECTION_KEY, tableApi } from './table-api';
import type { SpellCastChoice } from './tactical-api';

vi.mock('./table-api', async original => ({ ...await original<typeof import('./table-api')>(), tableApi:{defaults:vi.fn(),list:vi.fn(),create:vi.fn(),view:vi.fn(),options:vi.fn(),situation:vi.fn(),action:vi.fn(),text:vi.fn(),rollOptions:vi.fn(),creatureOptions:vi.fn(),sourceControlOptions:vi.fn()} }));
beforeEach(()=>{
  localStorage.clear();vi.resetAllMocks();
  vi.mocked(tableApi.defaults).mockResolvedValue(structuredClone(contract));
  vi.mocked(tableApi.list).mockResolvedValue([{id:'campaign',name:'Saved campaign'}]);
  vi.mocked(tableApi.options).mockResolvedValue(options);
  vi.mocked(tableApi.situation).mockResolvedValue({title:'',description:'',challenges:[]});
  vi.mocked(tableApi.rollOptions).mockResolvedValue({savage_attacker:null});
  vi.mocked(tableApi.creatureOptions).mockResolvedValue([]);
  localStorage.setItem(SELECTION_KEY,JSON.stringify({campaignId:'campaign',playerId:null}));
});
afterEach(()=>vi.unstubAllGlobals());

function consequenceView() {
  const view=emptyView();
  view.active_session={session_id:'session',display_name:'Courtyard',started_at_world:0,participants:[]};
  view.tactical={encounter_id:'encounter',phase:'active',execution:'EncounterReleaseV1',round:1,active_actor:'hag',battlefield:null,participants:[],observers:[],initiative:[],ties:[],budget:null,
    continuation:{actor:'hag',host_adjudication:false,choices:[{handle:'first',label:'Dart 1'},{handle:'second',label:'Dart 2'}]},
    may_fail_save:null,legendary_resistance:null,legendary_action:null,combatant_sources:[]};
  return view;
}

it('keeps successive accepted consequences reachable with fresh controls and their original handles',async()=>{
  const user=userEvent.setup();let view=consequenceView();
  vi.mocked(tableApi.view).mockImplementation(async()=>structuredClone(view));
  vi.mocked(tableApi.action).mockImplementation(async request=>{
    view={...view,revision:`revision-${request.command_id}`,tactical:{...view.tactical!,continuation:{actor:'hag',host_adjudication:false,choices:[{handle:'third',label:'Dart 3'}]}}};
    return {command_id:request.command_id,revision:view.revision,outcome:{message:'Encounter action recorded.'}};
  });
  render(TableApp);
  await waitFor(()=>expect(screen.getByRole('button',{name:'Dart 2 · 2'}).closest('fieldset')?.disabled).toBe(false));
  vi.stubGlobal('scrollY',1420);
  const oldGroup=screen.getByRole('group',{name:'Choose which consequence happens next'});
  await user.click(screen.getByRole('button',{name:'Dart 2 · 2'}));
  await waitFor(()=>expect(window.scrollTo).toHaveBeenCalledWith({left:0,top:1420,behavior:'instant'}));
  const freshGroup=screen.getByRole('group',{name:'Choose which consequence happens next'});
  expect(freshGroup).not.toBe(oldGroup);expect(document.activeElement).toBe(freshGroup);
  expect(screen.queryByRole('button',{name:'Dart 2 · 2'})).toBeNull();
  expect(vi.mocked(tableApi.action).mock.calls[0][0].action).toEqual({Tactical:{action:{ChooseTurnWork:{handle:'second'}}}});
  vi.stubGlobal('scrollY',1500);
  await user.click(screen.getByRole('button',{name:'Dart 3 · 1'}));
  await waitFor(()=>expect(window.scrollTo).toHaveBeenLastCalledWith({left:0,top:1500,behavior:'instant'}));
  expect(vi.mocked(tableApi.action).mock.calls[1][0].action).toEqual({Tactical:{action:{ChooseTurnWork:{handle:'third'}}}});
  expect(localStorage.getItem(REQUEST_KEY)).toBeNull();
});

it('clears previous physical faces while keeping the next roll reachable and using its new request',async()=>{
  const user=userEvent.setup();let view=consequenceView();view.tactical!.continuation=null;
  view.roll_channel='Tactical';view.roll={id:'roll-one',roller:'hag',dice:[{count:1,sides:4}],modifier:1,mode:'Normal',visibility:'Secret',reason:'Spell damage'};
  vi.mocked(tableApi.view).mockImplementation(async()=>structuredClone(view));
  vi.mocked(tableApi.action).mockImplementation(async request=>{
    view={...view,revision:'next-roll',roll:{...view.roll!,id:'roll-two'}};
    return {command_id:request.command_id,revision:view.revision,outcome:{message:'Encounter action recorded.'}};
  });
  render(TableApp);
  await waitFor(()=>expect(screen.getByLabelText('Die 1 · d4').closest('fieldset')?.disabled).toBe(false));
  vi.stubGlobal('scrollY',680);
  await user.type(screen.getByLabelText('Die 1 · d4'),'4');
  await user.click(screen.getByRole('button',{name:'Report these faces'}));
  await waitFor(()=>expect(window.scrollTo).toHaveBeenCalled());
  expect((screen.getByLabelText('Die 1 · d4') as HTMLInputElement).value).toBe('');
  expect(document.activeElement).toBe(screen.getByRole('group',{name:'Report physical dice'}));
  expect(window.scrollTo).toHaveBeenLastCalledWith({left:0,top:680,behavior:'instant'});
  await user.type(screen.getByLabelText('Die 1 · d4'),'2');
  await user.click(screen.getByRole('button',{name:'Report these faces'}));
  await waitFor(()=>expect(tableApi.action).toHaveBeenCalledTimes(2));
  expect(vi.mocked(tableApi.action).mock.calls.map(([request])=>request.action)).toEqual([
    {Tactical:{action:{SubmitRoll:{result:{request_id:'roll-one',source:'Physical',dice:[{sides:4,value:4}]}}}}},
    {Tactical:{action:{SubmitRoll:{result:{request_id:'roll-two',source:'Physical',dice:[{sides:4,value:2}]}}}}},
  ]);
});

it('keeps uncertain delivery on its error and exact saved request without restoring position',async()=>{
  const user=userEvent.setup();vi.mocked(tableApi.view).mockResolvedValue(consequenceView());
  vi.mocked(tableApi.action).mockRejectedValue({message:'Delivery uncertain.',retryable:true});
  render(TableApp);
  await waitFor(()=>expect(screen.getByRole('button',{name:'Dart 1 · 1'}).closest('fieldset')?.disabled).toBe(false));
  await user.click(screen.getByRole('button',{name:'Dart 1 · 1'}));
  await screen.findByText('Delivery uncertain.');
  expect(document.activeElement).toBe(screen.getByRole('alert'));
  expect(window.scrollTo).not.toHaveBeenCalled();
  expect(JSON.parse(localStorage.getItem(REQUEST_KEY)!).request).toEqual(vi.mocked(tableApi.action).mock.calls[0][0]);
  expect(screen.getByRole('button',{name:'Dart 2 · 2'}).closest('fieldset')?.disabled).toBe(true);
});

it.each(['consequence','turn'] as const)('brings a new physical roll into view after the %s decision',async decision=>{
  const user=userEvent.setup();let view=consequenceView();
  if(decision==='turn') {
    view.tactical!.continuation=null;
    view.tactical!.budget={movement_spent:0,attacks_remaining:0,action_spent:true,bonus_action_spent:false,reaction_available:true};
  }
  vi.mocked(tableApi.view).mockImplementation(async()=>structuredClone(view));
  vi.mocked(tableApi.action).mockImplementation(async request=>{
    view={...view,revision:'child-roll',tactical:{...view.tactical!,continuation:null},roll_channel:'Tactical',
      roll:{id:'concentration-child',roller:'hag',dice:[{count:1,sides:20}],modifier:3,mode:'Normal',visibility:'Secret',reason:'Concentration save'}};
    return {command_id:request.command_id,revision:view.revision,outcome:{message:'Encounter action recorded.'}};
  });
  render(TableApp);
  const actionName=decision==='turn'?'End turn':'Dart 1 · 1';
  await waitFor(()=>expect(screen.getByRole('button',{name:actionName}).closest('fieldset')?.disabled).toBe(false));
  vi.stubGlobal('scrollY',1800);
  await user.click(screen.getByRole('button',{name:actionName}));
  await waitFor(()=>expect(document.activeElement).toBe(screen.getByRole('group',{name:'Report physical dice'})));
  expect(vi.mocked(tableApi.action).mock.calls[0][0].action).toEqual({Tactical:{action:decision==='turn'?'EndTurn':{ChooseTurnWork:{handle:'first'}}}});
  expect((screen.getByLabelText('Die 1 · d20') as HTMLInputElement).value).toBe('');
  const calls=vi.mocked(Element.prototype.scrollIntoView);
  expect(calls).toHaveBeenCalledWith({block:'nearest',behavior:'instant'});
  expect(calls.mock.contexts.at(-1)).toBe(document.activeElement);
  await user.type(screen.getByLabelText('Die 1 · d20'),'13');
  await user.click(screen.getByRole('button',{name:'Report these faces'}));
  await waitFor(()=>expect(tableApi.action).toHaveBeenCalledTimes(2));
  expect(vi.mocked(tableApi.action).mock.calls[1][0].action).toEqual({Tactical:{action:{SubmitRoll:{result:{request_id:'concentration-child',source:'Physical',dice:[{sides:20,value:13}]}}}}});
});

it('does not restore tactical focus after navigating to host setup during a request',async()=>{
  const user=userEvent.setup();vi.mocked(tableApi.view).mockResolvedValue(consequenceView());
  let finish!:()=>void;
  vi.mocked(tableApi.action).mockImplementation(request=>new Promise(resolve=>{finish=()=>resolve({command_id:request.command_id,revision:'accepted',outcome:{message:'Encounter action recorded.'}});}));
  render(TableApp);
  await waitFor(()=>expect(screen.getByRole('button',{name:'Dart 1 · 1'}).closest('fieldset')?.disabled).toBe(false));
  await user.click(screen.getByRole('button',{name:'Dart 1 · 1'}));
  await user.click(screen.getByRole('button',{name:'Setup and host controls'}));
  finish();await screen.findByText('Encounter action recorded.');
  await waitFor(()=>expect(screen.getByRole('button',{name:'Refresh saved table'}).hasAttribute('disabled')).toBe(false));
  expect(screen.getByRole('heading',{name:'Current session'})).toBeTruthy();
  expect(screen.queryByRole('group',{name:'Choose which consequence happens next'})).toBeNull();
  expect(window.scrollTo).not.toHaveBeenCalled();
});

function shieldChoice(actor:string):SpellCastChoice {
  return {actor,spell_id:'shield',grant:{CreatureFeature:{feature_id:'protective-magic'}},resource:'SourceFeature',material:'None',mode:'Immediate'};
}
function participant(entity_id:string,public_label:string) {
  return {entity_id,public_label,position:{x:0,y:0,z:0},size:'Medium'};
}
function expectScrolledTo(element:HTMLElement) {
  expect(document.activeElement).toBe(element);
  const calls=vi.mocked(Element.prototype.scrollIntoView);
  expect(calls).toHaveBeenLastCalledWith({block:'nearest',behavior:'instant'});
  expect(calls.mock.contexts.at(-1)).toBe(element);
}

it('brings self-missile ordering, then the owned response, then blank physical dice into view after casting lower on the page',async()=>{
  const user=userEvent.setup();let view=consequenceView();let step=0;
  const choice:SpellCastChoice={actor:'hag',spell_id:'magic-missile',grant:{CreatureFeature:{feature_id:'spellcasting'}},resource:'SourceFeature',material:'None',mode:'Immediate'};
  view.tactical!.continuation=null;
  view.tactical!.participants=[participant('hag','Hag QA')];
  view.tactical!.budget={movement_spent:0,attacks_remaining:0,action_spent:false,bonus_action_spent:false,reaction_available:true};
  view.tactical!.casting_options={actor:'hag',unavailable:[],variants:[{choice,label:'Magic Missile',concentration:false,minimum_targets:6,maximum_targets:6,repeated_targets:true,targets:[{actor:'hag',label:'Hag QA'}]}]};
  vi.mocked(tableApi.view).mockImplementation(async()=>structuredClone(view));
  vi.mocked(tableApi.action).mockImplementation(async request=>{
    step++;
    view={...view,revision:`missile-step-${step}`,tactical:{...view.tactical!,budget:{...view.tactical!.budget!,action_spent:true}}};
    if(step===1) view.tactical!.missile={order:{key:'order-self-missile',actor:'hag',participants:[{actor:'hag',label:'Hag QA'}]},delegate:null,responses:[{key:'hag-intent',actor:'hag',selected:false,shield:[]}]};
    else if(step===2) view.tactical!.missile={...view.tactical!.missile!,order:null};
    else {
      view.tactical!.missile=null;view.roll_channel='Tactical';
      view.roll={id:'first-self-dart',roller:'hag',dice:[{count:1,sides:4}],modifier:1,mode:'Normal',visibility:'Secret',reason:'Magic Missile damage'};
    }
    return {command_id:request.command_id,revision:view.revision,outcome:{message:'Encounter action recorded.'}};
  });
  render(TableApp);
  await waitFor(()=>expect(screen.getByLabelText('Spell and resource').closest('fieldset')?.disabled).toBe(false));
  await user.selectOptions(screen.getByLabelText('Spell and resource'),JSON.stringify(choice));
  for(let dart=1;dart<=6;dart++) await user.selectOptions(screen.getByLabelText(`Spell target ${dart}`),'hag');
  vi.stubGlobal('scrollY',1900);
  await user.click(screen.getByRole('button',{name:'Cast spell'}));
  const order=await screen.findByRole('group',{name:'Order responses to Magic Missile'});
  await waitFor(()=>expectScrolledTo(order));
  expect(window.scrollTo).toHaveBeenLastCalledWith({left:0,top:1900,behavior:'instant'});
  expect(vi.mocked(tableApi.action).mock.calls[0][0].action).toEqual({Tactical:{action:{CastSpell:{choice,targets:{Entities:Array(6).fill('hag')}}}}});
  await user.selectOptions(within(order).getByLabelText('Other participants'),'AfterForward');
  await user.click(within(order).getByRole('button',{name:'Use this response order'}));
  await waitFor(()=>expectScrolledTo(screen.getByRole('group',{name:'Hag QA · Magic Missile response'})));
  expect(screen.queryByRole('group',{name:'Order responses to Magic Missile'})).toBeNull();
  expect(vi.mocked(tableApi.action).mock.calls[1][0].action).toEqual({Tactical:{action:{MissileResponse:{handle:'order-self-missile',decision:{Order:{instruction:{ranked:[],unlisted:'AfterForward'}}}}}}});
  await user.click(screen.getByRole('button',{name:'Continue without Shield'}));
  await waitFor(()=>expectScrolledTo(screen.getByRole('group',{name:'Report physical dice'})));
  expect((screen.getByLabelText('Die 1 · d4') as HTMLInputElement).value).toBe('');
  expect(vi.mocked(tableApi.action).mock.calls[2][0].action).toEqual({Tactical:{action:{MissileResponse:{handle:'hag-intent',decision:{Respond:{accept:false}}}}}});
  expect(localStorage.getItem(REQUEST_KEY)).toBeNull();
});

it.each(['hit','missile'] as const)('focuses the freshly selected owned %s Shield without retaining a resource or borrowing another actor',async kind=>{
  const user=userEvent.setup();let view=consequenceView();let step=0;
  const choice=shieldChoice('mage');view.tactical!.continuation=null;
  view.players=[{id:'owner',campaign_id:'campaign',display_name:'Arin'}];
  view.active_session!.participants=[{player_id:'owner',character_id:null,attendance:'Present'}];
  view.source_control={version:2,actors:[{actor:'mage',name:'Owned Mage',definition_id:'mage',controller:{Player:'owner'},hp:81,max_hp:81},{actor:'foreign',name:'Other Mage',definition_id:'mage',controller:{Player:'other-owner'},hp:81,max_hp:81}]};
  view.tactical!.participants=[participant('mage','Owned Mage'),participant('foreign','Other Mage')];
  const response={key:`${kind}-offer`,actor:'mage',selected:false,shield:[choice]};
  if(kind==='hit') view.tactical!.hit={order:null,delegate:null,response};
  else view.tactical!.missile={order:null,delegate:null,responses:[response,{key:'foreign-selected',actor:'foreign',selected:true,shield:[shieldChoice('foreign')]}]};
  localStorage.setItem(SELECTION_KEY,JSON.stringify({campaignId:'campaign',playerId:'owner',sourceActorId:'mage'}));
  vi.mocked(tableApi.view).mockImplementation(async()=>structuredClone(view));
  vi.mocked(tableApi.action).mockImplementation(async request=>{
    step++;view={...view,revision:`owned-${kind}-${step}`,tactical:{...view.tactical!}};
    if(step===1) {
      const selected={...response,key:`${kind}-selected`,selected:true};
      if(kind==='hit') view.tactical!.hit={order:null,delegate:null,response:selected};
      else view.tactical!.missile={order:null,delegate:null,responses:[selected,{key:'foreign-selected',actor:'foreign',selected:true,shield:[shieldChoice('foreign')]}]};
    } else { view.tactical!.hit=null;view.tactical!.missile=null; }
    return {command_id:request.command_id,revision:view.revision,outcome:{message:'Encounter action recorded.'}};
  });
  render(TableApp);
  await waitFor(()=>expect(screen.getByRole('button',{name:'Offer Shield response'}).closest('fieldset')?.disabled).toBe(false));
  const previous=screen.getByRole('button',{name:'Offer Shield response'}).closest('fieldset')!;
  vi.stubGlobal('scrollY',1550);
  await user.click(screen.getByRole('button',{name:'Offer Shield response'}));
  const label=kind==='hit'?'Resolve your Shield response':'Owned Mage · Magic Missile response';
  await waitFor(()=>expectScrolledTo(screen.getByRole('group',{name:label})));
  const selected=screen.getByRole('group',{name:label});
  expect(selected).not.toBe(previous);
  expect(selected.dataset.tacticalFocusId).not.toBe(previous.dataset.tacticalFocusId);
  expect((within(selected).getByLabelText('Shield resource') as HTMLSelectElement).value).toBe('');
  expect(screen.queryByRole('group',{name:'Other Mage · Magic Missile response'})).toBeNull();
  await user.selectOptions(within(selected).getByLabelText('Shield resource'),JSON.stringify(choice));
  await user.click(within(selected).getByRole('button',{name:'Cast Shield · spend Reaction'}));
  await waitFor(()=>expect(tableApi.action).toHaveBeenCalledTimes(2));
  const key=kind==='hit'?'HitResponse':'MissileResponse';
  expect(vi.mocked(tableApi.action).mock.calls[0][0]).toMatchObject({version:2,channel:{SourceCreature:{player_id:'owner',actor:'mage'}},action:{Tactical:{action:{[key]:{handle:`${kind}-offer`,decision:{Respond:{accept:true}}}}}}});
  expect(vi.mocked(tableApi.action).mock.calls[1][0]).toMatchObject({version:2,channel:{SourceCreature:{player_id:'owner',actor:'mage'}},action:{Tactical:{action:{[key]:{handle:`${kind}-selected`,decision:{Cast:{choice}}}}}}});
});

it('moves focus to a different Host-owned selected missile responder and clears the previous payment choice',async()=>{
  const user=userEvent.setup();let view=consequenceView();view.tactical!.continuation=null;
  view.tactical!.participants=[participant('red','Red Mage'),participant('blue','Blue Mage')];
  view.tactical!.missile={order:null,delegate:null,responses:[{key:'selected-red',actor:'red',selected:true,shield:[shieldChoice('red')]}]};
  vi.mocked(tableApi.view).mockImplementation(async()=>structuredClone(view));
  vi.mocked(tableApi.action).mockImplementation(async request=>{
    view={...view,revision:'blue-selected',tactical:{...view.tactical!,missile:{order:null,delegate:null,responses:[{key:'selected-blue',actor:'blue',selected:true,shield:[shieldChoice('blue')]}]}}};
    return {command_id:request.command_id,revision:view.revision,outcome:{message:'Encounter action recorded.'}};
  });
  render(TableApp);
  await waitFor(()=>expect(screen.getByLabelText('Shield resource').closest('fieldset')?.disabled).toBe(false));
  await user.selectOptions(screen.getByLabelText('Shield resource'),JSON.stringify(shieldChoice('red')));
  vi.stubGlobal('scrollY',1250);
  await user.click(screen.getByRole('button',{name:'Cast Shield · spend Reaction'}));
  await waitFor(()=>expectScrolledTo(screen.getByRole('group',{name:'Blue Mage · Magic Missile response'})));
  expect(screen.queryByRole('group',{name:'Red Mage · Magic Missile response'})).toBeNull();
  expect((screen.getByLabelText('Shield resource') as HTMLSelectElement).value).toBe('');
  expect((screen.getByRole('button',{name:'Cast Shield · spend Reaction'}) as HTMLButtonElement).disabled).toBe(true);
  expect(vi.mocked(tableApi.action).mock.calls[0][0].action).toEqual({Tactical:{action:{MissileResponse:{handle:'selected-red',decision:{Cast:{choice:shieldChoice('red')}}}}}});
});

it('brings Legendary Resistance into view after the owned physical save fails',async()=>{
  const user=userEvent.setup();let view=consequenceView();view.tactical!.continuation=null;
  view.roll_channel='Tactical';view.roll={id:'dragon-save',roller:'dragon',dice:[{count:1,sides:20}],modifier:5,mode:'Normal',visibility:'Secret',reason:'Dragon saving throw'};
  vi.mocked(tableApi.view).mockImplementation(async()=>structuredClone(view));
  vi.mocked(tableApi.action).mockImplementation(async request=>{
    view={...view,revision:'failed-save',roll:null,tactical:{...view.tactical!,legendary_resistance:'dragon'}};
    return {command_id:request.command_id,revision:view.revision,outcome:{message:'Encounter action recorded.'}};
  });
  render(TableApp);
  await waitFor(()=>expect(screen.getByLabelText('Die 1 · d20').closest('fieldset')?.disabled).toBe(false));
  await user.type(screen.getByLabelText('Die 1 · d20'),'1');
  await user.click(screen.getByRole('button',{name:'Report these faces'}));
  await waitFor(()=>expectScrolledTo(screen.getByRole('group',{name:'Legendary Resistance'})));
  expect(screen.queryByRole('group',{name:'Report physical dice'})).toBeNull();
  expect(vi.mocked(tableApi.action).mock.calls[0][0].action).toEqual({Tactical:{action:{SubmitRoll:{result:{request_id:'dragon-save',source:'Physical',dice:[{sides:20,value:1}]}}}}});
});

it('skips the previous opportunity prompt while its controls are disabled by the new physical attack roll',async()=>{
  const user=userEvent.setup();let view=consequenceView();view.tactical!.continuation=null;
  view.tactical!.opportunity={actor:'hag',target:{actor:'runner',label:'Runner'},weapons:null,unarmed:true,features:[]};
  vi.mocked(tableApi.view).mockImplementation(async()=>structuredClone(view));
  vi.mocked(tableApi.action).mockImplementation(async request=>{
    view={...view,revision:'opportunity-roll',roll_channel:'Tactical',roll:{id:'opportunity-attack',roller:'hag',dice:[{count:1,sides:20}],modifier:4,mode:'Normal',visibility:'Secret',reason:'Opportunity attack'}};
    return {command_id:request.command_id,revision:view.revision,outcome:{message:'Encounter action recorded.'}};
  });
  render(TableApp);
  await waitFor(()=>expect(screen.getByRole('button',{name:'Unarmed Strike (Strength damage)'}).closest('fieldset')?.disabled).toBe(false));
  await user.click(screen.getByRole('button',{name:'Unarmed Strike (Strength damage)'}));
  await waitFor(()=>expectScrolledTo(screen.getByRole('group',{name:'Report physical dice'})));
  expect(screen.getByRole('group',{name:'Other reaction choices'}).hasAttribute('disabled')).toBe(true);
  expect(document.activeElement).not.toBe(screen.getByRole('region',{name:'Opportunity attack'}));
  expect(vi.mocked(tableApi.action).mock.calls[0][0].action).toEqual({Tactical:{action:{OpportunityAttack:{choice:{UnarmedDamage:{ability:'Strength'}}}}}});
});

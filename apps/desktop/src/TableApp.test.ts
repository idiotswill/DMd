import { render, screen, waitFor } from '@testing-library/svelte';
import userEvent from '@testing-library/user-event';
import { beforeEach, describe, it, expect, vi } from 'vitest';
import TableApp from './TableApp.svelte';
import { contract, emptyView, options } from './components/table-fixtures.test-support';
import { REQUEST_KEY, SELECTION_KEY, tableApi, type UnconfirmedRequest } from './table-api';
vi.mock('./table-api', async (original) => ({ ...await original<typeof import('./table-api')>(), tableApi:{defaults:vi.fn(),list:vi.fn(),create:vi.fn(),view:vi.fn(),options:vi.fn(),situation:vi.fn(),action:vi.fn(),text:vi.fn(),rollOptions:vi.fn(),creatureOptions:vi.fn()} }));

beforeEach(() => { localStorage.clear(); vi.resetAllMocks(); vi.mocked(tableApi.defaults).mockResolvedValue(structuredClone(contract)); vi.mocked(tableApi.list).mockResolvedValue([{id:'campaign',name:'Saved campaign'}]); vi.mocked(tableApi.view).mockResolvedValue(emptyView()); vi.mocked(tableApi.options).mockResolvedValue(options); vi.mocked(tableApi.situation).mockResolvedValue({title:'',description:'',challenges:[]}); vi.mocked(tableApi.rollOptions).mockResolvedValue({savage_attacker:null}); vi.mocked(tableApi.creatureOptions).mockResolvedValue([]); });
describe('durable UI retry',()=>{
  it('prepares a newly queried source without using the historical catalog',async()=>{
    const user=userEvent.setup(); const view=emptyView();
    view.characters=[{character_id:'pc',entity_id:'actor',player_id:'player',name:'River',profile:null,sheet:null,details:null,second_wind_remaining:null}];
    view.creature_setup={catalog:[],creatures:[]};
    const mage={definition_id:'mage',name:'Mage',sizes:['Medium' as const],additional_languages:3,ammunition_required:false,item_count:2,abilities:['Spellcasting'],omitted_features:['light']};
    vi.mocked(tableApi.creatureOptions).mockResolvedValue([mage]);
    vi.mocked(tableApi.view).mockResolvedValue(view);
    vi.mocked(tableApi.action).mockRejectedValue({message:'Delivery uncertain.',retryable:true});
    localStorage.setItem(SELECTION_KEY,JSON.stringify({campaignId:'campaign',playerId:null}));
    render(TableApp);
    await user.click(await screen.findByRole('button',{name:'Setup and host controls'}));
    await screen.findByRole('option',{name:'Mage'});
    expect(tableApi.creatureOptions).toHaveBeenCalledWith({campaign_id:'campaign',channel:'Host',revision:view.revision});
    await user.type(screen.getByLabelText(/Additional languages/),'dwarvish, elvish, draconic');
    await user.click(screen.getByRole('button',{name:'Prepare creature'}));
    await screen.findByRole('alert');
    const saved=JSON.parse(localStorage.getItem(REQUEST_KEY)!);
    expect(saved.request.action.CreateCreature.creation).toMatchObject({definition_id:'mage',size:'Medium',additional_languages:['dwarvish','elvish','draconic'],ammunition_units:0});
    expect(saved.request.action.CreateCreature.creation.item_ids).toHaveLength(2);
  });
  it.each(['campaign','channel','revision'] as const)('discards late catalog data and errors after a %s switch',async(change)=>{
    const user=userEvent.setup();const view=emptyView();
    view.players=[{id:'player',campaign_id:'campaign',display_name:'Sam'}];
    view.creature_setup={catalog:[],creatures:[]};
    vi.mocked(tableApi.list).mockResolvedValue([{id:'campaign',name:'First'},{id:'other',name:'Second'}]);
    vi.mocked(tableApi.view).mockResolvedValue(view);
    let resolveOld!:(value:Awaited<ReturnType<typeof tableApi.creatureOptions>>)=>void;
    let rejectOld!:(reason:unknown)=>void;
    const pending=new Promise<Awaited<ReturnType<typeof tableApi.creatureOptions>>>((resolve,reject)=>{resolveOld=resolve;rejectOld=reject;});
    vi.mocked(tableApi.creatureOptions).mockReturnValueOnce(pending).mockResolvedValue([]);
    localStorage.setItem(SELECTION_KEY,JSON.stringify({campaignId:'campaign',playerId:null}));
    const mounted=render(TableApp);
    await waitFor(()=>expect(tableApi.creatureOptions).toHaveBeenCalledOnce());
    await user.click(screen.getByRole('button',{name:'Setup and host controls'}));
    expect(screen.queryByLabelText('Creature source')).toBeNull();
    const switchView=async()=>{
      if(change==='campaign') { vi.mocked(tableApi.view).mockResolvedValue({...view,campaign_id:'other'}); await user.selectOptions(screen.getByLabelText('Campaign'),'other'); }
      else if(change==='channel') await user.selectOptions(screen.getByLabelText('Local viewing and input channel'),'player');
      else {vi.mocked(tableApi.view).mockResolvedValue({...view,revision:'new-revision'});await user.click(screen.getByRole('button',{name:'Refresh saved table'}));}
      await waitFor(()=>expect(screen.getByRole('button',{name:'Refresh saved table'}).hasAttribute('disabled')).toBe(false));
    };
    await switchView();
    resolveOld([{definition_id:'old-secret',name:'Old private source',sizes:['Medium'],additional_languages:0,ammunition_required:false,item_count:0,abilities:[],omitted_features:[]}]);
    await pending;
    if(change!=='channel') await user.click(screen.getByRole('button',{name:'Setup and host controls'}));
    expect(screen.queryByRole('option',{name:'Old private source'})).toBeNull();
    mounted.unmount();
    // Repeat with an old error; it must not become the new table's error banner.
    vi.mocked(tableApi.view).mockResolvedValue(view);
    localStorage.setItem(SELECTION_KEY,JSON.stringify({campaignId:'campaign',playerId:null}));
    const failed=new Promise<Awaited<ReturnType<typeof tableApi.creatureOptions>>>((_,reject)=>{rejectOld=reject;});
    vi.mocked(tableApi.creatureOptions).mockReturnValueOnce(failed);
    render(TableApp);
    await waitFor(()=>expect(screen.getByRole('button',{name:'Refresh saved table'}).hasAttribute('disabled')).toBe(false));
    await switchView();
    rejectOld('Old private error');
    await failed.catch(()=>{});
    expect(screen.queryByRole('alert')).toBeNull();
  });
  it('queries the owned roll and retries the original two-set tactical request',async()=>{
    const user=userEvent.setup();const view=emptyView();
    view.players=[{id:'player',campaign_id:'campaign',display_name:'Sam'}];
    view.characters=[{character_id:'pc',entity_id:'actor',player_id:'player',name:'River',profile:null,sheet:null,details:null,second_wind_remaining:null}];
    view.active_session={session_id:'session',display_name:'Evening',started_at_world:0,participants:[{player_id:'player',character_id:'pc',attendance:'Present'}]};
    view.roll={id:'opaque-damage',roller:'actor',dice:[{count:2,sides:4}],modifier:3,mode:'Normal',visibility:'Public',reason:'Attack damage'};
    view.roll_channel='Tactical';
    vi.mocked(tableApi.view).mockResolvedValue(view);
    vi.mocked(tableApi.rollOptions).mockResolvedValue({savage_attacker:{weapon_dice:2,heroic_inspiration:false}});
    vi.mocked(tableApi.action).mockRejectedValueOnce({message:'Delivery uncertain.',retryable:true}).mockImplementationOnce(async request=>({command_id:request.command_id,revision:'accepted',outcome:{message:'Damage accepted.'}}));
    localStorage.setItem(SELECTION_KEY,JSON.stringify({campaignId:'campaign',playerId:'player'}));
    render(TableApp);
    await screen.findByLabelText('Use Savage Attacker (once per turn)');
    expect(tableApi.rollOptions).toHaveBeenCalledWith({campaign_id:'campaign',channel:{Player:{player_id:'player',character_id:'pc'}},revision:view.revision,roll_id:'opaque-damage'});
    await user.type(screen.getByLabelText('Die 1 · d4'),'1');await user.type(screen.getByLabelText('Die 2 · d4'),'2');
    await user.click(screen.getByLabelText('Use Savage Attacker (once per turn)'));
    await user.type(screen.getByLabelText('Second weapon die 1 · d4'),'3');await user.type(screen.getByLabelText('Second weapon die 2 · d4'),'4');
    await user.selectOptions(screen.getByLabelText('Damage set to use'),'First');
    await user.click(screen.getByRole('button',{name:'Report these faces'}));
    await screen.findByText('Delivery uncertain.');
    const saved=JSON.parse(localStorage.getItem(REQUEST_KEY)!) as UnconfirmedRequest;
    const expectedRoll={weapon_dice:2,chosen:'First',inspiration:null,
      first:{request_id:'opaque-damage',source:'Physical',dice:[{sides:4,value:1},{sides:4,value:2}]},
      second:{request_id:'opaque-damage',source:'Physical',dice:[{sides:4,value:3},{sides:4,value:4}]}};
    expect(saved).toMatchObject({kind:'action',request:{revision:view.revision,action:{Tactical:{action:{SubmitSavageAttacker:{roll:expectedRoll}}}}}});
    await user.click(screen.getByRole('button',{name:'Retry original request'}));
    await waitFor(()=>expect(localStorage.getItem(REQUEST_KEY)).toBeNull());
    expect(tableApi.action).toHaveBeenNthCalledWith(1,saved.request);expect(tableApi.action).toHaveBeenNthCalledWith(2,saved.request);
  });
  it('retries an actual source area declaration with its original aim and host ordering',async()=>{
    const user=userEvent.setup();const view=emptyView();
    view.active_session={session_id:'session',display_name:'Crossing',started_at_world:0,participants:[]};
    view.tactical={encounter_id:'encounter',phase:'active',execution:'ReactionsV1',round:1,active_actor:'chimera',battlefield:null,participants:[],observers:[],initiative:[],ties:[],budget:null,continuation:null,may_fail_save:null,legendary_resistance:null,legendary_action:null,combatant_sources:[],area_options:{actor:'chimera',controller:null,source_space:{min:{x:10,y:40,z:0},max:{x:30,y:60,z:12}},variants:[{feature_id:'fire-breath',label:'Fire Breath',length_feet:15}],unavailable:[]}};
    vi.mocked(tableApi.view).mockResolvedValue(view);
    localStorage.setItem(SELECTION_KEY,JSON.stringify({campaignId:'campaign',playerId:null}));
    vi.mocked(tableApi.action).mockRejectedValueOnce({message:'Delivery uncertain.',retryable:true}).mockImplementationOnce(async request=>({command_id:request.command_id,revision:'accepted-revision',outcome:{message:'Encounter action recorded.'}}));
    const mounted=render(TableApp);
    await waitFor(()=>expect(screen.getByLabelText('Ability').closest('fieldset')?.hasAttribute('disabled')).toBe(false));
    await user.selectOptions(screen.getByLabelText('Ability'),'fire-breath');
    await user.click(screen.getByRole('button',{name:'Use area ability'}));
    await screen.findByRole('alert');
    const saved=JSON.parse(localStorage.getItem(REQUEST_KEY)!);
    expect(saved.request.action).toEqual({Tactical:{action:{CreatureArea:{feature_id:'fire-breath',aim:{origin:{x:30,y:50,z:6},toward:{x:40,y:50,z:6},include_origin:false},ordering:'Host'}}}});
    expect(saved.request.channel).toBe('Host');expect(saved.request.session_id).toBe('session');
    expect(saved.request.revision).toBe(view.revision);
    mounted.unmount();render(TableApp);
    await waitFor(()=>expect(screen.getByRole('button',{name:'Retry original request'}).hasAttribute('disabled')).toBe(false));
    await user.click(screen.getByRole('button',{name:'Retry original request'}));
    await waitFor(()=>expect(localStorage.getItem(REQUEST_KEY)).toBeNull());
    expect(tableApi.action).toHaveBeenCalledTimes(2);
    expect(tableApi.action).toHaveBeenNthCalledWith(1,saved.request);expect(tableApi.action).toHaveBeenNthCalledWith(2,saved.request);
  });
  it('retains the actual shield form item, hand and original action head through uncertain restart',async()=>{
    const user=userEvent.setup();const view=emptyView();
    view.active_session={session_id:'session',display_name:'Courtyard',started_at_world:0,participants:[]};
    view.tactical={encounter_id:'encounter',phase:'active',execution:'ReactionsV1',round:3,active_actor:'source-actor',battlefield:null,participants:[],observers:[],initiative:[],ties:[],budget:null,continuation:null,may_fail_save:null,legendary_resistance:null,legendary_action:null,combatant_sources:[],shield_options:{actor:'source-actor',donned:null,shields:[{item:'carried-shield',hands:['Right']}]}};
    vi.mocked(tableApi.view).mockResolvedValue(view);
    localStorage.setItem(SELECTION_KEY,JSON.stringify({campaignId:'campaign',playerId:null}));
    vi.mocked(tableApi.action).mockRejectedValueOnce({message:'Delivery uncertain.',retryable:true}).mockImplementationOnce(async request=>({command_id:request.command_id,revision:'accepted-revision',outcome:{message:'Encounter action recorded.'}}));
    const first=render(TableApp);
    await waitFor(()=>expect(screen.getByRole('button',{name:'Don shield'}).closest('fieldset')?.hasAttribute('disabled')).toBe(false));
    await user.click(screen.getByRole('button',{name:'Don shield'}));
    await screen.findByRole('alert');
    const saved=JSON.parse(localStorage.getItem(REQUEST_KEY)!);
    expect(saved.request.action).toEqual({Tactical:{action:{DonShield:{shield:'carried-shield',hand:'Right'}}}});
    expect(saved.request.channel).toBe('Host');expect(saved.request.session_id).toBe('session');expect(saved.request.revision).toBe(view.revision);
    first.unmount();render(TableApp);
    await waitFor(()=>expect(screen.getByRole('button',{name:'Retry original request'}).hasAttribute('disabled')).toBe(false));
    await user.click(screen.getByRole('button',{name:'Retry original request'}));
    await waitFor(()=>expect(localStorage.getItem(REQUEST_KEY)).toBeNull());
    expect(tableApi.action).toHaveBeenCalledTimes(2);
    expect(tableApi.action).toHaveBeenNthCalledWith(1,saved.request);expect(tableApi.action).toHaveBeenNthCalledWith(2,saved.request);
  });
  it.each([
    { channel: 'Table' as const, phase: 'setup', sides: 20, label: 'Skill check' },
    { channel: 'Table' as const, phase: 'setup', sides: 10, label: 'Second Wind' },
    { channel: 'Tactical' as const, phase: 'initiative', sides: 20, label: 'Initiative' },
  ])('routes $label through its pending handler even with a prepared map', async ({channel,phase,sides,label}) => {
    const user=userEvent.setup();const view=emptyView();
    view.players=[{id:'player',campaign_id:'campaign',display_name:'Sam'}];
    view.characters=[{character_id:'pc',entity_id:'actor',player_id:'player',name:'River',profile:null,sheet:null,details:null,second_wind_remaining:null}];
    view.active_session={session_id:'session',display_name:'Evening',started_at_world:0,participants:[{player_id:'player',character_id:'pc',attendance:'Present'}]};
    view.tactical={encounter_id:'encounter',round:null,active_actor:null,phase,battlefield:null,participants:[],observers:[],initiative:[],ties:[],budget:null,continuation:null,may_fail_save:null,legendary_resistance:null,legendary_action:null,combatant_sources:[]};
    view.roll={id:'roll',roller:'actor',dice:[{count:1,sides}],modifier:2,mode:'Normal',visibility:'Public',reason:label};
    view.roll_channel=channel;
    vi.mocked(tableApi.view).mockResolvedValue(view);
    vi.mocked(tableApi.action).mockImplementation(async received=>({command_id:received.command_id,revision:'accepted-revision',outcome:{message:'Roll accepted.'}}));
    localStorage.setItem(SELECTION_KEY,JSON.stringify({campaignId:'campaign',playerId:'player'}));
    render(TableApp);
    const die=await screen.findByLabelText(`Die 1 · d${sides}`);
    await waitFor(()=>expect(die.closest('fieldset')?.disabled).toBe(false));
    await user.type(die,'7');
    await user.click(screen.getByRole('button',{name:'Report these faces'}));
    await waitFor(()=>expect(tableApi.action).toHaveBeenCalledOnce());
    expect(vi.mocked(tableApi.action).mock.calls[0][0]).toMatchObject({channel:{Player:{player_id:'player',character_id:'pc'}},action:channel==='Tactical'
      ? {Tactical:{action:{SubmitRoll:{result:{request_id:'roll',source:'Physical',dice:[{sides,value:7}]}}}}}
      : {SubmitPhysical:{request_id:'roll',faces:[7]}}});
  });
  it('retries NPC creation after uncertain delivery and restart with the same source and item IDs',async()=>{
    const user=userEvent.setup();
    const view=emptyView();
    view.characters=[{character_id:'pc',entity_id:'actor',player_id:'player',name:'River',profile:null,sheet:null,details:null,second_wind_remaining:null}];
    view.creature_setup={catalog:[{definition_id:'goblin-warrior',name:'Goblin Warrior',sizes:['Small'],additional_languages:0,ammunition_required:true,item_count:5,abilities:['Scimitar','Shortbow'],omitted_features:[]}],creatures:[]};
    vi.mocked(tableApi.creatureOptions).mockResolvedValue(view.creature_setup.catalog);
    vi.mocked(tableApi.view).mockResolvedValue(view);
    localStorage.setItem(SELECTION_KEY,JSON.stringify({campaignId:'campaign',playerId:null}));
    vi.mocked(tableApi.action).mockRejectedValueOnce('Connection interrupted').mockImplementationOnce(async received => ({command_id:received.command_id,revision:'accepted-revision',outcome:{message:'Host preparation recorded.'}}));
    const first=render(TableApp);
    await user.click(await screen.findByRole('button',{name:'Setup and host controls'}));
    await waitFor(()=>expect(screen.getByRole('button',{name:'Prepare creature'}).closest('fieldset')?.hasAttribute('disabled')).toBe(false));
    await user.type(screen.getByLabelText('Creature name'),'Private sentry');
    await user.clear(screen.getByLabelText('Starting ammunition per type'));
    await user.type(screen.getByLabelText('Starting ammunition per type'),'7');
    await user.click(screen.getByRole('button',{name:'Prepare creature'}));
    await screen.findByRole('alert');
    const request=JSON.parse(localStorage.getItem(REQUEST_KEY)!);
    expect(request.request.action.CreateCreature.creation).toMatchObject({name:'Private sentry',definition_id:'goblin-warrior',size:'Small',additional_languages:[],ammunition_units:7});
    expect(new Set(request.request.action.CreateCreature.creation.item_ids).size).toBe(5);
    expect(request.request.channel).toBe('Host');
    expect(request.request.revision).toBe(view.revision);
    first.unmount();render(TableApp);
    await waitFor(()=>expect(screen.getByRole('button',{name:'Retry original request'}).hasAttribute('disabled')).toBe(false));
    await user.click(screen.getByRole('button',{name:'Retry original request'}));
    await waitFor(()=>expect(localStorage.getItem(REQUEST_KEY)).toBeNull());
    expect(tableApi.action).toHaveBeenCalledTimes(2);
    expect(tableApi.action).toHaveBeenNthCalledWith(1,request.request);expect(tableApi.action).toHaveBeenNthCalledWith(2,request.request);
  });
  it('persists an actual casting form submission and retries its original targets and material after restart',async()=>{
    const user=userEvent.setup();const view=emptyView();
    const choice={actor:'source-caster',spell_id:'hold-person',grant:{CreatureFeature:{feature_id:'spellcasting'}},resource:'SourceFeature',material:{Material:{item:'actual-component'}},mode:'Immediate'} as const;
    view.active_session={session_id:'session',display_name:'Courtyard',started_at_world:0,participants:[]};
    view.tactical={encounter_id:'encounter',phase:'active',execution:'ReactionsV1',round:1,active_actor:'source-caster',battlefield:null,participants:[],observers:[],initiative:[],ties:[],budget:null,continuation:null,may_fail_save:null,legendary_resistance:null,legendary_action:null,combatant_sources:[],casting_options:{actor:'source-caster',unavailable:[],variants:[{choice,label:'Hold Person — source ability',concentration:true,minimum_targets:1,maximum_targets:1,repeated_targets:false,targets:[{actor:'selected-target',label:'Visible traveler'}]}]}};
    vi.mocked(tableApi.view).mockResolvedValue(view);
    localStorage.setItem(SELECTION_KEY,JSON.stringify({campaignId:'campaign',playerId:null}));
    vi.mocked(tableApi.action).mockRejectedValueOnce({message:'Delivery uncertain.',retryable:true}).mockImplementationOnce(async request=>({command_id:request.command_id,revision:'accepted-revision',outcome:{message:'Encounter action recorded.'}}));
    const mounted=render(TableApp);
    await waitFor(()=>expect(screen.getByLabelText('Spell and resource').closest('fieldset')?.hasAttribute('disabled')).toBe(false));
    await user.selectOptions(screen.getByLabelText('Spell and resource'),JSON.stringify(choice));
    await user.selectOptions(screen.getByLabelText('Spell target 1'),'selected-target');
    await user.click(screen.getByRole('button',{name:'Cast spell'}));
    await screen.findByRole('alert');
    const saved=JSON.parse(localStorage.getItem(REQUEST_KEY)!);
    expect(saved.request.action).toEqual({Tactical:{action:{CastSpell:{choice,targets:{Entities:['selected-target']}}}}});
    expect(saved.request.channel).toBe('Host');expect(saved.request.session_id).toBe('session');
    expect(saved.request.revision).toBe(view.revision);
    mounted.unmount();render(TableApp);
    await waitFor(()=>expect(screen.getByRole('button',{name:'Retry original request'}).hasAttribute('disabled')).toBe(false));
    await user.click(screen.getByRole('button',{name:'Retry original request'}));
    await waitFor(()=>expect(localStorage.getItem(REQUEST_KEY)).toBeNull());
    expect(tableApi.action).toHaveBeenCalledTimes(2);
    expect(tableApi.action).toHaveBeenNthCalledWith(1,saved.request);expect(tableApi.action).toHaveBeenNthCalledWith(2,saved.request);
  });
  it('retries NPC creation after uncertain delivery and restart with the same source and item IDs',async()=>{
    const user=userEvent.setup();
    const request:UnconfirmedRequest={kind:'action',request:{command_id:'npc-command',campaign_id:'campaign',expected_event_sequence:4,session_id:null,channel:'Host',action:{CreateCreature:{creation:{entity_id:'same-creature',name:'Private sentry',definition_id:'goblin-warrior',size:'Small',additional_languages:[],ammunition_units:7,item_ids:['arrows','armor','scimitar','shield','bow']}}}}};
    localStorage.setItem(REQUEST_KEY,JSON.stringify(request));
    vi.mocked(tableApi.action).mockRejectedValueOnce('Connection interrupted').mockResolvedValueOnce({command_id:'npc-command',revision:'accepted-revision',outcome:{message:'Host preparation recorded.'}});
    const first=render(TableApp);
    await waitFor(()=>expect(screen.getByRole('button',{name:'Retry original request'}).hasAttribute('disabled')).toBe(false));
    await user.click(screen.getByRole('button',{name:'Retry original request'}));await screen.findByRole('alert');
    expect(JSON.parse(localStorage.getItem(REQUEST_KEY)!)).toEqual(request);
    first.unmount();render(TableApp);
    await waitFor(()=>expect(screen.getByRole('button',{name:'Retry original request'}).hasAttribute('disabled')).toBe(false));
    await user.click(screen.getByRole('button',{name:'Retry original request'}));
    await waitFor(()=>expect(localStorage.getItem(REQUEST_KEY)).toBeNull());
    expect(tableApi.action).toHaveBeenCalledTimes(2);
    expect(tableApi.action).toHaveBeenNthCalledWith(1,request.request);expect(tableApi.action).toHaveBeenNthCalledWith(2,request.request);
  });
  it('reports raw tactical dice through the retained tactical action envelope',async()=>{
    const user=userEvent.setup();const view=emptyView();
    view.players=[{id:'player',campaign_id:'campaign',display_name:'Sam'}];
    view.characters=[{character_id:'pc',entity_id:'actor',player_id:'player',name:'River',profile:null,sheet:null,details:null,second_wind_remaining:null}];
    view.active_session={session_id:'session',display_name:'Encounter',started_at_world:0,participants:[{player_id:'player',character_id:'pc',attendance:'Present'}]};
    view.tactical={encounter_id:'encounter',phase:'initiative',round:null,active_actor:null,battlefield:null,participants:[],observers:[],initiative:[],ties:[],budget:null,continuation:null,may_fail_save:null,legendary_resistance:null,legendary_action:null,combatant_sources:[]};
    view.roll_channel='Tactical';
    view.roll={id:'initiative-roll',roller:'actor',dice:[{sides:20,count:1}],modifier:2,mode:'Disadvantage',visibility:'Public',reason:'Initiative'};
    vi.mocked(tableApi.view).mockResolvedValue(view);
    localStorage.setItem(SELECTION_KEY,JSON.stringify({campaignId:'campaign',playerId:'player'}));
    vi.mocked(tableApi.action).mockRejectedValue('Connection interrupted');
    const mounted=render(TableApp);
    await waitFor(()=>expect(screen.getByRole('button',{name:'Report these faces'}).closest('fieldset')?.hasAttribute('disabled')).toBe(false));
    await user.type(screen.getByLabelText('Die 1 · d20'),'17');await user.type(screen.getByLabelText('Die 2 · d20'),'4');
    await user.click(screen.getByRole('button',{name:'Report these faces'}));
    await screen.findByRole('alert');
    const saved=JSON.parse(localStorage.getItem(REQUEST_KEY)!);
    expect(saved.request.action).toEqual({Tactical:{action:{SubmitRoll:{result:{request_id:'initiative-roll',source:'Physical',dice:[{sides:20,value:17},{sides:20,value:4}]}}}}});
    mounted.unmount();render(TableApp);
    await waitFor(()=>expect(screen.getByRole('button',{name:'Retry original request'}).hasAttribute('disabled')).toBe(false));
    await user.click(screen.getByRole('button',{name:'Retry original request'}));
    expect(tableApi.action).toHaveBeenLastCalledWith(saved.request);
  });
  it('retries equipment preparation after restart with the original command and every item identity',async()=>{
    const user=userEvent.setup();
    const view=emptyView();
    view.characters=[{
      character_id:'character',entity_id:'actor',player_id:'player',name:'River',details:null,second_wind_remaining:2,
      equipment:{prepared:false,initial_item_count:3,items:[],worn_armor:null,shield:null,hands:{hands:['Free','Free']}},
      sheet:{Character:{ability_modifiers:[3,2,1,-1,0,1],proficiency_bonus:2,armor_class:12,hp:11,max_hp:11,temporary_hp:0,spell_save_dc:null}},
      profile:{entity_id:'actor',name:'River',pronouns:'they/them',description:'A traveler',alignment:'Neutral Good',backstory:'',
        class_id:'fighter',species_id:'human',background_id:'soldier',level:1,experience_points:0,size:'Medium',speed_feet:30,
        base_ability_scores:[15,14,13,8,10,12],background_boosts:[2,0,1,0,0,0],fighter_skills:['Perception','Survival'],human_skill:'Insight',skilled_skills:['Acrobatics','Stealth','Investigation'],
        languages:['common','dwarvish','elvish'],fighting_style:'Defense',features:[],masteries:['club','dagger','shortbow'],tool_proficiencies:['dice'],armor_training:['light'],weapon_proficiencies:['simple','martial'],
        equipment:[{item_id:'dagger',display_name:'Dagger',quantity:2,unit_cost_cp:200,source_page:90},{item_id:'arrows',display_name:'Arrows',quantity:20,unit_cost_cp:5,source_page:96}],worn_armor:null,shield:false,money_cp:20000},
    }];
    vi.mocked(tableApi.view).mockResolvedValue(view);
    localStorage.setItem(SELECTION_KEY,JSON.stringify({campaignId:'campaign',playerId:null}));
    vi.mocked(tableApi.action).mockRejectedValueOnce('Connection interrupted').mockImplementationOnce(async received => ({command_id:received.command_id,revision:'accepted-revision',outcome:{message:'Equipment ready.'}}));
    const first=render(TableApp);
    await user.click(await screen.findByText('Equipment and features'));
    await waitFor(()=>expect(screen.getByRole('button',{name:"Prepare River's equipment"}).hasAttribute('disabled')).toBe(false));
    await user.click(screen.getByRole('button',{name:"Prepare River's equipment"}));
    await screen.findByRole('alert');
    const request=JSON.parse(localStorage.getItem(REQUEST_KEY)!);
    expect(request.request.action.PrepareEquipment.character_id).toBe('character');
    const itemIds=request.request.action.PrepareEquipment.item_ids;
    expect(itemIds).toHaveLength(3);
    expect(new Set(itemIds).size).toBe(3);
    expect(request.request.channel).toBe('Host');
    expect(request.request.revision).toBe(view.revision);
    first.unmount();render(TableApp);
    await waitFor(()=>expect(screen.getByRole('button',{name:'Retry original request'}).hasAttribute('disabled')).toBe(false));
    await user.click(screen.getByRole('button',{name:'Retry original request'}));
    await waitFor(()=>expect(localStorage.getItem(REQUEST_KEY)).toBeNull());
    expect(tableApi.action).toHaveBeenCalledTimes(2);
    expect(tableApi.action).toHaveBeenNthCalledWith(1,request.request);
    expect(tableApi.action).toHaveBeenNthCalledWith(2,request.request);
  });
  it('retries exactly the saved nonce, head and channel after restart, then clears only on success',async()=>{
    const user=userEvent.setup();
    const request: UnconfirmedRequest={kind:'action',request:{command_id:'same-command',campaign_id:'campaign',expected_event_sequence:4,session_id:null,channel:'Host',action:{AddPlayer:{id:'same-player',name:'Sam'}}}};
    localStorage.setItem(REQUEST_KEY,JSON.stringify(request));
    vi.mocked(tableApi.action).mockImplementation(async received => { expect(received).toEqual(request.request); expect(JSON.parse(localStorage.getItem(REQUEST_KEY)!)).toEqual(request); return {command_id:'same-command',revision:'accepted-revision',outcome:{message:'Sam joined.'}}; });
    render(TableApp);
    await waitFor(()=>expect(screen.getByRole('button',{name:'Retry original request'}).hasAttribute('disabled')).toBe(false));
    await user.click(screen.getByRole('button',{name:'Retry original request'}));
    await waitFor(()=>expect(localStorage.getItem(REQUEST_KEY)).toBeNull());
    expect(tableApi.action).toHaveBeenCalledOnce();
    expect(screen.getByText('Sam joined.')).toBeTruthy();
  });
  it('keeps an uncertain failed request and does not regenerate identity',async()=>{
    const user=userEvent.setup(); const request:UnconfirmedRequest={kind:'action',request:{command_id:'nonce',campaign_id:'campaign',expected_event_sequence:3,session_id:null,channel:'Host',action:{AddPlayer:{id:'player',name:'Sam'}}}};
    localStorage.setItem(REQUEST_KEY,JSON.stringify(request)); vi.mocked(tableApi.action).mockRejectedValue('Connection interrupted');
    render(TableApp); await waitFor(()=>expect(screen.getByRole('button',{name:'Retry original request'}).hasAttribute('disabled')).toBe(false));
    await user.click(screen.getByRole('button',{name:'Retry original request'}));
    await screen.findByRole('alert'); expect(JSON.parse(localStorage.getItem(REQUEST_KEY)!)).toEqual(request);
    expect(screen.getByRole('button',{name:'New campaign'}).hasAttribute('disabled')).toBe(true);
  });
  it('requests the selected player projection instead of reusing a host view',async()=>{
    const user=userEvent.setup(); const host=emptyView();host.players=[{id:'player',campaign_id:'campaign',display_name:'Sam'}];
    vi.mocked(tableApi.view).mockResolvedValue(host);localStorage.setItem(SELECTION_KEY,JSON.stringify({campaignId:'campaign',playerId:null}));
    render(TableApp); await screen.findByRole('combobox',{name:'Local viewing and input channel'});
    await waitFor(()=>expect(screen.getByRole('combobox',{name:'Local viewing and input channel'}).hasAttribute('disabled')).toBe(false));
    await user.selectOptions(screen.getByRole('combobox',{name:'Local viewing and input channel'}),'player');
    await waitFor(()=>expect(tableApi.view).toHaveBeenLastCalledWith('campaign',{Player:'player'}));
    expect(screen.queryByRole('button',{name:'Setup and host controls'})).toBeNull();
  });
  it('releases a confirmed host rejection so the player can correct input',async()=>{
    const user=userEvent.setup();const request:UnconfirmedRequest={kind:'action',request:{command_id:'rejected',campaign_id:'campaign',expected_event_sequence:19,session_id:null,channel:'Host',action:{AddPlayer:{id:'player',name:'Sam'}}}};
    localStorage.setItem(REQUEST_KEY,JSON.stringify(request));vi.mocked(tableApi.action).mockRejectedValue({code:'table_action',message:'Correct this input.',retryable:false});
    render(TableApp);await waitFor(()=>expect(screen.getByRole('button',{name:'Retry original request'}).hasAttribute('disabled')).toBe(false));
    await user.click(screen.getByRole('button',{name:'Retry original request'}));
    await screen.findByText('Correct this input.');expect(localStorage.getItem(REQUEST_KEY)).toBeNull();expect(screen.queryByText('A request is awaiting confirmation')).toBeNull();
  });
  it('contains malformed saved input without invoking it or losing campaign navigation',async()=>{
    localStorage.setItem(REQUEST_KEY,JSON.stringify({kind:'action',request:{command_id:'broken',campaign_id:'campaign'}}));
    render(TableApp);await screen.findByText('The saved retry is incomplete or incompatible. Review the saved campaign before clearing this request.');
    await waitFor(()=>expect(tableApi.list).toHaveBeenCalledOnce());
    expect(tableApi.action).not.toHaveBeenCalled();expect(screen.queryByRole('button',{name:'Retry original request'})).toBeNull();
    expect(screen.getByRole('button',{name:'Refresh and release local retry'})).toBeTruthy();
  });
  it('clears private transient answers before changing the selected player',async()=>{
    const user=userEvent.setup();const view=emptyView();
    view.players=[{id:'one',campaign_id:'campaign',display_name:'One'},{id:'two',campaign_id:'campaign',display_name:'Two'}];
    view.characters=[{character_id:'pc',player_id:'one',entity_id:'actor',name:'River',profile:null,sheet:null,details:null,second_wind_remaining:null}];
    view.active_session={session_id:'session',display_name:'Evening',started_at_world:0,participants:[{player_id:'one',character_id:'pc',attendance:'Present'}]};
    vi.mocked(tableApi.view).mockResolvedValue(view);localStorage.setItem(SELECTION_KEY,JSON.stringify({campaignId:'campaign',playerId:'one'}));
    vi.mocked(tableApi.text).mockResolvedValue({Observed:{command_id:'question',revision:'answer-revision',text:'What is my health?',answer:'Private answer for One'}});
    render(TableApp);await waitFor(()=>expect(screen.getByRole('button',{name:'Send to the table'}).closest('fieldset')?.hasAttribute('disabled')).toBe(false));
    await user.type(screen.getByLabelText('Your declaration, question or correction'),'What is my health?');await user.click(screen.getByRole('button',{name:'Send to the table'}));
    await screen.findByText('Private answer for One');await waitFor(()=>expect(screen.getByRole('combobox',{name:'Local viewing and input channel'}).hasAttribute('disabled')).toBe(false));
    await user.selectOptions(screen.getByRole('combobox',{name:'Local viewing and input channel'}),'two');
    await waitFor(()=>expect(tableApi.view).toHaveBeenLastCalledWith('campaign',{Player:'two'}));expect(screen.queryByText('Private answer for One')).toBeNull();
  });
});

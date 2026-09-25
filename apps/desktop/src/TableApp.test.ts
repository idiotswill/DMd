import { render, screen, waitFor } from '@testing-library/svelte';
import userEvent from '@testing-library/user-event';
import { beforeEach, describe, it, expect, vi } from 'vitest';
import TableApp from './TableApp.svelte';
import { contract, emptyView, options } from './components/table-fixtures.test-support';
import { REQUEST_KEY, SELECTION_KEY, tableApi, type UnconfirmedRequest } from './table-api';
vi.mock('./table-api', async (original) => ({ ...await original<typeof import('./table-api')>(), tableApi:{defaults:vi.fn(),list:vi.fn(),create:vi.fn(),view:vi.fn(),options:vi.fn(),situation:vi.fn(),action:vi.fn(),text:vi.fn()} }));

beforeEach(() => { localStorage.clear(); vi.resetAllMocks(); vi.mocked(tableApi.defaults).mockResolvedValue(structuredClone(contract)); vi.mocked(tableApi.list).mockResolvedValue([{id:'campaign',name:'Saved campaign'}]); vi.mocked(tableApi.view).mockResolvedValue(emptyView()); vi.mocked(tableApi.options).mockResolvedValue(options); vi.mocked(tableApi.situation).mockResolvedValue({title:'',description:'',challenges:[]}); });
describe('durable UI retry',()=>{
  it('retains the actual shield form item, hand and original action head through uncertain restart',async()=>{
    const user=userEvent.setup();const view=emptyView();
    view.active_session={session_id:'session',display_name:'Courtyard',started_at_world:0,participants:[]};
    view.tactical={encounter_id:'encounter',phase:'active',round:3,active_actor:'source-actor',battlefield:null,participants:[],observers:[],initiative:[],ties:[],budget:null,continuation:null,may_fail_save:null,legendary_resistance:null,legendary_action:null,combatant_sources:[],shield_options:{actor:'source-actor',donned:null,shields:[{item:'carried-shield',hands:['Right']}]}};
    vi.mocked(tableApi.view).mockResolvedValue(view);
    localStorage.setItem(SELECTION_KEY,JSON.stringify({campaignId:'campaign',playerId:null}));
    vi.mocked(tableApi.action).mockRejectedValueOnce({message:'Delivery uncertain.',retryable:true}).mockImplementationOnce(async request=>({command_id:request.command_id,event_sequence:20,already_accepted:true,outcome:{message:'Encounter action recorded.',mechanics:null}}));
    const first=render(TableApp);
    await waitFor(()=>expect(screen.getByRole('button',{name:'Don shield'}).closest('fieldset')?.hasAttribute('disabled')).toBe(false));
    await user.click(screen.getByRole('button',{name:'Don shield'}));
    await screen.findByRole('alert');
    const saved=JSON.parse(localStorage.getItem(REQUEST_KEY)!);
    expect(saved.request.action).toEqual({Tactical:{action:{DonShield:{shield:'carried-shield',hand:'Right'}}}});
    expect(saved.request.channel).toBe('Host');expect(saved.request.session_id).toBe('session');expect(saved.request.expected_event_sequence).toBe(view.event_sequence);
    first.unmount();render(TableApp);
    await waitFor(()=>expect(screen.getByRole('button',{name:'Retry original request'}).hasAttribute('disabled')).toBe(false));
    await user.click(screen.getByRole('button',{name:'Retry original request'}));
    await waitFor(()=>expect(localStorage.getItem(REQUEST_KEY)).toBeNull());
    expect(tableApi.action).toHaveBeenCalledTimes(2);
    expect(tableApi.action).toHaveBeenNthCalledWith(1,saved.request);expect(tableApi.action).toHaveBeenNthCalledWith(2,saved.request);
  });
  it('retries NPC creation after uncertain delivery and restart with the same source and item IDs',async()=>{
    const user=userEvent.setup();
    const view=emptyView();
    view.characters=[{character_id:'pc',entity_id:'actor',player_id:'player',name:'River',profile:null,sheet:null,details:null,second_wind_remaining:null}];
    view.creature_setup={catalog:[{definition_id:'goblin-warrior',name:'Goblin Warrior',sizes:['Small'],additional_languages:0,ammunition_required:true,item_count:5,abilities:['Scimitar','Shortbow'],omitted_features:[]}],creatures:[]};
    vi.mocked(tableApi.view).mockResolvedValue(view);
    localStorage.setItem(SELECTION_KEY,JSON.stringify({campaignId:'campaign',playerId:null}));
    vi.mocked(tableApi.action).mockRejectedValueOnce('Connection interrupted').mockImplementationOnce(async received => ({command_id:received.command_id,event_sequence:20,already_accepted:true,outcome:{message:'Host preparation recorded.',mechanics:null}}));
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
    expect(request.request.expected_event_sequence).toBe(view.event_sequence);
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
    view.tactical={encounter_id:'encounter',phase:'active',round:1,active_actor:'source-caster',battlefield:null,participants:[],observers:[],initiative:[],ties:[],budget:null,continuation:null,may_fail_save:null,legendary_resistance:null,legendary_action:null,combatant_sources:[],casting_options:{actor:'source-caster',unavailable:[],variants:[{choice,label:'Hold Person — source ability',concentration:true,minimum_targets:1,maximum_targets:1,repeated_targets:false,targets:[{actor:'selected-target',label:'Visible traveler'}]}]}};
    vi.mocked(tableApi.view).mockResolvedValue(view);
    localStorage.setItem(SELECTION_KEY,JSON.stringify({campaignId:'campaign',playerId:null}));
    vi.mocked(tableApi.action).mockRejectedValueOnce({message:'Delivery uncertain.',retryable:true}).mockImplementationOnce(async request=>({command_id:request.command_id,event_sequence:20,already_accepted:true,outcome:{message:'Encounter action recorded.',mechanics:null}}));
    const mounted=render(TableApp);
    await waitFor(()=>expect(screen.getByLabelText('Spell and resource').closest('fieldset')?.hasAttribute('disabled')).toBe(false));
    await user.selectOptions(screen.getByLabelText('Spell and resource'),JSON.stringify(choice));
    await user.selectOptions(screen.getByLabelText('Spell target 1'),'selected-target');
    await user.click(screen.getByRole('button',{name:'Cast spell'}));
    await screen.findByRole('alert');
    const saved=JSON.parse(localStorage.getItem(REQUEST_KEY)!);
    expect(saved.request.action).toEqual({Tactical:{action:{CastSpell:{choice,targets:{Entities:['selected-target']}}}}});
    expect(saved.request.channel).toBe('Host');expect(saved.request.session_id).toBe('session');
    expect(saved.request.expected_event_sequence).toBe(view.event_sequence);
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
    vi.mocked(tableApi.action).mockRejectedValueOnce('Connection interrupted').mockResolvedValueOnce({command_id:'npc-command',event_sequence:5,already_accepted:true,outcome:{message:'Host preparation recorded.',mechanics:null}});
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
    vi.mocked(tableApi.action).mockRejectedValueOnce('Connection interrupted').mockImplementationOnce(async received => ({command_id:received.command_id,event_sequence:20,already_accepted:true,outcome:{message:'Equipment ready.',mechanics:null}}));
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
    expect(request.request.expected_event_sequence).toBe(view.event_sequence);
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
    vi.mocked(tableApi.action).mockImplementation(async received => { expect(received).toEqual(request.request); expect(JSON.parse(localStorage.getItem(REQUEST_KEY)!)).toEqual(request); return {command_id:'same-command',event_sequence:5,already_accepted:true,outcome:{message:'Sam joined.',mechanics:null}}; });
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
    vi.mocked(tableApi.text).mockResolvedValue({Observed:{meta:{id:'question',campaign_id:'campaign',session_id:'session',issuer:{Player:'one'},actor:{Entity:'actor'},expected_event_sequence:19},text:'What is my health?',answer:'Private answer for One'}});
    render(TableApp);await waitFor(()=>expect(screen.getByRole('button',{name:'Send to the table'}).closest('fieldset')?.hasAttribute('disabled')).toBe(false));
    await user.type(screen.getByLabelText('Your declaration, question or correction'),'What is my health?');await user.click(screen.getByRole('button',{name:'Send to the table'}));
    await screen.findByText('Private answer for One');await waitFor(()=>expect(screen.getByRole('combobox',{name:'Local viewing and input channel'}).hasAttribute('disabled')).toBe(false));
    await user.selectOptions(screen.getByRole('combobox',{name:'Local viewing and input channel'}),'two');
    await waitFor(()=>expect(tableApi.view).toHaveBeenLastCalledWith('campaign',{Player:'two'}));expect(screen.queryByText('Private answer for One')).toBeNull();
  });
});

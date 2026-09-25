import { render, screen, waitFor } from '@testing-library/svelte';
import userEvent from '@testing-library/user-event';
import { beforeEach, describe, it, expect, vi } from 'vitest';
import TableApp from './TableApp.svelte';
import { contract, emptyView, options } from './components/table-fixtures.test-support';
import { REQUEST_KEY, SELECTION_KEY, tableApi, type UnconfirmedRequest } from './table-api';
vi.mock('./table-api', async (original) => ({ ...await original<typeof import('./table-api')>(), tableApi:{defaults:vi.fn(),list:vi.fn(),create:vi.fn(),view:vi.fn(),options:vi.fn(),situation:vi.fn(),action:vi.fn(),text:vi.fn()} }));

beforeEach(() => { localStorage.clear(); vi.clearAllMocks(); vi.mocked(tableApi.defaults).mockResolvedValue(structuredClone(contract)); vi.mocked(tableApi.list).mockResolvedValue([{id:'campaign',name:'Saved campaign'}]); vi.mocked(tableApi.view).mockResolvedValue(emptyView()); vi.mocked(tableApi.options).mockResolvedValue(options); vi.mocked(tableApi.situation).mockResolvedValue({title:'',description:'',challenges:[]}); });
describe('durable UI retry',()=>{
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

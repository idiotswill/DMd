import {render,screen} from '@testing-library/svelte';
import {expect,it,vi} from 'vitest';
import userEvent from '@testing-library/user-event';
import type {OpportunityView,TacticalView} from '../tactical-api';
import EncounterPanel from './EncounterPanel.svelte';

function opportunity():OpportunityView {
  return {actor:'guard',target:{actor:'runner',label:'Departing creature'},unarmed:true,
    features:[{feature_id:'bite',label:'Bite',weapon:null}],
    weapons:{actor:'guard',hands:{hands:['Free',{Item:'blade'}]},targets:[{actor:'runner',label:'Departing creature'}],weapons:[
      {item:'blade',name:'Dagger',deliveries:['Melee'],abilities:['Strength','Dexterity'],grips:[{OneHand:'Right'}],purposes:['Normal'],ammunition_required:false,ammunition:[]},
    ]}};
}

it('uses the witnessed reaction choice and never borrows ordinary Attack equipment permission',async()=>{
  const user=userEvent.setup();const onAction=vi.fn();
  const tactical:TacticalView={encounter_id:'encounter',round:1,active_actor:'runner',phase:'active',battlefield:null,participants:[],combatant_sources:[],observers:[],initiative:[],ties:[],continuation:null,may_fail_save:null,legendary_resistance:null,legendary_action:null,budget:null,opportunity:opportunity()};
  const component=render(EncounterPanel,{tactical,characters:[],host:false,actor:'guard',player:'player',onAction});
  expect(screen.queryByLabelText('Ready or put away weapon')).toBeNull();
  await user.selectOptions(screen.getByLabelText('Target'),'runner');
  await user.selectOptions(screen.getByLabelText('Attack ability'),'Dexterity');
  await user.click(screen.getByRole('button',{name:/^Attack$/}));
  expect(onAction).toHaveBeenLastCalledWith({OpportunityAttack:{choice:{Weapon:{weapon:'blade',target:'runner',delivery:'Melee',ability:'Dexterity',grip:{OneHand:'Right'},purpose:'Normal',ammunition:null,equipment_change:null}}}});
  await user.click(screen.getByRole('button',{name:'Unarmed Strike (Strength damage)'}));
  expect(onAction).toHaveBeenLastCalledWith({OpportunityAttack:{choice:{UnarmedDamage:{ability:'Strength'}}}});
  await user.click(screen.getByRole('button',{name:'Use Bite'}));
  expect(onAction).toHaveBeenLastCalledWith({OpportunityAttack:{choice:{CreatureFeature:{feature_id:'bite',weapon:null}}}});
  await user.click(screen.getByRole('button',{name:'Let the creature pass'}));
  expect(onAction).toHaveBeenLastCalledWith('DeclineOpportunity');
  await component.rerender({actor:'other',player:'other-player'});
  expect(screen.queryByText('Opportunity attack')).toBeNull();
  expect(screen.queryByText('Departing creature')).toBeNull();
  await component.rerender({actor:'guard',player:'player',tactical:{...tactical,opportunity:null}});
  expect(screen.queryByRole('button',{name:'Let the creature pass'})).toBeNull();
});

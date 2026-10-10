import type { TableView } from './table-api';
import type { AttackOptions, WeaponUseChoice } from './tactical-api';
import { emptyView } from './components/table-fixtures.test-support';

export function heldOptions(): AttackOptions {
  return { actor: 'pc-actor', hands: { hands: [{ Item: 'sword' }, 'Free'] },
    targets: [{ actor: 'guard', label: 'Visible guard' }],
    weapons: [{ item: 'sword', name: 'Greatsword', deliveries: ['Melee'], abilities: ['Strength'],
      grips: ['TwoHands'], purposes: ['Normal'], ammunition_required: false, ammunition: [] }],
  };
}

export const ordinaryChoice = (): WeaponUseChoice => ({ weapon: 'sword', target: 'guard', delivery: 'Melee',
  ability: 'Strength', grip: 'TwoHands', purpose: 'Normal', ammunition: null, equipment_change: null });

export function ownedTurn(): TableView {
  return { ...emptyView(), physical: { version: 5, actors: [], controls: [] },
    players: [{ id: 'player', campaign_id: 'campaign', display_name: 'Sam' }, { id: 'other', campaign_id: 'campaign', display_name: 'Taylor' }],
    characters: [{ character_id: 'pc', entity_id: 'pc-actor', player_id: 'player', name: 'River', profile: null, sheet: null, details: null, second_wind_remaining: null }],
    active_session: { session_id: 'session', display_name: 'Evening', started_at_world: 0, participants: [{ player_id: 'player', character_id: 'pc', attendance: 'Present' }] },
    source_control: { version: 2, actors: [{ actor: 'source', name: 'Owned creature', definition_id: 'warhorse', controller: { Player: 'player' }, hp: 19, max_hp: 19 }] },
    tactical: { encounter_id: 'encounter', phase: 'active', execution: 'EncounterReleaseV1', round: 1, active_actor: 'pc-actor',
      battlefield: null, participants: [], observers: [], initiative: [], ties: [], continuation: null,
      may_fail_save: null, legendary_resistance: null, legendary_action: null,
      combatant_sources: [{ actor: 'pc-actor', source: 'Character', initiative_modifier: 0, normal_mode: 'Normal', surprised_mode: 'Disadvantage' }],
      budget: { movement_spent: 0, attacks_remaining: 0, action_spent: false, bonus_action_spent: false, reaction_available: true },
      attack_options: heldOptions(),
    },
  };
}

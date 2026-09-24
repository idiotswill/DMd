import type { Id } from './table-api';

export interface Point { x: number; y: number; z: number }
export interface Volume { min: Point; max: Point }
export interface Battlefield {
  bounds: Volume; floor_z: number; floor_surface: string; ambient_light: 'Bright' | 'Dim' | 'Darkness';
  terrain: { id: string; volume: Volume; difficult: boolean; observable: boolean; water: boolean; climbable: boolean; burrowable: boolean; supports_top: boolean; surface: string | null; obscuration: 'None' | 'Light' | 'Heavy'; magical_darkness: boolean }[];
  obstacles: { id: string; volume: Volume; blocks_movement: boolean; blocks_sight: boolean; observable: boolean; cover: 'None' | 'Half' | 'ThreeQuarters' | 'Total' }[];
  lights: { id: string; position: Point; bright_radius: number; dim_radius: number; attached_to: Id | null }[];
}
export interface BattlefieldSetup {
  encounter_id: Id; scene_id: Id; location_id: Id; name: string; battlefield: Battlefield;
  characters: { character_id: Id; position: Point; height: number; allies: Id[]; enemies: Id[] }[];
  creatures: { actor: Id; public_label: string; position: Point; height: number; allies: Id[]; enemies: Id[] }[];
  geometry_ruling: { basis: 'GmAdjudication'; reason: string };
}
export type TacticalAction =
  | 'EndTurn' | 'Disengage' | 'Dodge' | 'StandProne' | 'StartAttackAction' | 'VoluntarilyFailSave'
  | 'UseLegendaryResistance' | 'DeclineLegendaryResistance' | 'DeclineLegendaryAction'
  | { Dash: { speed: 'Speed'|'Climb'|'Swim'|'Fly'|'Burrow' } } | { ChooseTurnWork: { occurrence: number } }
  | { Begin: { combatants: { actor: Id; source: 'Character' | { Creature: { definition_id: string } }; surprised: boolean }[]; groups: { actors: Id[]; request_id: Id }[] } }
  | { SubmitRoll: { result: { request_id: Id; source: 'Physical'; dice: { sides: number; value: number }[] } } }
  | { ProposeInitiativeTie: { order: Id[] } } | { AcceptInitiativeTie: { total: number } };
export interface InitiativeTie { total: number; actors: Id[]; proposed_order: Id[] | null; accepted_by: Id[]; host_decided: boolean }
export interface TacticalView {
  encounter_id: Id; round: number | null; active_actor: Id | null; phase: string;
  battlefield: Battlefield | null;
  participants: { entity_id: Id; public_label: string; position: Point; size: string }[];
  combatant_sources: { actor: Id; source: 'Character' | { Creature: { definition_id: string } }; initiative_modifier: number; normal_mode: string; surprised_mode: string }[];
  observers: { observer: Id; position: Point | null; contacts: { entity_id: Id; label: string | null; position: Point; status: 'Seen' | 'Located' | 'Remembered'; modality: string }[]; cells: { position: Point; difficult: boolean; blocked: boolean; currently_seen: boolean }[] }[];
  initiative: { actor: Id; label: string; total: number | null }[];
  ties: InitiativeTie[];
  continuation: { actor: Id; host_adjudication: boolean; choices: { occurrence: number; label: string }[] } | null;
  may_fail_save: Id | null;
  legendary_resistance: Id | null;
  legendary_action: Id | null;
  budget: { movement_spent: number; attacks_remaining: number; action_spent: boolean; bonus_action_spent: boolean; reaction_available: boolean } | null;
}

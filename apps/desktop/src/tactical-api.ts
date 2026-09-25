import type { Ability, Id } from './table-api';

export type Hand = 'Left' | 'Right';
export type WeaponGrip = 'TwoHands' | { OneHand: Hand };
export type WeaponDelivery = 'Melee' | 'Thrown' | 'Shot';
export type WeaponAttackPurpose = 'Normal' | { LightBonus: { trigger: Id } } | { Nick: { trigger: Id } } | { Cleave: { trigger: Id } };
export interface WeaponUseChoice {
  weapon: Id; target: Id; delivery: WeaponDelivery; ability: Ability; grip: WeaponGrip;
  purpose: WeaponAttackPurpose;
  ammunition: Id | null;
  equipment_change: { timing: 'BeforeAttack' | 'AfterAttack'; operation: { Equip: { item: Id; hand: Hand } } | { Unequip: { item: Id } } } | null;
}
export interface AttackOptions {
  actor: Id; hands: { hands: ('Free' | { Item: Id })[] };
  weapons: { item: Id; name: string; deliveries: WeaponDelivery[]; abilities: Ability[]; grips: WeaponGrip[]; purposes: WeaponAttackPurpose[]; ammunition_required: boolean; ammunition: { id: Id; name: string; quantity: number }[]; source_features?: { feature_id: Id; label: string; weapon: Id|null }[] }[];
  targets: { actor: Id; label: string }[];
}
export interface SpellCastChoice {
  actor: Id; spell_id: string;
  grant: 'Prepared' | { CreatureFeature: { feature_id: string } };
  resource: 'Cantrip' | 'SourceFeature' | { Slot: { level: number } };
  material: 'None' | { Material: { item: Id } } | { Focus: { item: Id } } | { ComponentPouch: { item: Id } };
  mode: 'Immediate';
}
export interface CastingVariant {
  choice: SpellCastChoice; label: string; concentration: boolean;
  minimum_targets: number; maximum_targets: number; repeated_targets: boolean;
  targets: { actor: Id; label: string }[];
}
export interface CastingOptions { actor: Id; variants: CastingVariant[]; unavailable: string[] }
export interface ShieldOptions { actor: Id; donned: Id|null; shields: { item: Id; hands: Hand[] }[] }
export type ReactionUnlistedOrder = 'BeforeForward'|'BeforeReverse'|'AfterForward'|'AfterReverse';
export interface ReactionOrdering { ranked: Id[]; unlisted: ReactionUnlistedOrder }
export type HitDecision = { Order: { instruction: ReactionOrdering } } | 'Delegate'
  | { Respond: { accept: boolean } } | { Cast: { choice: SpellCastChoice } } | 'Decline';
export interface HitView {
  order: { key: Id; actor: Id; participants: { actor: Id; label: string }[] } | null;
  delegate: Id | null;
  response: { key: Id; actor: Id; selected: boolean; shield: SpellCastChoice[] } | null;
}

export interface AreaOptions {
  actor: Id; controller: Id | null; source_space: Volume;
  variants: { feature_id: string; label: string; length_feet: number }[]; unavailable: string[];
}
export interface Point { x: number; y: number; z: number }
export type MovementMode = 'Walk' | 'Crawl' | 'Climb' | 'Swim' | 'Fly' | 'Burrow' | 'Jump';
export interface MoveStep { destination: Point; mode: MovementMode }
export interface MovementOptions { actor: Id; position: Point; grid_units: number; modes: MovementMode[] }
export type MeleeChoice = { Weapon: WeaponUseChoice } | { UnarmedDamage: { ability: Ability } } | { CreatureFeature: { feature_id: string; weapon: Id | null } };
export interface OpportunityView { actor: Id; target: { actor: Id; label: string }; weapons: AttackOptions | null; unarmed: boolean; features: { feature_id: string; label: string; weapon: Id | null }[] }
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
  area_grid_policy?: 'OccupiedCellCentersV1' | null;
  geometry_ruling: { basis: 'GmAdjudication'; reason: string };
}
export interface SavageAttackerRoll {
  weapon_dice: number;
  first: { request_id: Id; source: 'Physical'; dice: { sides: number; value: number }[] };
  second: { request_id: Id; source: 'Physical'; dice: { sides: number; value: number }[] };
  chosen: 'First' | 'Second';
  inspiration: { roll: 'First' | 'Second'; die_index: number; replacement: { sides: number; value: number } } | null;
}
export type TacticalAction =
  | 'UpgradeExecution'
  | { UpgradeExecutionTo: { execution: 'ShieldHitV1' } }
  | { HitResponse: { handle: Id; decision: HitDecision } }
  | { AbandonReady: { actor: Id } }
  | { UnarmedStrike: { target: Id } }
  | { FirstAid: { target: Id; purpose: 'Stabilize' | 'EndKnockout' } }
  | { SubmitSavageAttacker: { roll: SavageAttackerRoll } }
  | 'SecondWind'
  | { DonShield: { shield: Id; hand: Hand } } | 'DoffShield'
  | { CreatureWeaponAttack: { feature_id: string; choice: Omit<WeaponUseChoice,'delivery'|'ability'|'purpose'> } }
  | 'EndTurn' | 'Disengage' | 'Dodge' | 'StandProne' | 'StartAttackAction' | 'VoluntarilyFailSave'
  | 'UseLegendaryResistance' | 'DeclineLegendaryResistance' | 'DeclineLegendaryAction'
  | { Attack: { choice: WeaponUseChoice } }
  | { CreatureArea: { feature_id: string; aim: { origin: Point; toward: Point; include_origin: boolean }; ordering: 'Host' | 'DelegateToHost' } }
  | { CastSpell: { choice: SpellCastChoice; targets: { Entities: Id[] } } }
  | { Move: { path: MoveStep[] } }
  | 'DeclineOpportunity' | { OpportunityAttack: { choice: MeleeChoice } }
  | { ChooseLiquidLanding: { choice: 'Athletics' | 'Acrobatics' | null } }
  | { ChooseAttackKnockout: { choice: 'NormalDamage' | 'KnockOut' } }
  | { ChooseAttackMastery: { choice: 'Decline' | 'Graze' } }
  | { Dash: { speed: 'Speed'|'Climb'|'Swim'|'Fly'|'Burrow' } } | { ChooseTurnWork: { handle: Id } }
  | { Begin: { execution: 'ShieldHitV1'; combatants: { actor: Id; source: 'Character' | { Creature: { definition_id: string } }; surprised: boolean }[]; groups: { actors: Id[]; request_id: Id }[] } }
  | { SubmitRoll: { result: { request_id: Id; source: 'Physical'; dice: { sides: number; value: number }[] } } }
  | { ProposeInitiativeTie: { order: Id[] } } | { AcceptInitiativeTie: { total: number } };
export interface InitiativeTie { total: number; actors: Id[]; proposed_order: Id[] | null; accepted_by: Id[]; host_decided: boolean }
export interface TacticalView {
  execution?: 'ReactionsV1' | 'ShieldHitV1' | null;
  hit?: HitView | null;
  ready?: { actor: Id; action: string; may_abandon: boolean }[];
  encounter_id: Id; round: number | null; active_actor: Id | null; phase: string;
  battlefield: Battlefield | null;
  participants: { entity_id: Id; public_label: string; position: Point; size: string }[];
  combatant_sources: { actor: Id; source: 'Character' | { Creature: { definition_id: string } }; initiative_modifier: number; normal_mode: string; surprised_mode: string }[];
  observers: { observer: Id; position: Point | null; contacts: { entity_id: Id; label: string | null; position: Point; status: 'Seen' | 'Located' | 'Remembered'; modality: string }[]; cells: { position: Point; difficult: boolean; blocked: boolean; currently_seen: boolean }[] }[];
  initiative: { actor: Id; label: string; total: number | null }[];
  ties: InitiativeTie[];
  continuation: { actor: Id; host_adjudication: boolean; choices: { handle: Id; label: string }[] } | null;
  may_fail_save: Id | null;
  legendary_resistance: Id | null;
  legendary_action: Id | null;
  attack_options?: AttackOptions | null;
  casting_options?: CastingOptions | null;
  area_options?: AreaOptions | null;
  movement_options?: MovementOptions | null;
  opportunity?: OpportunityView | null;
  liquid_landing?: { actor: Id } | null;
  shield_options?: ShieldOptions | null;
  attack_decision?: { actor: Id; kind: 'Knockout' | 'Graze' } | null;
  budget: { movement_spent: number; attacks_remaining: number; action_spent: boolean; bonus_action_spent: boolean; reaction_available: boolean } | null;
}

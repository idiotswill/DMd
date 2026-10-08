import { invoke } from '@tauri-apps/api/core';
import type { BattlefieldSetup, TacticalAction, TacticalView } from './tactical-api';

export type Id = string;
export const ABILITIES = ['Strength', 'Dexterity', 'Constitution', 'Intelligence', 'Wisdom', 'Charisma'] as const;
export type Ability = typeof ABILITIES[number];
export const SKILLS = ['Acrobatics', 'AnimalHandling', 'Arcana', 'Athletics', 'Deception', 'History', 'Insight', 'Intimidation', 'Investigation', 'Medicine', 'Nature', 'Perception', 'Performance', 'Persuasion', 'Religion', 'SleightOfHand', 'Stealth', 'Survival'] as const;
export type Skill = typeof SKILLS[number];
export type Viewer = 'Host' | { Player: Id };
export interface VersionedRef { id: string; version: string }
export interface TableContract {
  tone: string; humor: string; tactical_preference: string; exploration_preference: string;
  social_preference: string; lethality: string; ruleset: VersionedRef; permitted_content: VersionedRef[];
  advancement: string; optional_rules: string[]; house_rules: { ability_test_natural_extremes: boolean };
  house_rule_notes: string; pvp_policy: string; theft_and_secrets: string; retcon_policy: string;
  content_boundaries: string; explanation_depth: string; experience: string; absent_player_policy: string;
}
export interface Player { id: Id; campaign_id: Id; display_name: string }
export interface Participant { player_id: Id; character_id: Id | null; attendance: 'Present' | 'Absent' }
export interface EquipmentChoice { item_id: string; quantity: number }
export interface CharacterInput {
  name: string; pronouns: string; description: string; alignment: string; backstory: string;
  ability_scores: number[]; background_boosts: number[]; fighter_skills: Skill[]; human_skill: Skill;
  skilled_skills: Skill[]; size: 'Small' | 'Medium'; languages: string[];
  fighting_style: 'Defense' | 'Archery'; gaming_set: 'Dice' | 'Dragonchess' | 'PlayingCards' | 'ThreeDragonAnte';
  purchases: EquipmentChoice[]; worn_armor: string | null; shield: boolean; masteries: string[];
}
export interface CharacterProfile {
  creation_source?: CharacterCreationSourcePin;
  entity_id: Id; name: string; pronouns: string; description: string; alignment: string; backstory: string;
  class_id: string; species_id: string; background_id: string; level: number; experience_points: number;
  size: string; speed_feet: number; base_ability_scores: number[]; background_boosts: number[];
  fighter_skills: Skill[]; human_skill: Skill; skilled_skills: Skill[]; languages: string[]; fighting_style: string;
  features: { id: string; source_page: number; execution_gate: number; scope: string }[];
  masteries: string[]; tool_proficiencies: string[]; armor_training: string[]; weapon_proficiencies: string[];
  equipment: { item_id: string; display_name: string; quantity: number; unit_cost_cp: number; source_page: number }[];
  worn_armor: string | null; shield: boolean; money_cp: number;
}
export interface CharacterSheet { ability_modifiers: number[]; proficiency_bonus: number; armor_class: number; hp: number; max_hp: number; temporary_hp: number; spell_save_dc: number | null }
export interface SheetDetails { ability_scores: number[]; hit_dice: { sides: number; maximum: number; remaining: number }; heroic_inspiration: boolean; saving_throws: { ability: Ability; modifier: number; proficient: boolean }[]; skills: { skill: Skill; ability: Ability; modifier: number; proficiency: 'Proficient' | 'Expertise' | null }[]; conditions: string[]; exhaustion: number; death: { successes: number; failures: number; stable: boolean; dead: boolean } }
export interface EquipmentView { prepared: boolean; initial_item_count: number; items: { id: Id; name: string; quantity: number }[]; worn_armor: Id | null; shield: Id | null; hands: { hands: ('Free' | { Item: Id })[] } }
export interface CharacterView { character_id: Id; player_id: Id | null; entity_id: Id; name: string; profile: CharacterProfile | null; sheet: { Character: CharacterSheet } | null; details: SheetDetails | null; second_wind_remaining: number | null; equipment?: EquipmentView | null }
export interface CharacterCreationSourcePin {
  ruleset_id: string; ruleset_version: string; catalog_schema_version: number;
  profile_id: string; definition_fingerprint: string;
}
export interface CreationOptions {
  source: CharacterCreationSourcePin;
  catalog: { schema_version: number; ruleset_id: string; version: string; profile_id: string; starting_money_cp: number; source_pages: number[]; scope: string; items: { id: string; name: string; unit_cost_cp: number; purchase_multiple: number; source_page: number; weapon: boolean }[] };
  fighter_skills: Skill[]; fighter_masteries: string[]; standard_languages: string[]; alignments: string[];
}
export type CreatureSize = 'Tiny' | 'Small' | 'Medium' | 'Large' | 'Huge' | 'Gargantuan';
export interface CreatureSourcePin { ruleset_id: string; ruleset_version: string; definition_id: string; definition_fingerprint: string }
export interface CreatureCreation { entity_id: Id; name: string; definition_id: string; source?: CreatureSourcePin; size: CreatureSize; additional_languages: string[]; ammunition_units: number; item_ids: Id[] }
export interface CreatureOption { definition_id: string; source?: CreatureSourcePin; execution_limits?: string[]; name: string; sizes: CreatureSize[]; additional_languages: number; ammunition_required: boolean; item_count: number; abilities: string[]; omitted_features: string[] }
export interface CreatureView { actor: Id; name: string; definition_id: string; size: CreatureSize; hp: number; max_hp: number }
export interface CreatureSetupView { catalog: CreatureOption[]; creatures: CreatureView[] }
export type CreatureController = 'Autonomous' | 'Host' | { Player: Id };
export interface ControlledSourceActor { actor: Id; name: string; definition_id: string; controller: CreatureController; hp: number; max_hp: number }
export interface SourceAdoption { actor: Id; player_id: Id; source: { ruleset_id: string; ruleset_version: string; definition_id: string; definition_fingerprint: string }; profile_origin: Record<string, unknown>; control_origin: Record<string, unknown> }
export interface SourceControlView { version: 2; actors: ControlledSourceActor[] }
export interface SourceControlOptions { enabled: boolean; settled: boolean; adopted: SourceAdoption[]; actors: ControlledSourceActor[] }
export interface Challenge { id: string; title: string; description: string; phrases: string[]; kind: { Check: { ability: Ability; skill: Skill | null } }; dc: number; success: string; failure: string; resolution: { actor: Id; request_id: Id; success: boolean; total: number } | null }
export interface Situation { title: string; description: string; challenges: Challenge[] }
export interface PendingDecision { id: Id; revision: number; player_id: Id; character_id: Id; actor: Id; session_id: Id; text: string; intent: 'SecondWind' | { Check: { kind: { Check: { ability: Ability; skill: Skill | null } }; goal: string; challenge_id: string | null } } | { Unresolved: { question: string } } }
export interface RollRequest { id: Id; roller: Id | null; dice: { sides: number; count: number }[]; modifier: number; mode: 'Normal' | 'Advantage' | 'Disadvantage'; visibility: string; reason: string }
export interface PhysicalLoad { items: { item: Id; name: string; quantity: number; pounds: string | null; basis: string }[]; known_pounds: string; complete: boolean; unresolved: string[]; body_and_load_pounds: string | null }
export interface PhysicalControl { key: Id; label: string; kind: string; source: string | null; conditions: string[]; unit: boolean; gross: boolean; wallet_cp: number | null }
export interface PhysicalView { version: 5; actors: { actor: Id; name: string; body_pounds: string | null; load: PhysicalLoad | null }[]; controls: PhysicalControl[] }
export type PhysicalItemInput = 'Unknown' | { Catalog: { condition: string | null } } | { Unit: { pounds: string; condition: string | null } } | { Gross: { pounds: string } } | { NonstandardUnit: { description: string; pounds: string } };
export type PhysicalFactInput = 'Enable' | { Body: { pounds: string | null; reason: string } } | { Item: { choice: PhysicalItemInput; reason: string } } | { Coverage: { complete: boolean; reason: string } } | { PersonalItem: { name: string; pounds: string; separate_from_listed: boolean; reason: string } } | { Currency: { coins: { cp: number; sp: number; ep: number; gp: number; pp: number }; reason: string } };
export interface TableView {
  physical?: PhysicalView;
  inspiration_transfer?: { character_id: Id; choices: { key: Id; label: string }[] };
  grapple?: { version: 3 | 4; choices: { key: Id; actor: Id; label: string }[]; ground_drag?: { key: Id; actor: Id; label: string }[] };
  source_control?: SourceControlView;
  campaign_id: Id; name: string; revision: Id; diagnostics?: { canonical_event_sequence: number }; contract: TableContract; players: Player[]; characters: CharacterView[];
  active_session: { session_id: Id; display_name: string; started_at_world: number; participants: Participant[] } | null;
  pending: PendingDecision | null; roll: RollRequest | null; situation_title: string; situation_description: string;
  roll_channel: 'Table' | 'Tactical' | null;
  tactical?: TacticalView | null;
  creature_setup?: CreatureSetupView | null;
  transcript: { id: string; kind: string; speaker: string; text: string }[]; recap: string[];
}
export type TableAction =
  | 'EnablePhysicalFacts'
  | { PhysicalFact: { handle: Id; input: PhysicalFactInput } }
  | { AwardExcessInspiration: { character_id: Id; reason: string } }
  | { InspirationTransfer: { handle: Id } }
  | { AwardHeroicInspiration: { character_id: Id; reason: string } }
  | 'EnableGrappleAccess'
  | 'EnableGrappleTransport'
  | { UpdateContract: { contract: TableContract } } | { AddPlayer: { id: Id; name: string } }
  | { CreateCharacter: { character_id: Id; entity_id: Id; player_id: Id; input: CharacterInput } }
  | { CreateCharacterFromSource: { character_id: Id; entity_id: Id; player_id: Id; source: CharacterCreationSourcePin; input: CharacterInput } }
  | { PrepareEquipment: { character_id: Id; item_ids: Id[] } }
  | { CreateCreature: { creation: CreatureCreation } }
  | { EnableSourceActorAccess: { adopted: SourceAdoption[] } }
  | { SetSourceCreatureController: { actor: Id; controller: CreatureController } }
  | { PrepareBattlefield: { setup: BattlefieldSetup } } | { Tactical: { action: TacticalAction } }
  | { StartSession: { id: Id; name: string; participants: Participant[] } } | 'EndSession'
  | { SetSituation: { situation: Situation } }
  | { CancelDecision: { pending_id: Id; revision: number } }
  | { Adjudicate: { pending_id: Id; revision: number; request_id: Id } }
  | { SubmitPhysical: { request_id: Id; faces: number[] } };
export interface Receipt { command_id: Id; revision?: Id; outcome: { message: string } }
export type TextResult = { Accepted: Receipt } | { Observed: { command_id: Id; revision?: Id; text: string; answer: string } };
export type LocalChannel = 'Host' | { Player: { player_id: Id; character_id: Id } } | { SourceCreature: { player_id: Id; actor: Id } };
interface ChannelContext { command_id: Id; campaign_id: Id; session_id: Id | null; channel: LocalChannel }
export interface RequestContext extends ChannelContext { version: 1 | 2 | 3 | 4 | 5; revision: Id }
export interface LegacyRequestContext extends ChannelContext { expected_event_sequence: number }
type SavedContext = RequestContext | LegacyRequestContext;
export type UnconfirmedRequest = { kind: 'action'; request: SavedContext & { action: TableAction } } | { kind: 'text'; request: SavedContext & { text: string } } | { kind: 'create'; request: { id: Id; name: string; contract: TableContract } };export interface Selection { campaignId: Id | null; playerId: Id | null; sourceActorId?: Id | null }
export function channelPlayer(channel: LocalChannel): Id | null { return channel === 'Host' ? null : 'Player' in channel ? channel.Player.player_id : channel.SourceCreature.player_id; }
export const REQUEST_KEY = 'dmd.ui.unconfirmed.v1';
export const SELECTION_KEY = 'dmd.ui.selection.v1';
export function newId(): Id { return crypto.randomUUID(); }
export function label(value: string): string { return value.replace(/([a-z])([A-Z])/g, '$1 $2').replaceAll('-', ' ').replace(/^./, c => c.toUpperCase()); }
export function money(cp: number): string { return `${(cp / 100).toFixed(2)} GP`; }
export function signed(value: number): string { return value >= 0 ? `+${value}` : String(value); }
export function rawDice(request: RollRequest): number[] { return request.dice.flatMap(die => Array.from({ length: request.mode === 'Normal' ? die.count : 2 }, () => die.sides)); }
export function saveRequest(request: UnconfirmedRequest): void {
  try { localStorage.setItem(REQUEST_KEY, JSON.stringify(request)); }
  catch { throw new Error('The window could not retain this request, so no action was sent. Free local storage or reopen DMd, then try again.'); }
}
export function clearRequest(): void { localStorage.removeItem(REQUEST_KEY); }
function object(value: unknown): value is Record<string, unknown> { return typeof value === 'object' && value !== null && !Array.isArray(value); }
function id(value: unknown): value is string { return typeof value === 'string' && value.trim().length > 0; }
function validPhysicalInput(value: unknown): boolean {
  const keys = (v: Record<string, unknown>, expected: string[]) => Object.keys(v).length === expected.length && expected.every(k => k in v);
  if (!object(value) || Object.keys(value).length !== 1) return false;
  const [kind, payload] = Object.entries(value)[0];
  if (!object(payload) || typeof payload.reason !== 'string' || !payload.reason.trim() || new TextEncoder().encode(payload.reason).length > 1000) return false;
  const pounds = (v: unknown) => typeof v === 'string' && v.length <= 27 && /^(0|[1-9][0-9]*)(\.[0-9]{1,6})?$/.test(v) && /[1-9]/.test(v);
  if (kind === 'Body') return keys(payload, ['pounds','reason']) && (payload.pounds === null || pounds(payload.pounds));
  if (kind === 'Coverage') return keys(payload, ['complete','reason']) && typeof payload.complete === 'boolean';
  if (kind === 'PersonalItem') return keys(payload, ['name','pounds','separate_from_listed','reason']) && typeof payload.name === 'string' && pounds(payload.pounds) && typeof payload.separate_from_listed === 'boolean';
  if (kind === 'Currency') {
    const coins = payload.coins;
    return keys(payload, ['coins','reason']) && object(coins) && keys(coins, ['cp','sp','ep','gp','pp']) && ['cp','sp','ep','gp','pp'].every(k => typeof coins[k] === 'number' && Number.isInteger(coins[k]) && coins[k] >= 0 && coins[k] <= 4294967295);
  }
  if (kind !== 'Item' || !keys(payload, ['choice','reason'])) return false;
  if (payload.choice === 'Unknown') return true;
  if (!object(payload.choice) || Object.keys(payload.choice).length !== 1) return false;
  const [choice, fields] = Object.entries(payload.choice)[0];
  if (!object(fields)) return false;
  if (choice === 'Catalog') return keys(fields, ['condition']) && (fields.condition === null || typeof fields.condition === 'string');
  if (choice === 'Unit') return keys(fields, ['pounds','condition']) && pounds(fields.pounds) && (fields.condition === null || typeof fields.condition === 'string');
  if (choice === 'NonstandardUnit') return keys(fields, ['description','pounds']) && typeof fields.description === 'string' && fields.description.trim().length > 0 && pounds(fields.pounds);
  return choice === 'Gross' && keys(fields, ['pounds']) && pounds(fields.pounds);
}
function validSavedRequest(value: unknown): value is UnconfirmedRequest {
  if (!object(value) || !object(value.request)) return false;
  const request = value.request;
  if (value.kind === 'create') return id(request.id) && id(request.name) && object(request.contract) && object(request.contract.ruleset);
  if (!['action', 'text'].includes(String(value.kind)) || !id(request.command_id) || !id(request.campaign_id)
    || !(request.session_id === null || id(request.session_id))) return false;
  if ('revision' in request) {
    if ((request.version !== 1 && request.version !== 2 && request.version !== 3 && request.version !== 4 && request.version !== 5) || !id(request.revision) || 'expected_event_sequence' in request) return false;
  } else if ('version' in request || !Number.isSafeInteger(request.expected_event_sequence) || Number(request.expected_event_sequence) < 0) return false;
  const channel = request.channel;
  const pcChannel = object(channel) && Object.keys(channel).length === 1 && object(channel.Player) && id(channel.Player.player_id) && id(channel.Player.character_id);
  const sourceChannel = (request.version === 2 || request.version === 3 || request.version === 4 || request.version === 5) && object(channel) && Object.keys(channel).length === 1 && object(channel.SourceCreature) && id(channel.SourceCreature.player_id) && id(channel.SourceCreature.actor);
  if (channel !== 'Host' && !pcChannel && !sourceChannel) return false;
  if (value.kind === 'text') return pcChannel && typeof request.text === 'string' && request.text.trim().length > 0 && request.text.length <= 8000;
  if (request.action === 'EndSession') return !sourceChannel;
  if (request.action === 'EnablePhysicalFacts') return request.version === 5 && channel === 'Host';
  if (request.action === 'EnableGrappleTransport') return (request.version === 4 || request.version === 5) && channel === 'Host';
  if (request.action === 'EnableGrappleAccess') return (request.version === 3 || request.version === 5) && channel === 'Host';
  if (!object(request.action) || Object.keys(request.action).length !== 1) return false;
  const [kind, payload] = Object.entries(request.action)[0];
  if (kind === 'PhysicalFact') return request.version === 5 && channel === 'Host' && object(payload) && Object.keys(payload).length === 2 && id(payload.handle) && validPhysicalInput(payload.input);
  if (kind === 'InspirationTransfer') return (request.version === 4 || request.version === 5) && pcChannel && id(request.session_id)
    && object(payload) && Object.keys(payload).length === 1 && id(payload.handle);
  if (kind === 'AwardExcessInspiration') return (request.version === 4 || request.version === 5) && channel === 'Host' && id(request.session_id)
    && object(payload) && Object.keys(payload).length === 2 && id(payload.character_id)
    && typeof payload.reason === 'string' && payload.reason.trim().length > 0 && new TextEncoder().encode(payload.reason).length <= 2000;
  if (kind === 'AwardHeroicInspiration') return (request.version === 4 || request.version === 5) && channel === 'Host'
    && object(payload) && Object.keys(payload).length === 2
    && id(payload.character_id) && typeof payload.reason === 'string' && payload.reason.trim().length > 0;
  if (['EnableSourceActorAccess','SetSourceCreatureController'].includes(kind)) return (request.version === 2 || request.version === 3 || request.version === 4 || request.version === 5) && channel === 'Host' && object(payload);
  if (sourceChannel && kind !== 'Tactical') return false;
  return ['UpdateContract','AddPlayer','CreateCharacter','CreateCharacterFromSource','CreateCreature','PrepareEquipment','PrepareBattlefield','Tactical','StartSession','SetSituation','CancelDecision','Adjudicate','SubmitPhysical'].includes(kind) && object(payload);
}
export function loadRequest(): UnconfirmedRequest | null {
  const value = localStorage.getItem(REQUEST_KEY);
  if (!value) return null;
  let record: unknown;
  try { record = JSON.parse(value); } catch { throw new Error('The saved retry could not be read. Review the saved campaign before clearing this request.'); }
  if (!validSavedRequest(record)) throw new Error('The saved retry is incomplete or incompatible. Review the saved campaign before clearing this request.');
  return record;
}
export function loadSelection(): Selection {
  try {
    const value: unknown = JSON.parse(localStorage.getItem(SELECTION_KEY) ?? 'null');
    if (!object(value)) return { campaignId: null, playerId: null };
    return { campaignId: id(value.campaignId) ? value.campaignId : null, playerId: id(value.playerId) ? value.playerId : null, ...(id(value.sourceActorId) ? { sourceActorId: value.sourceActorId } : {}) };
  }
  catch { return { campaignId: null, playerId: null }; }
}
export function saveSelection(selection: Selection): void { localStorage.setItem(SELECTION_KEY, JSON.stringify(selection)); }
export function requestLabel(request: UnconfirmedRequest): string {
  if (request.kind === 'text') return `Your text: ${request.request.text}`;
  if (request.kind === 'create') return `Create campaign: ${request.request.name}`;
  if (typeof request.request.action === 'object' && 'AwardExcessInspiration' in request.request.action) return 'Award extra Heroic Inspiration';
  if (typeof request.request.action === 'object' && 'InspirationTransfer' in request.request.action) return 'Resolve your extra Heroic Inspiration';
  if (request.request.action === 'EnablePhysicalFacts') return 'Enable physical facts';
  if (typeof request.request.action === 'object' && 'PhysicalFact' in request.request.action) return 'Record physical information';
  if (request.request.action === 'EnableGrappleTransport') return 'Enable dragging and Inspiration for this table';
  if (typeof request.request.action === 'object' && 'AwardHeroicInspiration' in request.request.action) return 'Award Heroic Inspiration';
  if (request.request.action === 'EnableGrappleAccess') return 'Enable Grapple for this table';
  if (typeof request.request.action === 'object' && 'EnableSourceActorAccess' in request.request.action) return 'Enable source creature control';
  if (typeof request.request.action === 'object' && 'SetSourceCreatureController' in request.request.action) return 'Assign source creature control';
  const labels: Record<string, string> = { EndSession:'End the session',UpdateContract:'Update the table agreement',AddPlayer:'Add a player',CreateCharacter:'Create a character',CreateCharacterFromSource:'Create a character',CreateCreature:'Prepare a source creature',PrepareEquipment:'Prepare starting equipment',PrepareBattlefield:'Prepare the encounter map',Tactical:'Resolve an encounter action',StartSession:'Start a session',SetSituation:'Establish a situation',CancelDecision:'Withdraw a declaration',Adjudicate:'Request a supported roll',SubmitPhysical:'Report physical dice' };
  return labels[typeof request.request.action === 'string' ? request.request.action : Object.keys(request.request.action)[0]];
}

// The desktop adapter supplies trusted channels independently of natural-language text.
export const tableApi = {
  sourceControlOptions: (request: { campaign_id: Id; channel: LocalChannel; revision: Id }) => invoke<SourceControlOptions>('desktop_source_control_options', { request }),
  creatureOptions: (request: { campaign_id: Id; channel: RequestContext['channel']; revision: Id }) => invoke<CreatureOption[]>('desktop_creature_options', { request }),
  rollOptions: (request: { campaign_id: Id; channel: RequestContext['channel']; revision: Id; roll_id: Id }) => invoke<{ savage_attacker: { weapon_dice: number; heroic_inspiration: boolean } | null; heroic_inspiration?: boolean }>('desktop_roll_options', { request }),
  rollDetails: (request: { version: 1; campaign_id: Id; channel: RequestContext['channel']; revision: Id; roll_id: Id }) => invoke<{ version: 1; options: { savage_attacker: { weapon_dice: number; heroic_inspiration: boolean } | null; heroic_inspiration?: boolean }; display_reason: string }>('desktop_roll_details', { request }),
  defaults: () => invoke<TableContract>('desktop_default_contract'),
  list: () => invoke<{ id: Id; name: string }[]>('desktop_list_campaigns'),
  create: (request: { id: Id; name: string; contract: TableContract }) => invoke<TableView>('desktop_create_campaign', { request }),
  view: (campaignId: Id, viewer: Viewer) => invoke<TableView>('desktop_open_campaign', { request: { campaign_id: campaignId, viewer } }),
  options: (campaignId: Id) => invoke<CreationOptions>('desktop_creation_options', { request: { campaign_id: campaignId } }),
  situation: (campaignId: Id) => invoke<Situation>('desktop_host_situation', { request: { campaign_id: campaignId } }),
  action: async (request: SavedContext & { action: TableAction }): Promise<Receipt> => {
    let result: TextResult;
    if ('revision' in request) {
      const { action, ...context } = request;
      const decision = typeof action === 'object' && 'Tactical' in action && typeof action.Tactical.action === 'object' && 'ChooseTurnWork' in action.Tactical.action
        ? action.Tactical.action.ChooseTurnWork : null;
      const hit = typeof action === 'object' && 'Tactical' in action && typeof action.Tactical.action === 'object' && 'HitResponse' in action.Tactical.action
        ? action.Tactical.action.HitResponse : null;
      const missile = typeof action === 'object' && 'Tactical' in action && typeof action.Tactical.action === 'object' && 'MissileResponse' in action.Tactical.action
        ? action.Tactical.action.MissileResponse : null;
      const shove = typeof action === 'object' && 'Tactical' in action && typeof action.Tactical.action === 'object' && 'ShoveDecision' in action.Tactical.action
        ? action.Tactical.action.ShoveDecision : null;
      const grapple = typeof action === 'object' && 'Tactical' in action && typeof action.Tactical.action === 'object' && 'GrappleChoice' in action.Tactical.action
        ? action.Tactical.action.GrappleChoice : null;
      const equipment = typeof action === 'object' && 'Tactical' in action && typeof action.Tactical.action === 'object' && 'AttackEquipment' in action.Tactical.action
        ? action.Tactical.action.AttackEquipment : null;
      const groundDrag = typeof action === 'object' && 'Tactical' in action && typeof action.Tactical.action === 'object' && 'MoveGrappled' in action.Tactical.action
        ? action.Tactical.action.MoveGrappled : null;
      const transfer = typeof action === 'object' && 'InspirationTransfer' in action ? action.InspirationTransfer : null;
      const physical = typeof action === 'object' && 'PhysicalFact' in action ? action.PhysicalFact : null;
      const input = action === 'EnablePhysicalFacts' ? 'EnablePhysicalFacts' : physical ? { PhysicalFact: physical } : transfer ? { InspirationTransfer: transfer } : groundDrag ? { MoveGrappled: groundDrag } : grapple ? { GrappleChoice: grapple } : equipment ? { AttackEquipment: equipment } : shove ? { ShoveDecision: shove } : missile ? { MissileResponse: missile } : hit ? { HitResponse: hit } : decision ? { SelectWork: { handle: decision.handle } } : { Action: action };
      result = await invoke<TextResult>('desktop_submit_table', { request: { ...context, input } });
    } else {
      // A saved v1 nonce goes only to the recovery-only endpoint. Never translate
      // its numeric head into today's revision or allocate a replacement identity.
      result = await invoke<TextResult>('desktop_table_action', { request });
    }
    if (!('Accepted' in result)) throw new Error('The original action acknowledgement could not be confirmed.');
    return result.Accepted;
  },
  text: (request: SavedContext & { text: string }): Promise<TextResult> => {
    if ('revision' in request) {
      const { text, ...context } = request;
      return invoke<TextResult>('desktop_submit_table', { request: { ...context, input: { Text: { text } } } });
    }
    return invoke<TextResult>('desktop_table_text', { request });
  },
};

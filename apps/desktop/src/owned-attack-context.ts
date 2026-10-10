import type { TableView } from './table-api';
import type { AttackOptions } from './tactical-api';
import { heldMeleeWeapons } from './owned-attack-intent';

export interface OwnedAttackContext {
  key: string;
  options: AttackOptions;
  targetDetails: { actor: string; position: string | null }[];
}

export function distinguishableAttackTargets(context: OwnedAttackContext, targets: AttackOptions['targets']): boolean {
  const groups = new Map<string, string[]>();
  for (const target of targets) {
    const label = target.label.trim().replace(/\s+/gu, ' ').toLowerCase();
    groups.set(label, [...(groups.get(label) ?? []), target.actor]);
  }
  return [...groups.values()].every(actors => {
    if (actors.length === 1) return true;
    const positions = actors.map(actor => context.targetDetails.find(detail => detail.actor === actor)?.position);
    return positions.every(position => !!position) && new Set(positions).size === actors.length;
  });
}

/** Rendering context, not permission. Only the existing command path authorizes attacks. */
export function ownedAttackContext(view: TableView | null, player: string, source: string, generation: number): OwnedAttackContext | null {
  const tactical = view?.tactical;
  if (!player || source || !view?.active_session || view.pending || view.roll || view.inspiration_transfer
    || !tactical || tactical.phase !== 'active' || tactical.execution !== 'EncounterReleaseV1'
    || !tactical.active_actor || !tactical.budget
    || (tactical.budget.action_spent && tactical.budget.attacks_remaining === 0)
    || tactical.attack_equipment || tactical.shove || tactical.hit || tactical.missile
    || tactical.continuation || tactical.attack_decision || tactical.opportunity
    || tactical.liquid_landing || tactical.legendary_action || tactical.legendary_resistance
    || tactical.may_fail_save || !tactical.attack_options) return null;
  const bindings = view.active_session.participants.filter(participant => participant.player_id === player
    && participant.attendance === 'Present' && participant.character_id);
  if (bindings.length !== 1) return null;
  const character = view.characters.find(candidate => candidate.character_id === bindings[0].character_id
    && candidate.player_id === player && candidate.entity_id === tactical.active_actor);
  const options = tactical.attack_options;
  if (!character || options.actor !== character.entity_id
    || !tactical.combatant_sources.some(candidate => candidate.actor === character.entity_id && candidate.source === 'Character')
    || !options.targets.length || !heldMeleeWeapons(options).length) return null;
  // Only a display discriminator for already-offered IDs. Never resolve names or
  // discover candidates through participants, another observer, or remembered truth.
  const contacts = tactical.observers.find(observer => observer.observer === character.entity_id)?.contacts ?? [];
  const targetDetails = options.targets.map(target => {
    const contact = contacts.find(candidate => candidate.entity_id === target.actor && candidate.status !== 'Remembered');
    const point = contact?.position;
    const position = point && [point.x, point.y, point.z].every(Number.isSafeInteger)
      ? `${point.x / 2}, ${point.y / 2} feet; height ${point.z / 2} feet` : null;
    return { actor: target.actor, position };
  });
  return { options, targetDetails, key: JSON.stringify([
    view.campaign_id, player, source, character.character_id, character.entity_id,
    view.active_session.session_id, view.revision, tactical.encounter_id, tactical.round,
    view.physical?.version ?? view.grapple?.version ?? (view.source_control ? 2 : 1),
    tactical.budget, options, targetDetails, generation,
  ]) };
}

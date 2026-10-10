import type { IntrinsicAttackRequest, TableView } from './table-api';

/** A rendering hint only. The separate read and accepted command enforce authority. */
export function intrinsicAttackContext(view: TableView | null, player: string, source: string): { key: string; request: IntrinsicAttackRequest } | null {
  const tactical = view?.tactical;
  if (!view?.active_session || view.pending || view.roll || view.inspiration_transfer
    || !tactical || tactical.phase !== 'active' || tactical.execution !== 'EncounterReleaseV1'
    || !tactical.active_actor || !tactical.budget || tactical.budget.action_spent
    || tactical.attack_equipment || tactical.shove || tactical.hit || tactical.missile
    || tactical.continuation || tactical.attack_decision || tactical.opportunity
    || tactical.liquid_landing || tactical.legendary_action || tactical.legendary_resistance) return null;
  const actor = tactical.active_actor;
  const owner = view.source_control?.actors.find(candidate => candidate.actor === actor);
  if (player) {
    if (source !== actor || !owner || typeof owner.controller !== 'object' || owner.controller.Player !== player
      || !view.active_session.participants.some(participant => participant.player_id === player && participant.attendance === 'Present')) return null;
  } else if (typeof owner?.controller === 'object'
    || !tactical.combatant_sources.some(candidate => candidate.actor === actor && typeof candidate.source === 'object')) return null;
  const request: IntrinsicAttackRequest = {
    version: 1, campaign_id: view.campaign_id, revision: view.revision, actor,
    channel: player ? { SourceCreature: { player_id: player, actor } } : 'Host',
  };
  return { request, key: JSON.stringify([request, view.active_session.session_id, tactical.encounter_id, tactical.round]) };
}

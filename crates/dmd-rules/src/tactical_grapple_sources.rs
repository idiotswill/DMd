//! Pure source anatomy query, not Grapple admission or a hand-occupancy planner.
use crate::{RulesError, RulesPack, tactical_creatures, validate_character_intrinsics};
use dmd_domain::*;

/// Full supported Human reconstruction or a fully pinned annotated creature is
/// required. Default loadout slots and narrative/species/type strings prove nothing.
pub fn ordinary_grapple_anatomy(
    state: &CampaignState,
    actor: EntityId,
    pack: &RulesPack,
) -> Result<Option<GrappleAnatomyProof>, RulesError> {
    if state.campaign.ruleset.id != pack.id || state.campaign.ruleset.version != pack.version {
        return Err(RulesError::Incompatible(
            "anatomy source and campaign ruleset differ".into(),
        ));
    }
    let rules = state.rules.as_ref().ok_or(RulesError::Uninitialized)?;
    let mechanics = rules
        .entities
        .get(&actor)
        .ok_or_else(|| RulesError::Invalid("anatomy actor lacks mechanics".into()))?;
    let mut characters = state.characters.values().filter(|c| c.entity_id == actor);
    if let Some(character) = characters.next() {
        if characters.next().is_some()
            || rules
                .tactical_creatures
                .as_ref()
                .is_some_and(|c| c.profile(actor).is_some())
        {
            return Err(RulesError::Invalid(
                "ambiguous anatomy source identity".into(),
            ));
        }
        let Some(profile) = state
            .table
            .as_ref()
            .and_then(|t| t.character_profiles.get(&character.id))
        else {
            return Ok(None);
        };
        if profile.entity_id != actor || character.campaign_id != state.campaign_id() {
            return Err(RulesError::Invalid(
                "anatomy character/profile identity differs".into(),
            ));
        }
        validate_character_intrinsics(profile, mechanics, pack)?;
        return Ok(Some(GrappleAnatomyProof::HumanCreationV1 {
            character: character.id,
        }));
    }
    let Some(profile) = rules
        .tactical_creatures
        .as_ref()
        .and_then(|c| c.profile(actor))
    else {
        return Ok(None);
    };
    tactical_creatures::validate_creature_profile(state, profile, mechanics)
        .map_err(|e| RulesError::Invalid(e.to_string()))?;
    let source = tactical_creatures::source_for_profile(profile)
        .map_err(|e| RulesError::Invalid(e.to_string()))?;
    Ok(source
        .ordinary_hands
        .map(|ordinary_hands| GrappleAnatomyProof::Creature {
            source: profile.source.clone(),
            ordinary_hands,
        }))
}

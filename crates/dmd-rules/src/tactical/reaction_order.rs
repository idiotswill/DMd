//! ADR028: apply an explicitly selected total order after private collection.
//! This pure operation grants no response or ordering authority. Its caller must
//! authenticate the current occurrence/controller and revalidate each selected
//! source response before payment. Neither eligibility nor network order affects
//! admission of the instruction itself.
use super::*;
use std::collections::HashSet;

pub fn order_reaction_respondents(
    instruction: &TacticalReactionOrdering,
    initiative: &[EntityId],
    known: &HashSet<EntityId>,
    accepted: &[EntityId],
) -> Result<Vec<EntityId>, RulesError> {
    let participants = initiative.iter().copied().collect::<HashSet<_>>();
    let ranked = instruction.ranked.iter().copied().collect::<HashSet<_>>();
    if initiative.is_empty()
        || participants.len() != initiative.len()
        || participants.iter().any(|actor| actor.0.is_nil())
        || ranked.len() != instruction.ranked.len()
        || !ranked.is_subset(&participants)
        || !ranked.is_subset(known)
    {
        return Err(invalid(
            "reaction order must rank distinct known initiative participants",
        ));
    }
    let respondents = accepted.iter().copied().collect::<HashSet<_>>();
    if respondents.len() != accepted.len() || !respondents.is_subset(&participants) {
        return Err(invalid(
            "reaction respondents differ from the current initiative",
        ));
    }
    let mut unlisted = initiative
        .iter()
        .copied()
        .filter(|actor| !ranked.contains(actor))
        .collect::<Vec<_>>();
    if matches!(
        instruction.unlisted,
        ReactionUnlistedOrder::BeforeReverse | ReactionUnlistedOrder::AfterReverse
    ) {
        unlisted.reverse();
    }
    let mut total = if matches!(
        instruction.unlisted,
        ReactionUnlistedOrder::BeforeForward | ReactionUnlistedOrder::BeforeReverse
    ) {
        unlisted.extend_from_slice(&instruction.ranked);
        unlisted
    } else {
        let mut total = instruction.ranked.clone();
        total.extend(unlisted);
        total
    };
    total.retain(|actor| respondents.contains(actor));
    Ok(total)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn explicit_hidden_fallback_changes_priority_but_arrival_order_does_not() {
        let actors = [
            EntityId::new(),
            EntityId::new(),
            EntityId::new(),
            EntityId::new(),
        ];
        let known = HashSet::from([actors[1]]);
        let mut instruction = TacticalReactionOrdering {
            ranked: vec![actors[1]],
            unlisted: ReactionUnlistedOrder::BeforeForward,
        };
        let accepted = [actors[2], actors[0], actors[1]];
        let opposite = accepted.into_iter().rev().collect::<Vec<_>>();
        assert_eq!(
            order_reaction_respondents(&instruction, &actors, &known, &accepted).unwrap(),
            [actors[0], actors[2], actors[1]]
        );
        assert_eq!(
            order_reaction_respondents(&instruction, &actors, &known, &opposite).unwrap(),
            [actors[0], actors[2], actors[1]]
        );
        instruction.unlisted = ReactionUnlistedOrder::BeforeReverse;
        assert_eq!(
            order_reaction_respondents(&instruction, &actors, &known, &accepted).unwrap(),
            [actors[2], actors[0], actors[1]]
        );
        instruction.unlisted = ReactionUnlistedOrder::AfterForward;
        assert_eq!(
            order_reaction_respondents(&instruction, &actors, &known, &accepted).unwrap(),
            [actors[1], actors[0], actors[2]]
        );
        instruction.unlisted = ReactionUnlistedOrder::AfterReverse;
        assert_eq!(
            order_reaction_respondents(&instruction, &actors, &known, &accepted).unwrap(),
            [actors[1], actors[2], actors[0]]
        );
    }

    #[test]
    fn instruction_admission_uses_knowledge_even_when_no_named_participant_reacts() {
        let actors = [EntityId::new(), EntityId::new(), EntityId::new()];
        let known = HashSet::from([actors[1]]);
        let instruction = TacticalReactionOrdering {
            ranked: vec![actors[1]],
            unlisted: ReactionUnlistedOrder::AfterForward,
        };
        let original = serde_json::to_value(&instruction).unwrap();
        for accepted in [vec![], vec![actors[0]], vec![actors[2], actors[0]]] {
            assert!(order_reaction_respondents(&instruction, &actors, &known, &accepted).is_ok());
            assert_eq!(serde_json::to_value(&instruction).unwrap(), original);
            let unknown = TacticalReactionOrdering {
                ranked: vec![actors[0]],
                ..instruction.clone()
            };
            assert!(order_reaction_respondents(&unknown, &actors, &known, &accepted).is_err());
        }
        let duplicate = TacticalReactionOrdering {
            ranked: vec![actors[1], actors[1]],
            ..instruction.clone()
        };
        assert!(order_reaction_respondents(&duplicate, &actors, &known, &[]).is_err());
        assert!(
            order_reaction_respondents(&instruction, &actors, &known, &[actors[0], actors[0]])
                .is_err()
        );
        assert!(
            order_reaction_respondents(&instruction, &actors, &known, &[EntityId::new()]).is_err()
        );
        assert!(
            serde_json::from_value::<TacticalReactionOrdering>(serde_json::json!({"ranked":[]}))
                .is_err(),
            "omitting the fallback must not silently select one"
        );
    }
}

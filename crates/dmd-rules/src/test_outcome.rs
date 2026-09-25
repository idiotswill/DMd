//! One outcome policy for ordinary checks/saves. Attack and death-save source
//! rules remain separate; this never rewrites an accepted die face.
use crate::RulesError;
use dmd_domain::{HouseRules, ResolvedRoll};

pub(crate) fn ability_test_success(
    roll: &ResolvedRoll,
    dc: i32,
    house_rules: &HouseRules,
) -> Result<bool, RulesError> {
    let face = roll
        .kept_dice
        .first()
        .filter(|die| die.sides == 20 && (1..=20).contains(&die.value))
        .ok_or_else(|| RulesError::Invalid("ability test lacks its resolved d20".into()))?
        .value;
    Ok(match face {
        20 if house_rules.ability_test_natural_extremes => true,
        1 if house_rules.ability_test_natural_extremes => false,
        _ => roll.total >= dc,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ResolveRoll;
    use dmd_domain::*;

    #[test]
    fn only_opted_in_kept_extremes_override_the_actual_total() {
        let normal = HouseRules::default();
        let house = HouseRules {
            ability_test_natural_extremes: true,
        };
        for (faces, modifier, mode, raw_success, house_success) in [
            (vec![1], 20, RollMode::Normal, true, false),
            (vec![20], -15, RollMode::Normal, false, true),
            (vec![1, 17], 0, RollMode::Advantage, true, true),
            (vec![20, 5], 0, RollMode::Disadvantage, false, false),
        ] {
            let request = RollRequest {
                id: RollRequestId::new(),
                roller: None,
                dice: vec![DieSpec {
                    count: 1,
                    sides: 20,
                }],
                modifier,
                mode,
                visibility: RollVisibility::Public,
                reason: "Outcome policy".into(),
            };
            let raw = RollResult {
                request_id: request.id,
                source: RollSource::Physical,
                dice: faces
                    .iter()
                    .map(|value| DieResult {
                        sides: 20,
                        value: *value,
                    })
                    .collect(),
            };
            let roll = request.resolve(&raw).unwrap();
            assert_eq!(
                ability_test_success(&roll, 15, &normal).unwrap(),
                raw_success
            );
            assert_eq!(
                ability_test_success(&roll, 15, &house).unwrap(),
                house_success
            );
            assert_eq!(
                raw.dice.iter().map(|die| die.value).collect::<Vec<_>>(),
                faces
            );
        }
    }
}

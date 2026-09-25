pub use dmd_domain::{
    DieResult, DieSpec, EntityId, ResolvedRoll, RollMode, RollRequest, RollRequestId, RollResult,
    RollSource, RollVisibility,
};
use thiserror::Error;
pub mod kernel;
pub use kernel::*;
pub mod character_creation;
pub use character_creation::*;
pub mod spatial;
pub mod tactical_creature_equipment;
pub mod tactical_creatures;
pub mod tactical_definitions;
pub mod tactical_effect_adapter;
pub mod tactical_effects;
pub mod tactical_inventory;
mod tactical_vitality_adapter;
pub mod tactical_weapons;
mod test_outcome;

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum RollError {
    #[error("a roll request must contain at least one die")]
    EmptyDice,
    #[error("die count must be greater than zero")]
    ZeroDice,
    #[error("a die must have at least two sides, got d{sides}")]
    InvalidDie { sides: u16 },
    #[error("advantage/disadvantage requires exactly one d20 request")]
    InvalidAdvantageShape,
    #[error("roll result belongs to a different request")]
    RequestMismatch,
    #[error("roll result dice do not match the request")]
    DiceMismatch,
    #[error("reported d{sides} value {value} is outside 1..={sides}")]
    DieValueOutOfRange { sides: u16, value: u16 },
    #[error("resolved total overflowed i32")]
    TotalOverflow,
}

pub trait ResolveRoll {
    fn validate(&self) -> Result<(), RollError>;
    fn resolve(&self, result: &RollResult) -> Result<ResolvedRoll, RollError>;
}

impl ResolveRoll for RollRequest {
    fn validate(&self) -> Result<(), RollError> {
        if self.dice.is_empty() {
            return Err(RollError::EmptyDice);
        }

        for die in &self.dice {
            if die.count == 0 {
                return Err(RollError::ZeroDice);
            }
            if die.sides < 2 {
                return Err(RollError::InvalidDie { sides: die.sides });
            }
        }

        if self.mode != RollMode::Normal
            && (self.dice.len() != 1 || self.dice[0].count != 1 || self.dice[0].sides != 20)
        {
            return Err(RollError::InvalidAdvantageShape);
        }

        Ok(())
    }

    fn resolve(&self, result: &RollResult) -> Result<ResolvedRoll, RollError> {
        self.validate()?;
        if result.request_id != self.id {
            return Err(RollError::RequestMismatch);
        }

        for die in &result.dice {
            if die.sides < 2 || die.value == 0 || die.value > die.sides {
                return Err(RollError::DieValueOutOfRange {
                    sides: die.sides,
                    value: die.value,
                });
            }
        }

        let kept_dice = match self.mode {
            RollMode::Normal => {
                if !dice_match_specs(&self.dice, &result.dice) {
                    return Err(RollError::DiceMismatch);
                }
                result.dice.clone()
            }
            RollMode::Advantage | RollMode::Disadvantage => {
                if result.dice.len() != 2 || result.dice.iter().any(|die| die.sides != 20) {
                    return Err(RollError::DiceMismatch);
                }
                let selected = match self.mode {
                    RollMode::Advantage => result
                        .dice
                        .iter()
                        .max_by_key(|die| die.value)
                        .copied()
                        .expect("two validated dice must have a maximum"),
                    RollMode::Disadvantage => result
                        .dice
                        .iter()
                        .min_by_key(|die| die.value)
                        .copied()
                        .expect("two validated dice must have a minimum"),
                    RollMode::Normal => unreachable!("handled above"),
                };
                vec![selected]
            }
        };

        let die_total = kept_dice.iter().try_fold(0_i32, |total, die| {
            total
                .checked_add(i32::from(die.value))
                .ok_or(RollError::TotalOverflow)
        })?;
        let total = die_total
            .checked_add(self.modifier)
            .ok_or(RollError::TotalOverflow)?;

        Ok(ResolvedRoll {
            request_id: self.id,
            source: result.source,
            raw_dice: result.dice.clone(),
            kept_dice,
            modifier: self.modifier,
            total,
        })
    }
}

fn dice_match_specs(specs: &[DieSpec], results: &[DieResult]) -> bool {
    let expected_count = specs.iter().try_fold(0_usize, |total, spec| {
        total.checked_add(usize::from(spec.count))
    });
    if expected_count != Some(results.len()) {
        return false;
    }

    let mut unmatched = results.to_vec();
    for spec in specs {
        for _ in 0..spec.count {
            let Some(index) = unmatched
                .iter()
                .position(|result| result.sides == spec.sides)
            else {
                return false;
            };
            unmatched.swap_remove(index);
        }
    }

    unmatched.is_empty()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn d20_request(mode: RollMode, modifier: i32) -> RollRequest {
        RollRequest {
            id: RollRequestId::new(),
            roller: Some(EntityId::new()),
            dice: vec![DieSpec {
                count: 1,
                sides: 20,
            }],
            modifier,
            mode,
            visibility: RollVisibility::Public,
            reason: "test roll".into(),
        }
    }

    #[test]
    fn physical_roll_submits_raw_face_and_engine_applies_modifier() {
        let request = d20_request(RollMode::Normal, 5);
        let result = RollResult {
            request_id: request.id,
            source: RollSource::Physical,
            dice: vec![DieResult {
                sides: 20,
                value: 13,
            }],
        };

        let resolved = request.resolve(&result).expect("valid roll should resolve");
        assert_eq!(resolved.total, 18);
        assert_eq!(resolved.modifier, 5);
        assert_eq!(resolved.source, RollSource::Physical);
    }

    #[test]
    fn advantage_keeps_higher_d20_without_losing_raw_faces() {
        let request = d20_request(RollMode::Advantage, 2);
        let result = RollResult {
            request_id: request.id,
            source: RollSource::Digital,
            dice: vec![
                DieResult {
                    sides: 20,
                    value: 7,
                },
                DieResult {
                    sides: 20,
                    value: 16,
                },
            ],
        };

        let resolved = request.resolve(&result).expect("valid advantage roll");
        assert_eq!(resolved.raw_dice.len(), 2);
        assert_eq!(resolved.kept_dice[0].value, 16);
        assert_eq!(resolved.total, 18);
    }

    #[test]
    fn result_for_another_request_is_rejected() {
        let request = d20_request(RollMode::Normal, 0);
        let result = RollResult {
            request_id: RollRequestId::new(),
            source: RollSource::Physical,
            dice: vec![DieResult {
                sides: 20,
                value: 10,
            }],
        };

        assert_eq!(request.resolve(&result), Err(RollError::RequestMismatch));
    }

    #[test]
    fn impossible_physical_face_is_rejected() {
        let request = d20_request(RollMode::Normal, 0);
        let result = RollResult {
            request_id: request.id,
            source: RollSource::Physical,
            dice: vec![DieResult {
                sides: 20,
                value: 21,
            }],
        };

        assert_eq!(
            request.resolve(&result),
            Err(RollError::DieValueOutOfRange {
                sides: 20,
                value: 21
            })
        );
    }
}
pub mod tactical;
pub mod tactical_budget;
pub mod tactical_conditions;
pub mod tactical_damage;

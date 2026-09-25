use super::*;
use crate::ResolveRoll;

/// The central scheduler allocates the request ID and authenticates the actual
/// source controller. Every victim shares this one source-derived raw amount.
pub fn area_amount_request(
    program: &AreaProgram,
    id: RollRequestId,
    visibility: RollVisibility,
) -> Result<RollRequest, RulesError> {
    if id.0.is_nil() {
        return Err(invalid("area amount request has no allocated identity"));
    }
    let request = RollRequest {
        id,
        roller: Some(program.actor()),
        dice: program.dice().to_vec(),
        modifier: program.modifier(),
        mode: RollMode::Normal,
        visibility,
        reason: "Damage from the accepted area effect".into(),
    };
    request.validate()?;
    Ok(request)
}

/// Re-resolve the canonical shared raw amount; no supplied total is authoritative.
/// Success halves this occurrence BEFORE the ordinary vitality reducer applies
/// resistance/vulnerability. This save effect is neither an attack nor a knockout.
pub fn area_damage_operation(
    program: &AreaProgram,
    id: RollRequestId,
    raw: &RollResult,
    succeeded: bool,
) -> Result<VitalityOperation, RulesError> {
    let request = area_amount_request(program, id, RollVisibility::Secret)?;
    let amount = request.resolve(raw)?.total;
    let amount = u32::try_from(amount.max(0)).map_err(|_| invalid("area damage exceeds bounds"))?;
    let amounts = vec![if succeeded && !program.half_on_success() {
        0
    } else {
        amount
    }];
    let adjustments = if succeeded && program.half_on_success() {
        vec![DamageAdjustment::Multiply {
            numerator: 1,
            denominator: 2,
        }]
    } else {
        vec![]
    };
    Ok(VitalityOperation::Damage {
        packet: DamagePacket {
            cause: DamageCause::Other,
            components: vec![DamageComponent {
                damage_type: program.damage_type(),
                amounts,
                adjustments,
            }],
        },
        knockout: None,
    })
}

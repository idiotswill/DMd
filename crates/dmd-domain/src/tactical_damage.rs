//! Durable recovery causes and trusted internal vitality inputs (SRD 5.2.1 pp. 16–18).
//! HP remains exclusively on `MechanicalEntity`. These are not public player commands.
use crate::{CommandMeta, DamageType, EntityId, RollRequestId, RollResult, WorldInstant};
use serde::{Deserialize, Serialize};

/// Distinguishes several damage occurrences produced by one accepted command.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct VitalityOrigin {
    pub command: CommandMeta,
    pub occurrence: u16,
}

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TacticalRecovery {
    pub knockout: Option<KnockoutRecovery>,
    pub stable: Option<StableRecovery>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct KnockoutRecovery {
    pub origin: VitalityOrigin,
    pub inflicted_at: WorldInstant,
    /// Damage/initiative/non-cantrip casting interrupts this rest, not the condition.
    pub short_rest_started_at: Option<WorldInstant>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct StableRecovery {
    pub origin: VitalityOrigin,
    pub stabilized_at: WorldInstant,
    /// Accepted raw 1d4; the wake time is derived, never supplied by a controller.
    pub delay_roll: Option<RollResult>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum DamageAdjustment {
    Add(i32),
    /// Apply in source-defined order, rounding down after each rational multiplier.
    Multiply {
        numerator: u16,
        denominator: u16,
    },
}

/// All amounts of one damage type in this occurrence are summed BEFORE rounding.
/// Different components must have different types. Separate attacks are separate packets.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DamageComponent {
    pub damage_type: DamageType,
    pub amounts: Vec<u32>,
    pub adjustments: Vec<DamageAdjustment>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum DamageCause {
    /// A confirmed hit; the attack resolver has already doubled appropriate damage dice.
    Attack {
        attacker: EntityId,
        melee: bool,
        critical: bool,
    },
    /// Source mastery miss damage (SRD90), not a hit or an attack damage roll.
    /// The source resolver supplies the actual attack ability modifier.
    Graze {
        attacker: EntityId,
        ability_modifier: i32,
    },
    /// Saves, environmental damage and other non-attack occurrences are never critical hits.
    Other,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DamagePacket {
    pub cause: DamageCause,
    pub components: Vec<DamageComponent>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum KnockoutChoice {
    NormalDamage,
    KnockOut,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TemporaryHpChoice {
    KeepExisting,
    Replace,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MedicinePurpose {
    Stabilize,
    EndKnockout,
}

/// These operations are supplied only after the enclosing resolver establishes authority,
/// cost, source entitlement and action timing. Raw dice still pass exact request validation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum VitalityOperation {
    Damage {
        packet: DamagePacket,
        knockout: Option<KnockoutChoice>,
    },
    Heal {
        amount: u32,
    },
    TemporaryHitPoints {
        amount: u32,
        choice: Option<TemporaryHpChoice>,
    },
    /// The parent has resolved an authorized DC10 Wisdom (Medicine) Help/first-aid check.
    /// Total is engine-derived; this is not a client-submitted check result.
    Medicine {
        purpose: MedicinePurpose,
        total: i32,
    },
    DeathSave {
        request_id: RollRequestId,
        result: RollResult,
    },
    /// Controller chooses to fail a saving throw (SRD187); no fictional natural die.
    FailDeathSave,
    /// A validated source feature changes the failed save to an ordinary success.
    /// This neither rewrites its die nor grants the natural-20 healing exception.
    SucceedDeathSave,
    Stabilize,
    StableRecoveryRoll {
        origin: VitalityOrigin,
        request_id: RollRequestId,
        result: RollResult,
    },
    RecoverStable,
    InterruptKnockoutRest,
    StartKnockoutRest,
    CompleteShortRest,
    /// Only the temporary-HP expiry consequence; the parent owns rest eligibility/healing.
    CompleteLongRest,
    /// Source-derived new maximum, e.g. life drain. Not healing, including when increased.
    SetMaximumHitPoints {
        maximum: u32,
    },
    /// Called after independently installed/removed conditions, without inventing HP damage.
    ReconcileConditions,
}

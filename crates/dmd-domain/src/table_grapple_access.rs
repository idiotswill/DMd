//! Explicit table protocol activation; this record never supplies replay authority.
use crate::{
    AttackEquipmentOperation, EntityId, GrappleEscapeChoice, GrappleId, GrappleSaveAbility, Hand,
    TacticalWorkKey,
};
use crate::{CampaignState, CommandIssuer, CommandMeta};
use serde::{Deserialize, Serialize};

/// Canonical contents of a server-issued option. Desktop input carries only its
/// opaque audience handle; this enum is never an admission or a retained proof.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum TableGrappleChoice {
    Attempt {
        target: EntityId,
        hand: Hand,
        before_change: Option<AttackEquipmentOperation>,
    },
    Save {
        grip: GrappleId,
        ability: GrappleSaveAbility,
    },
    AfterEquipment {
        grip: GrappleId,
        work: TacticalWorkKey,
        operation: Option<AttackEquipmentOperation>,
    },
    Withdraw {
        grip: GrappleId,
    },
    Release {
        grip: GrappleId,
    },
    Escape {
        grip: GrappleId,
        choice: GrappleEscapeChoice,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TableGrappleOffer {
    pub actor: EntityId,
    pub choice: TableGrappleChoice,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TableGrappleAccessVersion {
    OrdinaryGrappleV1,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TableGrappleAccess {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ground_transport: Option<TableGrappleTransportAccess>,
    pub version: TableGrappleAccessVersion,
    pub origin: CommandMeta,
}

impl TableGrappleAccess {
    pub fn validate(&self, state: &CampaignState) -> Result<(), String> {
        if self.origin.id.0.is_nil()
            || self.origin.campaign_id != state.campaign_id()
            || self.origin.expected_event_sequence > state.applied_event_sequence
            || self.origin.issuer != CommandIssuer::Admin
            || self.origin.actor.is_some()
            || self.origin.session_id.is_none_or(|id| id.0.is_nil())
        {
            return Err("invalid table Grapple activation provenance".into());
        }
        if let Some(transport) = &self.ground_transport {
            let origin = &transport.origin;
            if origin.id.0.is_nil() || origin.id == self.origin.id
                || origin.campaign_id != state.campaign_id()
                || origin.expected_event_sequence <= self.origin.expected_event_sequence
                || origin.expected_event_sequence > state.applied_event_sequence
                || origin.issuer != CommandIssuer::Admin || origin.actor.is_some()
                || origin.session_id.is_none_or(|id| id.0.is_nil()) {
                return Err("invalid ground drag activation provenance".into());
            }
        }
        Ok(())
    }
}

/// A journaled capability upgrade, never a deserialized execution authority.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TableGrappleTransportAccess {
    pub version: crate::GrappleTransportKind,
    pub origin: CommandMeta,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TableGrappleTransportOffer {
    pub actor: EntityId,
    pub grip: GrappleId,
    pub kind: crate::GrappleTransportKind,
}

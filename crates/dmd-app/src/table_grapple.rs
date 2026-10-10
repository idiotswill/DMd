//! Owner-private Grapple choices; only opaque option keys leave presented transport.
use dmd_domain::*;

pub(crate) fn view(
    read: &dmd_rules::table::TableRead<'_>,
    viewer: &crate::TableViewer,
) -> Result<crate::TableGrappleView, String> {
    let state = read.state();
    let issuer = match viewer {
        crate::TableViewer::Host => CommandIssuer::Admin,
        crate::TableViewer::Player(player) => CommandIssuer::Player(*player),
    };
    let choices = read
        .grapple_choices(issuer)
        .map_err(|error| error.to_string())?;
    let mut shown = Vec::new();
    for offer in choices {
        let observer = state
            .encounter
            .as_ref()
            .map(|encounter| dmd_rules::spatial::project_actor_view(encounter, state, offer.actor))
            .transpose()
            .map_err(|error| error.to_string())?;
        let label = |target| {
            observer
                .as_ref()
                .and_then(|view| {
                    view.contacts
                        .iter()
                        .find(|contact| contact.entity_id == target)
                })
                .and_then(|contact| contact.label.as_deref())
                .unwrap_or("creature")
                .to_owned()
        };
        let hand = |hand| match hand {
            Hand::Left => "left hand",
            Hand::Right => "right hand",
        };
        let equipment = |operation| {
            let (item, operation) = match operation {
                AttackEquipmentOperation::Equip {
                    item,
                    hand: selected,
                } => (item, format!("equip in {}", hand(selected))),
                AttackEquipmentOperation::Pickup {
                    item,
                    hand: selected,
                } => (item, format!("pick up in {}", hand(selected))),
                AttackEquipmentOperation::Unequip { item } => (item, "stow".to_owned()),
            };
            let name = state
                .items
                .get(&item)
                .and_then(|item| {
                    dmd_rules::tactical_inventory::equipment_definition(&item.definition_id).ok()
                })
                .map(|definition| definition.display_name.clone())
                .unwrap_or_else(|| "item".into());
            format!("{operation} {name}")
        };
        let text = match &offer.choice {
            TableGrappleChoice::Attempt {
                target,
                hand: selected,
                before_change,
            } => {
                let before = before_change
                    .map(equipment)
                    .map(|text| format!("; first {text}"))
                    .unwrap_or_default();
                format!(
                    "Grapple {} with {}{before}",
                    label(*target),
                    hand(*selected)
                )
            }
            TableGrappleChoice::Save { ability, .. } => format!(
                "Resist Grapple with {}",
                match ability {
                    GrappleSaveAbility::Strength => "Strength",
                    GrappleSaveAbility::Dexterity => "Dexterity",
                }
            ),
            TableGrappleChoice::AfterEquipment { operation, .. } => operation
                .map(equipment)
                .map(|text| format!("After Grapple: {text}"))
                .unwrap_or_else(|| "Finish without changing equipment".into()),
            TableGrappleChoice::Withdraw { .. } => "Withdraw this Grapple attempt".into(),
            TableGrappleChoice::Release { grip } => {
                let live = state
                    .rules
                    .as_ref()
                    .and_then(|rules| rules.tactical_grapples.as_ref())
                    .and_then(|live| live.grip(*grip))
                    .ok_or("Offered grip is absent.")?;
                format!(
                    "Release {} from {}",
                    label(live.declaration.target),
                    hand(live.declaration.hand)
                )
            }
            TableGrappleChoice::Escape { grip, choice } => {
                let live = state
                    .rules
                    .as_ref()
                    .and_then(|rules| rules.tactical_grapples.as_ref())
                    .and_then(|live| live.grip(*grip))
                    .ok_or("Offered grip is absent.")?;
                format!(
                    "Escape {}'s {} grip using {}",
                    label(live.declaration.grappler),
                    hand(live.declaration.hand),
                    match choice {
                        GrappleEscapeChoice::Athletics => "Athletics",
                        GrappleEscapeChoice::Acrobatics => "Acrobatics",
                    }
                )
            }
        };
        shown.push(crate::TableGrappleOption {
            actor: offer.actor,
            key: offer,
            label: text,
        });
    }
    let ground_drag = read
        .grapple_transport_choices(issuer)
        .map_err(|error| error.to_string())?
        .into_iter()
        .map(|offer| {
            let grip = state
                .rules
                .as_ref()
                .and_then(|r| r.tactical_grapples.as_ref())
                .and_then(|g| g.grip(offer.grip))
                .ok_or("Offered drag grip is absent.")?;
            let observer = state.encounter.as_ref().ok_or("Drag encounter absent.")?;
            let view = dmd_rules::spatial::project_actor_view(observer, state, offer.actor)
                .map_err(|e| e.to_string())?;
            let name = view
                .contacts
                .iter()
                .find(|c| c.entity_id == grip.declaration.target)
                .and_then(|c| c.label.as_deref())
                .unwrap_or("creature");
            let hand = match grip.declaration.hand {
                Hand::Left => "left hand",
                Hand::Right => "right hand",
            };
            Ok(crate::TableGrappleOption {
                actor: offer.actor,
                label: format!("Drag {name} with {hand}"),
                key: offer,
            })
        })
        .collect::<Result<Vec<_>, String>>()?;
    Ok(crate::TableGrappleView {
        ground_drag,
        version: if dmd_rules::table::grapple_transport_enabled(state) {
            4
        } else {
            3
        },
        choices: shown,
    })
}

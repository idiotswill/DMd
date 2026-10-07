//! The original anchor contains ordinary world objects, never accepted mass authority.
//! Every positive mass record is produced by the same owned table dispatcher.
use super::*;
use crate::physical_facts::{self, execution::MassCommand};

fn owner() -> (CampaignExecution, EntityId, ItemId) {
    let contract = TableContract::default();
    let campaign = Campaign {
        id: CampaignId::new(),
        display_name: "Generic authored objects".into(),
        status: CampaignStatus::Active,
        world_seed: 7,
        ruleset: contract.ruleset.clone(),
        content_packs: vec![],
    };
    let mut anchor = CampaignState::empty(
        campaign,
        WorldClock {
            now: WorldInstant(0),
            calendar_id: "plain".into(),
        },
    );
    anchor.table = Some(TableState::new(contract));
    let actor = EntityId::new();
    anchor.entities.insert(
        actor,
        WorldEntity {
            id: actor,
            campaign_id: anchor.campaign_id(),
            display_name: "Authored bearer".into(),
            kind: EntityKind::Npc,
            existence: EntityExistence::Present,
            location_id: None,
        },
    );
    let item = ItemId::new();
    anchor.items.insert(
        item,
        ItemInstance {
            id: item,
            campaign_id: anchor.campaign_id(),
            definition_id: "dagger".into(),
            display_name: "Unclassified original object".into(),
            quantity: 1,
            owner: Ownership::Entity(actor),
            custody: Custody::Entity(actor),
            state: ItemState::Intact,
        },
    );
    let pack =
        RulesPack::from_json(include_str!("../../../../../content/srd-5.2.1/kernel.json")).unwrap();
    (
        CampaignExecution::from_original_anchor(anchor, pack).unwrap(),
        actor,
        item,
    )
}
fn meta(owner: &CampaignExecution) -> CommandMeta {
    CommandMeta {
        id: CommandId::new(),
        campaign_id: owner.read().state().campaign_id(),
        session_id: None,
        issuer: CommandIssuer::Admin,
        actor: None,
        expected_event_sequence: owner.read().state().applied_event_sequence,
    }
}
fn accept(owner: &mut CampaignExecution, offer: PhysicalFactOffer, input: PhysicalFactInput) {
    let command = meta(owner);
    let operation = TableOperation::PhysicalFact {
        acceptance: Box::new(PhysicalFactAcceptance { offer, input }),
    };
    owner.prepare_table(&command, &operation).unwrap().commit();
}
fn enable(owner: &mut CampaignExecution) {
    let offer = owner
        .read()
        .physical_fact_offers()
        .unwrap()
        .into_iter()
        .next()
        .unwrap();
    accept(owner, offer, PhysicalFactInput::Enable);
}
fn item_offer(owner: &CampaignExecution, item: ItemId) -> PhysicalFactOffer {
    owner
        .read()
        .physical_fact_offers()
        .unwrap()
        .into_iter()
        .find(
            |offer| matches!(offer,PhysicalFactOffer::Item { item:offered,.. } if offered.id==item),
        )
        .unwrap()
}
#[test]
fn registered_text_is_not_a_source_grant_but_explicit_host_classification_is_owned_and_retained() {
    let (mut owner, actor, item) = owner();
    enable(&mut owner);
    assert_eq!(owner.read().physical_load(actor).unwrap().known_pounds, "0");
    assert!(
        !owner
            .image
            .physical_sources
            .contains(&owner.image.state.items[&item])
    );
    let offer = item_offer(&owner, item);
    let attempted = TableOperation::PhysicalFact {
        acceptance: Box::new(PhysicalFactAcceptance {
            offer: offer.clone(),
            input: PhysicalFactInput::Item {
                choice: PhysicalItemInput::Unit {
                    pounds: "99".into(),
                    condition: None,
                },
                reason: "Unrestricted silent source override".into(),
            },
        }),
    };
    assert!(owner.prepare_table(&meta(&owner), &attempted).is_err());
    accept(
        &mut owner,
        offer,
        PhysicalFactInput::Item {
            choice: PhysicalItemInput::Catalog { condition: None },
            reason: "The exact original object is an ordinary catalog dagger.".into(),
        },
    );
    assert_eq!(owner.read().physical_load(actor).unwrap().known_pounds, "1");
    assert!(
        !owner
            .image
            .physical_sources
            .contains(&owner.image.state.items[&item])
    );
    let before = owner.read().state().items[&item].clone();
    let offer = item_offer(&owner, item);
    accept(&mut owner,offer,PhysicalFactInput::Item { choice:PhysicalItemInput::NonstandardUnit { description:"This particular original object's enlarged pommel is a nonstandard physical variant.".into(),pounds:"3.25".into() },reason:"Correct the prior ordinary applicability classification.".into() });
    assert_eq!(
        owner.read().physical_load(actor).unwrap().known_pounds,
        "3.25"
    );
    assert_eq!(owner.read().state().items[&item], before);
    assert!(
        owner
            .read()
            .state()
            .physical_facts
            .as_ref()
            .unwrap()
            .get(PhysicalSubject::Item(item))
            .unwrap()
            .previous
            .is_some()
    );
}
#[test]
fn mass_only_proof_is_candidate_bound_and_cannot_supply_grapple_or_raw_clone_authority() {
    let (mut owner, actor, _) = owner();
    enable(&mut owner);
    let command = meta(&owner);
    let mut candidate = Box::new(owner.image.state.clone());
    let mut execution = ExecutionContext::ordinary();
    execution.attach_mass(&owner.image, &candidate, &command);
    assert!(!execution.is_owned());
    execution
        .read(&candidate)
        .unwrap()
        .validate_physical_facts()
        .unwrap();
    assert!(
        execution
            .read(&candidate)
            .unwrap()
            .require_guarded("No Grapple activation")
            .is_err()
    );
    let clone = candidate.clone();
    assert!(execution.read(&clone).is_err());
    assert!(
        ReadContext::ordinary(&candidate)
            .validate_physical_facts()
            .is_err()
    );
    assert!(
        CampaignExecution::from_original_anchor((*candidate).clone(), owner.pack.clone()).is_err()
    );
    let offer = owner
        .read()
        .physical_fact_offers()
        .unwrap()
        .into_iter()
        .find(
            |offer| matches!(offer,PhysicalFactOffer::Body { actor:subject,.. } if *subject==actor),
        )
        .unwrap();
    let acceptance = PhysicalFactAcceptance {
        offer,
        input: PhysicalFactInput::Body {
            pounds: Some("150".into()),
            reason: "Actual unladen body fact.".into(),
        },
    };
    execution
        .accept_physical_fact(&mut candidate, &command, &acceptance)
        .unwrap();
    execution.validate_delta(&candidate).unwrap();
    candidate.physical_facts.as_mut().unwrap().records.clear();
    assert!(
        execution.validate_delta(&candidate).is_err(),
        "a current accepted fact cannot disappear from the produced collection"
    );
}
#[test]
fn producer_delta_refuses_forged_created_item_or_whole_attachment_disappearance() {
    let (mut owner, actor, _) = owner();
    enable(&mut owner);
    let offer=owner.read().physical_fact_offers().unwrap().into_iter().find(|offer|matches!(offer,PhysicalFactOffer::PersonalItem { actor:subject,.. } if *subject==actor)).unwrap();
    let command = meta(&owner);
    let mut candidate = Box::new(owner.image.state.clone());
    let mut proof = MassCommand::new(&owner.image, &candidate, &command);
    proof
        .apply(
            &mut candidate,
            &command,
            &PhysicalFactAcceptance {
                offer,
                input: PhysicalFactInput::PersonalItem {
                    name: "Distinct additional cloak".into(),
                    pounds: "2.5".into(),
                    separate_from_listed: true,
                    reason: "A separate untracked physical object.".into(),
                },
            },
        )
        .unwrap();
    proof.validate_delta(&candidate).unwrap();
    let retained = candidate.items[&ItemId(command.id.0)].clone();
    candidate.items.get_mut(&retained.id).unwrap().quantity = 2;
    assert!(proof.validate_delta(&candidate).is_err());
    candidate.items.insert(retained.id, retained.clone());
    candidate.items.remove(&retained.id);
    assert!(proof.validate_delta(&candidate).is_err());
    candidate.items.insert(retained.id, retained);
    candidate.physical_facts = None;
    assert!(proof.validate_delta(&candidate).is_err());
    assert!(
        physical_facts::enabled(owner.read().state()),
        "refused candidate mutations cannot alter the closed predecessor"
    );
}

fn operation(owner: &mut CampaignExecution, operation: TableOperation) {
    let command = meta(owner);
    owner.prepare_table(&command, &operation).unwrap().commit();
}
fn coverage(owner: &mut CampaignExecution, actor: EntityId) {
    let offer=owner.read().physical_fact_offers().unwrap().into_iter().find(|offer|matches!(offer,PhysicalFactOffer::Coverage { actor:subject,.. } if *subject==actor)).unwrap();
    accept(
        owner,
        offer,
        PhysicalFactInput::Coverage {
            complete: true,
            reason: "All distinct physical payload is listed.".into(),
        },
    );
}
fn classify(owner: &mut CampaignExecution, item: ItemId, choice: PhysicalItemInput) {
    let offer = item_offer(owner, item);
    accept(
        owner,
        offer,
        PhysicalFactInput::Item {
            choice,
            reason: "Actual applicability for this original physical object.".into(),
        },
    );
}
#[test]
fn original_nested_containers_count_each_object_once_and_refuse_inclusive_gross_over_children() {
    let (original, actor, dagger) = owner();
    // This ordinary original anchor authors only physical containment. It carries
    // no source grants, catalog classifications or accepted mass authority.
    let mut anchor = original.read().state().clone();
    let backpack = ItemId::new();
    let sack = ItemId::new();
    let waterskin = ItemId::new();
    let water = ItemId::new();
    for (id, definition, name, custody) in [
        (backpack, "backpack", "Outer pack", Custody::Entity(actor)),
        (sack, "sack", "Inner sack", Custody::Container(backpack)),
        (
            waterskin,
            "waterskin",
            "Partly filled waterskin",
            Custody::Container(backpack),
        ),
        (
            water,
            "authored-water-payload",
            "Separately tracked water",
            Custody::Container(waterskin),
        ),
    ] {
        anchor.items.insert(
            id,
            ItemInstance {
                id,
                campaign_id: anchor.campaign_id(),
                definition_id: definition.into(),
                display_name: name.into(),
                quantity: 1,
                owner: Ownership::Entity(actor),
                custody,
                state: ItemState::Intact,
            },
        );
    }
    anchor.items.get_mut(&dagger).unwrap().custody = Custody::Container(sack);
    let mut owner = CampaignExecution::from_original_anchor(anchor, original.pack.clone()).unwrap();
    enable(&mut owner);
    for item in [backpack, dagger] {
        classify(
            &mut owner,
            item,
            PhysicalItemInput::Catalog { condition: None },
        );
    }
    // The generic sack is outside this bounded source catalog. Its half-pound
    // weight is an explicit fact about this authored object, never a source claim.
    classify(
        &mut owner,
        sack,
        PhysicalItemInput::Unit {
            pounds: "0.5".into(),
            condition: None,
        },
    );
    classify(
        &mut owner,
        water,
        PhysicalItemInput::Unit {
            pounds: "2".into(),
            condition: None,
        },
    );
    coverage(&mut owner, actor);
    let before = owner.read().state().clone();
    for choice in [
        PhysicalItemInput::Catalog {
            condition: Some("full".into()),
        },
        PhysicalItemInput::Gross { pounds: "5".into() },
    ] {
        let operation = TableOperation::PhysicalFact {
            acceptance: Box::new(PhysicalFactAcceptance {
                offer: item_offer(&owner, waterskin),
                input: PhysicalFactInput::Item {
                    choice,
                    reason: "Inclusive amount cannot count separately tracked contents twice."
                        .into(),
                },
            }),
        };
        let command = meta(&owner);
        assert!(owner.prepare_table(&command, &operation).is_err());
        assert_eq!(owner.read().state(), &before);
    }
    let unresolved = owner.read().physical_load(actor).unwrap();
    assert_eq!(unresolved.known_pounds, "8.5");
    assert!(!unresolved.complete);
    assert_eq!(unresolved.items.len(), 5);
    // This is an explicit measured nonstandard EMPTY shell fact, not an inferred
    // subtraction from the printed full waterskin or a change to source content.
    classify(&mut owner,waterskin,PhysicalItemInput::NonstandardUnit { description:"This particular nonstandard empty shell weighs one quarter pound, excluding the tracked water.".into(),pounds:"0.25".into() });
    let load = owner.read().physical_load(actor).unwrap();
    assert!(load.complete);
    assert_eq!(load.known_pounds, "8.75");
    assert_eq!(load.items.len(), 5);
}
#[test]
fn authenticated_wallet_lot_never_uses_spent_ammunition_zero_mass_shortcut() {
    let (mut owner, _, _) = owner();
    let player = PlayerId::new();
    let actor = EntityId::new();
    let character = CharacterId::new();
    operation(
        &mut owner,
        TableOperation::AddPlayer {
            id: player,
            name: "Coin holder".into(),
        },
    );
    operation(
        &mut owner,
        TableOperation::CreateCharacterFromSource {
            character_id: character,
            entity_id: actor,
            player_id: player,
            source: crate::current_character_creation_source().unwrap(),
            input: crate::CharacterCreationInput {
                name: "Measured PC".into(),
                pronouns: "they/them".into(),
                description: "A traveler".into(),
                alignment: "Neutral Good".into(),
                backstory: "A private aspiration".into(),
                ability_scores: [15, 14, 13, 8, 10, 12],
                background_boosts: [2, 0, 1, 0, 0, 0],
                fighter_skills: [Skill::Perception, Skill::Survival],
                human_skill: Skill::Insight,
                skilled_skills: [Skill::Acrobatics, Skill::Stealth, Skill::Investigation],
                size: CharacterSize::Medium,
                languages: ["dwarvish".into(), "elvish".into()],
                fighting_style: FightingStyle::Defense,
                gaming_set: GamingSet::Dice,
                purchases: vec![
                    EquipmentChoice {
                        item_id: "leather-armor".into(),
                        quantity: 1,
                    },
                    EquipmentChoice {
                        item_id: "arrows".into(),
                        quantity: 20,
                    },
                ],
                worn_armor: Some("leather-armor".into()),
                shield: false,
                masteries: ["club".into(), "dagger".into(), "shortbow".into()],
            },
        },
    );
    let plan = crate::tactical_inventory::starting_equipment_plan(
        owner.read().state(),
        character,
        &owner.pack,
    )
    .unwrap();
    operation(
        &mut owner,
        TableOperation::PrepareEquipment {
            character_id: character,
            item_ids: (0..plan.identity_count).map(|_| ItemId::new()).collect(),
        },
    );
    enable(&mut owner);
    let value = owner
        .read()
        .state()
        .table
        .as_ref()
        .unwrap()
        .character_profiles[&character]
        .money_cp;
    assert!(value > 0);
    assert_eq!(value % 100, 0);
    let offer=owner.read().physical_fact_offers().unwrap().into_iter().find(|offer|matches!(offer,PhysicalFactOffer::Currency { character:subject,.. } if *subject==character)).unwrap();
    accept(
        &mut owner,
        offer,
        PhysicalFactInput::Currency {
            coins: PhysicalCoins {
                cp: 0,
                sp: 0,
                ep: 0,
                gp: value / 100,
                pp: 0,
            },
            reason: "These actual gold coins realize the existing unchanged wallet.".into(),
        },
    );
    coverage(&mut owner, actor);
    assert!(owner.read().physical_load(actor).unwrap().complete);
    let source = owner.read().state();
    let lot = source
        .items
        .values()
        .find(|item| item.definition_id == CURRENCY_LOT_DEFINITION)
        .unwrap()
        .id;
    let arrows = source
        .items
        .values()
        .find(|item| item.definition_id == "arrows")
        .unwrap()
        .id;
    let command = meta(&owner);
    let mut candidate = Box::new(source.clone());
    let mut execution = ExecutionContext::ordinary();
    execution.attach_mass(&owner.image, &candidate, &command);
    // Negative applicability probes, never accepted gameplay or a claimed spend:
    // retaining the genuine facts cannot turn an altered physical lot into cash.
    for (quantity, state) in [
        (0, ItemState::Spent),
        (0, ItemState::Intact),
        (1, ItemState::Intact),
        (source.items[&lot].quantity, ItemState::Damaged),
    ] {
        let item = candidate.items.get_mut(&lot).unwrap();
        item.quantity = quantity;
        item.state = state;
        let load = physical_facts::load(&execution.read(&candidate).unwrap(), actor).unwrap();
        assert!(!load.complete);
        assert!(load.unresolved.iter().any(|entry| entry.contains("wallet")));
    }
    let changed_definition = candidate.items.get_mut(&lot).unwrap();
    changed_definition.definition_id = "arrows".into();
    changed_definition.quantity = 0;
    changed_definition.state = ItemState::Spent;
    assert!(
        !physical_facts::load(&execution.read(&candidate).unwrap(), actor)
            .unwrap()
            .complete
    );
    candidate.items.insert(lot, source.items[&lot].clone());
    let item = candidate.items.get_mut(&arrows).unwrap();
    item.quantity = 0;
    item.state = ItemState::Spent;
    let ordinary_spent = physical_facts::load(&execution.read(&candidate).unwrap(), actor).unwrap();
    assert!(ordinary_spent.complete);
    assert_eq!(
        ordinary_spent
            .items
            .iter()
            .find(|item| item.item == arrows)
            .unwrap()
            .pounds
            .as_deref(),
        Some("0")
    );
    assert_eq!(
        owner.read().state(),
        source,
        "none of these probes commits a mutation"
    );
}

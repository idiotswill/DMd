# ADR 018 — Typed rules kernel and semantic replay

Status: **Accepted — Gate 2, [PR #17](https://github.com/idiotswill/DMd/pull/17), exact head `ca54b8215258468db4fbac7b5c15a648fdbb98d7`, [CI #303](https://github.com/idiotswill/DMd/actions/runs/36002412992); production integration verified in [PR #18](https://github.com/idiotswill/DMd/pull/18).**

## Problem and contract

Gate 2 needs mechanically faithful foundations on the existing campaign path. Arbitrary provider totals, free-form state patches, and campaign-specific rules cannot become authority. SRD 5.2.1 is the pinned licensed source; the source-ledger slice owns complete coverage accounting and attribution. This ADR advances the product rules fidelity, physical dice, explicit adjudication, local authority and exact-suspension requirements. It does not claim a complete tactical/noncombat game.

## State and boundaries

`dmd-domain` owns serializable mechanical state and the existing dice records. `CampaignState.rules` is absent until an explicit privileged initialization of existing world entities. Trusted imported character sheets supply bounded base scores, granted content IDs, armor formulas, proficiency, spell slots and resources. Modifiers, totals, AC, slot expenditure, roll success, damage, healing and timing are derived in `dmd-rules`. The character creation workflow belongs to Gate 3, progression and multiclass choices to Gate 5, and complete character option catalogs and grant data to Gate 6. Callers must not present `MechanicalEntity::basic` as completed character creation.

The rules crate has no SQL, application, conversation, network, RNG, or provider dependency. `resolve(state, meta, action, pack)` validates input, works on a clone, and returns one `RulesTransition`; failures have no mutation. `RulesPack::from_json` accepts only the pinned ID/version and a known schema with typed, bounded definitions. The application must independently verify the immutable content manifest and campaign pin before resolving; a structurally valid pack is not proof of file authenticity.

The kernel intentionally leaves `applied_event_sequence` unchanged. Persistence owns sequence allocation; the application increments its commit image by exactly one using checked arithmetic, and replay persistence advances the replay image after each event. No UUID or random face is generated within the resolver. Follow-up requests use the already-recorded command UUID wrapped as a roll-request ID. Used and canceled request IDs cannot be recycled.

## Authority and adjudication

Players choose owned actor/content IDs and raw die faces. They cannot supply DCs, modifiers, critical hits, resource grants, effects, damage amounts, or successful after-states. `CommandIssuer` remains separate from actor. Digital or secret results require System/Admin; every supplied actor must still match the pending roller. Read ownership permits inspection of a player's dead/retired characters, while action eligibility requires an active controller and mechanically capable actor.

Privileged actions carry a `Ruling` identifying a source page, enabled known house rule, or explicit GM adjudication plus a nonempty explanation. There is one optional house-rule switch, disabled by default: natural 1/20 overrides for ordinary checks/saves. Attack and death-save natural extremes follow their distinct source rules regardless of this switch. The generic `UseBonusAction` and `UseReaction` operations consume a budget under an explicit trusted feature/trigger ruling; they do not invent the feature's mechanical effect.

Until Gate 4 owns authoritative spatial/perceptual derivation, attacks/spells need a one-use privileged contextual permission. The permission identifies actor, target, content, source ruling and issuer command. It is consumed only by the immediately following matching action. `within_five_feet` is distance to the target; `Circumstances.ranged_threat` separately records whether a hostile, non-incapacitated creature within 5 feet can see the attacker. Contextual visibility, frightened-source sight and other situational advantage/disadvantage are explicit adjudication inputs. No arbitrary modifier or final total is accepted through this boundary.

## Dice and exact continuation

Pending requests, typed purposes, originating commands, content/context causes, accepted original/final raw inputs and resolved totals are saved in mechanical state. Restore validation reconstructs pending dice/modifiers/mode from those causes. Damage requests must follow a recorded successful attack, and the attack is revalidated against the content definition. Healing requests retain spell and slot level; concentration and hit-die requests retain their rule-specific prerequisites. Unrelated mechanical mutation is rejected while a roll is pending.

Heroic Inspiration is a non-stacking resource. `SubmitRollWithInspiration` records the initial raw faces plus one replacement die, spends the resource, and resolves using the replacement even when worse. This is the immediate choice boundary before any result becomes authoritative, suitable for physical table dice. Digital presentation/choice windows are application/UI work; callers must not claim a post-commit arbitrary rollback or a complete digital Inspiration UX from this primitive.

Queries are read-only. Pending secret requests are invisible to players; private/public pending requests are exposed only to their controller through the query API. Full authoritative state/events are trusted application data, not player-safe view models. The application is responsible for never sending raw campaign state or secret event outcomes to untrusted clients.

## Mechanical scope and precise deferrals

Implemented foundations include score/proficiency/expertise derivation, checks/saves, advantage cancellation, passives, weapon/spell attack rolls, natural extremes, critical damage dice, resistances/vulnerability/immunity ordering, temporary HP, healing, zero HP/death saves, conditions, concentration, bounded effects, generic resources, initiative/turn/action/bonus/reaction budgets and rest recovery. Heavy armor explicitly ignores Dexterity; light/medium/unarmored formulas retain negative Dexterity. Unconsciousness leaves independent Prone state after healing. Rest hit-die spending is a completion window that closes on unrelated time/activity; taking zero damage does not interrupt rest.

Gate 4 retains complete tactical geometry, movement/standing costs, ranges/cover/line of sight, threat derivation, surprise and hidden-actor perception, complete condition clauses, weapon properties/masteries, reactions/ready/opportunity attacks, simultaneous/ongoing effects, multi-target effects, stabilization/first aid, death/knockout workflows and full combat action catalogs. The kernel's typed conditions do not imply all these clauses are automated. General save-producing environmental/magical effects can request privileged typed saves, but complete saving-throw/area spell pipelines remain that gate's work.

Specifically, Invisible initiative advantage is derived, while whether an attacker's invisibility is defeated by a target's special perception is supplied through privileged circumstances. Frightened source line-of-sight is likewise contextual; the kernel does not impose its disadvantage when the source cannot be seen. Whether a check requires sight or hearing under Blinded/Deafened, Grappled/Restrained movement restrictions and escape actions, Charmed social-context clauses, prone standing/movement costs, and interactions with terrain/perception remain with the complete spatial/noncombat systems. The primitive cannot decide the applicability of those missing contexts, and callers must not label these conditions fully automated.

Gate 5 retains full rest interruption/resumption scheduling (including the extra hour per long-rest interruption), autonomous sleep/light-activity tracking, ritual/long-casting workflows, exploration/travel/downtime/environmental rules, noncombat care and elapsed-time recovery of stable creatures. The primitive permits starting a new rest after interruption; it does not silently resume the previous rest or claim that workflow complete. Reduced ability-score/max-HP restoration, multiclass hit-die selections and class-specific recovery are not represented by the current base-sheet schema and belong to their full effects/content systems.

The sample definitions are a deliberately limited licensed catalog: Club, Dagger and Shortbow single-attack damage; Cure Wounds healing/upcasting; Fire Bolt creature-attack damage/cantrip scaling; Dancing Lights concentration/duration only. Fire Bolt ignition of objects and Dancing Lights actual lights, placement, visibility and movement are deferred to spatial/world systems. `SupportedContent` returns typed spell support metadata so a client can distinguish these partial primitives. Complete catalogs, character classes/features and adventure content remain Gate 6. A concentration marker is not a completed spell effect.

## Replay and save integrity

`rules.action_resolved` version 1 contains the original trusted command metadata, typed action (including raw dice inputs) and derived outcome. It never carries an arbitrary replacement campaign state. `replay` re-runs the resolver and compares the complete recomputed event/outcome. Wrong campaign/head/actor, unsupported content, malformed data, impossible requests and mismatched outcomes fail closed.

Mechanical restore validation also checks reference/identity/range invariants, contextual/ruling command authority, duplicate/canceled roll IDs, raw roll arithmetic, single-die Inspiration replacement and timing/effect relationships. Local structural consistency cannot authenticate a coherently forged journal or snapshot. The application/persistence composition must bind retained command provenance to audited journal records and replay available history, including exported/imported snapshots; raw deserialization is not acceptance.

## Evidence and costs

The mechanics tests exercise real pure transitions and replay each multi-step scenario, including hostile inputs and restored-state forgeries. Source-review regressions cover negative Dexterity, close spell attacks, conditional ranged threats, concentration DC caps, death/prone continuity, initiative conditions, rest interruptions and invalid expiry mappings. PR #18 adds verified durable commit/reopen/export/replay and content/version failure tests; the Gate 2 checkpoint records exact production evidence and scope limits.

The initial representation retains roll/ruling history and clones the mechanical snapshot for validation/resolution. This is correctness-first work. Bounded archives/projections and performance tuning remain measurable debt for Gates 7/13/14; do not substitute this representation for endurance evidence.

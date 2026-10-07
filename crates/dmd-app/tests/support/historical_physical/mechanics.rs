//! Independent concrete expectations for the unchanged old-build corpus.
use super::*;
fn flow(s: &CampaignState) -> &TacticalFlow {
    s.encounter.as_ref().unwrap().flow.as_ref().unwrap()
}
fn timing(s: &CampaignState) -> &CombatTiming {
    s.rules.as_ref().unwrap().timing.as_ref().unwrap()
}
pub fn captured(a: &archive::Archive) {
    if a.name == "ground-v4" {
        let paid = decoded(&a.cuts[3].after);
        let issued = decoded(&a.cuts[4].after);
        let released = decoded(&a.cuts[5].after);
        let answered = decoded(&a.cuts[6].after);
        let mover = timing(&paid).order[0].actor;
        let target = timing(&paid).order[1].actor;
        let reactor = timing(&paid).order[2].actor;
        assert!(timing(&paid).action_spent);
        assert!(timing(&paid).reactions_spent.is_empty());
        assert_eq!(timing(&issued).reactions_spent, vec![reactor]);
        assert_eq!(timing(&released), timing(&issued));
        assert_eq!(timing(&answered), timing(&issued));
        let progress = flow(&paid).resolution.as_ref().unwrap();
        let prefix = &progress
            .grapple
            .as_ref()
            .unwrap()
            .transport
            .as_ref()
            .unwrap()
            .steps;
        assert_eq!(prefix.len(), 1);
        assert_eq!(prefix[0].holder.actor, mover);
        assert_eq!(prefix[0].target.actor, target);
        assert_eq!(prefix[0].holder.from, SpatialPoint { x: 10, y: 10, z: 0 });
        assert_eq!(prefix[0].holder.to, SpatialPoint { x: 0, y: 10, z: 0 });
        assert_eq!(prefix[0].target.from, SpatialPoint { x: 20, y: 10, z: 0 });
        assert_eq!(prefix[0].target.to, SpatialPoint { x: 10, y: 10, z: 0 });
        assert_eq!((prefix[0].ordinary_cost, prefix[0].haul_cost), (10, 10));
        assert_eq!(progress.movement.as_ref().unwrap().next_step, 1);
        assert_eq!(flow(&paid).budget.movement_spent, 20);
        let ir = flow(&issued).resolution.as_ref().unwrap();
        let rr = flow(&released).resolution.as_ref().unwrap();
        assert_eq!(ir.attack, rr.attack);
        assert_eq!(ir.frames, rr.frames);
        assert_eq!(
            &rr.grapple
                .as_ref()
                .unwrap()
                .transport
                .as_ref()
                .unwrap()
                .steps,
            prefix
        );
        assert_eq!(rr.grapple.as_ref().unwrap().ends.len(), 1);
        assert_eq!(
            rr.grapple.as_ref().unwrap().ends[0].cause,
            GrappleEndCause::Released
        );
        let pending = issued.rules.as_ref().unwrap().pending.as_ref().unwrap();
        assert_eq!(pending.request.roller, Some(reactor));
        assert_eq!(
            released.rules.as_ref().unwrap().pending.as_ref(),
            Some(pending)
        );
        let result = answered.rules.as_ref().unwrap().rolls.last().unwrap();
        assert_eq!(result.request, pending.request);
        assert_eq!(result.issued_by, pending.issued_by);
        assert_eq!(result.purpose, pending.purpose);
        assert_eq!(result.result.source, RollSource::Physical);
        assert_eq!(
            result.result.dice,
            vec![DieResult {
                sides: 20,
                value: 1
            }]
        );
        assert_eq!(result.resolved.total, 2);
        assert_eq!(
            answered.rules.as_ref().unwrap().rolls.len(),
            issued.rules.as_ref().unwrap().rolls.len() + 1
        );
        assert_eq!(answered.items, paid.items);
        assert_eq!(released.items, issued.items);
        for actor in [mover, target, reactor] {
            assert_eq!(
                answered.rules.as_ref().unwrap().entities[&actor].hp,
                paid.rules.as_ref().unwrap().entities[&actor].hp
            );
        }
        let end = flow(&answered).last_movement.as_ref().unwrap();
        assert_eq!(
            (end.requested_steps, end.completed_steps, end.spent_after),
            (2, 1, 20)
        );
        assert_eq!(end.reason, TacticalMovementEnd::Stopped);
        assert_eq!(end.endpoint, prefix[0].holder.to);
        assert_eq!(
            end.transport.as_ref().unwrap().target_endpoint,
            prefix[0].target.to
        );
        assert!(flow(&answered).resolution.is_none());
    } else {
        let before = decoded(&a.cuts[4].before);
        let after = decoded(&a.cuts[4].after);
        let owner = before
            .rules
            .as_ref()
            .unwrap()
            .pending
            .as_ref()
            .unwrap()
            .request
            .roller
            .unwrap();
        assert!(before.rules.as_ref().unwrap().entities[&owner].heroic_inspiration);
        assert!(!after.rules.as_ref().unwrap().entities[&owner].heroic_inspiration);
        let pending = before.rules.as_ref().unwrap().pending.as_ref().unwrap();
        assert!(
            matches!(pending.purpose,PendingPurpose::TacticalResolution {key,..} if key.role==TacticalRollRole::GrappleSave)
        );
        assert_eq!(pending.request.reason, "Grapple saving throw");
        let visible = a.cuts[4]
            .before_views
            .iter()
            .filter_map(|v| v.presented.roll.as_ref())
            .collect::<Vec<_>>();
        assert!(!visible.is_empty());
        for roll in visible {
            assert_eq!(roll.reason, "Unsupported roll");
        }
        for v in &a.cuts[4].before_views {
            if let Some(roll) = &v.raw.roll {
                assert_eq!(roll.reason, "Unsupported roll");
            }
        }
        let result = after.rules.as_ref().unwrap().rolls.last().unwrap();
        assert_eq!(result.request, pending.request);
        assert_eq!(result.purpose, pending.purpose);
        assert_eq!(result.issued_by, pending.issued_by);
        assert_eq!(
            result.original_result.as_ref().unwrap().source,
            RollSource::Physical
        );
        assert_eq!(
            result.original_result.as_ref().unwrap().dice,
            vec![DieResult {
                sides: 20,
                value: 1
            }]
        );
        assert_eq!(result.result.source, RollSource::Physical);
        assert_eq!(
            result.result.dice,
            vec![DieResult {
                sides: 20,
                value: 20
            }]
        );
        assert_eq!((result.request.modifier, result.resolved.total), (5, 25));
        assert_eq!(
            after.rules.as_ref().unwrap().rolls.len(),
            before.rules.as_ref().unwrap().rolls.len() + 1
        );
        assert_eq!(after.items, before.items);
        let other = match a.cuts[4].after_views[2].channel {
            TableTransportChannel::Player { character_id, .. } => {
                after.characters[&character_id].entity_id
            }
            _ => panic!("original unrelated player channel"),
        };
        assert_ne!(owner, other);
        assert_eq!(
            after.rules.as_ref().unwrap().entities[&other].heroic_inspiration,
            a.name == "inspiration-recipient-v4"
        );
    }
}

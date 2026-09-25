# Gate 4 lethal completion and dead-target admission regression

Writer: bootstrap_audit. Branch `codex/gate4-dead-target-recovery`, from reviewed
PR33 draft `b7f5dfb2c03c346c7de3f159530460b8e4915167`. Root alone owns the production
admission fix. This slice owns an application test and its registration only.

## Scope and acceptance

The combined review found that an ordinary attack on a located already-dead target
could spend its Action and open dice that the living vitality reducer cannot finish.
Preserve Gate4's source-faithful damage and atomic recovery contracts, root AGENTS,
ADR024/026, and player-authority/product persistence requirements. A pre-cost
admission guard must not become a historical validator that rejects legitimate
lethal completion. This test does not implement corpse/object rules or expose a
secret death flag by filtering player contacts.

Use only actual TableAction commands against a file SQLite database: supported PC
creation buys a Dagger, source Goblin equipment is materialized, a normal Attack
equips the weapon and requests real critical damage, then the controller chooses
normal lethal damage. Verify the original lethal completion restores and replays.
End turns normally until the PC has its next Action. Attempt a fresh ordinary
attack at the same located target with a new command identity. Prove rejection
without any durable export change or action/equipment cost. Reopen and restore
again; the valid lethal history must remain intact.

Compare all durable export fields exactly for the same database, normalizing only
the request-time `exported_at_utc`. Heap-pin large scenario phases and keep the
default Windows stack. Do not inject HP/death/gear, bypass source choices, or edit
production. Root will supply the reviewed admission-fix commit before compilation.

## Verification and next action

Plan committed before test implementation. The complete boxed file-SQLite case is
now authored in `crates/dmd-app/tests/support/table_dead_target_cases.rs`, reusing
the actual public source-scene and raw-roll helpers. It also cold-retries the valid
original lethal declaration before advancing turns, then reopens after the rejected
fresh declaration and checks replay/portable restore again. Formatting and diff
checks pass. Independent exact test review is clear. Root integrated the scenario
as `9910ce7` with admission correction `a58d4dc`. First compilation caught the
next-turn helper taking an immutable fixture although the host helper needs a
mutable reference; its signature and call were corrected without gameplay changes.
The actual file-SQLite test then passed on the default Windows stack: one passed,
24 unrelated cases filtered, 9.66 seconds. Log: `../tooling/pr33-dead-target-app-r2.log`.
All lethal completion, cold retry, next-turn, no-write rejection, replay and restore
assertions ran. Root now owns combined canonical/UI/CI verification and integration.
Combined code/test6291776 subsequently passes canonical verification (595 local
Windows GNU tests), desktop45/check/build and all six CI checks: Linux36133381708
(596 Rust tests), Windows36133381673 (598 native Rust tests, MSRV and installer).
This real recovery scenario is included in each Rust run. Final evidence-head
review/CI, protected merge and post-merge verification remain in the integration
plan. Gate4 remains active; this is not whole-gate acceptance.

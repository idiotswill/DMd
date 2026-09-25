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
checks pass; **the new test is uncompiled** pending root's production fix. Root now
owns the compiler and will integrate this test into its focused/canonical batch,
avoiding a second conflicting build. Read-only review and actual test evidence remain
required before acceptance. Root owns PR integration and final gate decisions.
Gate4 remains active; no native or whole-gate acceptance is claimed.

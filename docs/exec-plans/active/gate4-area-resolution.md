# Gate 4 source area saving throws

Writer: bootstrap_audit. Branch: `codex/gate4-area-resolution`, based on verified
integration `220ee804d7344564a265f02c4d42cb57a7e560c2`. Movement owns the shared
falling/turn cursor until its coherent handoff; environment_audit owns attack
modules and source physical weapon actions. This writer starts with new domain
and rules area modules, then coordinates the single shared resolution integration.

## Objective and source

Resolve real canonical creature SaveArea actions through authoritative spatial
binding, one shared raw damage roll, separate target saves and defenses, and
individual vitality/concentration consequences. Preserve finite source activation,
raw controller authority, exact suspension/replay and private target information.

The initial complete source programs are Young Red Dragon Fire Breath (SRD5.2.1
p318:30ft Cone, Dexterity DC17,16d6 Fire, half on success), Adult Red Dragon Fire
Breath (p319:60ft Cone, Dexterity DC21,17d6 Fire, half), and Chimera Fire Breath
(p273:15ft Cone, Dexterity DC15,7d8 Fire, half). All recharge on5–6. Canonical
definitions already contain these programs; no source catalog or invented grant
is needed. Own-turn direct Action activation is the first admission; source
Multiattack replacement will reuse the same proof after its enclosing continuation
is implemented, and remains active Gate4 work.

SRD16 requires one damage roll when the same effect forces simultaneous saves,
with half damage rounded down; p17 orders damage adjustments before resistance
and vulnerability. Areas and origin-to-area blocking are p177, Cone p179, source
recharge p257, and simultaneous effect ordering p187. Existing opt-in natural1/20
save policy must agree with other tactical saves; default RAW uses the total.

Relevant contract: root AGENTS, product-definition tactical timing/privacy/normal
intent and production integration requirements, Gate4 checkpoint, ledger rows
`targeting-areas`, `combat-conditions`, `monster-running` and `spell-effects`, and
the eighteen-family checklist in `gate-4-effect-lifecycle.md`. SaveArea is one
mechanism slice, not completion of those rows or Gate4.

## Planned slices and ownership

1. Define source-bound area choice and durable area occurrence contracts in new
   domain/rules modules. Client choices contain area aim only, never victim IDs,
   save DC, damage, source counters or a prepaid flag. Bind the real profile and
   canonical feature to a genuine scheduler activation receipt before costs.
2. Use existing area/cover geometry and record any required boundary ruling rather
   than converting a preview into authority. Source area membership includes all
   actually affected creatures, including allies and unseen targets. Public
   declaration/preview must not leak unseen actors through target lists, errors,
   outcome summaries, raw requests or simultaneous-work choices.
3. After shared-file handoff, compose new work into TacticalResolution frames.
   Source Action and recharge expenditure occur once; one immutable damage result
   feeds each distinct target. Each target save, Legendary Resistance decision,
   ordinary defenses, temporary HP/death and concentration child completes under
   existing raw/controller/provenance rules before parent work resumes. No second
   queue, arbitrary effect patch, automatic PC choice or generated die face.
4. Add exact partition/provenance validation and replay tests at every pending
   stage, then expose through the existing table envelope in a coordinated app
   follow-up. Root owns integrated canonical/CI/desktop acceptance.

## Acceptance and verification

- Public source-created/materialized creature actions derive printed dimensions,
  DC, dice and recharge. Wrong source, depleted recharge, malformed area, foreign
  authority and unsupported programs reject without action/resource mutation.
- Geometry covers boundaries, positive occupied volume, different creature sizes,
  origin exclusion/inclusion and Total Cover; the declaration cannot identify or
  probe a hidden target. Spatial policy and any ambiguity are explicit.
- One raw amount is reused across mixed save outcomes, resistance/vulnerability,
  zero HP and multiple independent concentration saves; no attack critical or
  knockout path is invented for a saving-throw effect.
- RAW and explicit house natural extremes, automatic/voluntary save failure and
  source Legendary Resistance retain actual accepted dice or explicit no-roll
  decisions, with correct controller and causal metadata.
- Save/reconstruct/replay every pending stage. Reject duplicate/missing victims,
  reused amount rolls, forged source/geometry/origins and competing work without
  mutating the input. Coordinate app origin/label additions before integration.
- Run focused domain/rules tests and strict all-target Clippy only after obtaining
  the serialized Rust slot. Run format/diff checks, independent full review and
  exact final delta review. Root performs full canonical and real SQLite/native
  checks; no helper-only or unrun native acceptance claim.

## Current state and remaining scope

Root approved an optional host-selected encounter policy, absent in old states;
absence rejects area activation before costs. `OccupiedCellCentersV1` samples the
center of each occupied part of a5ft horizontal cell and vertical band. Tiny spaces
and partial top bands use their occupied midpoint, rounded toward the minimum if
it falls between half-foot coordinates. One in-shape sample with a clear effect
path includes the creature. Total-cover terrain blocking at least half or three
quarters of all occupied samples grants Half or ThreeQuarters cover; authored
cover and intervening creature cover combine by taking the greatest grade. A fully
blocked target is excluded. This named map adjudication avoids hidden-victim-
dependent requests for extra cover rulings; it is not a source-prescribed ray
percentage and does not grant around-corner effects. GM setup must explain the
consequence in ordinary language and retain its accepted origin.

New domain/source/geometry/raw-amount contracts and ten source/geometry tests are
drafted. The tests use actual source
builds for all three admitted breaths and cover policy absence, malformed aim,
large/Tiny/tall bodies, cover thresholds and full exclusion, private identity/sight
independence, canonical raw amounts and explicit origin inclusion. Format and diff
checks pass; no compiler slot has been taken and no test pass is claimed. The
source/geometry leaf at652c857 received independent read-only review with no
concrete finding. Verified falling checkpoint ebbff59 was integrated as b89f4fd;
the movement author handed over shared queue ownership. The optional policy now
belongs to TacticalEncounter and TableBattlefieldSetup, with setup-origin equality
and old absent fields omitted from serialization.

The public CreatureArea source action, roles13/14 and areas[] record now stage one
amount, all target saves, a simultaneous damage frame and completion in the same
TacticalResolution. Fourteen public reducer tests now accompany the ten leaf
tests. They cover three source breath grants/costs/recharge, mixed defenses and
independent concentration, explicit house policy, automatic/voluntary/Legendary
Resistance saves, zero-HP source damage, raw/partition/source corruption, hidden
victims, stored cover, old policy bytes and pending physical consequences. All have passed locally. Target saving throws precede the simultaneous damage frame so
one target's death/concentration loss cannot retroactively change another save.

Parent approved two explicit admission boundaries after review. Any active sourced
Charmed restriction uniformly closes this bounded area path before geometry; no
cone-dependent hidden-charmer error or invented victim exemption is allowed.
This also rejects otherwise legal cones away from the charmer, and is an
unsupported Charmed-area adjudication path that must be resolved within Gate4,
not a claimed source rule. An idle encounter with any due fall similarly rejects
before aim/cost. Supported preceding transitions pump falling before returning
idle; area binding cannot accidentally precede an old falling consequence.
Thus pre-damage validation rebinds exact source geometry/cover and verifies the
initial source invocation. After damage starts it retains historical membership;
event replay from the accepted setup/pre-tactical anchor remains required proof.

The first compiler rejected sorting EntityId directly; the deterministic sort now
uses its UUID value. The source fixture initially passed level-zero NPC mechanics
through the legacy initializer without its profile; it now installs the actual
source build and profile together after basic initialization, matching the other
source fixtures. The runtime tests also corrected an invalid space-containing map
identifier and a corruption case that swapped an empty frame instead of the actual
save/phase frames. The existing whole-state validator already rejects unsupported
idle falling before area admission; a stronger real action scenario now proves
area damage causes falling, landing finishes, and another victim's concentration
pause retains the original area volume after displacement. No validation was
weakened to make a fixture pass.

Independent review also required completed-save requests to reconstruct their
source modifiers/mode/disposition while other saves remain pending. This is now
checked before damage and tested against an arithmetic-consistent forged modifier.
Post-damage requests remain historical facts authenticated by journal replay.

Local Windows verification on the final source tree:

- Ten area leaf tests passed (`research/gate4-areas/leaf-third.txt`).
- Fourteen public area cases passed (`runtime-fourth.txt`), including semantic
  replay and serialization at every accepted transition.
- The affected full suites passed:67 attack,30 movement/falling,24 turn tests
  (`shared-suite.txt`), including the fourteen area cases. Together with the leaf
  tests this is131 distinct rules tests.
- `cargo clippy -p dmd-domain -p dmd-rules --all-targets -- -D warnings` passed
  (`clippy-first.txt`); format/diff checks passed. The global compiler was released
  directly to the movement app writer after completion.

The geometry leaf and complete driver have independent source reviews; exact
final-delta review is requested after this evidence checkpoint. Root owns canonical
workspace/CI verification. App/UI area action/privacy/labels/origins and meaningful
GM policy explanation remain integration work, not completed acceptance. Existing
generic simultaneous cards must not expose one card per hidden victim. A playable
explicit controller delegation/ordering contract is required before area UI exposure;
source turn-control ownership must be preserved without a count/order oracle.

Next: final independent exact-head source review, then integrate application origin
proof and the private ordering contract plus real SQLite and desktop controls.
No packaged/native area acceptance has been run or claimed.

Remaining tactical work is not reassigned to Gate6. Fireball must also implement
its mandatory flammable unworn/uncarried-object ignition and Burning hazard
(pp131,178), not discard the second program node. Persistent zones, Ready,
Counterspell, richer spell programs and all other checklist mechanisms remain
active Gate4. Broad catalog completion belongs Gate6 and noncombat magic/rituals
remain Gate5. Native/table acceptance remains pending.

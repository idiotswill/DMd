# Gate 4 tactical damage and recovery

## Objective and scope

Verification branch `codex/gate4-damage-verified`, based on merged main `580f487`;
the original `codex/gate4-damage-resolution` source branch is retained. Root is the
sole writer of the verification branch. Implement an internal,
pure, source-derived HP/damage/death reducer and durable recovery contracts without
changing legacy kernel event semantics. The parent encounter slice owns authority,
continuation storage, atomic journal/application integration and effect-group removal.

Relevant contracts: product-definition rules fidelity and durable authoritative state;
Gate 4 damage/death, conditions and concentration criteria; accepted ADR 025 and
the integration branch's proposed ADR 026 interruptible encounter resolution.
Source: pinned SRD 5.2.1 pp. 16–18, 179, 187, 191.

## Acceptance criteria

- Damage adjustments precede resistance and vulnerability; temporary HP absorb damage
  without suppressing damage/concentration obligations; each occurrence remains distinct.
- Zero-HP failures, critical hits, massive damage, monster death and maximum-HP-zero
  death obey source rules. Healing cannot revive or clear unrelated condition causes.
- A controller explicitly chooses temporary HP replacement or eligible melee knockout.
  Knockout rests, first aid, stabilization and raw recovery/death dice are durable inputs.
- Invalid inputs leave original state unchanged; arithmetic, dice and records are bounded.
- Focused source regressions and strict Clippy pass in an agreed serial build slot.

## Slices and decisions

1. Agree public contracts with the parent; add domain types and internal reducer.
2. Add meaningful source/invalid-input regression tests; independent review.
3. Run focused verification, commit the coherent slice, hand integration to the parent.

HP and death remain on `MechanicalEntity`; recovery records contain only source-tagged
causes and timers. No RNG, player/admin authorization, attack resolution, legacy API
rewrites, resurrection, or duplicate HP/effect authority belongs in this module.

Source-specific decisions: SRD90 Graze is miss damage, with its own cause and exact
ability-modifier amount. Its restriction that damage can increase only through the
ability modifier bars ordinary bonuses, added dice and vulnerability doubling; ordinary
reductions, resistance and immunity still apply (SRD17). SRD17 condition immunity prevents
the knockout condition and its involuntary recovery rest, while the explicit attacker
choice still leaves the target at 1 HP. Damage interrupts a knockout rest (SRD187), but
does not end its Unconscious cause: SRD17 only names regained HP, successful first aid,
or completing the short rest. Stabilizing and ending knockout require distinct declared
Medicine purposes even though both checks have DC10.

## Verification and remaining work

Source passages and existing kernel damage/death helpers reviewed. The parent reviewed
the reducer branches and identified an immune-knockout forced-rest issue; it is corrected
and covered by a regression. Independent effect and weapon/scheduler authors reviewed
the complete source reducer and tests. Their same-command future-occurrence finding is
corrected, with immutable same-ID metadata and valid later-command history regressions.
The scheduler review also requested explicit voluntary death-save failure (SRD187); it
now records exactly one failure without inventing a natural die, and retains normal
eligibility/death-at-three checks. Final follow-up delta signoff remains pending. The
public contract was shared with the parent and turn-scheduler author before integration.

Verified on the final code tree in the agreed serial build slot:

- `cargo test -p dmd-rules tactical_damage --lib`: 30 passed.
- `cargo clippy -p dmd-domain -p dmd-rules --all-targets -- -D warnings`: passed.
- `cargo fmt --all -- --check`: passed.
- `git diff --check`: passed.

Regression coverage includes damage type grouping/rounding, the SRD17 arithmetic example,
Graze's specific restriction, temporary-HP absorption, concentration DC floor/cap and
occurrence identity, massive damage, zero-HP critical failures, monster death policy,
controller choices, raw natural1/20 death saves, stable raw1d4 recovery, interrupted rests,
owned-condition removal, later immunity without cause loss, corrupted/future provenance,
unknown JSON fields, replayable raw faces, integer/time bounds, and rejection without mutation.

Root integration obligations: persist optional recovery records outside encounters;
authenticate and journal each original operation exactly once; bind raw-request IDs and
controller choices to durable continuations; apply all follow-ups atomically; synchronize
world death; project only active recovery conditions; retain Prone after awakening; and
apply narrowly scoped tactical validation for source-valid dead maximum-HP-zero and
condition immunity. The parent owns full verification and production-path evidence.

The isolated six-file PR contains the reviewed `dbe8cf7` source plus reviewed `b21f2eb`
ordinary death-save success operation (31 focused damage tests passed on its source branch).
Successful/failed saves chosen by a rule do not fabricate a natural die face. No optional
RulesState attachment or old event/save interpretation changes are included in this PR.

Exact next action: run canonical full verification and exact-head CI on this main-based
branch, verify the complete final diff, and merge with expected-head protection. Then
refresh the parent integration onto main. This reducer slice does not claim end-user
damage support or Gate 4 acceptance before the production integration above.

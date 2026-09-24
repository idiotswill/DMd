# Technical debt ledger

This file tracks debt that the project has deliberately accepted across gates. Gate-specific open risks remain in their checkpoint review until they are accepted into the ongoing codebase.

Do not use this file as a generic TODO list. An entry belongs here when we knowingly ship/merge a compromise that should later be removed or strengthened.

## Entry format

```markdown
### TD-XXX — short title

Status: open | mitigated | closed
Introduced: <PR/gate>
Owner area: <subsystem>
Severity: low | medium | high

Why accepted:
...

Risk:
...

Exit criteria:
- ...
```

## Open debt

### TD-001 — durable gameplay replay coverage grows with event families

Status: open
Introduced: Gate 1 / PR #7
Owner area: persistence + gameplay event owners
Severity: medium

Why accepted:
Gate 1 establishes the mechanical snapshot/journal replay boundary and requires typed `ReplayEventApplier` semantics. It deliberately does not invent replay semantics for gameplay systems whose durable event contracts do not yet exist.

Risk:
A future durable event family is not fully recoverable from snapshot+journal history until its owning gameplay subsystem defines stable kind/version semantics and a compatible typed applier.

Exit criteria:
- every production durable gameplay event family has versioned replay semantics and regression coverage;
- save-compatibility policy defines how supported historical event versions are retained or migrated.

### TD-002 — projection strategy is correctness-first, not scale-tuned

Status: open
Introduced: Gate 1 / PR #10
Owner area: persistence projections
Severity: low

Why accepted:
Whole-image projection replacement and head/count validation keep atomicity, deletion, drift detection, and replay-backed repair mechanically simple at the current Gate 1 scale.

Risk:
High-volume future simulation may make whole-image replacement expensive. Head/count validation also does not independently checksum arbitrary same-count manual in-place payload tampering.

Exit criteria:
- endurance/volume measurements establish whether incremental projection maintenance is needed;
- any replacement preserves equivalent atomicity, campaign isolation, fail-closed validation, and deterministic replay rebuild guarantees;
- strengthen tamper detection if supported admin/recovery workflows create a demonstrated need beyond the current constrained production write path.

### TD-003 — portability/content integrity is not cryptographic authenticity

Status: open
Introduced: Gate 1 / PRs #9 and #11
Owner area: campaign portability + content trust
Severity: medium

Why accepted:
Gate 1 needs deterministic local corruption/integrity checks and validated portable exports, not a publisher trust/DRM/signature system. Manifest v1 therefore uses `fnv1a64` for corruption detection, and export v1 validates structure/domain/provenance without signatures.

Risk:
These mechanisms can detect ordinary damage and inconsistency but do not prove publisher identity or protect an export/content pack from deliberate modification by an attacker with local file access.

Exit criteria:
- define product requirements for authenticity/trust based on actual distribution and threat models;
- if required, introduce versioned cryptographic digests/signatures/trust metadata without changing the meaning of existing v1 artifacts;
- provide supported backup/content provenance UX appropriate to those requirements.

### TD-004 — runnable boundary is not yet a finished product shell

Status: open
Introduced: Gate 1 / PR #12
Owner area: application composition
Severity: medium

Why accepted:
Gate 1 establishes `dmd-app::CampaignRuntime`/`RunnableCampaign` as the production-intended composition boundary before UI, voice, and later gameplay application surfaces exist. Raw persistence lifecycle APIs intentionally remain public for diagnostics/recovery.

Risk:
Future gameplay-facing entrypoints could accidentally bypass the runnable capability and treat raw `OpenCampaign` as playable state unless integration discipline is extended mechanically as the application grows.

Exit criteria:
- player-facing application entrypoints route campaign play through `RunnableCampaign`;
- add mechanical dependency/API guards wherever new layers could otherwise adopt raw persistence state for gameplay;
- retain a clearly separated recovery/admin path for unresolved campaigns.

### TD-005 — content catalog is fully reloaded at every runnable boundary

Status: open
Introduced: Gate 1 / PR #12
Owner area: application/content runtime
Severity: low

Why accepted:
Reloading and validating configured local manifests on create/open/resume/restore gives a simple fail-closed guarantee when content disappears, changes version, or becomes corrupt while the process is alive.

Risk:
Large future content installations may make repeated full discovery/integrity verification unnecessarily expensive.

Exit criteria:
- measure catalog load cost with realistic content volume;
- introduce caching/change watching only if needed and only with equivalent detection of removal, modification, corruption, compatibility changes, and exact-version availability.

### TD-006 — snapshot cadence and concurrent-write retry policy are not endurance-tuned

Status: open
Introduced: Gate 1 / PRs #6 and #7
Owner area: persistence operations
Severity: low

Why accepted:
The 100-event snapshot cadence and SQLite contention behavior are safe initial policies for establishing the persistence authority/recovery boundary before realistic long-horizon workload measurements exist.

Risk:
Future sustained simulation may expose replay-latency, storage, or contention characteristics that require different snapshot scheduling or application-level per-campaign write scheduling/retry behavior.

Exit criteria:
- measure snapshot/replay cost and concurrent write behavior under realistic endurance workloads;
- tune snapshot policy without weakening immutable recovery anchors;
- define application-level retry/scheduling behavior that always re-reads/re-resolves after non-committed contention failures.

# Gate 13 — Recovery, administration, performance and release

Status: **Planned**

## Product requirements advanced

- normal-user installation/configuration;
- backup/restore/correction/admin;
- diagnostics;
- content/model setup;
- performance hardening;
- licensing notices;
- supportability/accessibility.

## Objective

Turn the complete system into a distributable, recoverable and supportable release candidate rather than a developer-operated build.

## Entry conditions

Gates 2–12 accepted.

## Acceptance criteria

Provide:
- installer/first-run;
- update strategy;
- campaign backups/export/restore;
- corruption/failure recovery;
- provenance-preserving admin corrections;
- diagnostics understandable by normal users;
- content install/update/remediation UX;
- model/provider configuration and health checks;
- audio/microphone/speaker setup;
- table-contract/rules-version/house-rule inspection;
- adjudication/correction history;
- transcript/campaign knowledge browsing;
- admin truth/visibility debugging;
- performance tuning and long-session soak;
- licensing/attribution notices;
- accessible text-only fallback.
No ordinary recovery requires Rust/SQLite/Git/JSON editing.

## Architecture invariants

Admin escape hatch is explicit and provenance-preserving. Recovery cannot silently rewrite journal/history. Provider/content configuration cannot bypass runnable validation.

## Explicit non-goals

New gameplay systems except blockers discovered during release/endurance testing.

## Required failure/recovery behavior

Installer/update/provider/content failures preserve existing campaigns. Backup/restore validated end-to-end. Recovery flow distinguishes content-unavailable from state-corrupt cases.

## Merge/pause boundaries

Codex may merge in-scope Gate 13 migration/save/release/licensing work autonomously after exact-head verification when it satisfies the accepted product contract and preserves recoverability. Irreversible/destructive behavior outside the checkpoint intent or an unresolved data-loss risk is a true blocker that must be surfaced. Pause at the end of Gate 13 with release-candidate evidence, including reference-hardware results or a clear external test requirement.
## Candidate workstreams

Packaging, updater, settings, diagnostics, backup/restore, admin correction, performance, accessibility, legal notices, support runbooks.

## Deferred requirements

Only Gate 14 endurance acceptance.

## Open questions

Define supported OS/version matrix and concrete reference-hardware budgets from measured Gate 10 data.

## Production integration acceptance

On a clean supported machine, install/configure/run DMd without development tools; create/open campaigns; configure local providers; recover representative failures through UI; backup/restore; run multi-hour soak on reference hardware; verify legal notices/content provenance.

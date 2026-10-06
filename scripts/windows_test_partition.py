#!/usr/bin/env python3
"""Run and prove an exhaustive two-job partition of native workspace tests.

Only libtest's stable human output is parsed. Raw native stdout/stderr share one
pipe before any CI streaming; package validation reparses the uploaded raw logs.
No test concurrency, profile, feature, fixture or assertion changes are made.
"""

import argparse
import hashlib
import json
import os
from pathlib import Path
import posixpath
import re
import subprocess
import sys


TARGET = "x86_64-pc-windows-msvc"
KEY = ("table_missile_cases::"
       "missile_zero_one_two_eligible_sources_keep_uniform_ordering_and_private_acknowledgments")
ORIGINAL = {
    "genuine_flow4_selected_owned_shield_finishes_as_a_miss_without_replacing_dice",
    "seven_original_flow4_exports_restore_cold_and_retry_every_original_binding",
    "selected_owned_flow4_missile_shields_pay_once_then_prevent_all_six_darts",
    "three_flow4_faces_continue_through_barrier_and_six_concentration_children",
    "all_flow4_faces_keep_original_impacts_and_concentration_children",
    "first_flow4_concentration_child_precedes_the_remaining_five_darts",
    "fifth_flow4_child_resumes_the_last_automatic_singleton_and_its_save",
    "committed_flow4_hold_person_keeps_its_paid_source_and_original_first_save",
}
BASE = ["cargo", "test", "--locked", "--workspace", "--target", TARGET]
SCHEMA = 1
ANSI = re.compile(r"\x1b\[[0-9;]*m")
BOUNDARY = re.compile(r"^\s*Running (.+) \((.+)\)$")
DOC = re.compile(r"^\s*Doc-tests ([A-Za-z0-9_]+)$")
LIST_ITEM = re.compile(r"^(.+): (test|benchmark)$")
LIST_END = re.compile(r"^(\d+) tests?, (\d+) benchmarks?$")
RUN_START = re.compile(r"^running (\d+) tests?$")
RESULT = re.compile(r"^test (.+) \.\.\. (ok|FAILED|ignored(?:, .*)?|bench.*)$")
PROGRESS = re.compile(r"^test (.+) has been running for over \d+ seconds$")
SUMMARY = re.compile(
    r"^test result: (ok|FAILED)\. (\d+) passed; (\d+) failed; (\d+) ignored; "
    r"(\d+) measured; (\d+) filtered out; finished in [0-9.]+s$")
DOC_TIME = re.compile(
    r"^all doctests ran in [0-9.]+s; merged doctests compilation took [0-9.]+s$")


class EvidenceError(ValueError):
    pass


def require(condition, message):
    if not condition:
        raise EvidenceError(message)


def digest(data):
    return hashlib.sha256(data).hexdigest()


def canonical(value):
    return json.dumps(value, sort_keys=True, separators=(",", ":"))


def path_text(value):
    return posixpath.normpath(value.replace("\\", "/"))


def absolute(value, root):
    value = path_text(value)
    if value.startswith("/") or re.match(r"^[A-Za-z]:/", value):
        return value
    return path_text(root + "/" + value)


def relative(value, root):
    value, root = absolute(value, root), path_text(root).rstrip("/")
    require(value.lower().startswith(root.lower() + "/"),
            f"path outside workspace: {value}")
    return value[len(root) + 1:]


def lines(raw):
    text = raw.decode("utf-8-sig", errors="strict").replace("\r\n", "\n")
    text = ANSI.sub("", text)
    require(not re.search(r"[\x00-\x08\x0b-\x1f\x7f]", text),
            "unknown terminal control or bare carriage return")
    return text.splitlines()


def commands(partition):
    require(partition in {"remainder", "isolated"}, "unknown partition")
    return {
        "rustc": ["rustc", "-vV"],
        "cargo": ["cargo", "--version"],
        "metadata": ["cargo", "metadata", "--locked", "--no-deps", "--format-version", "1"],
        "artifacts": BASE + ["--no-run", "--message-format", "json-render-diagnostics"],
        "discovery": BASE + ["--", "--list"],
        # Cargo-level TESTNAME switches default targets and omits doctests.
        # Keep the filter on libtest's side to retain full workspace selection.
        "execution": (BASE + ["--", "--skip", KEY] if partition == "remainder"
                      else BASE + ["--", KEY, "--exact"]),
    }


class Registry:
    """Map Cargo's run-local binary paths to portable workspace target IDs."""

    def __init__(self, metadata, artifact_raw):
        require(metadata.get("version") == 1, "unsupported Cargo metadata version")
        self.root = path_text(metadata["workspace_root"])
        self.targets = {}
        self.executables = {}
        self.docs = {}
        members = set(metadata["workspace_members"])
        packages = {p["id"]: p for p in metadata["packages"] if p["id"] in members}
        require(set(packages) == members and members, "incomplete workspace packages")
        target_keys = {}
        for package_id, package in packages.items():
            manifest = relative(package["manifest_path"], self.root)
            for target in package["targets"]:
                kind = target["kind"]
                # New selectors/custom kinds need deliberate review, not an omitted suite.
                if not target["test"] and not target["doctest"]:
                    continue
                require(not target.get("required-features"),
                        "required-feature target needs explicit selection reconciliation")
                require(kind in [["lib"], ["bin"], ["test"], ["example"]],
                        f"unsupported test target kind: {kind}")
                info = {"package": package["name"], "manifest": manifest,
                        "kind": kind[0], "name": target["name"],
                        "source": relative(target["src_path"], self.root)}
                identity = canonical(info)
                key = (package_id, target["name"], tuple(kind), info["source"])
                require(key not in target_keys, "duplicate Cargo target")
                if target["test"]:
                    require(identity not in self.targets, "duplicate portable target identity")
                    self.targets[identity] = info
                    target_keys[key] = identity
                if target["doctest"]:
                    require(kind == ["lib"], "unsupported doc-test target")
                    doc_info = dict(info, kind="doc")
                    doc_identity = canonical(doc_info)
                    require(target["name"] not in self.docs, "ambiguous doc-test crate name")
                    self.docs[target["name"]] = doc_identity
                    self.targets[doc_identity] = doc_info
        seen_targets = set()
        finished = []
        for line in lines(artifact_raw):
            if not line.startswith("{"):
                continue  # Cargo diagnostics are not artifact records.
            record = json.loads(line)
            if record.get("reason") == "build-finished":
                finished.append(record["success"])
            if record.get("reason") != "compiler-artifact" or not record.get("executable"):
                continue
            if record["package_id"] not in members or not record["profile"]["test"]:
                continue
            target = record["target"]
            key = (record["package_id"], target["name"], tuple(target["kind"]),
                   relative(target["src_path"], self.root))
            require(key in target_keys, "unexpected compiled test target")
            identity = target_keys[key]
            executable = absolute(record["executable"], self.root).lower()
            require(identity not in seen_targets and executable not in self.executables,
                    "duplicate test artifact")
            seen_targets.add(identity)
            self.executables[executable] = identity
        expected = {key for key, info in self.targets.items() if info["kind"] != "doc"}
        require(finished == [True], "missing/duplicate/failed Cargo build-finished")
        require(seen_targets == expected, "missing or extra compiled workspace harness")

    def boundary(self, line):
        match = BOUNDARY.fullmatch(line)
        if match:
            executable = absolute(match[2], self.root).lower()
            require(executable in self.executables, f"unknown running executable: {match[2]}")
            return self.executables[executable]
        match = DOC.fullmatch(line)
        if match:
            require(match[1] in self.docs, "unknown doc-test crate")
            return self.docs[match[1]]
        return None


def parse_discovery(raw, registry):
    groups, current, pending = {}, None, []

    def close():
        if current is not None:
            require(not pending and groups[current], "incomplete discovery subgroup")
            require(registry.targets[current]["kind"] == "doc" or len(groups[current]) == 1,
                    "multiple discovery summaries for one binary")
            require(len(groups[current]) == 1 or all(groups[current]),
                    "duplicate or unexplained empty discovery subgroup")

    for line in lines(raw):
        identity = registry.boundary(line)
        if identity is not None:
            close()
            require(identity not in groups, "duplicate discovery harness")
            current = identity
            groups[current] = []
            continue
        if not line.strip() or (current is not None and DOC_TIME.fullmatch(line)):
            continue
        item, end = LIST_ITEM.fullmatch(line), LIST_END.fullmatch(line)
        if item:
            require(current is not None, "test name outside discovery harness")
            require(item[2] == "test", "benchmark inventory needs explicit reconciliation")
            require(item[1] not in pending and
                    all(item[1] not in group for group in groups[current]),
                    "duplicate discovered test identity")
            pending.append(item[1])
        elif end:
            require(current is not None and int(end[1]) == len(pending) and int(end[2]) == 0,
                    "discovery count mismatch")
            groups[current].append(sorted(pending))
            pending = []
        else:
            require(current is None and not line.startswith(("test ", "running ")),
                    f"unrecognized discovery output: {line}")
    close()
    require(set(groups) == set(registry.targets), "missing/extra discovery harness or docs")
    return {identity: sorted(parts, key=canonical) for identity, parts in groups.items()}


def inventory_names(discovery):
    return {identity: {name for group in groups for name in group}
            for identity, groups in discovery.items()}


def required_cases(registry, discovery):
    names = inventory_names(discovery)
    matches = [(identity, name) for identity, items in names.items() for name in items if KEY in name]
    require(len(matches) == 1 and matches[0][1] == KEY, "absent/duplicate/substring-colliding key")
    identity = matches[0][0]
    info = registry.targets[identity]
    require((info["package"], info["kind"], info["name"]) == ("dmd-app", "test", "table_loop"),
            "partition key in wrong target")
    originals = [key for key, target in registry.targets.items()
                 if (target["package"], target["kind"], target["name"]) ==
                 ("dmd-app", "test", "legacy_shield_missile_v1_replay")]
    require(len(originals) == 1 and ORIGINAL <= names[originals[0]],
            "missing original eight recovery cases")
    return identity, originals[0]


def parse_execution(raw, registry, discovery, partition):
    key_target, original_target = required_cases(registry, discovery)
    names = inventory_names(discovery)
    reports, current, active, serial = {}, None, None, None

    def test_name(display):
        require(current is not None, "outcome outside a harness")
        candidates = [display]
        for mode in ("should panic", "compile fail", "compile"):
            suffix = " - " + mode
            if display.endswith(suffix):
                candidates.append(display[:-len(suffix)])
        matches = [name for name in candidates if name in names[current]]
        require(len(matches) == 1, f"unknown/ambiguous test name: {display}")
        return matches[0]

    def close():
        require(active is None and serial is None, "incomplete runtime subgroup")
        if current is not None:
            groups = reports[current]
            require(len(groups) == len(discovery[current]), "runtime/discovery subgroup count mismatch")
            require(sorted(group["passed"] + group["filtered"] for group in groups) ==
                    sorted(len(group) for group in discovery[current]),
                    "runtime/discovery subgroup size mismatch")

    for line in lines(raw):
        identity = registry.boundary(line)
        if identity is not None:
            close()
            require(identity not in reports, "duplicate runtime harness")
            current = identity
            reports[current] = []
            continue
        if not line.strip() or (current is not None and DOC_TIME.fullmatch(line)):
            continue
        start, result = RUN_START.fullmatch(line), RESULT.fullmatch(line)
        progress, summary = PROGRESS.fullmatch(line), SUMMARY.fullmatch(line)
        if start:
            require(current is not None and active is None, "missing boundary/overlapping runtime subgroup")
            active = {"running": int(start[1]), "names": []}
        elif result or (serial is not None and line == "ok"):
            require(active is not None, "outcome outside active runtime subgroup")
            name = test_name(result[1]) if result else serial
            require((result[2] if result else "ok") == "ok", "non-passing named outcome")
            require(name not in active["names"] and
                    all(name not in group["names"] for group in reports[current]),
                    "duplicate named outcome")
            active["names"].append(name)
            serial = None
        elif " ... test " in line and line.endswith(" seconds"):
            # A one-thread default harness may emit its long-test warning after
            # the un-terminated initial name. Preserve the pending result name.
            prefix, warning = line.split(" ... ", 1)
            warning_match = PROGRESS.fullmatch(warning)
            require(active is not None and serial is None and prefix.startswith("test ")
                    and warning_match is not None, "malformed serial progress")
            serial = test_name(prefix[5:])
            require(test_name(warning_match[1]) == serial, "serial progress name mismatch")
        elif progress:
            require(active is not None, "progress outside active runtime subgroup")
            test_name(progress[1])
        elif summary:
            require(active is not None and serial is None, "summary without runtime start")
            passed, failed, ignored, measured, filtered = map(int, summary.groups()[1:])
            require(summary[1] == "ok" and failed == ignored == measured == 0,
                    "non-passing runtime summary")
            require(passed == active["running"] == len(active["names"]), "runtime count mismatch")
            reports[current].append({"names": sorted(active["names"]),
                                     "passed": passed, "filtered": filtered})
            active = None
        else:
            require(current is None and not line.startswith(("test ", "running ")),
                    f"unrecognized runtime output: {line}")
    close()
    require(set(reports) == set(discovery), "missing/extra runtime harness or docs")
    for identity, groups in reports.items():
        passed = {name for group in groups for name in group["names"]}
        expected = (names[identity] - ({KEY} if identity == key_target else set())
                    if partition == "remainder" else ({KEY} if identity == key_target else set()))
        require(passed == expected, "missing/extra actual passing test")
        require(sum(group["filtered"] for group in groups) == len(names[identity] - expected),
                "unexpected filtered count")
    if partition == "remainder":
        require(sum(g["filtered"] for g in reports[original_target]) == 0,
                "original recovery cases filtered")
    return reports


def inspect_logs(raw, partition):
    metadata = json.loads("\n".join(lines(raw["metadata"])))
    registry = Registry(metadata, raw["artifacts"])
    discovery = parse_discovery(raw["discovery"], registry)
    report = parse_execution(raw["execution"], registry, discovery, partition)
    return {"targets": registry.targets, "discovery": discovery, "results": report}


def git(root, *args):
    return subprocess.check_output(["git", *args], cwd=root).decode("utf-8").strip()


def source_identity(root):
    require(not git(root, "status", "--porcelain", "--untracked-files=normal"), "checkout is not clean")
    return {"head": git(root, "rev-parse", "HEAD"), "tree": git(root, "rev-parse", "HEAD^{tree}")}


def job_identity(root):
    value = source_identity(root)
    require(value["head"] == os.environ.get("DMD_SOURCE_SHA"), "checkout differs from requested source")
    for field, env in (("run", "GITHUB_RUN_ID"), ("attempt", "GITHUB_RUN_ATTEMPT")):
        value[field] = os.environ.get(env, "")
        require(re.fullmatch(r"[1-9][0-9]*", value[field]), f"missing {env}")
    value["target"] = TARGET
    value["repository"] = os.environ.get("GITHUB_REPOSITORY", "")
    require(value["repository"], "missing repository identity")
    return value


def save_json(path, value):
    with path.open("x", encoding="utf-8", newline="\n") as handle:
        json.dump(value, handle, sort_keys=True, indent=2)
        handle.write("\n")


def capture(args, root, path):
    # A single native pipe preserves cross-stream process ordering. Do not use
    # two reader threads, shell redirection, Tee-Object or GitHub's merged log.
    with path.open("xb") as output:
        process = subprocess.Popen(args, cwd=root, stdout=subprocess.PIPE, stderr=subprocess.STDOUT)
        try:
            while block := os.read(process.stdout.fileno(), 65536):
                output.write(block)
                output.flush()
                sys.stdout.buffer.write(block)
                sys.stdout.buffer.flush()
        except BaseException:
            process.kill()
            process.wait()
            raise
        finally:
            process.stdout.close()
        return process.wait()


def run_partition(root, output, partition):
    identity = job_identity(root)
    output.mkdir(parents=True, exist_ok=False)
    receipts, raw = {}, {}
    try:
        for name, args in commands(partition).items():
            require(job_identity(root) == identity, "source/run identity changed before command")
            path = output / (name + ".log")
            code = capture(args, root, path)
            raw[name] = path.read_bytes()
            receipts[name] = {"args": args, "exit": code, "file": path.name, "sha256": digest(raw[name])}
            save_json(output / (name + ".receipt.json"), receipts[name])
            require(code == 0, f"{name} command failed ({code}); raw evidence preserved")
            require(job_identity(root) == identity, "source/run identity changed after command")
            if name == "discovery":
                registry = Registry(json.loads("\n".join(lines(raw["metadata"]))), raw["artifacts"])
                required_cases(registry, parse_discovery(raw[name], registry))
        verified = inspect_logs(raw, partition)
        manifest = {"schema": SCHEMA, "partition": partition, "identity": identity,
                    "job": os.environ.get("GITHUB_JOB"), "commands": receipts,
                    "verified": verified}
        require(manifest["job"], "missing job identity")
        save_json(output / "manifest.json", manifest)
    except Exception as error:
        save_json(output / "failure.json", {"error": str(error), "identity": identity,
                                           "partition": partition, "commands": receipts})
        raise


def read_bundle(directory, partition, identity):
    manifest = json.loads((directory / "manifest.json").read_text(encoding="utf-8"))
    require(not (directory / "failure.json").exists(), "failed execution cannot supply success evidence")
    require(manifest["schema"] == SCHEMA and manifest["partition"] == partition, "wrong manifest schema/partition")
    require(manifest["identity"] == identity and manifest.get("job"), "wrong source/tree/target/run/attempt/job identity")
    expected = commands(partition)
    require(set(manifest["commands"]) == set(expected), "missing/extra command receipt")
    raw = {}
    for name, args in expected.items():
        receipt = manifest["commands"][name]
        require(receipt["args"] == args and receipt["exit"] == 0 and receipt["file"] == name + ".log",
                "wrong command, failed execution or unsafe log path")
        raw[name] = (directory / receipt["file"]).read_bytes()
        require(digest(raw[name]) == receipt["sha256"], "tampered raw log/hash")
        saved_receipt = json.loads((directory / (name + ".receipt.json")).read_text(encoding="utf-8"))
        require(receipt == saved_receipt, "receipt mismatch")
    verified = inspect_logs(raw, partition)
    require(verified == manifest["verified"], "manifest results disagree with literal raw logs")
    return manifest, raw


def prove_union(remainder, isolated):
    require(remainder["identity"] == isolated["identity"], "partition identity mismatch")
    left, right = remainder["verified"], isolated["verified"]
    require(left["targets"] == right["targets"] and left["discovery"] == right["discovery"],
            "partition inventories differ")
    inventory = {(identity, name) for identity, names in inventory_names(left["discovery"]).items()
                 for name in names}
    def passed(report):
        values = [(identity, name) for identity, groups in report["results"].items()
                  for group in groups for name in group["names"]]
        require(len(values) == len(set(values)), "duplicate passing identity")
        return set(values)
    a, b = passed(left), passed(right)
    require(not a & b and a | b == inventory and len(b) == 1,
            "actual passing sets are not a disjoint exhaustive partition")
    return {"discovered": len(inventory), "remainder_passed": len(a), "isolated_passed": len(b),
            "harnesses": len(left["discovery"]),
            "doc_harnesses": sum(t["kind"] == "doc" for t in left["targets"].values())}


def validate_pair(root, remainder_dir, isolated_dir, output):
    identity = job_identity(root)
    remainder, left_raw = read_bundle(remainder_dir, "remainder", identity)
    isolated, right_raw = read_bundle(isolated_dir, "isolated", identity)
    require(remainder["job"] != isolated["job"], "partitions must come from separate jobs")
    for name in ("rustc", "cargo"):
        require(left_raw[name] == right_raw[name] and left_raw[name].strip(), "toolchain mismatch")
    proof = prove_union(remainder, isolated)
    require(job_identity(root) == identity, "package checkout changed during validation")
    proof.update({"schema": SCHEMA, "identity": identity,
                  "manifests": {"remainder": digest((remainder_dir / "manifest.json").read_bytes()),
                                "isolated": digest((isolated_dir / "manifest.json").read_bytes())}})
    save_json(output, proof)
    print(json.dumps(proof, indent=2))


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--root", type=Path, default=Path(__file__).resolve().parents[1])
    sub = parser.add_subparsers(dest="operation", required=True)
    run = sub.add_parser("run")
    run.add_argument("partition", choices=("remainder", "isolated"))
    run.add_argument("--output", required=True, type=Path)
    check = sub.add_parser("validate")
    check.add_argument("--remainder", required=True, type=Path)
    check.add_argument("--isolated", required=True, type=Path)
    check.add_argument("--output", required=True, type=Path)
    args = parser.parse_args()
    try:
        if args.operation == "run":
            run_partition(args.root, args.output, args.partition)
        else:
            validate_pair(args.root, args.remainder, args.isolated, args.output)
    except (EvidenceError, OSError, ValueError, KeyError, TypeError) as error:
        print(f"Windows test evidence rejected: {error}", file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    sys.exit(main())

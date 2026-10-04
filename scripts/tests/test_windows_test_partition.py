"""Pure adversarial evidence tests; never launch Cargo, Rust or gameplay tests."""

import copy
import io
import json
import os
from pathlib import Path
import subprocess
import sys
from tempfile import TemporaryDirectory
import unittest
from unittest.mock import Mock, patch

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
import windows_test_partition as guard


ROOT = "C:/a/DMd/DMd"
IDENTITY = {"head": "a" * 40, "tree": "b" * 40, "run": "123", "attempt": "2",
            "target": guard.TARGET, "repository": "idiotswill/DMd"}


def fixture():
    """Small transcripts use real Cargo/libtest shapes and genuine required names.

    Include empty binary/doc targets and several doctest bundles; counts are
    deliberately different from today's real suite to detect hardcoded totals.
    """
    packages = []
    artifacts = []
    specs = []
    for package, targets in (
        ("dmd-app", [("dmd_app", "lib", "src/lib.rs", [["unit_case"]]),
                     ("table_loop", "test", "tests/table_loop.rs", [[guard.KEY, "ordinary_table_case"]]),
                     ("legacy_shield_missile_v1_replay", "test",
                      "tests/legacy_shield_missile_v1_replay.rs", [sorted(guard.ORIGINAL)])]),
        ("dmd-core", [("dmd_core", "lib", "src/lib.rs", [[]])]),
        ("dmd-desktop", [("dmd-desktop", "bin", "src/main.rs", [[]])]),
    ):
        package_id = f"path+file:///{ROOT}/crates/{package}#0.1.0"
        records = []
        for name, kind, source, groups in targets:
            full_source = f"{ROOT}/crates/{package}/{source}"
            target = {"kind": [kind], "crate_types": ["lib" if kind == "lib" else "bin"],
                      "name": name, "src_path": full_source, "edition": "2024", "doc": True,
                      "doctest": kind == "lib", "test": True}
            records.append(target)
            executable = f"{ROOT}/target/{guard.TARGET}/debug/deps/{name}-012345.exe"
            artifacts.append({"reason": "compiler-artifact", "package_id": package_id,
                              "target": target, "profile": {"test": True},
                              "executable": executable})
            label = "unittests " + full_source if kind != "test" else source
            relative_exe = executable[len(ROOT) + 1:].replace("/", "\\")
            specs.append((f"     Running {label} ({relative_exe})", groups, False))
        packages.append({"id": package_id, "name": package,
                         "manifest_path": f"{ROOT}/crates/{package}/Cargo.toml", "targets": records})
    metadata = {"version": 1, "workspace_root": ROOT, "packages": packages,
                "workspace_members": [p["id"] for p in packages]}
    specs += [("   Doc-tests dmd_app", [[]], True),
              ("   Doc-tests dmd_core", [["src/lib.rs - demo (line 7)",
                                         "src/lib.rs - compile_only (line 17)"],
                                        ["src/lib.rs - invalid (line 25)"]], True)]
    artifacts.append({"reason": "build-finished", "success": True})
    return metadata, artifacts, specs


def list_output(specs):
    text = ["    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.02s"]
    for boundary, groups, doc in specs:
        text.append(boundary)
        for group in groups:
            text.extend(name + ": test" for name in group)
            text.append(f"{len(group)} tests, 0 benchmarks")
        if doc and any(groups):
            text.append("all doctests ran in 0.02s; merged doctests compilation took 0.01s")
    return ("\n".join(text) + "\n").encode()


def execution_output(specs, partition):
    text = ["    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.01s"]
    for boundary, groups, doc in specs:
        text.append(boundary)
        for group in groups:
            chosen = [n for n in group if (n != guard.KEY if partition == "remainder" else n == guard.KEY)]
            text.append(f"running {len(chosen)} tests")
            if chosen:
                text.append(f"test {chosen[0]} has been running for over 60 seconds")
            for name in reversed(chosen):  # Completion need not follow discovery order.
                mode = (" - compile" if "compile_only" in name else
                        " - compile fail" if "invalid" in name else
                        " - should panic" if name == "unit_case" else "")
                text.append(f"test {name}{mode} ... ok")
            text.append(f"test result: ok. {len(chosen)} passed; 0 failed; 0 ignored; "
                        f"0 measured; {len(group) - len(chosen)} filtered out; finished in 0.01s")
        if doc and any(groups):
            text.append("all doctests ran in 0.02s; merged doctests compilation took 0.01s")
    return ("\n".join(text) + "\n").encode()


def raw_logs(partition):
    metadata, artifacts, specs = fixture()
    return {"metadata": json.dumps(metadata).encode(),
            "artifacts": ("\n".join(json.dumps(a) for a in artifacts) + "\n").encode(),
            "discovery": list_output(specs), "execution": execution_output(specs, partition),
            "rustc": b"rustc 1.98.1\nhost: x86_64-pc-windows-msvc\n",
            "cargo": b"cargo 1.98.1\n"}


def write_bundle(directory, partition, raw=None):
    raw = raw_logs(partition) if raw is None else raw
    directory.mkdir()
    receipts = {}
    for name, args in guard.commands(partition).items():
        (directory / (name + ".log")).write_bytes(raw[name])
        receipts[name] = {"args": args, "exit": 0, "file": name + ".log",
                          "sha256": guard.digest(raw[name])}
        (directory / (name + ".receipt.json")).write_text(json.dumps(receipts[name]), encoding="utf-8")
    manifest = {"schema": guard.SCHEMA, "partition": partition, "identity": copy.deepcopy(IDENTITY),
                "job": "windows" if partition == "remainder" else "isolated", "commands": receipts,
                "verified": guard.inspect_logs(raw, partition)}
    (directory / "manifest.json").write_text(json.dumps(manifest), encoding="utf-8")
    return manifest


class TranscriptTests(unittest.TestCase):
    def test_valid_pair_covers_every_name_once_and_preserves_empty_docs(self):
        remainder = guard.inspect_logs(raw_logs("remainder"), "remainder")
        isolated = guard.inspect_logs(raw_logs("isolated"), "isolated")
        proof = guard.prove_union({"identity": IDENTITY, "verified": remainder},
                                 {"identity": IDENTITY, "verified": isolated})
        self.assertEqual(proof, {"discovered": 14, "remainder_passed": 13,
                                 "isolated_passed": 1, "harnesses": 7, "doc_harnesses": 2})
        app_docs = next(key for key, t in remainder["targets"].items()
                        if t["kind"] == "doc" and t["package"] == "dmd-app")
        self.assertEqual(remainder["discovery"][app_docs], [[]])
        core_docs = next(key for key, t in remainder["targets"].items()
                         if t["kind"] == "doc" and t["package"] == "dmd-core")
        self.assertEqual(len(remainder["results"][core_docs]), 2)

    def test_full_workspace_selection_for_every_cargo_test_command(self):
        self.assertEqual(guard.BASE, ["cargo", "test", "--locked", "--workspace",
                                      "--target", "x86_64-pc-windows-msvc"])
        for partition in ("remainder", "isolated"):
            for name in ("artifacts", "discovery", "execution"):
                args = guard.commands(partition)[name]
                self.assertIn("--workspace", args)
                self.assertNotIn("-p", args)
                self.assertNotIn("--test", args)
            expected_suffixes = {
                "discovery": ["--list"],
                "execution": (["--skip", guard.KEY] if partition == "remainder"
                              else [guard.KEY, "--exact"]),
            }
            for name, suffix in expected_suffixes.items():
                args = guard.commands(partition)[name]
                separator = args.index("--")
                self.assertEqual(args[:separator], guard.BASE)
                self.assertEqual(args[separator + 1:], suffix)
            self.assertNotIn("--test-threads", " ".join(guard.commands(partition)["execution"]))

    def test_ansi_crlf_bom_and_windows_paths_keep_the_same_proof(self):
        raw = raw_logs("remainder")
        expected = guard.inspect_logs(raw, "remainder")
        for name in ("artifacts", "discovery", "execution"):
            raw[name] = b"\xef\xbb\xbf" + raw[name].replace(b"Running ", b"\x1b[1m\x1b[92mRunning\x1b[0m ").replace(b"\n", b"\r\n")
        self.assertEqual(guard.inspect_logs(raw, "remainder"), expected)

    def test_portable_identity_does_not_depend_on_checkout_or_executable_hash(self):
        left = guard.inspect_logs(raw_logs("remainder"), "remainder")
        raw = raw_logs("remainder")
        for name in ("metadata", "artifacts", "discovery", "execution"):
            raw[name] = raw[name].replace(ROOT.encode(), b"D:/other/checkout").replace(b"012345", b"abcdef")
        self.assertEqual(guard.inspect_logs(raw, "remainder"), left)

    def test_reordered_doc_subgroups_have_canonical_inventory(self):
        metadata, artifacts, specs = fixture()
        before = guard.inspect_logs(raw_logs("remainder"), "remainder")
        boundary, groups, doc = specs[-1]
        specs[-1] = (boundary, list(reversed(groups)), doc)
        raw = raw_logs("remainder")
        raw["discovery"], raw["execution"] = list_output(specs), execution_output(specs, "remainder")
        after = guard.inspect_logs(raw, "remainder")
        self.assertEqual(before["discovery"], after["discovery"])

    def test_serial_long_running_warning_requires_the_same_pending_name(self):
        raw = raw_logs("isolated")
        raw["execution"] = raw["execution"].replace(
            f"test {guard.KEY} has been running for over 60 seconds\ntest {guard.KEY} ... ok".encode(),
            f"test {guard.KEY} ... test {guard.KEY} has been running for over 60 seconds\nok".encode())
        guard.inspect_logs(raw, "isolated")
        raw["execution"] = raw["execution"].replace(b" ... test table_missile", b" ... test unknown_missile")
        with self.assertRaises(guard.EvidenceError):
            guard.inspect_logs(raw, "isolated")

    def test_key_absent_duplicate_substring_or_wrong_target_fails(self):
        for scenario in ("absent", "duplicate", "substring", "wrong_target"):
            with self.subTest(scenario=scenario):
                metadata, artifacts, specs = fixture()
                if scenario == "absent":
                    specs[1][1][0].remove(guard.KEY)
                elif scenario == "duplicate":
                    specs[0][1][0].append(guard.KEY)
                elif scenario == "substring":
                    specs[0][1][0].append("prefix_" + guard.KEY)
                else:
                    specs[1][1][0].remove(guard.KEY)
                    specs[0][1][0].append(guard.KEY)
                raw = raw_logs("remainder")
                raw["discovery"] = list_output(specs)
                with self.assertRaises(guard.EvidenceError):
                    guard.inspect_logs(raw, "remainder")

    def test_missing_original_case_rejects_even_a_self_consistent_smaller_suite(self):
        metadata, artifacts, specs = fixture()
        specs[2][1][0].pop()
        raw = raw_logs("remainder")
        raw["discovery"], raw["execution"] = list_output(specs), execution_output(specs, "remainder")
        with self.assertRaisesRegex(guard.EvidenceError, "original eight"):
            guard.inspect_logs(raw, "remainder")

    def test_named_outcomes_cannot_be_missing_extra_failed_ignored_or_duplicated(self):
        original = b"test ordinary_table_case ... ok\n"
        for replacement in (b"", b"test unknown ... ok\n", original * 2,
                            b"test ordinary_table_case ... FAILED\n",
                            b"test ordinary_table_case ... ignored, expensive\n"):
            with self.subTest(replacement=replacement):
                raw = raw_logs("remainder")
                self.assertIn(original, raw["execution"])
                raw["execution"] = raw["execution"].replace(original, replacement)
                with self.assertRaises(guard.EvidenceError):
                    guard.inspect_logs(raw, "remainder")

    def test_zero_isolated_pass_cannot_hide_behind_success_summaries(self):
        raw = raw_logs("isolated")
        raw["execution"] = raw["execution"].replace(f"test {guard.KEY} ... ok\n".encode(), b"")
        raw["execution"] = raw["execution"].replace(b"running 1 tests", b"running 0 tests")
        raw["execution"] = raw["execution"].replace(b"1 passed; 0 failed; 0 ignored; 0 measured; 1 filtered",
                                                   b"0 passed; 0 failed; 0 ignored; 0 measured; 2 filtered")
        with self.assertRaises(guard.EvidenceError):
            guard.inspect_logs(raw, "isolated")

    def test_duplicate_summary_or_new_unexplained_empty_doc_group_fails(self):
        for extra in ("test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s\n",
                      "running 0 tests\ntest result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s\n"):
            raw = raw_logs("remainder")
            raw["execution"] += extra.encode()
            with self.assertRaises(guard.EvidenceError):
                guard.inspect_logs(raw, "remainder")

    def test_duplicate_empty_discovery_subgroup_cannot_be_another_doc_bundle(self):
        raw = raw_logs("remainder")
        raw["discovery"] = raw["discovery"].replace(
            b"Doc-tests dmd_app\n0 tests, 0 benchmarks",
            b"Doc-tests dmd_app\n0 tests, 0 benchmarks\n0 tests, 0 benchmarks")
        with self.assertRaisesRegex(guard.EvidenceError, "empty discovery subgroup"):
            guard.inspect_logs(raw, "remainder")

    def test_incomplete_doc_subgroup_and_missing_empty_harness_fail(self):
        raw = raw_logs("remainder")
        raw["execution"] = raw["execution"].rsplit(b"test result:", 1)[0]
        with self.assertRaises(guard.EvidenceError):
            guard.inspect_logs(raw, "remainder")
        metadata, artifacts, specs = fixture()
        del specs[5]  # Empty dmd_app doc group still belongs to the inventory.
        raw = raw_logs("remainder")
        raw["discovery"] = list_output(specs)
        with self.assertRaisesRegex(guard.EvidenceError, "missing/extra discovery"):
            guard.inspect_logs(raw, "remainder")

    def test_isolated_pass_cannot_replace_missing_all_or_empty_doc_runtime(self):
        _, _, specs = fixture()
        for missing in ("all", "empty"):
            with self.subTest(missing=missing):
                raw = raw_logs("isolated")
                kept = [(boundary, groups, doc) for boundary, groups, doc in specs
                        if not (doc and (missing == "all" or not any(groups)))]
                raw["execution"] = execution_output(kept, "isolated")
                self.assertIn(f"test {guard.KEY} ... ok\n".encode(), raw["execution"])
                with self.assertRaisesRegex(guard.EvidenceError, "missing/extra runtime harness"):
                    guard.inspect_logs(raw, "isolated")

    def test_unexpected_filter_count_and_duplicate_discovery_name_fail(self):
        raw = raw_logs("remainder")
        raw["execution"] = raw["execution"].replace(b"1 filtered out", b"2 filtered out")
        with self.assertRaises(guard.EvidenceError):
            guard.inspect_logs(raw, "remainder")
        raw = raw_logs("remainder")
        raw["discovery"] = raw["discovery"].replace(b"ordinary_table_case: test", b"unit_case: test\nunit_case: test")
        with self.assertRaises(guard.EvidenceError):
            guard.inspect_logs(raw, "remainder")

    def test_nonempty_benchmarks_unknown_controls_and_packaging_running_are_rejected(self):
        for name, old, new in (("discovery", b"unit_case: test", b"unit_case: benchmark"),
                               ("execution", b"running 1 tests", b"\x1b[2Krunning 1 tests"),
                               ("execution", b"running 1 tests", b"Running beforeBuildCommand npm build")):
            raw = raw_logs("remainder")
            raw[name] = raw[name].replace(old, new)
            with self.assertRaises(guard.EvidenceError):
                guard.inspect_logs(raw, "remainder")

    def test_compilation_success_cannot_replace_discovery_or_execution(self):
        for name in ("discovery", "execution"):
            raw = raw_logs("remainder")
            raw[name] = b'{"reason":"build-finished","success":true}\n'
            with self.assertRaises(guard.EvidenceError):
                guard.inspect_logs(raw, "remainder")

    def test_missing_duplicate_or_unknown_compiler_artifact_fails(self):
        for scenario in ("missing", "duplicate", "unknown", "failed_build"):
            metadata, artifacts, specs = fixture()
            if scenario == "missing":
                del artifacts[0]
            elif scenario == "duplicate":
                artifacts.insert(0, artifacts[0])
            elif scenario == "unknown":
                artifacts[0] = copy.deepcopy(artifacts[0])
                artifacts[0]["target"]["name"] = "surprise"
            else:
                artifacts[-1]["success"] = False
            raw = raw_logs("remainder")
            raw["artifacts"] = ("\n".join(json.dumps(a) for a in artifacts) + "\n").encode()
            with self.subTest(scenario=scenario), self.assertRaises(guard.EvidenceError):
                guard.inspect_logs(raw, "remainder")

    def test_unknown_executable_or_duplicate_harness_fails(self):
        for replacement in (b"unknown-012345.exe", b"table_loop-ffffff.exe"):
            raw = raw_logs("remainder")
            raw["execution"] = raw["execution"].replace(b"table_loop-012345.exe", replacement)
            with self.assertRaises(guard.EvidenceError):
                guard.inspect_logs(raw, "remainder")
        raw = raw_logs("remainder")
        raw["execution"] += raw["execution"]
        with self.assertRaises(guard.EvidenceError):
            guard.inspect_logs(raw, "remainder")


class BundleTests(unittest.TestCase):
    def setUp(self):
        self.temp = TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.base = Path(self.temp.name)
        self.left, self.right = self.base / "remainder", self.base / "isolated"
        self.manifest = write_bundle(self.left, "remainder")
        write_bundle(self.right, "isolated")

    def change_manifest(self, change):
        value = copy.deepcopy(self.manifest)
        change(value)
        (self.left / "manifest.json").write_text(json.dumps(value), encoding="utf-8")

    def test_pair_reparses_raw_logs_before_writing_proof(self):
        output = self.base / "proof.json"
        with patch.object(guard, "job_identity", return_value=IDENTITY):
            guard.validate_pair(self.base, self.left, self.right, output)
        self.assertEqual(json.loads(output.read_text())["isolated_passed"], 1)

    def test_every_identity_field_must_match_the_package_checkout_and_attempt(self):
        for field in IDENTITY:
            with self.subTest(field=field):
                self.change_manifest(lambda m: m["identity"].update({field: "wrong"}))
                with self.assertRaises(guard.EvidenceError):
                    guard.read_bundle(self.left, "remainder", IDENTITY)

    def test_wrong_commands_failure_receipts_unsafe_path_and_missing_command_fail(self):
        for change in (
            lambda m: m["commands"]["execution"].update(args=["cargo", "test", "-p", "dmd-app"]),
            lambda m: m["commands"]["execution"].update(exit=101),
            lambda m: m["commands"]["execution"].update(file="../another.log"),
            lambda m: m["commands"].pop("discovery"),
        ):
            self.change_manifest(change)
            with self.assertRaises(guard.EvidenceError):
                guard.read_bundle(self.left, "remainder", IDENTITY)

    def test_old_positional_isolated_argv_fails_even_with_consistent_receipts(self):
        path = self.right / "manifest.json"
        value = json.loads(path.read_text())
        value["commands"]["execution"]["args"] = guard.BASE + [guard.KEY, "--", "--exact"]
        path.write_text(json.dumps(value))
        (self.right / "execution.receipt.json").write_text(
            json.dumps(value["commands"]["execution"]))
        with self.assertRaisesRegex(guard.EvidenceError, "wrong command"):
            guard.read_bundle(self.right, "isolated", IDENTITY)

    def test_tampered_raw_log_cannot_be_hidden_by_manifest_pass_counts(self):
        path = self.left / "execution.log"
        path.write_bytes(path.read_bytes() + b"tampered\n")
        with self.assertRaisesRegex(guard.EvidenceError, "tampered"):
            guard.read_bundle(self.left, "remainder", IDENTITY)

    def test_rehashed_tampered_log_still_has_to_parse_and_pass(self):
        path = self.left / "execution.log"
        path.write_bytes(path.read_bytes().replace(b"test ordinary_table_case ... ok", b"test other ... ok"))
        self.manifest["commands"]["execution"]["sha256"] = guard.digest(path.read_bytes())
        (self.left / "execution.receipt.json").write_text(json.dumps(self.manifest["commands"]["execution"]))
        self.change_manifest(lambda m: None)
        with self.assertRaises(guard.EvidenceError):
            guard.read_bundle(self.left, "remainder", IDENTITY)

    def test_invented_manifest_results_and_explicit_failure_bundle_are_rejected(self):
        self.change_manifest(lambda m: m["verified"].update(results={}))
        with self.assertRaisesRegex(guard.EvidenceError, "literal raw logs"):
            guard.read_bundle(self.left, "remainder", IDENTITY)
        self.change_manifest(lambda m: None)
        (self.left / "failure.json").write_text("{}")
        with self.assertRaisesRegex(guard.EvidenceError, "failed execution"):
            guard.read_bundle(self.left, "remainder", IDENTITY)

    def test_mismatched_inventory_and_overlapping_passes_are_rejected(self):
        left, _ = guard.read_bundle(self.left, "remainder", IDENTITY)
        right, _ = guard.read_bundle(self.right, "isolated", IDENTITY)
        changed = copy.deepcopy(right)
        changed["verified"]["discovery"].pop(next(iter(changed["verified"]["discovery"])))
        with self.assertRaisesRegex(guard.EvidenceError, "inventories"):
            guard.prove_union(left, changed)
        changed = copy.deepcopy(right)
        changed["verified"]["results"] = copy.deepcopy(left["verified"]["results"])
        with self.assertRaisesRegex(guard.EvidenceError, "disjoint exhaustive"):
            guard.prove_union(left, changed)

    def test_toolchain_mismatch_and_same_job_fail_before_proof(self):
        path = self.right / "cargo.log"
        path.write_bytes(b"cargo different\n")
        manifest_path = self.right / "manifest.json"
        value = json.loads(manifest_path.read_text())
        value["commands"]["cargo"]["sha256"] = guard.digest(path.read_bytes())
        (self.right / "cargo.receipt.json").write_text(json.dumps(value["commands"]["cargo"]))
        manifest_path.write_text(json.dumps(value))
        with patch.object(guard, "job_identity", return_value=IDENTITY), self.assertRaisesRegex(guard.EvidenceError, "toolchain"):
            guard.validate_pair(self.base, self.left, self.right, self.base / "proof.json")
        value["job"] = "windows"
        manifest_path.write_text(json.dumps(value))
        with patch.object(guard, "job_identity", return_value=IDENTITY), self.assertRaisesRegex(guard.EvidenceError, "separate jobs"):
            guard.validate_pair(self.base, self.left, self.right, self.base / "proof.json")


class CaptureTests(unittest.TestCase):
    def test_native_capture_merges_before_streaming_and_retains_exit_code(self):
        process = Mock()
        process.wait.return_value = 17
        process.stdout.fileno.return_value = 9
        sink = Mock(buffer=io.BytesIO())
        with TemporaryDirectory() as directory:
            path = Path(directory) / "raw.log"
            with patch.object(guard.subprocess, "Popen", return_value=process) as launch, \
                    patch.object(guard.os, "read", side_effect=[b"stderr\r\n", b"stdout\n", b""]), \
                    patch.object(guard.sys, "stdout", sink):
                code = guard.capture(["fake", "--arg"], Path(directory), path)
            self.assertEqual(code, 17)
            self.assertEqual(path.read_bytes(), b"stderr\r\nstdout\n")
            self.assertEqual(sink.buffer.getvalue(), path.read_bytes())
            self.assertIs(launch.call_args.kwargs["stderr"], subprocess.STDOUT)
            self.assertNotIn("shell", launch.call_args.kwargs)

    def test_failed_command_preserves_evidence_without_success_manifest(self):
        with TemporaryDirectory() as directory:
            output = Path(directory) / "evidence"
            def fail(args, root, path):
                path.write_bytes(b"actual failure\n")
                return 101
            with patch.object(guard, "job_identity", return_value=IDENTITY), \
                    patch.object(guard, "capture", side_effect=fail):
                with self.assertRaises(guard.EvidenceError):
                    guard.run_partition(Path(directory), output, "remainder")
            self.assertFalse((output / "manifest.json").exists())
            self.assertEqual(json.loads((output / "failure.json").read_text())["commands"]["rustc"]["exit"], 101)

    def test_zero_exit_commands_with_missing_docs_preserve_failure_only(self):
        raw = raw_logs("isolated")
        _, _, specs = fixture()
        raw["execution"] = execution_output([spec for spec in specs if not spec[2]], "isolated")
        with TemporaryDirectory() as directory:
            output = Path(directory) / "evidence"
            calls = []
            def capture(args, root, path):
                name = path.stem
                calls.append(name)
                path.write_bytes(raw[name])
                return 0
            with patch.object(guard, "job_identity", return_value=IDENTITY), \
                    patch.object(guard, "capture", side_effect=capture):
                with self.assertRaisesRegex(guard.EvidenceError, "missing/extra runtime harness"):
                    guard.run_partition(Path(directory), output, "isolated")
            self.assertEqual(calls, ["rustc", "cargo", "metadata", "artifacts", "discovery", "execution"])
            self.assertFalse((output / "manifest.json").exists())
            failure = json.loads((output / "failure.json").read_text())
            self.assertEqual(failure["identity"], IDENTITY)
            self.assertEqual(failure["partition"], "isolated")
            self.assertEqual(failure["error"], "missing/extra runtime harness or docs")
            self.assertEqual(set(failure["commands"]), set(calls))
            for name in calls:
                receipt = failure["commands"][name]
                self.assertEqual(receipt["exit"], 0)
                self.assertEqual(receipt["args"], guard.commands("isolated")[name])
                self.assertEqual(receipt["sha256"], guard.digest(raw[name]))
                self.assertEqual((output / receipt["file"]).read_bytes(), raw[name])
                self.assertEqual(json.loads((output / (name + ".receipt.json")).read_text()), receipt)

    def test_missing_key_stops_before_execution(self):
        raw = raw_logs("isolated")
        raw["discovery"] = raw["discovery"].replace(guard.KEY.encode(), b"renamed_case")
        with TemporaryDirectory() as directory:
            output = Path(directory) / "evidence"
            calls = []
            def capture(args, root, path):
                name = path.stem
                calls.append(name)
                path.write_bytes(raw[name])
                return 0
            with patch.object(guard, "job_identity", return_value=IDENTITY), \
                    patch.object(guard, "capture", side_effect=capture):
                with self.assertRaises(guard.EvidenceError):
                    guard.run_partition(Path(directory), output, "isolated")
            self.assertNotIn("execution", calls)
            self.assertFalse((output / "manifest.json").exists())


if __name__ == "__main__":
    unittest.main()

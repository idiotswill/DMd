"""Lightweight verification-runner tests; never compile or run DMd."""
import copy
import importlib.util
import json
import os
from pathlib import Path
import sys
import tempfile
from types import SimpleNamespace
import unittest
from unittest.mock import patch

SPEC = importlib.util.spec_from_file_location('ci_runtime', Path(__file__).parents[1] / 'ci_runtime.py')
ci = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(ci)


def complete(name='example', count=1):
    return (f'test {name} ... ok\n' if count else '') + (
        f'test result: ok. {count} passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s\n')


def receipts():
    source = {'head': 'a' * 40, 'tree': 'b' * 40, 'lock_sha256': 'c' * 64}
    universe = [{'id': 'test:app:integration:case', 'cases': ['case'], 'assigned': 'table-loop'},
                {'id': 'doc:app:app', 'cases': [], 'assigned': 'remainder'}]
    for item in universe:
        item.update(original_args=[], cwd='$WORKSPACE', cargo_manifest_dir='$WORKSPACE')
    common = {'schema': ci.SCHEMA, 'source': source, 'platform': 'linux', 'run_id': '10',
              'run_attempt': '1', 'universe': universe,
              'absent_allocations': ['legacy-missile', 'grapple-public']}
    result = []
    for allocation in ci.ALLOCATIONS:
        records = []
        for target in universe:
            record = copy.deepcopy(target)
            record['state'] = 'executed' if target['assigned'] == allocation else 'listed-only'
            if record['state'] == 'executed':
                record['outcome'] = {'passed': len(record['cases']), 'summaries': 1}
            records.append(record)
        result.append({'schema': ci.SCHEMA, 'status': 'complete', 'allocation': allocation,
                       'common': copy.deepcopy(common), 'common_sha256': ci.digest(common), 'records': records,
                       'normalization_roots': {}})
    return result, source


class ListingsAndOutcomes(unittest.TestCase):
    def test_zero_case_listing_and_real_completion_are_required(self):
        self.assertEqual(ci.parse_listing('0 tests, 0 benchmarks\n'), [])
        self.assertEqual(ci.parse_result(complete(count=0), []), {'passed': 0, 'summaries': 1})
        with self.assertRaises(ValueError):
            ci.parse_result('', [])

    def test_listing_requires_unique_names_and_matching_total(self):
        self.assertEqual(ci.parse_listing('b: test\na: test\n2 tests, 0 benchmarks\n'), ['a', 'b'])
        for invalid in ('a: test\n', 'a: test\n2 tests, 0 benchmarks\n',
                        'a: test\na: test\n2 tests, 0 benchmarks\n',
                        'a: benchmark\n0 tests, 1 benchmark\n', 'arbitrary custom harness\n'):
            with self.subTest(invalid=invalid), self.assertRaises(ValueError):
                ci.parse_listing(invalid)

    def test_partial_duplicate_filtered_ignored_and_wrong_case_refuse(self):
        good = complete()
        self.assertEqual(ci.parse_result(good, ['example'])['passed'], 1)
        for text in (good.split('test result:')[0], good + good, good.replace('0 filtered', '1 filtered'),
                     good.replace('0 ignored', '1 ignored'), good.replace('... ok', '... FAILED'),
                     good.replace('example', 'other')):
            with self.subTest(text=text), self.assertRaises(ValueError):
                ci.parse_result(text, ['example'])

    def test_doctest_groups_reconcile_all_names(self):
        text = complete('src/lib.rs - one (line 2)') + complete('src/lib.rs - two (line 8)')
        names = ['src/lib.rs - one (line 2)', 'src/lib.rs - two (line 8)']
        self.assertEqual(ci.parse_result(text, names, docs=True)['passed'], 2)
        with self.assertRaises(ValueError):
            ci.parse_result(text, names)


class CompleteUnion(unittest.TestCase):
    def validate(self, data, source):
        return ci.validate_receipts(data, source, 'linux', '10', '1')

    def test_complete_union_includes_zero_case_docs_and_absent_partition(self):
        data, source = receipts()
        self.assertEqual(self.validate(data, source)['targets'], 2)
        self.assertEqual(self.validate(data, source)['cases'], 1)

    def test_missing_duplicate_and_incomplete_allocations_refuse(self):
        data, source = receipts()
        variants = [data[:-1], data + data[:1], [data[0], data[0], *data[2:]]]
        failed = copy.deepcopy(data)
        failed[0]['status'] = 'cancelled'
        variants.append(failed)
        for variant in variants:
            with self.subTest(variant=variant), self.assertRaises(ValueError):
                self.validate(variant, source)

    def test_source_platform_and_run_identity_cannot_transfer_pass(self):
        data, source = receipts()
        for changes in ({'head': 'd' * 40}, {'tree': 'e' * 40}, {'lock_sha256': 'f' * 64}):
            with self.subTest(changes=changes), self.assertRaises(ValueError):
                self.validate(data, {**source, **changes})
        for platform, run, attempt in [('windows', '10', '1'), ('linux', '11', '1'), ('linux', '10', '2')]:
            with self.subTest(platform=platform, run=run, attempt=attempt), self.assertRaises(ValueError):
                ci.validate_receipts(data, source, platform, run, attempt)

    def test_discovery_change_or_false_listed_pass_refuses(self):
        data, source = receipts()
        for mutation in ('discovery', 'record', 'state', 'outcome'):
            changed = copy.deepcopy(data)
            if mutation == 'discovery':
                changed[1]['common']['universe'][0]['cases'].append('new_test')
                changed[1]['common_sha256'] = ci.digest(changed[1]['common'])
            elif mutation == 'record':
                changed[0]['records'].pop()
            elif mutation == 'state':
                changed[0]['records'][0]['state'] = 'listed-only'
            else:
                changed[0]['records'][0]['outcome']['passed'] = 0
            with self.subTest(mutation=mutation), self.assertRaises(ValueError):
                self.validate(changed, source)

    def test_listed_only_outcomes_wrong_absence_and_changed_cwd_refuse(self):
        data, source = receipts()
        for mutation in ('false-outcome', 'absence', 'cwd'):
            changed = copy.deepcopy(data)
            if mutation == 'false-outcome':
                changed[1]['records'][0]['outcome'] = {'passed': 1, 'summaries': 1}
            elif mutation == 'cwd':
                changed[0]['records'][0]['cwd'] = '/another-package'
            else:
                for receipt in changed:
                    receipt['common']['absent_allocations'] = []
                    receipt['common_sha256'] = ci.digest(receipt['common'])
            with self.subTest(mutation=mutation), self.assertRaises(ValueError):
                self.validate(changed, source)


class InvocationAndInventory(unittest.TestCase):
    def test_aggregate_reads_original_logs_and_refuses_missing_or_changed_evidence(self):
        data, source = receipts()
        library = {'name': 'app', 'kind': ['lib'], 'doctest': True}
        metadata = {'workspace_members': ['app'], 'packages': [
            {'id': 'app', 'name': 'app', 'targets': [library]}]}
        executable = '/original/target/debug/app'
        artifact = {'reason': 'compiler-artifact', 'package_id': 'app', 'target': library,
                    'profile': {'test': True}, 'features': [], 'filenames': [executable],
                    'executable': executable}
        compiled = '\n'.join(json.dumps(x) for x in [artifact, {'reason': 'build-finished', 'success': True}])
        graph, _ = ci.artifact_graph(compiled, {})
        with tempfile.TemporaryDirectory() as directory:
            directory = Path(directory)
            target = {'id': 'test:app:lib:app', 'cases': ['case'], 'assigned': 'remainder',
                      'original_args': [], 'cwd': '/original', 'cargo_manifest_dir': '/original'}
            doc = {**target, 'id': 'doc:app:app', 'cases': []}
            for receipt in data:
                output = directory / receipt['allocation']
                output.mkdir()

                def log_pair(name, stdout):
                    pair = {}
                    for stream, text in [('stdout', stdout), ('stderr', '')]:
                        path = output / f'{name}.{stream}.log'
                        path.write_text(text, encoding='utf8')
                        pair[stream] = {'path': path.name, 'sha256': ci.file_hash(path)}
                    return pair

                receipt['common'].update(metadata=metadata, graph=graph, universe=[target, doc],
                                         absent_allocations=list(ci.ALLOCATIONS[:-1]))
                receipt['common_sha256'] = ci.digest(receipt['common'])
                receipt['executables'] = [{'executable': executable}]
                receipt['command_logs'] = [log_pair('metadata', json.dumps(metadata)),
                                           log_pair('build', compiled), log_pair('cargo-execution', compiled)]
                receipt['records'] = []
                for item in [target, doc]:
                    record = copy.deepcopy(item)
                    prefix = ci.digest(item['id'])
                    names = ''.join(f'{name}: test\n' for name in item['cases'])
                    record['logs'] = [log_pair(prefix + '-list', names + f"{len(item['cases'])} tests, 0 benchmarks\n"),
                                      log_pair(prefix + '-ignored', '0 tests, 0 benchmarks\n')]
                    record['state'] = 'listed-only'
                    if receipt['allocation'] == 'remainder':
                        record['state'] = 'executed'
                        record['logs'].append(log_pair(prefix + '-run', complete('case', len(item['cases']))))
                        record['outcome'] = {'passed': len(item['cases']), 'summaries': 1}
                    receipt['records'].append(record)
                ci.write_json(output / 'complete.json', receipt)
            args = SimpleNamespace(directory=str(directory), platform='linux')
            with patch.object(ci, 'source_identity', return_value=source), patch.dict(
                    os.environ, {'GITHUB_RUN_ID': '10', 'GITHUB_RUN_ATTEMPT': '1'}):
                ci.aggregate(args)
                listing = directory / 'remainder' / f'{ci.digest(target["id"])}-list.stdout.log'
                original = listing.read_bytes()
                listing.write_text('0 tests, 0 benchmarks\n')
                with self.assertRaisesRegex(ValueError, 'changed original harness log'):
                    ci.aggregate(args)
                listing.write_bytes(original)
                listing.unlink()
                with self.assertRaises(FileNotFoundError):
                    ci.aggregate(args)

    def test_original_metadata_and_artifacts_reconstruct_complete_coverage(self):
        library = {'name': 'dmd_app', 'kind': ['lib'], 'doctest': True}
        integration = {'name': 'table_loop', 'kind': ['test'], 'doctest': False}
        metadata = {'workspace_members': ['app'], 'packages': [
            {'id': 'app', 'name': 'dmd-app', 'targets': [library, integration]}]}
        messages = [{'package_id': 'app', 'target': target} for target in [library, integration]]
        expected = ci.discovered_assignments(metadata, messages)
        self.assertEqual(expected, {'test:dmd-app:lib:dmd_app': 'remainder',
                                    'test:dmd-app:test:table_loop': 'table-loop',
                                    'doc:dmd-app:dmd_app': 'remainder'})
        for invalid in (messages + messages[:1], [{'package_id': 'unknown', 'target': library}],
                        [{'package_id': 'app', 'target': {'name': 'invented'}}]):
            with self.subTest(invalid=invalid), self.assertRaises(ValueError):
                ci.discovered_assignments(metadata, invalid)

    def test_only_exact_instrumentation_is_removed_from_rustdoc(self):
        runner = ['python', '-B', 'ci_runtime.py', 'runner']
        original = ['--crate-name', 'demo', '--test', 'src/lib.rs', '--test-run-directory', '/pkg']
        injected = original + ['--test-runtool', runner[0]]
        for arg in runner[1:]:
            injected += ['--test-runtool-arg', arg]
        self.assertEqual(ci.restore_rustdoc_args(injected, runner), original)
        for invalid in (original, injected + ['--test-args', 'only_one'],
                        injected + ['--test-runtool', 'other'], injected[:-1] + ['different']):
            with self.subTest(invalid=invalid), self.assertRaises(ValueError):
                ci.restore_rustdoc_args(invalid, runner)

    def test_compilation_graph_keeps_compile_only_and_platform_profiles(self):
        base = {'reason': 'compiler-artifact', 'package_id': 'path+file:///repo#demo@0.1.0',
                'target': {'name': 'demo', 'kind': ['lib'], 'src_path': '/repo/src/lib.rs'},
                'profile': {'test': False, 'opt_level': '0'}, 'features': [],
                'filenames': ['/repo/target/debug/libdemo.rlib'], 'executable': None}
        test = copy.deepcopy(base)
        test['profile']['test'] = True
        test['executable'] = '/repo/target/debug/deps/demo-hash'
        test['filenames'] = [test['executable']]
        text = '\n'.join(json.dumps(x) for x in [base, test, {'reason': 'build-finished', 'success': True}])
        graph, executables = ci.artifact_graph(text, {'$WORKSPACE': '/repo'})
        self.assertEqual(len(graph), 2)
        self.assertEqual(len(executables), 1)
        self.assertIn('$WORKSPACE', ci.canonical(graph))
        with self.assertRaises(ValueError):
            ci.artifact_graph(text.replace('"success": true', '"success": false'), {})

    def test_custom_harness_is_refused_before_invocation(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory).resolve()
            manifest = root / 'Cargo.toml'
            manifest.write_text('[package]\nname="demo"\nversion="0.1.0"\n[[test]]\nname="custom"\nharness=false\n')
            metadata = {'workspace_members': ['demo'], 'packages': [
                {'id': 'demo', 'name': 'demo', 'manifest_path': str(manifest), 'targets': []}]}
            with self.assertRaisesRegex(ValueError, 'harness=false'):
                ci.workspace_targets(metadata, root)

    def test_real_python_child_preserves_cwd_environment_and_distinguishes_listing(self):
        with tempfile.TemporaryDirectory() as directory:
            output = Path(directory)
            fixture = output / 'fixture.py'
            fixture.write_text("import os,sys\n"
                               "assert os.environ['DMD_CI_UNIT_PROBE']=='inherited'\n"
                               "assert os.getcwd()==os.environ['DMD_CI_UNIT_CWD']\n"
                               "if '--ignored' in sys.argv: print('0 tests, 0 benchmarks')\n"
                               "elif '--list' in sys.argv: print('example: test\\n1 test, 0 benchmarks')\n"
                               "else: print('test example ... ok\\ntest result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s')\n")
            with patch.dict(os.environ, {'DMD_CI_UNIT_PROBE': 'inherited', 'DMD_CI_UNIT_CWD': os.getcwd()}):
                ci.execute_harness('probe', [sys.executable, str(fixture)], [], 'remainder', 'remainder', output)
                receipt = ci.read_json(output / f'{ci.digest("probe")}.receipt.json')
                self.assertEqual(receipt['state'], 'executed')
                ci.execute_harness('listed', [sys.executable, str(fixture)], [], 'table-loop', 'remainder', output)
                listed = ci.read_json(output / f'{ci.digest("listed")}.receipt.json')
                self.assertEqual(listed['state'], 'listed-only')
                self.assertNotIn('outcome', listed)
                self.assertEqual(len(listed['logs']), 2)


class ExecutableDiagnostics(unittest.TestCase):
    def test_changed_executable_records_both_hashes_and_refuses_before_any_harness(self):
        with tempfile.TemporaryDirectory() as directory:
            output = Path(directory).resolve()
            executable = output / 'fixture.exe'
            executable.write_bytes(b'original')
            target = {'id': 'test:fixture:bin:fixture', 'executable': str(executable),
                      'sha256': ci.file_hash(executable), 'package_root': str(Path.cwd()),
                      'allocation': 'remainder'}
            context = output / 'context.json'
            ci.write_json(context, {'output': str(output), 'allocation': 'remainder', 'executables': [target]})
            executable.write_bytes(b'rebuilt')
            with patch.dict(os.environ, {'DMD_CI_RUNTIME_CONTEXT': str(context)}), patch.object(
                    ci, 'execute_harness') as execute:
                with self.assertRaisesRegex(ValueError, 'executable changed after canonical no-run build'):
                    ci.delegate('runner', [str(executable)])
                execute.assert_not_called()
            diagnostic = ci.read_json(output / f'diagnostic-executable-{ci.digest(target["id"])}-before-execution.json')
            self.assertEqual(diagnostic['expected_sha256'], target['sha256'])
            self.assertEqual(diagnostic['actual_sha256'], ci.file_hash(executable))
            self.assertNotEqual(diagnostic['actual_sha256'], diagnostic['expected_sha256'])
            self.assertTrue(diagnostic['diagnostic_only'])
            self.assertEqual(diagnostic['target'], target)
            self.assertEqual(list(output.glob('*.receipt.json')), [])
            self.assertFalse((output / 'complete.json').exists())

    def test_matching_bytes_execute_and_later_changes_still_refuse(self):
        with tempfile.TemporaryDirectory() as directory:
            output = Path(directory).resolve()
            executable = output / 'fixture.exe'
            executable.write_bytes(b'original')
            target = {'id': 'test:fixture:bin:fixture', 'executable': str(executable),
                      'sha256': ci.file_hash(executable), 'package_root': str(Path.cwd()),
                      'allocation': 'remainder'}
            context = output / 'context.json'
            ci.write_json(context, {'output': str(output), 'allocation': 'remainder', 'executables': [target]})
            with patch.dict(os.environ, {'DMD_CI_RUNTIME_CONTEXT': str(context)}), patch.object(
                    ci, 'execute_harness') as execute:
                self.assertEqual(ci.delegate('runner', [str(executable)]), 0)
                execute.assert_called_once_with(target['id'], [str(executable)], [], 'remainder', 'remainder', output)
                self.assertEqual(list(output.glob('diagnostic-*.json')), [])
                execute.side_effect = lambda *args: executable.write_bytes(b'changed-during-execution')
                with self.assertRaisesRegex(ValueError, 'executable changed during execution'):
                    ci.delegate('runner', [str(executable)])
            diagnostic = ci.read_json(output / f'diagnostic-executable-{ci.digest(target["id"])}-after-execution.json')
            self.assertEqual(diagnostic['expected_sha256'], target['sha256'])
            self.assertEqual(diagnostic['actual_sha256'], ci.file_hash(executable))
            with self.assertRaisesRegex(ValueError, 'final executable hash changed'):
                ci.verify_executable(target, output, 'final', 'final executable hash changed')
            self.assertFalse((output / 'complete.json').exists())

    def test_phase_diagnostic_records_exact_invocation_and_only_allowlisted_environment(self):
        with tempfile.TemporaryDirectory() as directory:
            output = Path(directory).resolve()
            argv = ['cargo', 'test', '--locked', '--workspace', '--no-run']
            env = {'CARGO_LOG': ci.FINGERPRINT_LOG, 'RUSTDOC': '/instrumented/rustdoc',
                   'UNRELATED_PRIVATE_VALUE': 'must-not-be-recorded'}
            source = {'head': 'a' * 40, 'tree': 'b' * 40}
            ci.record_phase(output, 'build', argv, env, output, source)
            diagnostic = ci.read_json(output / 'diagnostic-phase-build.json')
            self.assertEqual(diagnostic['argv'], argv)
            self.assertEqual(diagnostic['source'], source)
            self.assertEqual(diagnostic['cwd'], str(output))
            self.assertEqual(diagnostic['environment'], {key: env.get(key) for key in ci.DIAGNOSTIC_ENV})
            self.assertNotIn('must-not-be-recorded', ci.canonical(diagnostic))
            self.assertEqual(diagnostic['python']['sha256'], ci.file_hash(sys.executable))
            self.assertEqual(diagnostic['runner']['sha256'], ci.file_hash(ci.__file__))
            self.assertEqual(list(output.glob('*.receipt.json')), [])


if __name__ == '__main__':
    unittest.main()

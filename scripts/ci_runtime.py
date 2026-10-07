#!/usr/bin/env python3
"""Partition whole Cargo test executables; require their complete executed union.

Cargo owns resolution, compilation, cwd, environment and runtime library paths.
The runner changes only which complete harness executes in each hosted job.
Listed-only harnesses are never represented as successful executions.
"""
import argparse
import hashlib
import json
import os
from pathlib import Path
import re
import shlex
import subprocess
import sys
import tomllib

ALLOCATIONS = ('table-loop', 'legacy-missile', 'grapple-public', 'remainder')
SPECIAL = {'table_loop': 'table-loop', 'legacy_shield_missile_v1_replay': 'legacy-missile',
           'table_grapple_public': 'grapple-public'}
SCHEMA = 1
ANSI = re.compile(r'\x1b\[[0-?]*[ -/]*[@-~]')
FINGERPRINT_LOG = 'cargo::compiler::fingerprint=info'
DIAGNOSTIC_ENV = ('CARGO_LOG', 'RUSTDOC', 'DMD_CI_RUNTIME_CONTEXT', 'CARGO_HOME', 'RUSTUP_HOME',
                  'RUSTUP_TOOLCHAIN', 'RUSTC', 'RUSTC_WRAPPER', 'RUSTC_WORKSPACE_WRAPPER',
                  'RUSTFLAGS', 'CARGO_ENCODED_RUSTFLAGS', 'RUSTDOCFLAGS',
                  'CARGO_ENCODED_RUSTDOCFLAGS', 'CARGO_INCREMENTAL')


def require(condition, message):
    if not condition:
        raise ValueError(message)


def canonical(value):
    return json.dumps(value, sort_keys=True, separators=(',', ':'), ensure_ascii=False)


def digest(value):
    return hashlib.sha256(canonical(value).encode()).hexdigest()


def file_hash(path):
    h = hashlib.sha256()
    with Path(path).open('rb') as stream:
        for block in iter(lambda: stream.read(1024 * 1024), b''):
            h.update(block)
    return h.hexdigest()


def write_json(path, value):
    path = Path(path)
    path.parent.mkdir(parents=True, exist_ok=True)
    with path.open('x', encoding='utf8', newline='\n') as stream:
        json.dump(value, stream, indent=2, sort_keys=True)
        stream.write('\n')


def read_json(path):
    return json.loads(Path(path).read_text(encoding='utf8'))


def record_phase(output, phase, argv, env, root, source):
    write_json(output / f'diagnostic-phase-{phase}.json', {
        'diagnostic_only': True, 'phase': phase, 'argv': argv, 'cwd': str(root), 'source': source,
        'run_id': os.environ.get('GITHUB_RUN_ID'), 'run_attempt': os.environ.get('GITHUB_RUN_ATTEMPT'),
        'environment': {key: env.get(key) for key in DIAGNOSTIC_ENV},
        'python': {'path': sys.executable, 'sha256': file_hash(sys.executable), 'version': sys.version},
        'runner': {'path': str(Path(__file__).resolve()), 'sha256': file_hash(__file__)}})


def verify_executable(target, output, phase, message):
    path = Path(target['executable']).resolve()
    actual = file_hash(path)
    if actual != target['sha256']:
        write_json(output / f'diagnostic-executable-{digest(target["id"])}-{phase}.json', {
            'diagnostic_only': True, 'phase': phase, 'target': target, 'path': str(path),
            'expected_sha256': target['sha256'], 'actual_sha256': actual})
    require(actual == target['sha256'], message)


def command(args, cwd=None):
    return subprocess.check_output(args, cwd=cwd, text=True, encoding='utf8').strip()


def source_identity(root):
    require(not command(['git', 'status', '--porcelain', '--untracked-files=normal'], root),
            'source must be committed and clean')
    return {'head': command(['git', 'rev-parse', 'HEAD'], root),
            'tree': command(['git', 'rev-parse', 'HEAD^{tree}'], root),
            'lock_sha256': file_hash(root / 'Cargo.lock')}


def normalize(value, roots):
    if isinstance(value, str):
        value = value.replace('\\', '/')
        for label, path in roots.items():
            value = value.replace(str(path).replace('\\', '/'), label)
        return value
    if isinstance(value, list):
        return [normalize(x, roots) for x in value]
    if isinstance(value, dict):
        return {k: normalize(v, roots) for k, v in value.items()}
    return value


def allocation(package, target, kind):
    if package == 'dmd-app' and kind == ['test']:
        return SPECIAL.get(target, 'remainder')
    return 'remainder'


def parse_listing(text):
    names, totals = [], []
    for line in ANSI.sub('', text).splitlines():
        if not line.strip():
            continue
        if match := re.fullmatch(r'(.+): test', line):
            names.append(match[1])
        elif match := re.fullmatch(r'(\d+) tests?, (\d+) benchmarks?', line):
            require(int(match[2]) == 0, 'benchmark semantics unsupported')
            totals.append(int(match[1]))
        else:
            raise ValueError(f'unsupported test listing line: {line}')
    require(totals and sum(totals) == len(names), 'incomplete listing')
    require(len(names) == len(set(names)), 'duplicate listed case')
    return sorted(names)


def parse_result(text, expected, docs=False):
    outcomes, summaries = [], []
    for line in ANSI.sub('', text).splitlines():
        if match := re.fullmatch(r'test (.+?) \.\.\. (ok|FAILED|ignored)(?: .*)?', line):
            outcomes.append((match[1], match[2]))
        elif match := re.fullmatch(
                r'test result: (ok|FAILED)\. (\d+) passed; (\d+) failed; (\d+) ignored; '
                r'(\d+) measured; (\d+) filtered out; finished in ([\d.]+)(?:s|ms)', line):
            require(match[1] == 'ok' and all(int(match[i]) == 0 for i in range(3, 7)),
                    'failed, ignored, measured or filtered outcomes')
            summaries.append(int(match[2]))
    require(summaries and (docs or len(summaries) == 1), 'missing/ambiguous harness completion')
    require(all(outcome == 'ok' for _, outcome in outcomes), 'non-passing named outcome')
    require(len(outcomes) == len(set(name for name, _ in outcomes)), 'duplicate executed case')
    require(sorted(name for name, _ in outcomes) == expected, 'executed case inventory differs')
    require(sum(summaries) == len(expected), 'summary count differs from named outcomes')
    return {'passed': len(expected), 'summaries': len(summaries)}


def artifact_graph(text, roots):
    graph, executables, finished = {}, [], []
    for line in text.splitlines():
        if not line.startswith('{'):
            continue
        message = json.loads(line)
        reason = message.get('reason')
        if reason == 'build-finished':
            finished.append(message['success'])
        elif reason == 'compiler-artifact':
            record = {k: message[k] for k in ('package_id', 'target', 'profile', 'features')}
            # Keep actual host/target output location and extension, without
            # requiring independently linked binary bytes/PDBs to be identical.
            record['outputs'] = sorted({str(Path(p).parent / Path(p).suffix)
                                        for p in message['filenames']})
            record = normalize(record, roots)
            graph[canonical(record)] = record
            if message['profile']['test'] and message.get('executable'):
                executables.append(message)
        elif reason == 'build-script-executed':
            record = normalize({k: message[k] for k in
                                ('package_id', 'linked_libs', 'linked_paths', 'cfgs', 'env', 'out_dir')}, roots)
            graph[canonical(record)] = record
    require(finished == [True], 'Cargo compilation did not finish successfully')
    require(graph and executables, 'empty canonical compilation inventory')
    return sorted(graph.values(), key=canonical), executables


def workspace_targets(metadata, root):
    members = set(metadata['workspace_members'])
    packages = {p['id']: p for p in metadata['packages'] if p['id'] in members}
    require(set(packages) == members and packages, 'missing workspace package')
    docs = []
    for package in packages.values():
        manifest = Path(package['manifest_path']).resolve()
        require(manifest.is_relative_to(root), 'workspace package outside checkout')
        content = tomllib.loads(manifest.read_text(encoding='utf8'))
        declarations = [content.get('lib', {})]
        for kind in ('bin', 'test', 'bench', 'example'):
            declarations.extend(content.get(kind, []))
        require(all(d.get('harness', True) is True for d in declarations),
                'harness=false requires a separately reviewed executor')
        for target in package['targets']:
            if target['doctest']:
                require(target['kind'] == ['lib'], 'unsupported doctest target kind')
                docs.append({'id': f"doc:{package['name']}:{target['name']}",
                             'package': package['name'], 'target': target['name'],
                             'source': str(Path(target['src_path']).resolve()),
                             'package_root': str(manifest.parent)})
    return packages, docs


def inventory_executables(messages, packages, root):
    records, seen = [], set()
    for message in messages:
        require(message['package_id'] in packages, 'test executable outside workspace')
        package = packages[message['package_id']]
        target = message['target']
        path = Path(message['executable']).resolve()
        require(path.is_relative_to(root / 'target') and path.is_file(), 'unexpected executable path')
        identity = f"test:{package['name']}:{','.join(target['kind'])}:{target['name']}"
        require(identity not in seen, 'duplicate test executable identity')
        require(target in package['targets'], 'compiled target not in Cargo metadata')
        seen.add(identity)
        records.append({'id': identity, 'package': package['name'], 'target': target['name'],
                        'kind': target['kind'], 'package_root': str(Path(package['manifest_path']).parent),
                        'executable': str(path), 'sha256': file_hash(path),
                        'allocation': allocation(package['name'], target['name'], target['kind'])})
    return sorted(records, key=lambda x: x['id'])


def discovered_assignments(metadata, messages):
    """Reconstruct coverage from original Cargo output without executable transfer."""
    members = set(metadata['workspace_members'])
    packages = {p['id']: p for p in metadata['packages'] if p['id'] in members}
    require(set(packages) == members and packages, 'missing workspace metadata')
    expected = {}
    for message in messages:
        require(message['package_id'] in packages, 'unexpected compiled test package')
        package = packages[message['package_id']]
        target = message['target']
        require(target in package['targets'], 'compiled target missing from metadata')
        identity = f"test:{package['name']}:{','.join(target['kind'])}:{target['name']}"
        require(identity not in expected, 'duplicate compiled test identity')
        expected[identity] = allocation(package['name'], target['name'], target['kind'])
    for package in packages.values():
        for target in package['targets']:
            if target['doctest']:
                require(target['kind'] == ['lib'], 'unsupported doctest target kind')
                identity = f"doc:{package['name']}:{target['name']}"
                require(identity not in expected, 'duplicate doctest identity')
                expected[identity] = 'remainder'
    return expected


def run_logged(args, prefix, *, cwd=None, env=None):
    """No shell; retain Cargo's inherited runtime cwd/env and jobserver handles."""
    prefix = Path(prefix)
    prefix.parent.mkdir(parents=True, exist_ok=True)
    out, err = prefix.with_suffix('.stdout.log'), prefix.with_suffix('.stderr.log')
    with out.open('xb') as stdout, err.open('xb') as stderr:
        code = subprocess.call(args, cwd=cwd, env=env, stdout=stdout, stderr=stderr,
                               close_fds=False)
    logs = {'stdout': {'path': out.name, 'sha256': file_hash(out)},
            'stderr': {'path': err.name, 'sha256': file_hash(err)}}
    if code:
        print(err.read_text(encoding='utf8', errors='replace')[-12000:], file=sys.stderr)
        print(out.read_text(encoding='utf8', errors='replace')[-12000:], file=sys.stderr)
    require(code == 0, f'command failed ({code}); complete logs at {prefix}')
    return out.read_text(encoding='utf8'), logs


def restore_rustdoc_args(args, runner):
    remaining, actual = [], []
    index = 0
    while index < len(args):
        arg = args[index]
        if arg in ('--test-runtool', '--test-runtool-arg'):
            require(index + 1 < len(args), 'incomplete rustdoc runtool')
            if arg == '--test-runtool':
                require(not actual, 'duplicate rustdoc runtool')
            else:
                require(actual, 'runtool argument without tool')
            actual.append(args[index + 1])
            index += 2
        else:
            require(not arg.startswith(('--test-runtool=', '--test-runtool-arg=')),
                    'unexpected rustdoc runtool syntax')
            remaining.append(arg)
            index += 1
    require(actual == runner, 'rustdoc runtool differs from exact injected runner')
    require('--test-args' not in remaining and not any(x.startswith('@') for x in remaining),
            'unexpected doctest filters or argument file')
    return remaining


def execute_harness(identity, base, original_args, allocation_name, assigned, output, *, docs=False,
                    injected_args=None):
    key = digest(identity)
    listing, list_logs = run_logged(base + original_args +
                                   (['--test-args', '--list'] if docs else ['--list']), output / f'{key}-list')
    names = parse_listing(listing)
    ignored, ignored_logs = run_logged(base + original_args +
                                      (['--test-args', '--list', '--test-args', '--ignored']
                                       if docs else ['--list', '--ignored']), output / f'{key}-ignored')
    require(not parse_listing(ignored), 'ignored cases are not permitted')
    record = {'id': identity, 'cases': names, 'assigned': assigned, 'state': 'listed-only',
              'logs': [list_logs, ignored_logs], 'original_args': original_args,
              'cwd': str(Path.cwd()), 'cargo_manifest_dir': os.environ.get('CARGO_MANIFEST_DIR')}
    if injected_args is not None:
        record['injected_args'] = injected_args
    if assigned == allocation_name:
        print(f'Executing whole harness {identity} ({len(names)} cases)', flush=True)
        result, result_logs = run_logged(base + original_args, output / f'{key}-run')
        record['outcome'] = parse_result(result, names, docs)
        record['logs'].append(result_logs)
        record['state'] = 'executed'
        print(f'Completed whole harness {identity}: {len(names)} passed', flush=True)
    write_json(output / f'{key}.receipt.json', record)


def delegate(mode, args):
    context = read_json(os.environ['DMD_CI_RUNTIME_CONTEXT'])
    output = Path(context['output'])
    if mode == 'runner':
        require(len(args) == 1, 'canonical harness received unexpected arguments')
        path = Path(args[0]).resolve()
        matches = [x for x in context['executables'] if Path(x['executable']).resolve() == path]
        require(len(matches) == 1, 'Cargo invoked an unlisted executable')
        target = matches[0]
        require(Path.cwd().resolve() == Path(target['package_root']).resolve(), 'Cargo package cwd differs')
        verify_executable(target, output, 'before-execution', 'executable changed after canonical no-run build')
        execute_harness(target['id'], [str(path)], [], context['allocation'],
                        target['allocation'], output)
        verify_executable(target, output, 'after-execution', 'executable changed during execution')
    else:
        tool = context['rustdoc']
        if '--test' not in args:
            return subprocess.call([tool] + args, close_fds=False)
        restored = restore_rustdoc_args(args, context['runner'])
        require(restored.count('--crate-name') == 1 and restored.count('--test-run-directory') == 1,
                'unsupported rustdoc command identity')
        name = restored[restored.index('--crate-name') + 1]
        directory = Path(restored[restored.index('--test-run-directory') + 1]).resolve()
        matches = [x for x in context['docs'] if x['target'].replace('-', '_') == name
                   and Path(x['package_root']).resolve() == directory]
        require(len(matches) == 1, 'unlisted doctest target')
        target = matches[0]
        sources = [Path(x).resolve() for x in restored if x.endswith('.rs')]
        require(sources == [Path(target['source']).resolve()], 'rustdoc source differs from metadata')
        require(Path.cwd().resolve() in (Path(context['root']).resolve(), directory),
                'unexpected Cargo rustdoc cwd')
        execute_harness(target['id'], [tool], restored, context['allocation'], 'remainder', output,
                        docs=True, injected_args=args)
    return 0


def clean_configuration(root):
    forbidden = ('RUSTFLAGS', 'CARGO_ENCODED_RUSTFLAGS', 'RUSTDOCFLAGS', 'CARGO_ENCODED_RUSTDOCFLAGS',
                 'RUSTDOC', 'RUSTC_WRAPPER', 'RUSTC_WORKSPACE_WRAPPER', 'RUST_TEST_THREADS',
                 'RUST_TEST_NOCAPTURE', 'RUST_TEST_TIME_UNIT', 'RUST_TEST_TIME_INTEGRATION',
                 'RUST_MIN_STACK', 'CARGO_BUILD_TARGET', 'CARGO_TARGET_DIR', 'CARGO_BUILD_TARGET_DIR')
    require(not any(os.environ.get(key) for key in forbidden), 'unsupported ambient test/compiler override')
    require(not any(key.startswith(('CARGO_PROFILE_', 'CARGO_UNSTABLE_')) or
                    (key.startswith('CARGO_TARGET_') and key.endswith(('_RUNNER', '_RUSTFLAGS')))
                    for key in os.environ), 'unsupported Cargo profile/runner configuration')
    dirs = [root, *root.parents, Path(os.environ.get('CARGO_HOME', Path.home() / '.cargo'))]
    for parent in dirs:
        candidates = [parent / '.cargo' / 'config', parent / '.cargo' / 'config.toml']
        if parent == dirs[-1]:
            candidates += [parent / 'config', parent / 'config.toml']
        require(not any(path.exists() for path in candidates), 'custom Cargo config requires review')


def make_shim(path, script):
    if os.name == 'nt':
        # Cargo/Windows supports .cmd tools. No interpolation of command arguments
        # occurs in Python; %* forwards Cargo's own quoted argument vector.
        require(not any(c in str(script) + sys.executable for c in '%!\r\n'), 'unsupported shim path')
        text = f'@"{sys.executable}" -B "{script}" rustdoc %*\r\n'
    else:
        text = '#!/bin/sh\nexec ' + shlex.quote(sys.executable) + ' -B ' + shlex.quote(str(script)) + ' rustdoc "$@"\n'
    with path.open('x', encoding='utf8', newline='') as stream:
        stream.write(text)
    path.chmod(0o755)


def run_allocation(args):
    root = Path(__file__).resolve().parent.parent
    clean_configuration(root)
    identity = source_identity(root)
    output = root / 'target' / 'ci-runtime' / args.allocation
    require(not output.exists(), 'allocation output must be fresh')
    output.mkdir(parents=True)
    rustc = command(['rustc', '-vV'])
    cargo = command(['cargo', '-vV'])
    host = next(line.removeprefix('host: ') for line in rustc.splitlines() if line.startswith('host: '))
    require((args.platform == 'windows' and host == 'x86_64-pc-windows-msvc') or
            (args.platform == 'linux' and host == 'x86_64-unknown-linux-gnu'), 'unexpected host')
    target = ['--target', 'x86_64-pc-windows-msvc'] if args.platform == 'windows' else []
    base = ['cargo', 'test', '--locked', '--workspace', *target, '--message-format=json']
    roots = {'$WORKSPACE': str(root), '$CARGO_HOME': os.environ.get('CARGO_HOME', str(Path.home() / '.cargo')),
             '$RUSTUP_HOME': os.environ.get('RUSTUP_HOME', str(Path.home() / '.rustup'))}
    metadata_text, metadata_logs = run_logged(['cargo', 'metadata', '--locked', '--format-version=1'],
                                             output / 'metadata', cwd=root)
    metadata = json.loads(metadata_text)
    packages, docs = workspace_targets(metadata, root)
    build_env = os.environ.copy()
    build_env['CARGO_LOG'] = FINGERPRINT_LOG
    record_phase(output, 'build', base + ['--no-run'], build_env, root, identity)
    built, build_logs = run_logged(base + ['--no-run'], output / 'build', cwd=root, env=build_env)
    graph, messages = artifact_graph(built, roots)
    executables = inventory_executables(messages, packages, root)
    script = Path(__file__).resolve()
    runner = [sys.executable, '-B', str(script), 'runner']
    context = {'root': str(root), 'output': str(output), 'allocation': args.allocation,
               'executables': executables, 'docs': docs, 'runner': runner,
               'rustdoc': command(['rustup', 'which', 'rustdoc'])}
    context_path = output / 'context.json'
    write_json(context_path, context)
    shim = output / ('rustdoc.cmd' if os.name == 'nt' else 'rustdoc')
    make_shim(shim, script)
    env = os.environ.copy()
    env['CARGO_LOG'] = FINGERPRINT_LOG
    env['DMD_CI_RUNTIME_CONTEXT'] = str(context_path)
    env['RUSTDOC'] = str(shim)
    config = f'target.{host}.runner={json.dumps(runner)}'
    record_phase(output, 'execution', base + ['--config', config], env, root, identity)
    executed, execution_logs = run_logged(base + ['--config', config], output / 'cargo-execution', cwd=root, env=env)
    runtime_graph, runtime_messages = artifact_graph(executed, roots)
    require(runtime_graph == graph, 'Cargo build graph changed between no-run and execution')
    require(inventory_executables(runtime_messages, packages, root) == executables,
            'Cargo changed executable identity between no-run and execution')
    records = [read_json(path) for path in sorted(output.glob('*.receipt.json'))]
    expected = {x['id']: x['allocation'] for x in executables}
    expected.update({x['id']: 'remainder' for x in docs})
    require(len(records) == len(expected) and {x['id'] for x in records} == set(expected),
            'Cargo did not invoke every expected executable/doctest')
    for record in records:
        require(record['assigned'] == expected[record['id']], 'wrong allocation')
        require(record['state'] == ('executed' if record['assigned'] == args.allocation else 'listed-only'),
                'assigned harness did not execute')
    require(source_identity(root) == identity, 'source changed during allocation')
    for executable in executables:
        verify_executable(executable, output, 'final', 'final executable hash changed')
    universe = [normalize({k: x[k] for k in ('id', 'cases', 'assigned', 'original_args',
                                           'cwd', 'cargo_manifest_dir')}, roots) for x in records]
    universe.sort(key=lambda x: x['id'])
    common = {'schema': SCHEMA, 'source': identity, 'platform': args.platform, 'host': host,
              'rustc': rustc, 'cargo': cargo, 'base_command': base,
              'run_id': os.environ['GITHUB_RUN_ID'], 'run_attempt': os.environ['GITHUB_RUN_ATTEMPT'],
              'metadata': normalize(metadata, roots), 'graph': graph, 'universe': universe,
              'absent_allocations': [name for name in ALLOCATIONS if name not in expected.values()]}
    # Full per-job artifact hashes/paths stay outside cross-job equality because
    # separately linked artifacts need not be byte reproducible.
    receipt = {'schema': SCHEMA, 'status': 'complete', 'allocation': args.allocation,
               'common': common, 'common_sha256': digest(common), 'executables': executables,
               'records': records, 'normalization_roots': roots,
               'command_logs': [metadata_logs, build_logs, execution_logs]}
    write_json(output / 'complete.json', receipt)
    print(f'Allocation {args.allocation} complete; overall workspace pass awaits all {len(ALLOCATIONS)} allocations.')


def validate_receipts(receipts, source, platform, run_id, run_attempt):
    require(len(receipts) == len(ALLOCATIONS), 'missing/extra allocation receipt')
    require({r['allocation'] for r in receipts} == set(ALLOCATIONS), 'duplicate/missing allocation')
    baseline = receipts[0]['common']
    require(baseline['schema'] == SCHEMA, 'unsupported discovery schema')
    require(baseline['source'] == source and baseline['platform'] == platform,
            'wrong source or platform')
    require(baseline['run_id'] == run_id and baseline['run_attempt'] == run_attempt, 'wrong workflow run')
    universe = baseline['universe']
    require(len(universe) == len({x['id'] for x in universe}) and universe, 'duplicate/empty universe')
    for item in universe:
        require(item['assigned'] in ALLOCATIONS and item['cases'] == sorted(set(item['cases'])),
                'invalid assignment or case inventory')
    require(baseline['absent_allocations'] ==
            [name for name in ALLOCATIONS if name not in {x['assigned'] for x in universe}],
            'incorrect absent-allocation inventory')
    expected = {x['id']: x for x in universe}
    executed = []
    for receipt in receipts:
        require(receipt['schema'] == SCHEMA and receipt['status'] == 'complete', 'incomplete allocation')
        require(receipt['common'] == baseline and receipt['common_sha256'] == digest(baseline),
                'discovery/configuration changed across jobs')
        records = receipt['records']
        require(len(records) == len(universe) and {x['id'] for x in records} == set(expected),
                'incomplete per-job universe')
        for record in records:
            item = expected[record['id']]
            actual = normalize({k: record[k] for k in item}, receipt['normalization_roots'])
            require(actual == item, 'case listing, invocation or assignment changed')
            required = record['assigned'] == receipt['allocation']
            require(record['state'] == ('executed' if required else 'listed-only'), 'false execution state')
            if required:
                require(record.get('outcome', {}).get('passed') == len(record['cases']),
                        'missing/incomplete runtime outcome')
                executed.append(record['id'])
            else:
                require('outcome' not in record, 'listed-only target carries a false runtime outcome')
    require(len(executed) == len(set(executed)) and set(executed) == set(expected),
            'executed union is not complete and disjoint')
    return {'targets': len(expected), 'cases': sum(len(x['cases']) for x in universe),
            'common_sha256': digest(baseline)}


def aggregate(args):
    root = Path(__file__).resolve().parent.parent
    identity = source_identity(root)
    directory = Path(args.directory)
    paths = sorted(directory.glob('*/complete.json'))
    receipts = [read_json(path) for path in paths]
    result = validate_receipts(receipts, identity, args.platform,
                               os.environ['GITHUB_RUN_ID'], os.environ['GITHUB_RUN_ATTEMPT'])
    # Re-read original complete logs, not only the compact success statements.
    for path, receipt in zip(paths, receipts):
        for record in receipt['records']:
            require(len(record['logs']) == (3 if record['state'] == 'executed' else 2),
                    'incomplete or unexpected harness logs')
            for pair in record['logs']:
                require(set(pair) == {'stdout', 'stderr'}, 'incomplete log streams')
                for log in pair.values():
                    require(Path(log['path']).name == log['path'], 'unsafe log path')
                    log_path = path.parent / log['path']
                    require(log_path.parent == path.parent and file_hash(log_path) == log['sha256'],
                            'missing/changed original harness log')
            require(parse_listing((path.parent / record['logs'][0]['stdout']['path']).read_text(encoding='utf8'))
                    == record['cases'], 'original listing disagrees')
            require(not parse_listing((path.parent / record['logs'][1]['stdout']['path']).read_text(encoding='utf8')),
                    'ignored cases in original log')
            if record['state'] == 'executed':
                parsed = parse_result((path.parent / record['logs'][2]['stdout']['path']).read_text(encoding='utf8'),
                                      record['cases'], record['id'].startswith('doc:'))
                require(parsed == record['outcome'], 'original execution differs from receipt')
        require(len(receipt['command_logs']) == 3, 'incomplete canonical Cargo logs')
        for pair in receipt['command_logs']:
            require(set(pair) == {'stdout', 'stderr'}, 'incomplete Cargo log streams')
            for log in pair.values():
                require(Path(log['path']).name == log['path'], 'unsafe Cargo log path')
                require(file_hash(path.parent / log['path']) == log['sha256'], 'missing/changed Cargo log')
        texts = [(path.parent / pair['stdout']['path']).read_text(encoding='utf8')
                 for pair in receipt['command_logs']]
        roots = receipt['normalization_roots']
        metadata = json.loads(texts[0])
        require(normalize(metadata, roots) == receipt['common']['metadata'],
                'original Cargo metadata differs')
        for original in texts[1:]:
            graph, messages = artifact_graph(original, roots)
            require(graph == receipt['common']['graph'], 'original Cargo graph differs')
            require(discovered_assignments(metadata, messages) ==
                    {x['id']: x['assigned'] for x in receipt['common']['universe']},
                    'runtime universe differs from original Cargo discovery')
            require(sorted(normalize(x['executable'], roots) for x in messages) ==
                    sorted(normalize(x['executable'], roots) for x in receipt['executables']),
                    'original Cargo executable inventory differs')
    print(json.dumps({'status': 'complete-full-workspace', 'source': identity, **result}, indent=2))


def main():
    if len(sys.argv) > 1 and sys.argv[1] in ('runner', 'rustdoc'):
        return delegate(sys.argv[1], sys.argv[2:])
    parser = argparse.ArgumentParser(description=__doc__)
    sub = parser.add_subparsers(dest='mode', required=True)
    run = sub.add_parser('run')
    run.add_argument('--allocation', choices=ALLOCATIONS, required=True)
    run.add_argument('--platform', choices=('linux', 'windows'), required=True)
    final = sub.add_parser('aggregate')
    final.add_argument('--directory', required=True)
    final.add_argument('--platform', choices=('linux', 'windows'), required=True)
    args = parser.parse_args()
    if args.mode == 'run':
        run_allocation(args)
    else:
        aggregate(args)
    return 0


if __name__ == '__main__':
    try:
        sys.exit(main())
    except (ValueError, KeyError, OSError, subprocess.CalledProcessError) as error:
        print(f'CI runtime verification refused: {error}', file=sys.stderr)
        sys.exit(1)

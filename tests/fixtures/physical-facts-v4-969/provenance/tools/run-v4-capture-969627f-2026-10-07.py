"""Root-scheduled fresh-target GNU verification; importing never starts a run."""
import argparse
import ast
import datetime
import hashlib
import json
import os
from pathlib import Path
import re
import subprocess
import uuid

T = Path(__file__).resolve().parent
R = T.parent / 'gate4-v4-compatibility-captures'
MANIFEST = T / 'v4-capture-969627f-runner-manifest-2026-10-07.json'
ANSI = re.compile(r'\x1b\[[0-?]*[ -/]*[@-~]')

def require(condition, message):
    if not condition:
        raise RuntimeError(message)

def sha(data):
    return hashlib.sha256(data).hexdigest()

def now():
    return datetime.datetime.now(datetime.timezone.utc).isoformat()

def git(*args):
    return subprocess.check_output(['git', '-C', str(R), *args])

def snapshot():
    return {'head': git('rev-parse', 'HEAD').decode().strip(),
            'tree': git('rev-parse', 'HEAD^{tree}').decode().strip(),
            'status': git('status', '--porcelain=v1', '--untracked-files=all').decode().strip()}

def outcomes(log, expected):
    value = ANSI.sub('', log.read_text(encoding='utf-8', errors='strict'))
    summaries = re.findall(r'^test result: (\w+)\. (\d+) passed; (\d+) failed; (\d+) ignored; (\d+) measured; (\d+) filtered out;', value, re.M)
    require(len(summaries) == 1, 'Expected exactly one complete libtest harness')
    status, passed, failed, ignored, measured, filtered = summaries[0]
    require([status, passed, failed, ignored, measured] == ['ok', '1', '0', '0', '0'], 'Exact case did not pass once')
    rows = re.findall(r'^test (.+?) \.\.\. (.+?)\s*$', value, re.M)
    require(rows == [(expected, 'ok')], 'Missing, extra, skipped or wrong named case')
    require(re.findall(r'^running (\d+) tests?\s*$', value, re.M) == ['1'], 'Wrong harness running count')
    return {'passed_names': [expected], 'passed': 1, 'filtered': int(filtered)}


def file_sha(path):
    value = hashlib.sha256()
    with path.open('rb') as stream:
        for chunk in iter(lambda: stream.read(1024 * 1024), b''):
            value.update(chunk)
    return value.hexdigest()

def capture_executable(log, target):
    messages = []
    for line in log.read_text(encoding='utf-8').splitlines():
        if line.startswith('{'):
            messages.append(json.loads(line))
    artifacts = [row for row in messages if row.get('reason') == 'compiler-artifact'
                 and row.get('target', {}).get('name') == 'table_grapple_public'
                 and row.get('target', {}).get('kind') == ['test'] and row.get('executable')]
    require(len(artifacts) == 1, 'Expected exactly one capture test executable artifact')
    require([row for row in messages if row.get('reason') == 'build-finished'] ==
            [{'reason': 'build-finished', 'success': True}], 'Compiler did not report successful build-finished')
    path = Path(artifacts[0]['executable']).resolve()
    require(path.is_file() and path.is_relative_to(target.resolve()) and path.suffix == '.exe',
            'Compiler artifact is not an actual executable inside the fresh target')
    return {'path': str(path), 'bytes': path.stat().st_size, 'sha256': file_sha(path),
            'compiler_artifact': artifacts[0], 'compile_log_sha256': file_sha(log)}

def archive_receipt(directory, config, expected_cuts):
    manifest_path = directory/'manifest.json'
    require(manifest_path.is_file(), 'Case did not complete its archive manifest')
    manifest = json.loads(manifest_path.read_text(encoding='utf-8'))
    require(manifest['schema_version'] == 1 and manifest['status'] == 'COMPLETE_GENUINE_APPLICATION_CAPTURES',
            'Incomplete or unsupported capture manifest')
    require(manifest['scenario'] == directory.name, 'Wrong scenario manifest')
    require(manifest['source']['head'] == config['prepared_for_head'] and
            manifest['source']['tree'] == config['prepared_for_tree'] and
            manifest['source']['base'] == '58696ac1d0c55ef7f71cfb546fb92e97747ee437' and
            manifest['source']['compiled_capture_bytes_match'] is True, 'Wrong archive producer source')
    require([row['label'] for row in manifest['cuts']] == expected_cuts, 'Missing/reordered/extra capture cuts')
    require(all(row['version'] == 4 and row['after_sequence'] == row['before_sequence'] + 1
                for row in manifest['cuts']), 'Wrong cut transport or accepted sequence')
    source = {row['path']: row for row in manifest['source']['entries']}
    require(source.keys() == config['source_inventory'].keys(), 'Archive source inventory differs')
    for path, entry in source.items():
        require(entry['git_entry'] == config['source_inventory'][path]['git_entry'] and
                entry['working_sha256'] == file_sha(R/path), 'Archive source identity differs: '+path)
    files = {}
    for artifact in manifest['artifacts']:
        name = artifact['path']
        require(Path(name).name == name and name not in files and name != 'manifest.json', 'Unsafe/duplicate artifact name')
        path = directory/name
        require(path.is_file() and path.stat().st_size == artifact['bytes'] and file_sha(path) == artifact['sha256'],
                'Artifact bytes or hash differ: '+name)
        files[name] = artifact['sha256']
    require(set(path.name for path in directory.iterdir()) == set(files) | {'manifest.json'},
            'Capture archive has unexpected or missing files')
    suffixes = ['before-export', 'before-audiences', 'next-request', 'accepted-response',
                'accepted-event', 'binding', 'expected-after-export', 'expected-after-audiences']
    expected_files = {f'{index:02d}-{label}-{suffix}.json' for index, label in enumerate(expected_cuts) for suffix in suffixes}
    require(expected_files <= files.keys(), 'A cut lacks one of its eight complete artifacts')
    retries = (['v-three-after-upgrade', 'v-three-after-ground-completion'] if directory.name == 'ground-v4'
               else ['first-award-after-transfer', 'first-award-at-end', 'duplicate-at-end', 'transfer-at-end', 'reroll-at-end'])
    require(set(files) == expected_files | {'retained-'+label+'.json' for label in retries}, 'Unexpected retained retry inventory')
    return {'scenario': directory.name, 'path': str(directory), 'expected_cuts': expected_cuts,
            'manifest_sha256': file_sha(manifest_path), 'artifact_sha256': files,
            'source_head': config['prepared_for_head'], 'source_tree': config['prepared_for_tree'],
            'qualification': 'Independent runner hash/inventory/source check; not a substitute for mechanical copied-artifact review or future consumer tests.'}

def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--head', required=True, help='Full reviewed capture commit SHA')
    parser.add_argument('--attempt', default='01', help='Fresh evidence/target suffix, e.g. 01')
    mode = parser.add_mutually_exclusive_group(required=True)
    mode.add_argument('--preflight-only', action='store_true', help='Read source and host inventory only; no toolchain or project execution')
    mode.add_argument('--heavy-slot-transferred', action='store_true', help='Root has allocated the serial local heavy slot')
    args = parser.parse_args()
    require(re.fullmatch(r'[0-9a-f]{40}', args.head) is not None, 'Use the complete lowercase exact head SHA')
    require(re.fullmatch(r'[a-z0-9-]{1,24}', args.attempt) is not None, 'Unsafe attempt suffix')
    require(sha(MANIFEST.read_bytes()) == 'd4e02d0ddbaafad874ae45e4088de676f68f669d21baea0443d1f5e7c28b2e12',
            'Reviewed selection manifest changed')
    config = json.loads(MANIFEST.read_text(encoding='utf-8'))
    require(args.head == config['prepared_for_head'], 'Runner is frozen to the reviewed capture head')
    scanner = T / config['scanner']
    require(sha(scanner.read_bytes()) == config['scanner_sha256'], 'Source scanner changed')
    nodes = [n for n in ast.parse(scanner.read_text()).body if isinstance(n, ast.FunctionDef) and n.name == 'end_block']
    require(len(nodes) == 1, 'Missing source-only Rust block scanner')
    scope = {'re': re}
    exec(compile(ast.Module(body=nodes, type_ignores=[]), str(scanner), 'exec'), scope)
    initial = snapshot()
    require(initial['head'] == args.head and initial['tree'] == config['prepared_for_tree'] and not initial['status'],
            'Expected head/tree is not the clean checkout')
    for ancestor in config['required_ancestors']:
        require(subprocess.run(['git', '-C', str(R), 'merge-base', '--is-ancestor', ancestor, args.head],
                               stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL).returncode == 0,
                'Combined head lacks a reviewed correction ancestor: '+ancestor)
    mapped = []
    for case in config['cases']:
        text = (R/case['path']).read_text(encoding='utf-8')
        matches = list(re.finditer(r'#\[(?:tokio::)?test\]\s*(?:async\s+)?fn\s+'+re.escape(case['leaf'])+r'\s*\([^)]*\)[^{]*\{', text))
        require(len(matches) == 1, 'Missing/ambiguous original case: '+case['leaf'])
        m = matches[0]
        body = text[m.start():scope['end_block'](text, m.end()-1)]
        require(sha(body.encode()) == case['body_sha256'], 'Reviewed case body changed: '+case['leaf'])
        mapped.append(case['qualified'])
    for path, digest in config['canonical_script_sha256'].items():
        require(sha((R/path).read_bytes().replace(b'\r\n', b'\n')) == digest,
                'Canonical verification script changed: '+path)
    require(str(R) == config['destination_checkout'], 'Wrong worktree-specific receiving checkout')
    paths = git('ls-files', '-z').decode().rstrip('\0').split('\0')
    require(set(paths) == set(config['source_inventory']), 'Tracked inventory differs from frozen Git source')
    tree_entries = {row.split('\t', 1)[1]: row.split('\t', 1)[0]
                    for row in git('ls-tree', '-r', args.head).decode().splitlines()}
    for path, expected in config['source_inventory'].items():
        require(tree_entries[path] == expected['git_entry'], 'Frozen tree mode/blob changed: '+path)
        require(sha((R/path).read_bytes().replace(b'\r\n', b'\n')) == expected['normalized_sha256'],
                'Working source differs from reviewed Git blob: '+path)
    for registration in config['module_registrations']:
        require((R/registration['path']).read_text(encoding='utf8').count(registration['declaration']) == 1,
                'Selected module registration changed: '+registration['module'])
    protected_by_path = {}
    for original in config['protected_original_tests']:
        protected_by_path.setdefault(original['path'], []).append(original)
    full_pattern = re.compile(r'#\[(?:tokio::)?test(?:\([^]]*\))?\]\s*(?:#\[[^\]]+\]\s*)*(?:async\s+)?fn\s+(\w+)\s*\([^)]*\)[^{]*\{')
    for path, originals in protected_by_path.items():
        source = (R/path).read_text(encoding='utf8')
        actual = [(match[1], sha(source[match.start():scope['end_block'](source, match.end()-1)].encode()))
                  for match in full_pattern.finditer(source)]
        for original in originals:
            require(actual.count((original['name'], original['sha256'])) == 1,
                    'Protected original body absent/changed/duplicated: '+path+'::'+original['name'])
    source_hashes = {p: sha((R/p).read_bytes()) for p in paths}
    tools = config['tool_sha256']
    def guard_tools():
        require(all(file_sha(Path(path)) == digest for path, digest in tools.items()),
                'Pinned tool executable changed or is absent')
    guard_tools()
    immutable = {str(p): sha(p.read_bytes()) for p in [Path(__file__), MANIFEST, scanner]}
    def guard():
        require(snapshot() == initial, 'Exact clean head/tree moved during verification')
        require({p: sha((R/p).read_bytes()) for p in paths} == source_hashes, 'Tracked source bytes changed')
        require(all(sha(Path(p).read_bytes()) == h for p, h in immutable.items()), 'Runner/manifest/scanner changed')
        guard_tools()
    guard()
    prefix = 'gate4-v4-compatibility-capture-'+args.head[:12]+'-'+args.attempt+'-2026-10-07'
    evidence, target = T/prefix, T/(prefix+'-target')
    captures = T/(prefix+'-captures')
    require(not evidence.exists() and not target.exists() and not captures.exists(),
            'Preserve prior runs: evidence, target and capture parent must all be fresh')
    require(not captures.resolve().is_relative_to(R.resolve()), 'Capture parent must be outside checkout')
    lock = T/'flow4-local-heavy.lock'
    probe = r"""$ErrorActionPreference='Stop'; $p=@(Get-CimInstance Win32_Process | Where-Object { $_.Name -match '^(cargo|rustc|rustfmt|clippy-driver|DMd|table_loop.*|flow4.*)\.exe$' -or ($_.ExecutablePath -like '*\DMD\tooling\*target\*' -and $_.Name -like '*.exe') -or ($_.Name -eq 'node.exe' -and $_.CommandLine -match 'vitest|svelte-check|vite[/\\]bin|tauri') } | Select-Object ProcessId,Name); $m=Get-CimInstance Win32_PerfFormattedData_PerfOS_Memory | Select-Object AvailableBytes,CommittedBytes,CommitLimit; $d=Get-PSDrive C | Select-Object Name,Free; [pscustomobject]@{processes=$p;memory=$m;disk=$d} | ConvertTo-Json -Depth 5"""
    def host_preflight():
        result = json.loads(subprocess.check_output(['powershell', '-NoProfile', '-Command', probe], text=True))
        result['existing_heavy_lock'] = lock.exists()
        return result
    env = os.environ.copy()
    forbidden = ['RUST_MIN_STACK', 'RUSTFLAGS', 'CARGO_ENCODED_RUSTFLAGS', 'RUSTC', 'RUSTDOC',
                 'RUSTC_BOOTSTRAP', 'RUSTDOCFLAGS', 'CARGO_ENCODED_RUSTDOCFLAGS', 'RUSTC_WRAPPER',
                 'RUSTC_WORKSPACE_WRAPPER', 'RUST_TEST_THREADS', 'RUST_TEST_NOCAPTURE',
                 'RUST_TEST_SHUFFLE', 'RUST_TEST_SHUFFLE_SEED', 'CARGO_BUILD_TARGET',
                 'CARGO_BUILD_RUSTFLAGS', 'CARGO_BUILD_RUSTC', 'CARGO_BUILD_RUSTDOC',
                 'CARGO_BUILD_RUSTC_WRAPPER', 'CARGO_BUILD_RUSTC_WORKSPACE_WRAPPER', 'BASH_ENV', 'DMD_V4_CAPTURE_DIR']
    require(not any(k in env for k in forbidden) and not any(k.startswith('CARGO_PROFILE_') for k in env),
            'Unexpected stack/compiler/profile/test/shell override')
    require(all(k == 'CARGO_TARGET_X86_64_PC_WINDOWS_GNU_RUSTFLAGS' and v == '-C link-self-contained=yes'
                for k,v in env.items() if k.startswith('CARGO_TARGET_') and k.endswith('_RUSTFLAGS')),
            'Unexpected per-target compiler flags')
    env.update(CARGO_HOME=str(T/'cargo'), RUSTUP_HOME=str(T/'rustup'),
               RUSTUP_TOOLCHAIN='stable-x86_64-pc-windows-gnu', CARGO_TARGET_DIR=str(target),
               CARGO_BUILD_JOBS='1', CARGO_INCREMENTAL='0', CARGO_TERM_COLOR='never',
               CC_X86_64_PC_WINDOWS_GNU=str(T/'w64devkit/bin/gcc.exe'),
               AR_X86_64_PC_WINDOWS_GNU=str(T/'w64devkit/bin/ar.exe'),
               CARGO_TARGET_X86_64_PC_WINDOWS_GNU_RUSTFLAGS='-C link-self-contained=yes')
    env['PATH'] = os.pathsep.join([str(T/'cargo/bin'), str(T/'w64devkit/bin'),
        'C:/Program Files/Git/usr/bin', 'C:/Program Files/Git/bin', env.get('PATH', '')])
    cargo = str(T/'cargo/bin/cargo.exe')
    bash = 'C:/Program Files/Git/bin/bash.exe'
    require(Path(cargo).is_file() and Path(bash).is_file(), 'Configured tool executables absent')
    steps = [{'name': '01-canonical-verify-fast', 'argv': [bash, './scripts/verify-fast']},
             {'name': '02-compile-capture-executable', 'argv': [cargo, 'test', '--locked', '-p', 'dmd-app',
              '--test', 'table_grapple_public', '--no-run', '--message-format=json'], 'compile_capture': True}]
    for index, case in enumerate(config['cases'], 3):
        selection = ['--lib'] if case['package'] == 'dmd-rules' else ['--test', 'table_grapple_public']
        steps.append({'name': f'{index:02d}-'+case['label'],
                      'argv': [cargo, 'test', '--locked', '-p', case['package'], *selection,
                               case['qualified'], '--', '--exact'], 'expected': case['qualified'],
                      'scenario': case['scenario'], 'expected_cuts': case['expected_cuts']})
    steps.append({'name': '06-strict-workspace-clippy',
                  'argv': [cargo, 'clippy', '--locked', '--workspace', '--all-targets', '--', '-D', 'warnings']})
    preflight = host_preflight()
    preview = {'source': initial, 'repository': str(R), 'tracked_hash_count': len(source_hashes),
        'selected_cases': mapped, 'protected_original_test_count': len(config['protected_original_tests']),
        'selection_scope': config['scope'], 'steps': steps, 'evidence': str(evidence), 'fresh_target': str(target),
        'runner_files': immutable, 'tool_executable_sha256': tools,
        'capture_parent': str(captures), 'capture_environment': {'DMD_V4_CAPTURE_DIR': str(captures)},
        'effective_controlled_environment': {key: env[key] for key in [
            'CARGO_HOME', 'RUSTUP_HOME', 'RUSTUP_TOOLCHAIN', 'CARGO_TARGET_DIR', 'CARGO_BUILD_JOBS',
            'CARGO_INCREMENTAL', 'CARGO_TERM_COLOR', 'CC_X86_64_PC_WINDOWS_GNU',
            'AR_X86_64_PC_WINDOWS_GNU', 'CARGO_TARGET_X86_64_PC_WINDOWS_GNU_RUSTFLAGS', 'PATH']},
        'preflight': preflight, 'project_execution': 'UNRUN'}
    if args.preflight_only:
        print(json.dumps(preview, indent=2))
        return 0
    require(not preflight['existing_heavy_lock'], 'Another local heavy allocation owns the lock')
    require(not preflight['processes'], 'Native/compiler/test/frontend work is already running')
    require(preflight['memory']['CommitLimit']-preflight['memory']['CommittedBytes'] >= 4*1024**3,
            'Less than 4 GiB committed-memory headroom')
    require(preflight['disk']['Free'] >= 12*1024**3, 'Less than 12 GiB fresh-target disk headroom')
    token = json.dumps({'pid': os.getpid(), 'token': uuid.uuid4().hex, 'head': args.head, 'target': str(target)})
    with lock.open('x', encoding='utf-8') as stream:
        stream.write(token)
    result = dict(preview, started_utc=now(), results=[], failure=None, all_planned_passed=False,
        qualification='Canonical verify-fast + normal-profile capture executable build/pin + three exact genuine v4 capture cases + strict all-target workspace Clippy; not full canonical verify, full workspace tests, future consumer verification, CI, native package, MSRV or gate acceptance.')
    try:
        evidence.mkdir()
        target.mkdir()
        captures.mkdir()
        def save(name, value):
            with (evidence/name).open('x', encoding='utf-8', newline='\n') as stream:
                json.dump(value, stream, indent=2)
                stream.write('\n')
        guard()
        version = subprocess.check_output([str(T/'cargo/bin/rustc.exe'), '-vV'], env=env).decode()
        require('release: 1.98.1' in version and 'host: x86_64-pc-windows-gnu' in version,
                'Unexpected Rust toolchain: expected GNU 1.98.1')
        result['rustc_verbose'] = version
        result['cargo_version'] = subprocess.check_output([cargo, '--version'], env=env).decode().strip()
        save('source-working-sha256.json', source_hashes)
        save('start.json', result)
        executable = None
        frozen_archives = {}
        def guard_archives():
            for scenario, receipt in frozen_archives.items():
                require(archive_receipt(captures/scenario, config, receipt['expected_cuts']) == receipt,
                        'Completed capture archive changed: '+scenario)
        for step in steps:
            try:
                guard()
                guard_archives()
                step_env = env.copy()
                if 'expected' in step:
                    require(executable is not None, 'Capture executable was not pinned')
                    require(file_sha(Path(executable['path'])) == executable['sha256'], 'Capture executable changed')
                    require(not (captures/step['scenario']).exists(), 'Scenario must be absent before its one run')
                    step_env['DMD_V4_CAPTURE_DIR'] = str(captures)
                save(step['name']+'-start.json', dict(step, at=now(), executable=executable,
                    capture_environment={'DMD_V4_CAPTURE_DIR': str(captures)} if 'expected' in step else {}))
                print(json.dumps({'starting': step['name'], 'at': now()}), flush=True)
                log = evidence/(step['name']+'.log')
                with log.open('xb') as stream:
                    done = subprocess.run(step['argv'], cwd=R, env=step_env, stdout=stream, stderr=subprocess.STDOUT)
                row = dict(step, exit=done.returncode, ended_utc=now(), log=str(log), log_sha256=sha(log.read_bytes()))
                result['results'].append(row)
                save(step['name']+'-exit.json', row)
                require(done.returncode == 0, 'Actual command failed; read '+str(log))
                guard()
                if step.get('compile_capture'):
                    executable = capture_executable(log, target)
                    result['capture_executable'] = executable
                    save('capture-executable.json', executable)
                if 'expected' in step:
                    require(file_sha(Path(executable['path'])) == executable['sha256'], 'Capture executable changed during case')
                    running = re.findall(r'^\s*Running tests[\\/]table_grapple_public\.rs \((.+\.exe)\)\s*$',
                                         ANSI.sub('', log.read_text(encoding='utf-8')), re.M)
                    require(len(running) == 1 and Path(running[0]).resolve() == Path(executable['path']).resolve(),
                            'Cargo did not run the pinned capture executable')
                    receipt = archive_receipt(captures/step['scenario'], config, step['expected_cuts'])
                    frozen_archives[step['scenario']] = receipt
                    row['capture_archive'] = receipt
                    save(step['name']+'-capture-audit.json', receipt)
                    guard_archives()
                    row['outcomes'] = outcomes(log, step['expected'])
                    save(step['name']+'-outcomes.json', row['outcomes'])
                elif step['name'] == '01-canonical-verify-fast':
                    text = log.read_text(encoding='utf-8', errors='replace')
                    require(all(marker in text for marker in ['==> rustfmt', '==> cargo check', 'Fast verification passed.']),
                            'Canonical fmt/check completion missing')
                print(json.dumps({'completed': step['name'], 'exit': done.returncode}), flush=True)
            except Exception as error:
                result['failure'] = {'step': step['name'], 'type': type(error).__name__, 'reason': str(error)}
                break
        guard_archives()
        if executable is not None:
            require(file_sha(Path(executable['path'])) == executable['sha256'], 'Final capture executable changed')
        result['all_planned_passed'] = (result['failure'] is None and len(result['results']) == len(steps)
                                        and len(frozen_archives) == len(config['cases']))
        result['capture_archives'] = frozen_archives
        save('capture-archives.json', frozen_archives)
        result['project_execution'] = 'EXECUTED; see actual per-command outcomes'
        result['ended_utc'] = now()
        result['source_after'] = snapshot()
        save('result.json', result)
        print(json.dumps(result), flush=True)
        return 0 if result['all_planned_passed'] else 1
    finally:
        # Delete only the exact lock file this invocation created; never a directory.
        if lock.exists() and lock.read_text(encoding='utf-8') == token:
            lock.unlink()

if __name__ == '__main__':
    raise SystemExit(main())

"""Prepare frozen verification by Git/source inspection only; never run/import engine."""
import ast
import copy
import hashlib
import json
import re
import subprocess
from datetime import datetime, timezone
from pathlib import Path

T = Path(__file__).resolve().parent
R = T.parent / 'gate4-grapple-attack-read-context'
B = '418eeb770404e8c93de3407b839704caa3fc303d'
C = 'c2e0647b3302dbc116e9aa4bf1431f6ad757928d'
U = '9768a405c611aae3442eb8a64e7a246b641b6962'
H = 'ff0e10351df4c8c1e9341e4c1ee2dffde2283dc6'
TREE = 'e7d0d9f8eaf848235c8ebe553b34324b8245985e'
P = 'crates/dmd-rules/src/tactical/grapple/execution/attack_tests.rs'
D = 'docs/exec-plans/active/gate4-grapple-attack-read-context.md'
HELPER = 'with_held_cultist_and_caster'
FAILED = {'actual_concentration_child_retains_completed_attack_evidence_through_release',
          'damage_release_and_real_knockout_complete_current_equipment_once'}
NAME = 'grapple-ff0e103-focused82-2026-10-06'
OLD = T / 'grapple-418eeb7-focused82-2026-10-05-manifest.json'
OLDW = T / 'run-grapple-418eeb7-focused82-2026-10-05.py'
ENGINE = T / 'frozen-focused-runner-v2-2026-10-04.py'

def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()

def digest(data):
    return hashlib.sha256(data).hexdigest()

def git(*args):
    return subprocess.check_output(['git', '-C', str(R), *args])

def norm(data):
    return data.replace(b'\r\n', b'\n')

def source(ref, path=P):
    return norm(git('show', ref + ':' + path)).decode()

def entries(ref):
    return {row.split('\t', 1)[1]: row.split('\t', 1)[0]
            for row in git('ls-tree', '-r', ref).decode().splitlines()}

def end_block(s, i):
    assert s[i] == '{'
    depth = 0
    while i < len(s):
        if s.startswith('//', i):
            j = s.find('\n', i)
            i = len(s) if j < 0 else j + 1
            continue
        if s.startswith('/*', i):
            level = 1
            i += 2
            while level:
                assert i < len(s)
                if s.startswith('/*', i): level += 1; i += 2
                elif s.startswith('*/', i): level -= 1; i += 2
                else: i += 1
            continue
        raw = re.match(r'(?:br|r)(#*)"', s[i:])
        if raw:
            terminal = '"' + raw[1]
            j = s.find(terminal, i + raw.end())
            assert j >= 0
            i = j + len(terminal)
            continue
        if s[i] == '"':
            i += 1
            while True:
                assert i < len(s)
                if s[i] == '\\': i += 2
                elif s[i] == '"': i += 1; break
                else: i += 1
            continue
        char = re.match(r"'(?:\\(?:u\{[^}]+\}|x[0-9a-fA-F]{2}|.)|[^'\\\n])'", s[i:])
        if char:
            i += char.end()
            continue
        if s[i] == '{': depth += 1
        if s[i] == '}':
            depth -= 1
            if depth == 0: return i + 1
        i += 1
    raise AssertionError('unclosed Rust source block')

def functions(s):
    result = {}
    for m in re.finditer(r'(?m)^fn (\w+)\b', s):
        opening = s.index('{', m.start())
        assert m[1] not in result
        result[m[1]] = s[m.start():end_block(s, opening)]
    return result

def tokens(s):
    # These three inspected functions contain ordinary quoted strings only.
    # Keep each whole string; ignore whitespace and optional trailing commas.
    assert not re.search(r'\b(?:r|br)#*"', s)
    values = re.findall(r'"(?:\\.|[^"\\])*"|[A-Za-z_]\w*|\d+|[^\s]', s)
    return [v for i, v in enumerate(values)
            if not (v == ',' and i + 1 < len(values) and values[i + 1] in ')]}')]

def blobs_at_head(paths):
    payload = ''.join(H + ':' + p + '\n' for p in paths).encode()
    data = subprocess.check_output(['git', '-C', str(R), 'cat-file', '--batch'], input=payload)
    offset = 0
    result = {}
    for path in paths:
        end = data.index(b'\n', offset)
        _, kind, size = data[offset:end].split()
        assert kind == b'blob'
        start = end + 1
        stop = start + int(size)
        result[path] = data[start:stop]
        assert data[stop:stop+1] == b'\n'
        offset = stop + 1
    assert offset == len(data)
    return result

assert git('rev-parse', 'HEAD').decode().strip() == H
assert git('rev-parse', 'HEAD^{tree}').decode().strip() == TREE
assert git('rev-parse', H + '^').decode().strip() == U
assert not git('status', '--porcelain').strip()
be, ce, ue, he = (entries(ref) for ref in [B, C, U, H])
assert set(be) == set(ce) == set(ue) == set(he)
assert {p for p in be if be[p] != he[p]} == {P, D}
assert {p for p in ue if ue[p] != he[p]} == {P, D}
old, prior, unformatted, new = (source(ref) for ref in [B, C, U, H])
oldf, priorf, unf, newf = (functions(s) for s in [old, prior, unformatted, new])
assert set(priorf) == set(unf) == set(newf)
assert set(newf) - set(oldf) == {HELPER}
assert {n for n in priorf if priorf[n] != unf[n]} == {HELPER}
assert {n for n in unf if unf[n] != newf[n]} == FAILED | {HELPER}
assert all(tokens(unf[n]) == tokens(newf[n]) for n in FAILED | {HELPER})
inverse = new
for name in FAILED | {HELPER}:
    assert inverse.count(newf[name]) == 1
    inverse = inverse.replace(newf[name], unf[name])
assert inverse == unformatted
assert {n for n in oldf if oldf[n] != newf[n]} == FAILED
tests = re.findall(r'#\[test\]\s*fn (\w+)', old)
assert len(tests) == 15 and tests == re.findall(r'#\[test\]\s*fn (\w+)', new)
assert len(oldf) == 37
assert all(newf[n] == oldf[n] for n in oldf if n not in FAILED)
assert all(tokens(priorf[n]) == tokens(newf[n]) for n in FAILED)
assert source(H, D).endswith(source(U, D).split('\n\n', 1)[1])
subprocess.run(['git', '-C', str(R), 'diff', '--check', B, H], check=True, capture_output=True)
assert sha(OLD) == '81d7c10d4d2f2956af5a479b8b81de1fb65a893f50140ff0b1a58f06178fcb7e'
assert sha(OLDW) == '9925fad896bc3f0c7ba48a86018abd062efa7e681ad3afc2ae3f4f404831b757'
assert sha(ENGINE) == 'ad291ba8db4a41e530010fabefb3664e6a5e152066031ffa2e6ee938ec2e4fc0'
preserved = {str(p): sha(p) for p in [OLD, OLDW, ENGINE]}
attempts = []
for name, prefix, exits in [
    ('grapple-0df1754-focused82-2026-10-05', '0df1754', [0,101]),
    ('grapple-58b6a90-focused82-2026-10-05', '58b6a90', [0,101]),
    ('grapple-ade8e93-focused82-2026-10-05', 'ade8e93', [0,0,0,101]),
    ('grapple-418eeb7-focused82-2026-10-05', '418eeb7', [0,0,0,0,101]),
]:
    directory = T / name
    result = json.loads((directory / 'run-result.json').read_text())
    assert result['initial'] == result['final'] and result['initial']['head'].startswith(prefix)
    assert [r['exit'] for r in result['results']] == exits and not result['all_planned_passed']
    for row in result['results']:
        assert sha(Path(row['log'])) == row['log_sha256']
    for path in [*filter(Path.is_file, directory.iterdir()), T/(name+'-manifest.json'), T/('run-'+name+'.py')]:
        preserved[str(path)] = sha(path)
    attempts.append(result)
old_cfg = json.loads(OLD.read_text())
assert old_cfg['head'] == B
for row, step in zip(attempts[-1]['results'], old_cfg['steps']):
    assert row['argv'][1:] == step['args']
log = T / old_cfg['evidence'] / '04-attack-fifteen.log'
assert sha(log) == '3888686e850ee4a5280f51f18f2a8f168b4f478459f5144d8592d7c81162acf5'
outcomes = re.findall(r'^test (.+?) \.\.\. (ok|FAILED)$', log.read_text(encoding='utf-8-sig'), re.M)
assert len(outcomes) == 15 and sum(s == 'ok' for _, s in outcomes) == 13
assert {n.rsplit('::',1)[1] for n,s in outcomes if s == 'FAILED'} == FAILED
assert {n for n,_ in outcomes} == set(old_cfg['steps'][4]['expected_pass_names'])
assert sum(step.get('expected_count', 0) for step in old_cfg['steps'][5:]) == 56
cfg = copy.deepcopy(old_cfg)
cfg.update(head=H, tree=TREE, source_map_base=H, evidence=NAME)
blob_map = blobs_at_head(list(he))
for rel, data in blob_map.items():
    assert norm((R/rel).read_bytes()) == norm(data)
changed_hashes = []
for rel, previous in old_cfg['source_file_sha256'].items():
    cfg['source_file_sha256'][rel] = sha(R/rel)
    if cfg['source_file_sha256'][rel] != previous: changed_hashes.append(rel)
assert changed_hashes == [P] and len(cfg['source_file_sha256']) == 328
line_changes, attributes = [], []
for entry in cfg['test_source_map']:
    lines = (R/entry['path']).read_text().splitlines()
    token = 'fn ' + entry['name'].rsplit('::', 1)[-1] + '('
    found = [i+1 for i,line in enumerate(lines) if token in line]
    assert len(found) == 1
    at = found[0]
    context = '\n'.join(lines[max(0,at-3):at-1])
    assert '#[test]' in context and '#[ignore' not in context
    attributes.append({'name': entry['name'], 'path': entry['path'], 'line': at, 'attributes': context})
    if at != entry['line']:
        assert entry['path'] == P
        line_changes.append({'name': entry['name'], 'old': entry['line'], 'new': at})
        entry['line'] = at
assert cfg['steps'] == old_cfg['steps']
names = [n for step in cfg['steps'] for n in step.get('expected_pass_names', [])]
assert len(names) == len(set(names)) == 82
assert {e['name'] for e in cfg['test_source_map']} == set(names)
for step in cfg['steps']:
    if 'expected_pass_names' in step:
        assert step['args'][step['args'].index('--')+1:] == ['--exact', *step['expected_pass_names']]
        assert step['expected_count'] == len(step['expected_pass_names'])
assert not (T/NAME).exists() and not (T/(NAME+'-target')).exists()
cfg['qualification'] = ('Prepared UNRUN at exact ff0e103 after actual418 fmt/strict3pkgClippy/opportunity1/owner10 PASS and attack13PASS/2FAIL; remaining56UNRUN. Corrections preserve source-typed paid no-effect and actual one-HP knockout semantics. Added genuine Host Cultist helper shares identical-creature initiative, submits3 physical rolls, and accepts a real Host tie. Only helper plus2 failed attack bodies changed;22 old helpers/13 passing attack bodies and all595 other Git entries remain exact. Root reviewed unformatted976 and allocated direct edited-file rustfmt only, now passed. Same82 names/argv/counts and328 mapped source hashes; only attack_tests.rs hash/locations change. Fresh target/evidence; unchanged v2 GNU1.98.1/jobs1/inc0/default profiles/stack/test threads, no explicit --target; wrapper rejects CARGO_BUILD_TARGET. Root sole heavy slot; engine lock check is not atomic. All4 prior failed attempts and evidence preserved. No current runtime/CI/canonical/public/native/Gate4 acceptance.')
m = T / (NAME+'-manifest.json')
m.open('x', encoding='utf-8', newline='\n').write(json.dumps(cfg, indent=2)+'\n')
w = T / ('run-'+NAME+'.py')
code = OLDW.read_text().replace(OLD.name, m.name).replace(sha(OLD), sha(m))
ast.parse(code)
assert code.replace(m.name, OLD.name).replace(sha(m), sha(OLD)) == OLDW.read_text()
w.open('x', encoding='utf-8', newline='\n').write(code)
inventory = T / (NAME+'-command-inventory.md')
parts = ['# Prepared ff0e103 focused82 inventory\n\nUNRUN. Every step/argument/name/count is identical to418. Source locations refer to frozenff0e103.\n']
for step in cfg['steps']:
    parts += ['\n## '+step['name']+'\n\n```text\ncargo '+' '.join(step['args'])+'\n```\n']
    if 'expected_pass_names' in step:
        parts += ['\nExpected exactly '+str(step['expected_count'])+' passing names; no ignored/measured/failing cases.\n']
        for name in step['expected_pass_names']:
            entry = next(e for e in cfg['test_source_map'] if e['name'] == name)
            parts += ['- `'+name+'` — `'+entry['path']+':'+str(entry['line'])+'`\n']
inventory.open('x', encoding='utf-8', newline='\n').write(''.join(parts))
patch = T / (NAME+'-complete-correction.patch')
patch.open('xb').write(git('diff', B, H, '--', D, P))
format_patch = T / (NAME+'-format-only.patch')
format_patch.open('xb').write(git('diff', U, H, '--', P))
protected = [{'path': p, 'git_entry': he[p], 'sha256': digest(blob_map[p])}
             for p in he if p != P and ('/tests/' in p or '/fixtures/' in p or
                p.endswith(('_tests.rs','.test.ts','.test-support.ts')) or p.startswith('content/'))]
assert all(be[row['path']] == row['git_entry'] for row in protected)
assert all(sha(Path(path)) == value for path,value in preserved.items())
assert git('rev-parse', 'HEAD').decode().strip() == H and not git('status','--porcelain').strip()
audit = {'at_utc': datetime.now(timezone.utc).isoformat(), 'head': H, 'tree': TREE,
         'base': B, 'unformatted_reviewed_head': U, 'all595_other_entries_exact': True,
         'all_working_files_match_git_after_crlf_normalization': True,
         'all22_old_helpers_and13_passing_bodies_exact': True,
         'all_formatting_is_within_three_permitted_functions': True,
         'formatting_tokens_equal_except_optional_trailing_commas': True,
         'entire_unformatted_attack_file_inverse_exact': True,
         'corrected_c2_body_tokens_preserved': True,
         'protected_entries': protected, 'source_hash_changes': changed_hashes,
         'source_line_changes': line_changes, 'literal_test_attributes': attributes,
         'all82_exact_names_arrays_counts_unchanged': True, 'all328_hashes_verified': True,
         'fresh_directories_absent': True, 'old_attack_outcomes': outcomes,
         'prior_attempts': attempts, 'prior_files_preserved': preserved,
         'files': {p.name: sha(p) for p in [m,w,ENGINE,inventory,patch,format_patch,Path(__file__)]},
         'qualification': cfg['qualification']}
out = T / (NAME+'-author-readiness.json')
out.open('x', encoding='utf-8', newline='\n').write(json.dumps(audit, indent=2)+'\n')
print(json.dumps({'head':H, 'tree':TREE, 'files':{**audit['files'],out.name:sha(out)},
                  'source_hash_changes':changed_hashes, 'source_line_changes':line_changes,
                  'step_counts':{s['name']:s.get('expected_count') for s in cfg['steps']}},indent=2))

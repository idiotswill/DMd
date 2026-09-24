import assert from 'node:assert/strict';
import { mkdtemp, mkdir, readFile, readdir, symlink, writeFile } from 'node:fs/promises';
import os from 'node:os';
import path from 'node:path';
import test from 'node:test';
import { copyNoticeFiles, prepareNoticeOutput } from './desktop-notice-files.mjs';
import { copySupplementalNotices } from './desktop-notice-supplements.mjs';

test('copies upstream license naming and directory layouts while excluding application source', async () => {
  const temporary = await mkdtemp(path.join(os.tmpdir(), 'dmd-notices-'));
  const root = path.join(temporary, 'package');
  const output = path.join(temporary, 'output');
  // Real layouts: Tauri/whoami underscores, crc-catalog LICENSES, and TypeScript notices.
  const expected = ['LICENSE_MIT', 'LICENSE_APACHE-2.0', 'LICENSES/MIT.txt',
    'LICENSES/Apache-2.0.txt', 'ThirdPartyNoticeText.txt', 'legal/grant.txt'];
  for (const file of [...expected, 'src/main.rs', 'README.md']) {
    const target = path.join(root, file);
    await mkdir(path.dirname(target), { recursive: true });
    await writeFile(target, `upstream ${file}\n`);
  }
  assert.deepEqual(await copyNoticeFiles(root, output, 'legal/grant.txt'), [...expected].sort());
  for (const file of expected) assert.equal(await readFile(path.join(output, file), 'utf8'), `upstream ${file}\n`);
  assert.ok(!(await readdir(output)).includes('src'));
  assert.ok(!(await readdir(output)).includes('README.md'));
});

test('fails closed when an explicitly declared license is missing or outside its package', async () => {
  const root = await mkdtemp(path.join(os.tmpdir(), 'dmd-invalid-notice-'));
  await assert.rejects(copyNoticeFiles(root, path.join(root, 'output'), '../private.txt'), /inside its package/);
  await assert.rejects(copyNoticeFiles(root, path.join(root, 'output'), 'missing-license.txt'), { code: 'ENOENT' });
});

test('rejects a declared license that escapes its package through a directory link', async (context) => {
  const temporary = await mkdtemp(path.join(os.tmpdir(), 'dmd-linked-notice-'));
  const root = path.join(temporary, 'package');
  const outside = path.join(temporary, 'outside');
  await mkdir(root);
  await mkdir(outside);
  await writeFile(path.join(outside, 'grant.txt'), 'must not be packaged');
  try { await symlink(outside, path.join(root, 'legal'), process.platform === 'win32' ? 'junction' : 'dir'); }
  catch (error) { if (error.code === 'EPERM') { context.skip('Host does not permit directory links.'); return; } throw error; }
  await assert.rejects(copyNoticeFiles(root, path.join(temporary, 'output'), 'legal/grant.txt'), /resolve inside its package/);
});

test('repeated collection excludes its own generated output inside the desktop crate', async () => {
  const root = await mkdtemp(path.join(os.tmpdir(), 'dmd-repeat-notice-'));
  const output = path.join(root, 'licenses', 'dependencies');
  const destination = path.join(output, 'rust', 'dmd-desktop-0.1.0');
  await mkdir(output, { recursive: true });
  await writeFile(path.join(root, 'LICENSE'), 'source notice');
  await writeFile(path.join(output, 'LICENSE.generated'), 'must not be recursively repackaged');
  for (let pass = 0; pass < 3; pass += 1) {
    assert.deepEqual(await copyNoticeFiles(root, destination, null, output), ['LICENSE']);
    assert.deepEqual(await readdir(destination), ['LICENSE']);
    assert.equal(await readFile(path.join(destination, 'LICENSE'), 'utf8'), 'source notice');
  }
});

test('preparation removes stale generated notices and refuses unrelated or linked directories', async (context) => {
  const root = await mkdtemp(path.join(os.tmpdir(), 'dmd-notice-output-'));
  const output = path.join(root, 'crates', 'dmd-desktop', 'licenses', 'dependencies');
  await prepareNoticeOutput(root, output);
  await writeFile(path.join(output, 'LICENSE.stale'), 'old generated notice');
  await prepareNoticeOutput(root, output);
  assert.deepEqual(await readdir(output), []);
  const unrelated = path.join(root, 'private');
  await mkdir(unrelated);
  await writeFile(path.join(unrelated, 'keep.txt'), 'keep');
  await assert.rejects(prepareNoticeOutput(root, unrelated), /repository generated/);
  assert.equal(await readFile(path.join(unrelated, 'keep.txt'), 'utf8'), 'keep');
  const linkedRoot = await mkdtemp(path.join(os.tmpdir(), 'dmd-linked-output-'));
  const licenses = path.join(linkedRoot, 'crates', 'dmd-desktop', 'licenses');
  await mkdir(licenses, { recursive: true });
  const linked = path.join(licenses, 'dependencies');
  try { await symlink(unrelated, linked, process.platform === 'win32' ? 'junction' : 'dir'); }
  catch (error) { if (error.code === 'EPERM') { context.skip('Host does not permit directory links.'); return; } throw error; }
  await assert.rejects(prepareNoticeOutput(linkedRoot, linked), /must not resolve through a link/);
  assert.equal(await readFile(path.join(unrelated, 'keep.txt'), 'utf8'), 'keep');
});

test('packages the pinned upstream copyright text for each WebView2 crate', async () => {
  const manifest = JSON.parse(await readFile(new URL('../third-party-notices/supplements.json', import.meta.url), 'utf8'));
  for (const entry of manifest.packages) {
    const root = await mkdtemp(path.join(os.tmpdir(), 'dmd-supplement-'));
    const destination = path.join(root, 'output');
    await writeFile(path.join(root, '.cargo-checksum.json'), JSON.stringify({ package: entry.crate_sha256 }));
    await writeFile(path.join(root, '.cargo_vcs_info.json'), JSON.stringify({ git: { sha1: entry.revision } }));
    assert.deepEqual(await copySupplementalNotices('rust', entry.name, entry.version, 'MIT', root, destination), ['LICENSE.upstream', 'NOTICE.source.json']);
    assert.match(await readFile(path.join(destination, 'LICENSE.upstream'), 'utf8'), /Copyright \(c\) 2021 Bill Avery/);
    assert.deepEqual(JSON.parse(await readFile(path.join(destination, 'NOTICE.source.json'), 'utf8')), entry);
    await assert.rejects(copySupplementalNotices('rust', entry.name, '99.0.0', 'MIT', root, destination), /requires review/);
    await writeFile(path.join(root, '.cargo_vcs_info.json'), JSON.stringify({ git: { sha1: 'unreviewed' } }));
    await assert.rejects(copySupplementalNotices('rust', entry.name, entry.version, 'MIT', root, destination), /source pin/);
    await writeFile(path.join(root, '.cargo_vcs_info.json'), JSON.stringify({ git: { sha1: entry.revision } }));
    await writeFile(path.join(root, '.cargo-checksum.json'), JSON.stringify({ package: 'unreviewed' }));
    await assert.rejects(copySupplementalNotices('rust', entry.name, entry.version, 'MIT', root, destination), /source pin/);
  }
});

test('accepts an aliased repository root while clearing only its canonical generated output', async (context) => {
  const temporary = await mkdtemp(path.join(os.tmpdir(), 'dmd-notice-alias-'));
  const root = path.join(temporary, 'checkout');
  const alias = path.join(temporary, 'checkout-alias');
  await mkdir(root);
  try { await symlink(root, alias, process.platform === 'win32' ? 'junction' : 'dir'); }
  catch (error) { if (error.code === 'EPERM') { context.skip('Host does not permit directory links.'); return; } throw error; }
  const output = path.join(alias, 'crates', 'dmd-desktop', 'licenses', 'dependencies');
  const canonicalOutput = path.join(root, 'crates', 'dmd-desktop', 'licenses', 'dependencies');
  await prepareNoticeOutput(alias, output);
  await writeFile(path.join(output, 'LICENSE.stale'), 'old generated output');
  await prepareNoticeOutput(alias, output);
  assert.deepEqual(await readdir(canonicalOutput), []);
});

import assert from 'node:assert/strict';
import { mkdtemp, mkdir, readFile, readdir, symlink, writeFile } from 'node:fs/promises';
import os from 'node:os';
import path from 'node:path';
import test from 'node:test';
import { copyNoticeFiles } from './desktop-notice-files.mjs';

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

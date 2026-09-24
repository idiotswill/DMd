// Collect upstream notice files from the exact installed/locked dependency trees.
import { spawnSync } from 'node:child_process';
import { readdir, readFile, writeFile } from 'node:fs/promises';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import { copyNoticeFiles, prepareNoticeOutput } from './desktop-notice-files.mjs';
import { copySupplementalNotices } from './desktop-notice-supplements.mjs';

if (!process.argv[2]) throw new Error('A notice output directory is required.');
const output = await prepareNoticeOutput(fileURLToPath(new URL('../', import.meta.url)), process.argv[2]);
const records = [];

async function collect(kind, name, version, license, root, licenseFile = null) {
  const destination = path.join(output, kind, `${name.replaceAll('/', '__')}-${version}`);
  const files = await copyNoticeFiles(root, destination, licenseFile, output);
  files.push(...await copySupplementalNotices(kind, name, version, license, root, destination, lockfile));
  records.push({ kind, name, version, license: license ?? null, notices: files.map((file) => path.relative(output, path.join(destination, file)).replaceAll('\\', '/')) });
}

const cargo = spawnSync('cargo', ['metadata', '--format-version', '1', '--locked'], { encoding: 'utf8', maxBuffer: 64 * 1024 * 1024 });
if (cargo.status !== 0) throw new Error(cargo.stderr || 'Cargo metadata failed.');
const lockfile = await readFile('Cargo.lock', 'utf8');
for (const pkg of JSON.parse(cargo.stdout).packages) {
  await collect('rust', pkg.name, pkg.version, pkg.license, path.dirname(pkg.manifest_path), pkg.license_file);
}

async function visitModules(directory) {
  for (const entry of await readdir(directory, { withFileTypes: true })) {
    if (!entry.isDirectory() || entry.name.startsWith('.')) continue;
    const packageRoot = path.join(directory, entry.name);
    if (entry.name.startsWith('@')) { await visitModules(packageRoot); continue; }
    let json;
    try { json = JSON.parse(await readFile(path.join(packageRoot, 'package.json'), 'utf8')); }
    catch (error) { if (error.code === 'ENOENT') continue; throw error; }
    await collect('npm', json.name, json.version, json.license, packageRoot);
    try { await visitModules(path.join(packageRoot, 'node_modules')); }
    catch (error) { if (error.code !== 'ENOENT') throw error; }
  }
}
await visitModules(path.resolve('apps/desktop/node_modules'));
records.sort((left, right) => `${left.kind}/${left.name}/${left.version}`.localeCompare(`${right.kind}/${right.name}/${right.version}`));
await writeFile(path.join(output, 'index.json'), `${JSON.stringify(records, null, 2)}\n`);

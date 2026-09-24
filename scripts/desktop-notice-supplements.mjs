import { createHash } from 'node:crypto';
import { mkdir, readFile, writeFile } from 'node:fs/promises';
import path from 'node:path';

const manifestUrl = new URL('../third-party-notices/supplements.json', import.meta.url);
const manifest = JSON.parse(await readFile(manifestUrl, 'utf8'));

// Some published crates omit repository license texts. Bind each offline supplement
// to the reviewed archive, source revision and license; upgrades require a new audit.
export async function copySupplementalNotices(kind, name, version, license, root, destination) {
  const entry = manifest.packages.find((item) => item.kind === kind && item.name === name);
  if (!entry) return [];
  if (entry.version !== version || entry.license !== license) {
    throw new Error(`Supplemental notice requires review for ${kind}/${name}@${version}.`);
  }
  const checksum = JSON.parse(await readFile(path.join(root, '.cargo-checksum.json'), 'utf8'));
  const vcs = JSON.parse(await readFile(path.join(root, '.cargo_vcs_info.json'), 'utf8'));
  if (checksum.package !== entry.crate_sha256 || vcs.git?.sha1 !== entry.revision) {
    throw new Error(`Supplemental notice source pin does not match ${name}@${version}.`);
  }
  const text = await readFile(new URL(entry.notice_file, manifestUrl));
  if (createHash('sha256').update(text).digest('hex') !== entry.notice_sha256) {
    throw new Error(`Supplemental notice checksum does not match ${name}@${version}.`);
  }
  await mkdir(destination, { recursive: true });
  await writeFile(path.join(destination, 'LICENSE.upstream'), text);
  await writeFile(path.join(destination, 'NOTICE.source.json'), `${JSON.stringify(entry, null, 2)}\n`);
  return ['LICENSE.upstream', 'NOTICE.source.json'];
}

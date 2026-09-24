import { copyFile, lstat, mkdir, readdir, realpath, rm } from 'node:fs/promises';
import path from 'node:path';

const noticeName = /^(licen[cs]es?|copying|notices?|copyright)([._-]|$)/i;
const thirdPartyNotice = /^third[-_ ]?party(?:notices?|licenses?)[\w .-]*$/i;

export async function prepareNoticeOutput(repositoryRoot, requestedOutput) {
  const root = await realpath(repositoryRoot);
  const expected = path.join(root, 'crates', 'dmd-desktop', 'licenses', 'dependencies');
  if (path.relative(expected, path.resolve(requestedOutput)) !== '') {
    throw new Error('Notice output must be the repository generated dependency-notice directory.');
  }
  await mkdir(expected, { recursive: true });
  const resolved = await realpath(expected);
  if (path.relative(expected, resolved) !== '') throw new Error('Generated notice output must not resolve through a link.');
  // The resolved absolute target is exactly the generated directory in this checkout.
  // Clear stale notices so repeated preparation cannot preserve prior nested output.
  await rm(resolved, { recursive: true, force: true });
  await mkdir(resolved, { recursive: true });
  return resolved;
}

// Copy upstream notice trees, never arbitrary source trees or symbolic links.
export async function copyNoticeFiles(root, destination, licenseFile = null, excludedDirectory = null) {
  const notices = new Set();
  const excluded = excludedDirectory ? await realpath(excludedDirectory) : null;
  function isExcluded(directory) {
    if (!excluded) return false;
    const relative = path.relative(excluded, directory);
    return relative === '' || (!relative.startsWith(`..${path.sep}`) && relative !== '..' && !path.isAbsolute(relative));
  }
  async function visit(directory, recognizedDirectory = false) {
    if (isExcluded(await realpath(directory))) return;
    for (const entry of await readdir(directory, { withFileTypes: true })) {
      const recognized = recognizedDirectory || noticeName.test(entry.name) || thirdPartyNotice.test(entry.name);
      const source = path.join(directory, entry.name);
      if (entry.isDirectory() && recognized) await visit(source, true);
      else if (entry.isFile() && recognized) notices.add(path.relative(root, source));
    }
  }
  await visit(root);
  if (licenseFile) {
    const source = path.resolve(root, licenseFile);
    const relative = path.relative(root, source);
    if (relative.startsWith(`..${path.sep}`) || relative === '..' || path.isAbsolute(relative)) {
      throw new Error(`Declared license file must remain inside its package: ${licenseFile}`);
    }
    const canonicalSource = await realpath(source);
    if (isExcluded(canonicalSource)) throw new Error('Declared license file points into generated notice output.');
    const canonicalRelative = path.relative(await realpath(root), canonicalSource);
    if (canonicalRelative.startsWith(`..${path.sep}`) || canonicalRelative === '..' || path.isAbsolute(canonicalRelative)) {
      throw new Error(`Declared license file must resolve inside its package: ${licenseFile}`);
    }
    if (!(await lstat(source)).isFile()) throw new Error(`Declared license file is not a regular file: ${licenseFile}`);
    notices.add(relative);
  }
  const files = [...notices].sort();
  for (const relative of files) {
    const target = path.join(destination, relative);
    await mkdir(path.dirname(target), { recursive: true });
    await copyFile(path.join(root, relative), target);
  }
  return files.map((file) => file.replaceAll('\\', '/'));
}

import { copyFile, lstat, mkdir, readdir, realpath } from 'node:fs/promises';
import path from 'node:path';

const noticeName = /^(licen[cs]es?|copying|notices?|copyright)([._-]|$)/i;
const thirdPartyNotice = /^third[-_ ]?party(?:notices?|licenses?)[\w .-]*$/i;

// Copy upstream notice trees, never arbitrary source trees or symbolic links.
export async function copyNoticeFiles(root, destination, licenseFile = null) {
  const notices = new Set();
  async function visit(directory, recognizedDirectory = false) {
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
    const canonicalRelative = path.relative(await realpath(root), await realpath(source));
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

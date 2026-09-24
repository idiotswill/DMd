"""Check declared Cargo package identities, including aliases and workspace inheritance.

Uses only Python 3.11+ standard-library modules; no Cargo resolution or network access.
"""

import os
from pathlib import Path
import re
import sys
import tomllib


FORBIDDEN = {
    "dmd-domain": {"dmd-core", "dmd-rules", "dmd-persistence", "dmd-conversation", "dmd-app"},
    "dmd-conversation": {"dmd-core", "dmd-rules", "dmd-persistence", "dmd-app", "sqlx"},
    "dmd-core": {"dmd-persistence", "dmd-conversation", "dmd-app", "sqlx"},
    "dmd-rules": {"dmd-persistence", "dmd-conversation", "dmd-app", "sqlx"},
    "dmd-persistence": {"dmd-core", "dmd-rules", "dmd-conversation", "dmd-app"},
    "dmd-desktop": {"dmd-core", "dmd-rules", "dmd-persistence", "dmd-conversation", "sqlx"},
}
DEPENDENCY_KINDS = ("dependencies", "dev-dependencies", "build-dependencies")
SOURCE_NAMES = {
    crate: ("dmd_persistence",)
    for crate in ("dmd-domain", "dmd-conversation", "dmd-core", "dmd-rules")
}
SOURCE_NAMES["dmd-desktop"] = ("dmd_persistence", "dmd_rules", "sqlx")


class BoundaryError(Exception):
    pass


def table(value, context):
    if not isinstance(value, dict):
        raise BoundaryError(f"{context} must be a TOML table")
    return value


def read_manifest(path):
    try:
        with path.open("rb") as source:
            return tomllib.load(source)
    except (OSError, UnicodeError, tomllib.TOMLDecodeError) as error:
        raise BoundaryError(f"Cannot read manifest {path}: {error}") from error


def package_name(alias, spec, inherited, context):
    if isinstance(spec, str):
        return alias
    spec = table(spec, context)
    if "workspace" in spec:
        if spec["workspace"] is not True or alias not in inherited:
            raise BoundaryError(f"{context} has invalid workspace inheritance")
        # Root workspace dependencies cannot themselves inherit from another workspace.
        return package_name(alias, inherited[alias], {}, f"workspace.dependencies.{alias}")
    name = spec.get("package", alias)
    if not isinstance(name, str) or not name:
        raise BoundaryError(f"{context} has an invalid package name")
    return name


def dependencies(manifest):
    scopes = [("", manifest)]
    for target, spec in table(manifest.get("target", {}), "target").items():
        scopes.append((f"target.{target}.", table(spec, f"target.{target}")))
    for prefix, scope in scopes:
        for kind in DEPENDENCY_KINDS:
            context = prefix + kind
            for alias, spec in table(scope.get(kind, {}), context).items():
                yield f"{context}.{alias}", alias, spec


def fail_read(error):
    raise error


def check_sources(root, crate, names):
    source_dir = root / "crates" / crate / "src"
    if not source_dir.is_dir():
        raise BoundaryError(f"Missing production source directory: {source_dir}")
    pattern = re.compile(r"(?<![a-zA-Z0-9_])(?:" + "|".join(names) + r")(?![a-zA-Z0-9_])")
    try:
        # Included files count as production sources too; traversal/read errors fail closed.
        for directory, _, files in os.walk(source_dir, onerror=fail_read):
            for name in sorted(files):
                path = Path(directory) / name
                for line, text in enumerate(path.read_text(encoding="utf-8").splitlines(), 1):
                    if pattern.search(text):
                        raise BoundaryError(f"{path}:{line}: forbidden production reference: {text.strip()}")
    except (OSError, UnicodeError) as error:
        raise BoundaryError(f"Cannot inspect production sources: {error}") from error


def check_repository(root):
    workspace = table(read_manifest(root / "Cargo.toml").get("workspace", {}), "workspace")
    inherited = table(workspace.get("dependencies", {}), "workspace.dependencies")
    for crate, forbidden in FORBIDDEN.items():
        manifest_path = root / "crates" / crate / "Cargo.toml"
        # The desktop adapter is optional until its separately reviewed integration lands.
        if crate == "dmd-desktop" and not manifest_path.exists():
            continue
        manifest = read_manifest(manifest_path)
        for context, alias, spec in dependencies(manifest):
            name = package_name(alias, spec, inherited, context)
            if name in forbidden:
                raise BoundaryError(f"{manifest_path}: {context} must not depend on {name}")
        if crate in SOURCE_NAMES:
            check_sources(root, crate, SOURCE_NAMES[crate])


def main():
    root = Path(sys.argv[1]) if len(sys.argv) > 1 else Path(__file__).resolve().parent.parent
    try:
        check_repository(root)
    except BoundaryError as error:
        print(f"Architecture boundary violation: {error}", file=sys.stderr)
        return 1
    print("Architecture boundary guard passed.")
    return 0


if __name__ == "__main__":
    sys.exit(main())

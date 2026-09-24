"""Regenerate the pinned SRD distribution manifest after a reviewed content edit.

Developer tooling only; Python is not required by the local game runtime.
"""

import json
from pathlib import Path


def main():
    root = Path(__file__).resolve().parent.parent / "content" / "srd-5.2.1"
    kernel = json.loads((root / "kernel.json").read_text(encoding="utf-8"))
    if (kernel["id"], kernel["version"]) != ("srd-5.2", "5.2.1"):
        raise ValueError("Review source/version changes before regenerating this manifest")
    files = []
    for name in ("NOTICE.md", "source.json", "kernel.json"):
        data = (root / name).read_bytes()
        if b"\r" in data:
            raise ValueError(f"{name} must retain the pinned LF content line endings")
        digest = 0xCBF29CE484222325
        for byte in data:
            digest = ((digest ^ byte) * 0x100000001B3) & 0xFFFFFFFFFFFFFFFF
        files.append({"path": name, "byte_len": len(data), "checksum": {
            "algorithm": "fnv1a64", "value": f"{digest:016x}"}})
    manifest = {
        "manifest_schema_version": 1, "content_contract_version": 1,
        "kind": "ruleset", "id": kernel["id"], "version": kernel["version"],
        "compatible_rulesets": [], "dependencies": [], "files": files,
    }
    (root / "manifest.json").write_text(
        json.dumps(manifest, indent=2) + "\n", encoding="utf-8", newline="\n")


if __name__ == "__main__":
    main()

# Supplemental dependency notices

The three pinned WebView2 Rust crates declare MIT but their published archives omit
the repository's license text. `supplements.json` records each archive checksum and
published `.cargo_vcs_info.json` revision. The checked-in `webview2-rs/LICENSE` is the
unchanged MIT text at both recorded upstream revisions, including Bill Avery's
copyright notice. Its bytes are pinned by SHA-256 and checked during packaging.

The offline collector copies the text and its per-package provenance into each
dependency's generated notice directory. A version, archive, revision, license or
text hash mismatch stops packaging and requires this mapping to be reviewed.
These notices do not change DMd's own MIT license or the SRD's separate license.

"""Regression fixtures for valid Cargo forms that bypassed the old line-based guard."""

from pathlib import Path
import sys
from tempfile import TemporaryDirectory
import unittest

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
from check_boundaries import BoundaryError, FORBIDDEN, check_repository


class BoundaryTests(unittest.TestCase):
    def setUp(self):
        self.temp = TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name)
        self.workspace = self.root / "Cargo.toml"
        self.workspace.write_text("[workspace]\n", encoding="utf-8")
        for crate in FORBIDDEN:
            directory = self.root / "crates" / crate
            (directory / "src").mkdir(parents=True)
            (directory / "src" / "lib.rs").write_text("", encoding="utf-8")
            self.manifest(crate).write_text(f'[package]\nname = "{crate}"\n', encoding="utf-8")

    def manifest(self, crate="dmd-desktop"):
        return self.root / "crates" / crate / "Cargo.toml"

    def write_dependencies(self, text, crate="dmd-desktop"):
        self.manifest(crate).write_text(f'[package]\nname = "{crate}"\n{text}', encoding="utf-8")

    def test_legal_outer_adapter_and_comments_are_allowed(self):
        self.write_dependencies('[dependencies]\ndmd-app = "1"\ndmd-domain = "1"\n'
                                '# dmd-persistence = "1" is forbidden, not a dependency\n')
        check_repository(self.root)

    def test_original_alias_bypass_is_rejected(self):
        self.write_dependencies('[dependencies]\nstore = { package = "dmd-persistence", '
                                'path = "../dmd-persistence" }\n')
        (self.manifest().parent / "src" / "lib.rs").write_text(
            "use store::open_campaign;\n", encoding="utf-8")
        with self.assertRaisesRegex(BoundaryError, "must not depend on dmd-persistence"):
            check_repository(self.root)

    def test_dependency_tables_and_every_target_kind_cannot_hide_aliases(self):
        for prefix in ("", 'target.\'cfg(windows)\'.'):
            for kind in ("dependencies", "dev-dependencies", "build-dependencies"):
                with self.subTest(prefix=prefix, kind=kind):
                    self.write_dependencies(f'[{prefix}{kind}.storage]\n'
                                            'package = "sqlx"\nversion = "0.8"\n')
                    with self.assertRaisesRegex(BoundaryError, "must not depend on sqlx"):
                        check_repository(self.root)

    def test_workspace_aliases_are_resolved_by_package_identity(self):
        self.workspace.write_text('[workspace.dependencies]\n'
                                  'storage = { package = "dmd-persistence", path = "crates/dmd-persistence" }\n'
                                  'app = { package = "dmd-app", path = "crates/dmd-app" }\n', encoding="utf-8")
        self.write_dependencies('[dependencies]\napp.workspace = true\n')
        check_repository(self.root)
        self.write_dependencies('[target.\'cfg(windows)\'.dependencies]\nstorage.workspace = true\n')
        with self.assertRaisesRegex(BoundaryError, "must not depend on dmd-persistence"):
            check_repository(self.root)

    def test_existing_inner_layer_rules_also_reject_renamed_packages(self):
        for crate, package in (("dmd-domain", "dmd-core"), ("dmd-conversation", "dmd-rules"),
                               ("dmd-core", "dmd-persistence"), ("dmd-rules", "sqlx"),
                               ("dmd-persistence", "dmd-app")):
            with self.subTest(crate=crate):
                self.write_dependencies(f'[dependencies]\nalias = {{ package = "{package}", version = "1" }}\n', crate)
                with self.assertRaisesRegex(BoundaryError, f"must not depend on {package}"):
                    check_repository(self.root)
                self.write_dependencies("", crate)

    def test_source_scans_and_missing_sources_fail_closed(self):
        source = self.manifest().parent / "src" / "lib.rs"
        source.write_text("use dmd_rules::RulesAction;\n", encoding="utf-8")
        with self.assertRaisesRegex(BoundaryError, "forbidden production reference"):
            check_repository(self.root)
        source.unlink()
        source.parent.rmdir()
        with self.assertRaisesRegex(BoundaryError, "Missing production source"):
            check_repository(self.root)

    def test_malformed_missing_and_unresolved_manifests_fail_closed(self):
        for text in ('[dependencies', '[dependencies]\nunknown.workspace = true\n'):
            with self.subTest(text=text):
                self.write_dependencies(text)
                with self.assertRaises(BoundaryError):
                    check_repository(self.root)
        self.write_dependencies("")
        for crate in ("dmd-rules", "dmd-desktop"):
            with self.subTest(missing=crate):
                self.manifest(crate).unlink()
                with self.assertRaisesRegex(BoundaryError, "Cannot read manifest"):
                    check_repository(self.root)
                self.write_dependencies("", crate)

    def test_linked_source_directories_are_rejected_instead_of_silently_skipped(self):
        outside = self.root / "shared-source"
        outside.mkdir()
        (outside / "storage.rs").write_text("use dmd_persistence::open_campaign;\n", encoding="utf-8")
        link = self.manifest().parent / "src" / "linked"
        try:
            link.symlink_to(outside, target_is_directory=True)
        except OSError as error:
            if getattr(error, "winerror", None) == 1314:
                self.skipTest("Windows user lacks symbolic-link privilege; CI exercises this fixture")
            raise
        with self.assertRaisesRegex(BoundaryError, "Linked production source directory"):
            check_repository(self.root)


if __name__ == "__main__":
    unittest.main()

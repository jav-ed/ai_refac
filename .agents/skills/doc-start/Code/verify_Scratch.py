#!/usr/bin/env python3
"""Exercise the scaffold and destructive helper only in temporary repositories."""
import os
from pathlib import Path
import subprocess
import tempfile
import unittest

Skill_Root = Path(__file__).resolve().parent.parent
Expected_Folders = ["Agent_Tasks", "Audit", "Design", "Screenshots"]


class Scratch_Tests(unittest.TestCase):
    def setUp(self):
        temporary = tempfile.TemporaryDirectory(prefix="doc-start-scratch-")
        self.addCleanup(temporary.cleanup)
        self.temporary = Path(temporary.name)
        self.project = self.temporary / "project with spaces"
        self.project.mkdir()

    def run_Command(self, *args, succeeds=True, cwd=None):
        result = subprocess.run(args, cwd=cwd or self.project, capture_output=True, text=True)
        if succeeds:
            self.assertEqual(result.returncode, 0, result.stdout + result.stderr)
        else:
            self.assertNotEqual(result.returncode, 0, result.stdout + result.stderr)
        return result

    def bootstrap(self):
        return self.run_Command("bash", str(Skill_Root / "Code/bootstrap.sh"), str(self.project))

    def install(self, succeeds=True):
        return self.run_Command(
            "bash", str(Skill_Root / "Code/install_Scratch.sh"), str(self.project), succeeds=succeeds
        )

    def write_File(self, relative, content="disposable"):
        path = self.project / relative
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_text(content)
        return path

    def assert_Empty_Scratch(self):
        scratch = self.project / "Scratch"
        self.assertEqual(sorted(p.name for p in scratch.iterdir()), Expected_Folders)
        for folder in scratch.iterdir():
            self.assertTrue(folder.is_dir())
            self.assertFalse(folder.is_symlink())
            self.assertEqual(list(folder.iterdir()), [])

    def test_Full_Reset_And_Safe_Default(self):
        note = self.write_File("Scratch/Agent_Tasks/task.md")
        self.bootstrap()
        self.assertTrue(note.exists(), "Bootstrap must not wipe existing Scratch files")
        self.run_Command("just")
        self.assertTrue(note.exists(), "Bare just must not invoke cleanup")
        cache = self.write_File("Cache/CI/Artifacts/release.tar.gz", "retained archive")
        for name in [".hidden.json", "output.log", "image.png", "copied/src/old.ts"]:
            self.write_File("Scratch/" + name)
        outside = self.temporary / "outside"
        outside.mkdir()
        original = outside / "original.ts"
        original.write_text("live source")
        (self.project / "Scratch/nested").mkdir()
        (self.project / "Scratch/nested/source").symlink_to(outside, target_is_directory=True)
        (self.project / "Scratch/dangling").symlink_to(outside / "absent")
        os.link(original, self.project / "Scratch/hard-link.ts")
        self.run_Command(
            "just", "--justfile", str(self.project / "justfile"), "scratch-clean", cwd=outside
        )
        self.assert_Empty_Scratch()
        self.assertEqual(original.read_text(), "live source")
        self.assertEqual(cache.read_text(), "retained archive")

    def test_Repeat_Bootstrap_Preserves_Existing_Default(self):
        justfile = self.write_File("Justfile", "greet:\n    @echo original-default\n")
        self.bootstrap()
        installed = justfile.read_bytes()
        self.bootstrap()
        self.assertEqual(justfile.read_bytes(), installed)
        self.assertIn("original-default", self.run_Command("just").stdout)
        self.run_Command("just", "scratch-clean")
        self.assert_Empty_Scratch()

    def test_Imported_Cleanup_Is_Retained(self):
        contents = "import 'tools.just'\n"
        justfile = self.write_File("justfile", contents)
        self.write_File("tools.just", "scratch-clean:\n    @echo existing-cleaner\n")
        self.install()
        self.assertEqual(justfile.read_text(), contents)
        self.assertFalse((self.project / "Code/Development/Scratch/clean.sh").exists())
        self.assertIn("existing-cleaner", self.run_Command("just", "scratch-clean").stdout)

    def test_Hidden_Justfile_Is_Extended(self):
        self.write_File(".justfile", "greet:\n    @echo original-default\n")
        self.install()
        self.assertFalse((self.project / "justfile").exists())
        self.run_Command("just", "scratch-clean")
        self.assert_Empty_Scratch()

    def test_Settings_Only_Justfile_Gets_Safe_Default(self):
        self.write_File("justfile", "set shell := ['bash', '-cu']\n")
        note = self.write_File("Scratch/keep.md")
        self.install()
        self.run_Command("just")
        self.assertTrue(note.exists())
        self.run_Command("just", "scratch-clean")
        self.assert_Empty_Scratch()

    def test_Conflicting_Helper_Is_Not_Overwritten(self):
        helper = self.write_File("Code/Development/Scratch/clean.sh", "custom implementation\n")
        result = self.install(succeeds=False)
        self.assertIn("differs from the template", result.stderr)
        self.assertEqual(helper.read_text(), "custom implementation\n")
        self.assertFalse((self.project / "justfile").exists())

    def test_Ambiguous_Justfiles_Are_Refused(self):
        for name in ["justfile", "Justfile"]:
            self.write_File(name, "greet:\n    @echo keep\n")
        result = self.install(succeeds=False)
        self.assertIn("Multiple Justfiles", result.stderr)
        self.assertFalse((self.project / "Code/Development/Scratch/clean.sh").exists())

    def test_Symlinked_And_File_Scratch_Roots_Are_Refused(self):
        self.install()
        outside = self.temporary / "outside"
        outside.mkdir()
        original = outside / "keep.md"
        original.write_text("keep")
        scratch = self.project / "Scratch"
        for target in [outside, self.temporary / "absent"]:
            with self.subTest(target=target):
                scratch.symlink_to(target, target_is_directory=True)
                result = self.run_Command("just", "scratch-clean", succeeds=False)
                self.assertIn("must be a real directory", result.stderr)
                self.assertEqual(original.read_text(), "keep")
                scratch.unlink()
        scratch.write_text("keep")
        self.run_Command("just", "scratch-clean", succeeds=False)
        self.assertEqual(scratch.read_text(), "keep")

    def test_Arguments_And_Uninstalled_Template_Are_Refused(self):
        self.install()
        note = self.write_File("Scratch/task.md")
        self.run_Command(
            "bash", str(self.project / "Code/Development/Scratch/clean.sh"), "unexpected",
            succeeds=False,
        )
        self.assertTrue(note.exists())
        result = self.run_Command("bash", str(Skill_Root / "Templates/clean_Scratch.sh"), succeeds=False)
        self.assertIn("Install this helper", result.stderr)

    def test_Missing_And_Empty_Scratch_Reset(self):
        self.install()
        for _ in range(2):
            self.run_Command("just", "scratch-clean")
            self.assert_Empty_Scratch()


if __name__ == "__main__":
    unittest.main(verbosity=2)

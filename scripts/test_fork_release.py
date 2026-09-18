from __future__ import annotations

import hashlib
from pathlib import Path
import struct
import tarfile
import tempfile
import unittest
import zipfile

from scripts import fork_release as release


class ForkReleaseTests(unittest.TestCase):
    def test_only_exact_stable_version_tags_are_accepted(self) -> None:
        release.validate_tag("0.9.2", None)
        release.validate_tag("0.9.2", "v0.9.2")
        for tag in ("v0.9.1", "0.9.2", "v0.9.2-preview.1", "v0.9.2+build",
                    "v00.9.2", "v0.9.2\n", "v0.9.2; echo unsafe"):
            with self.subTest(tag=tag), self.assertRaises(ValueError):
                release.validate_tag("0.9.2", tag)

    def test_macos_package_contains_renamed_executable_and_license(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            binary = root / "input"
            binary.write_bytes(struct.pack("<II", 0xFEEDFACF, 0x0100000C) + b"fixture")
            output = root / release.ASSETS[0]
            release.package_macos(binary, output)
            with tarfile.open(output) as archive:
                self.assertEqual(archive.getnames(), ["herdr-dumb", "LICENSE"])
                self.assertEqual(archive.getmember("herdr-dumb").mode, 0o755)
                self.assertEqual(archive.extractfile("herdr-dumb").read(), binary.read_bytes())

    def test_wrong_macos_architecture_and_non_macho_are_rejected(self) -> None:
        for data in (b"", b"MZ000000", struct.pack("<II", 0xFEEDFACF, 0x01000007)):
            with self.subTest(data=data), self.assertRaises(ValueError):
                release.validate_macos_binary(data)

    def make_assets(self, root: Path, *, windows_exe: str = "herdr-dumb.exe") -> Path:
        binary = root / "input"
        binary.write_bytes(struct.pack("<II", 0xFEEDFACF, 0x0100000C))
        directory = root / "dist"
        directory.mkdir()
        release.package_macos(binary, directory / release.ASSETS[0])
        with zipfile.ZipFile(directory / release.ASSETS[1], "w") as archive:
            for name in (windows_exe, "conpty/conpty.dll", "conpty/herdr-conpty.json",
                         "conpty/x64/OpenConsole.exe", "conpty/arm64/OpenConsole.exe"):
                archive.writestr(name, b"test fixture")
        return directory

    def test_checksums_cover_both_exact_archives(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            directory = self.make_assets(Path(temporary))
            output = release.write_checksums(directory)
            expected = "".join(
                f"{hashlib.sha256((directory / name).read_bytes()).hexdigest()}  {name}\n"
                for name in release.ASSETS
            )
            self.assertEqual(output.read_text(), expected)

    def test_partial_or_extra_artifacts_are_rejected(self) -> None:
        for mode in ("partial", "extra"):
            with self.subTest(mode=mode), tempfile.TemporaryDirectory() as temporary:
                directory = self.make_assets(Path(temporary))
                if mode == "partial":
                    (directory / release.ASSETS[1]).unlink()
                else:
                    (directory / "unrelated.txt").write_text("unexpected")
                with self.assertRaisesRegex(ValueError, "exactly"):
                    release.write_checksums(directory)
                self.assertFalse((directory / "SHA256SUMS").exists())

    def test_upstream_windows_executable_is_rejected(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            directory = self.make_assets(Path(temporary), windows_exe="herdr.exe")
            with self.assertRaisesRegex(ValueError, "renamed binary"):
                release.write_checksums(directory)

    def test_macos_archive_with_symlink_is_rejected(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            directory = self.make_assets(Path(temporary))
            with tarfile.open(directory / release.ASSETS[0], "w:gz") as archive:
                link = tarfile.TarInfo("herdr-dumb")
                link.type = tarfile.SYMTYPE
                link.linkname = "/outside"
                archive.addfile(link)
                archive.addfile(tarfile.TarInfo("LICENSE"))
            with self.assertRaisesRegex(ValueError, "regular files"):
                release.write_checksums(directory)


if __name__ == "__main__":
    unittest.main()

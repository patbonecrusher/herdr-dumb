"""Offline validation and packaging for the herdr-dumb GitHub workflow."""
from __future__ import annotations

import argparse
import hashlib
from pathlib import Path
import re
import struct
import tarfile
import tomllib
import zipfile


ROOT = Path(__file__).resolve().parent.parent
ASSETS = ("herdr-dumb-macos-arm64.tar.gz", "herdr-dumb-windows-x86_64.zip")


def validate_tag(version: str, tag: str | None) -> None:
    if tag is not None and (
        re.fullmatch(r"v(?:0|[1-9][0-9]*)\.(?:0|[1-9][0-9]*)\.(?:0|[1-9][0-9]*)", tag) is None
        or tag != f"v{version}"
    ):
        raise ValueError(f"release tag {tag!r} must exactly match Cargo.toml version v{version}")


def validate_macos_binary(binary: bytes) -> None:
    if len(binary) < 8 or struct.unpack_from("<II", binary) != (0xFEEDFACF, 0x0100000C):
        raise ValueError("expected a macOS ARM64 Mach-O executable")


def package_macos(binary: Path, output: Path) -> None:
    validate_macos_binary(binary.read_bytes())
    output.parent.mkdir(parents=True, exist_ok=True)
    with tarfile.open(output, "w:gz") as archive:
        for path, name, mode in ((binary, "herdr-dumb", 0o755), (ROOT / "LICENSE", "LICENSE", 0o644)):
            info = archive.gettarinfo(str(path), arcname=name)
            info.mode = mode
            info.uid = info.gid = 0
            info.uname = info.gname = ""
            with path.open("rb") as source:
                archive.addfile(info, source)


def write_checksums(directory: Path) -> Path:
    # Only the two expected archives may be published; never silently publish a
    # partial matrix, upstream binary, or unrelated downloaded artifact.
    actual = {path.name for path in directory.iterdir()}
    if actual != set(ASSETS):
        raise ValueError(f"expected exactly {ASSETS}; found {sorted(actual)}")
    for name in ASSETS:
        path = directory / name
        if path.is_symlink() or not path.is_file():
            raise ValueError(f"not a regular release asset: {path}")
    with tarfile.open(directory / ASSETS[0], "r:gz") as archive:
        members = archive.getmembers()
        if len(members) != 2 or {m.name for m in members} != {"herdr-dumb", "LICENSE"}:
            raise ValueError("unexpected macOS archive layout")
        if not all(m.isfile() for m in members):
            raise ValueError("macOS archive must contain only regular files")
        if archive.getmember("herdr-dumb").mode & 0o111 != 0o111:
            raise ValueError("macOS binary must be executable")
        source = archive.extractfile("herdr-dumb")
        if source is None:
            raise ValueError("macOS binary missing")
        with source:
            validate_macos_binary(source.read(8))
    with zipfile.ZipFile(directory / ASSETS[1]) as archive:
        names = archive.namelist()
        required = {"herdr-dumb.exe", "conpty/conpty.dll", "conpty/herdr-conpty.json",
                    "conpty/x64/OpenConsole.exe", "conpty/arm64/OpenConsole.exe"}
        if len(names) != len(set(names)) or not required.issubset(names) or "herdr.exe" in names:
            raise ValueError("Windows archive is missing the renamed binary or ConPTY runtime")
    checksums = directory / "SHA256SUMS"
    lines = []
    for name in ASSETS:
        with (directory / name).open("rb") as source:
            digest = hashlib.file_digest(source, "sha256").hexdigest()
        lines.append(f"{digest}  {name}\n")
    checksums.write_text("".join(lines), encoding="utf-8")
    return checksums


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    commands = parser.add_subparsers(dest="command", required=True)
    version = commands.add_parser("check-version")
    version.add_argument("--tag")
    macos = commands.add_parser("package-macos")
    macos.add_argument("--binary", type=Path, required=True)
    macos.add_argument("--output", type=Path, required=True)
    checksums = commands.add_parser("checksums")
    checksums.add_argument("--directory", type=Path, required=True)
    args = parser.parse_args()
    if args.command == "check-version":
        with (ROOT / "Cargo.toml").open("rb") as source:
            value = tomllib.load(source)["package"]["version"]
        validate_tag(value, args.tag)
        print(value)
    elif args.command == "package-macos":
        package_macos(args.binary, args.output)
    else:
        print(write_checksums(args.directory))


if __name__ == "__main__":
    main()

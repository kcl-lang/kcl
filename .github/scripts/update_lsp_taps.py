#!/usr/bin/env python3
"""Regenerate the kcl-language-server Homebrew formula and Scoop manifest.

The Homebrew tap and the Scoop bucket live in separate repositories that GoReleaser
only manages for the kcl CLI, so kcl-lsp has to be bumped by hand. This renders
the files for a given tag so the release workflow can commit them.

Usage:
    update_lsp_taps.py --tag v0.13.2 --assets-dir artifacts --out-dir tap-files

`--assets-dir` is searched recursively, so it accepts both a flat directory and the
per-target subdirectories that `actions/download-artifact` produces.
"""

import argparse
import hashlib
import pathlib
import re
import sys

REPO = "kcl-lang/kcl"
BINARY = "kcl-language-server"
WINDOWS_BINARY = "kcl-language-server.exe"

# The language server only ships for these four targets, unlike kcl-lib which
# also has musl and wasm builds. This order matches the formula layout.
FORMULA_TARGETS = ("darwin-amd64", "darwin-arm64", "linux-amd64", "linux-arm64")
WINDOWS_TARGET = "windows-amd64"

FORMULA_TEMPLATE = '''\
class {klass} < Formula
  desc "A constraint-based record & functional language mainly used in configuration and policy scenarios."
  homepage "https://kcl-lang.io"
  version "{version}"
  license "Apache License"

  on_macos do
    if Hardware::CPU.intel?
      url "{darwin_amd64_url}"
      sha256 "{darwin_amd64_sha}"
    end
    if Hardware::CPU.arm?
      url "{darwin_arm64_url}"
      sha256 "{darwin_arm64_sha}"
    end
  end

  on_linux do
    if Hardware::CPU.intel?
      url "{linux_amd64_url}"
      sha256 "{linux_amd64_sha}"
    end
    if Hardware::CPU.arm?
      url "{linux_arm64_url}"
      sha256 "{linux_arm64_sha}"
    end
  end

  def install
    bin.install "{binary}"
  end
end
'''

SCOOP_TEMPLATE = """\
{{
    "version": "{version}",
    "architecture": {{
        "64bit": {{
            "url": "{windows_url}",
            "bin": [
                "{windows_binary}"
            ],
            "hash": "{windows_sha}"
        }}
    }},
    "homepage": "https://github.com/{repo}",
    "license": "Apache-2.0",
    "description": "KCL Language Server"
}}
"""


def sha256_of(path):
    digest = hashlib.sha256()
    with path.open("rb") as f:
        for chunk in iter(lambda: f.read(1024 * 1024), b""):
            digest.update(chunk)
    return digest.hexdigest()


def find_asset(assets_dir, name):
    matches = sorted(pathlib.Path(assets_dir).rglob(name))
    if not matches:
        return None
    if len(matches) > 1:
        sys.exit(f"error: found multiple copies of {name}: {matches}")
    return matches[0]


def parse_version(tag):
    match = re.fullmatch(r"v(\d+)\.(\d+)\.(\d+)", tag)
    if not match:
        sys.exit(f"error: {tag!r} is not a vMAJOR.MINOR.PATCH tag")
    return match.group(1), match.group(2), match.group(3)


def formula_klass(alias):
    """Homebrew class name for a formula file: KclLsp, KclLspAT013, KclLspAT0132."""
    if alias is None:
        return "KclLsp"
    return "KclLspAT" + alias.replace(".", "")


def collect_checksums(assets_dir, tag):
    checksums = {}
    for target in FORMULA_TARGETS + (WINDOWS_TARGET,):
        name = f"{BINARY}-{tag}-{target}"
        name += ".zip" if target == WINDOWS_TARGET else ".tar.gz"
        path = find_asset(assets_dir, name)
        if path is None:
            sys.exit(f"error: missing asset {name} under {assets_dir}")
        checksums[target] = sha256_of(path)
    return checksums


def write(path, content):
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(content, newline="\n")
    print(f"wrote {path}")


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--tag", required=True, help="release tag, e.g. v0.13.2")
    parser.add_argument(
        "--assets-dir",
        required=True,
        help="directory holding the downloaded language server archives",
    )
    parser.add_argument(
        "--out-dir",
        required=True,
        help="directory to render HomebrewFormula/ and Scoops/ into",
    )
    args = parser.parse_args()

    major, minor, patch = parse_version(args.tag)
    version = f"{major}.{minor}.{patch}"
    checksums = collect_checksums(args.assets_dir, args.tag)

    def url_for(target, suffix=".tar.gz"):
        return f"https://github.com/{REPO}/releases/download/{args.tag}/{BINARY}-{args.tag}-{target}{suffix}"

    out_dir = pathlib.Path(args.out_dir)
    formula_dir = out_dir / "HomebrewFormula"

    # The unversioned formula tracks the latest release; the pinned ones let
    # `brew install kcl-lsp@0.13` and `kcl-lsp@0.13.2` resolve without guessing.
    aliases = [None, f"{major}.{minor}.{patch}", f"{major}.{minor}", f"{major}"]
    for alias in aliases:
        filename = "kcl-lsp.rb" if alias is None else f"kcl-lsp@{alias}.rb"
        write(
            formula_dir / filename,
            FORMULA_TEMPLATE.format(
                klass=formula_klass(alias),
                version=version,
                binary=BINARY,
                darwin_amd64_url=url_for("darwin-amd64"),
                darwin_amd64_sha=checksums["darwin-amd64"],
                darwin_arm64_url=url_for("darwin-arm64"),
                darwin_arm64_sha=checksums["darwin-arm64"],
                linux_amd64_url=url_for("linux-amd64"),
                linux_amd64_sha=checksums["linux-amd64"],
                linux_arm64_url=url_for("linux-arm64"),
                linux_arm64_sha=checksums["linux-arm64"],
            ),
        )

    # The Windows zip holds the executable at its root, not under bin/.
    write(
        out_dir / "Scoops" / "kcl-lsp.json",
        SCOOP_TEMPLATE.format(
            version=version,
            windows_url=url_for(WINDOWS_TARGET, ".zip"),
            windows_binary=WINDOWS_BINARY,
            windows_sha=checksums[WINDOWS_TARGET],
            repo=REPO,
        ),
    )


if __name__ == "__main__":
    main()

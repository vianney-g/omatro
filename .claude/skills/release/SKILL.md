---
name: release
description: Publishes a new version of omatro, end to end - checks, version bump, git tag, the GitHub release built by Actions (with release notes), then the AUR package (PKGBUILD, checksums, .SRCINFO, push). Use it whenever the maintainer wants to ship, release, publish, tag or "sortir" a version (« sors la 0.2.0 », « on publie », « nouvelle version »), to update or fix the AUR package (pkgrel bump, first AUR upload), or to redo a release whose workflow failed.
---

# Releasing omatro

A release goes out in two places: a **GitHub release** (built by
`.github/workflows/release.yml` when a `v*` tag is pushed: Arch container, tests,
archive with the binary, launcher entry, icon and a `.sha256`) and the **AUR package**
`omatro`, which builds from the tag's source tarball (`pkgbuild/PKGBUILD` in this repo
is the reference copy; the AUR repo is a separate git repo).

Pushing a tag and pushing to the AUR publish things other people install: when the
maintainer asked for a release, that is the go-ahead, but stop and report instead of
pushing whenever a check fails. Talk to the maintainer in French; commit messages,
release notes and the PKGBUILD stay in English.

## 0. Where things stand

```sh
git status --short && git fetch -q && git status -sb | head -1   # clean, master, in sync
git describe --tags --abbrev=0                                   # previous version
git log --oneline $(git describe --tags --abbrev=0)..HEAD        # what this release holds
```

A dirty tree or unpushed commits: ask whether they belong to the release before going
on. No new commits since the last tag: there is nothing to release (maybe they want an
AUR-only fix, see step 5).

**Version**: use the one asked for. Otherwise propose one from the log (semver, still
0.x: new content or features → minor, `0.2.0`; only fixes → patch, `0.1.1`) and wait
for a yes.

## 1. Checks

The release must not ship a broken game. Run, in the background as they take minutes:

```sh
make test
make bot ARGS="1000 1" && make bot ARGS="1000 1 --chaos --lang fr"
```

Each bot run must end with `Problems: none` (and `Untranslated into fr: 0`); `Refused
commands` should be empty. Anything else is a bug to fix (or to report) before
releasing, not after.

## 2. Bump, commit, tag

Set `version = "X.Y.Z"` in `app/Cargo.toml` and `engine/Cargo.toml`, then refresh the
lockfile and check it builds (the workflow uses `--locked`, so a stale `Cargo.lock`
fails the release):

```sh
cargo update -w && cargo build -q --release -p omatro
git commit -am "Release X.Y.Z" -m "Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
git push && git tag vX.Y.Z && git push origin vX.Y.Z
```

## 3. Watch the workflow

```sh
gh run list --limit 1                       # the run started by the tag
gh run watch <id> --exit-status             # in the background: about 10 minutes
```

On failure, read `gh run view <id> --log-failed`, fix, and move the tag onto the fix:
`git tag -d vX.Y.Z && git push origin :refs/tags/vX.Y.Z`, delete a half-made release
with `gh release delete vX.Y.Z --yes` if one exists, then tag and push again. Only do
this while the release is minutes old: once people may have downloaded it, release
X.Y.(Z+1) instead.

## 4. Release notes

The workflow writes the install instructions. Put what changed above them, for
players: read `git log vPREV..vX.Y.Z`, keep what a player notices (new content,
mechanics, interface, fixed bugs), merge related commits, drop internal work (tests,
docs, refactors, tooling). A few short bullets in English, in the README's tone.

```sh
gh release view vX.Y.Z --json body -q .body > /tmp/body.md
# write /tmp/notes.md: "## What's new" + bullets, a blank line, then /tmp/body.md
gh release edit vX.Y.Z --notes-file /tmp/notes.md
```

## 5. AUR

`scripts/aur.sh X.Y.Z` (in this skill's directory) does the mechanical part: clones the
AUR repo if needed (`$AUR_DIR`, default `~/Work/omatro-aur`), sets `pkgver` and
`pkgrel=1` in `pkgbuild/PKGBUILD`, fetches the tag's tarball and updates the checksum,
builds and tests the package with `makepkg` in a scratch directory, and copies
`PKGBUILD` and a fresh `.SRCINFO` into the AUR repo. It does not commit or push.
For a packaging-only fix (same version, changed PKGBUILD), pass `--pkgrel N`.

Then look at `git -C ~/Work/omatro-aur diff` (pkgver, checksum, nothing unexpected) and
publish:

```sh
git -C ~/Work/omatro-aur add PKGBUILD .SRCINFO
git -C ~/Work/omatro-aur commit -m "Update to X.Y.Z"
git -C ~/Work/omatro-aur push
git add pkgbuild/PKGBUILD && git commit -m "AUR: X.Y.Z" -m "Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>" && git push
```

The AUR repo's branch is `master`. If the clone or push says `Permission denied
(publickey)`, the maintainer's SSH key is not registered on aur.archlinux.org (My
Account → SSH Public Key) or `~/.ssh/config` lacks the `Host aur.archlinux.org` /
`User aur` entry: say so and stop there; everything before is done and stays valid.

Why `options=('!lto')` stays in the PKGBUILD: makepkg's default LTO turns cxx-qt's C++
objects into bitcode the Rust linker can't resolve (undefined `qstring_*` symbols).

## 6. Report

In French, briefly: the version, the release link (`gh release view vX.Y.Z --json url
-q .url`), the AUR page (https://aur.archlinux.org/packages/omatro), the release notes
as written, and anything skipped or left to do.

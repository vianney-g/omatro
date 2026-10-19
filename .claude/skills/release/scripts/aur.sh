#!/bin/bash
# Prepares the AUR package of omatro for a version already tagged on GitHub:
# updates pkgbuild/PKGBUILD (pkgver, pkgrel, checksum), builds and tests it
# with makepkg, and copies PKGBUILD and .SRCINFO into the AUR repo. Commits
# and pushes nothing.
#
# Usage: aur.sh X.Y.Z [--pkgrel N]
set -euo pipefail

version=${1:?usage: aur.sh X.Y.Z [--pkgrel N]}
pkgrel=1
if [ "${2:-}" = "--pkgrel" ]; then pkgrel=${3:?--pkgrel needs a number}; fi

repo=$(git rev-parse --show-toplevel)
aur=${AUR_DIR:-$HOME/Work/omatro-aur}
pkgbuild=$repo/pkgbuild/PKGBUILD

if ! git -C "$repo" ls-remote --exit-code --tags origin "v$version" > /dev/null; then
  echo "error: tag v$version is not on GitHub yet" >&2
  exit 1
fi

if [ ! -d "$aur/.git" ]; then
  git clone ssh://aur@aur.archlinux.org/omatro.git "$aur"
fi
git -C "$aur" pull -q --ff-only 2> /dev/null || true   # empty before the first upload

sed -i -e "s/^pkgver=.*/pkgver=$version/" -e "s/^pkgrel=.*/pkgrel=$pkgrel/" "$pkgbuild"

build=$(mktemp -d)
trap 'rm -rf "$build"' EXIT
cp "$pkgbuild" "$build/"
(
  cd "$build"
  updpkgsums
  makepkg -f --noconfirm > makepkg.log 2>&1 || { tail -40 makepkg.log; exit 1; }
  makepkg --printsrcinfo > .SRCINFO
)
cp "$build/PKGBUILD" "$pkgbuild"
cp "$build/PKGBUILD" "$build/.SRCINFO" "$aur/"

echo "omatro $version-$pkgrel built and tested; ready in $aur:"
git -C "$aur" status --short

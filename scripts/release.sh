#!/bin/sh

set -eu

ROOT="$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)"
cd "$ROOT"

SOURCE_REPOSITORY="${SOURCE_REPOSITORY:-knothhe/scroll-split}"
HOMEBREW_TAP_REPOSITORY="${HOMEBREW_TAP_REPOSITORY:-knothhe/homebrew-tap}"
VERSION="${1:-$(cargo metadata --no-deps --format-version 1 | ruby -rjson -e 'puts JSON.parse(STDIN.read).fetch("packages").find { |package| package.fetch("name") == "scrollsplit" }.fetch("version")')}"
TAG="v${VERSION}"
ARM_TARGET="aarch64-apple-darwin"
INTEL_TARGET="x86_64-apple-darwin"
DIST="$ROOT/dist"
TAP_DIR="$(mktemp -d)"

cleanup() {
  rm -rf "$TAP_DIR"
}
trap cleanup EXIT INT TERM

command -v cargo >/dev/null
command -v rustup >/dev/null
command -v gh >/dev/null
command -v ruby >/dev/null

test "$(uname -s)" = "Darwin" || {
  echo "release must run on macOS" >&2
  exit 1
}

test -z "$(git status --porcelain)" || {
  echo "working tree must be clean before release" >&2
  exit 1
}

package_version="$(cargo metadata --no-deps --format-version 1 | ruby -rjson -e 'puts JSON.parse(STDIN.read).fetch("packages").find { |package| package.fetch("name") == "scrollsplit" }.fetch("version")')"
test "$VERSION" = "$package_version" || {
  echo "requested version $VERSION does not match Cargo.toml version $package_version" >&2
  exit 1
}

gh auth status >/dev/null
gh repo view "$SOURCE_REPOSITORY" >/dev/null
gh repo view "$HOMEBREW_TAP_REPOSITORY" >/dev/null || {
  echo "Homebrew tap repository does not exist: https://github.com/$HOMEBREW_TAP_REPOSITORY" >&2
  echo "Create it first with: gh repo create $HOMEBREW_TAP_REPOSITORY --public" >&2
  exit 1
}
gh repo clone "$HOMEBREW_TAP_REPOSITORY" "$TAP_DIR"

git rev-parse "$TAG" >/dev/null 2>&1 && {
  echo "tag already exists: $TAG" >&2
  exit 1
}
gh release view "$TAG" --repo "$SOURCE_REPOSITORY" >/dev/null 2>&1 && {
  echo "GitHub release already exists: $TAG" >&2
  exit 1
}

cargo test --locked
rustup target add "$ARM_TARGET" "$INTEL_TARGET"
cargo build --locked --release --target "$ARM_TARGET"
cargo build --locked --release --target "$INTEL_TARGET"

mkdir -p "$DIST"
arm_archive="$DIST/scrollsplit-${TAG}-${ARM_TARGET}.tar.gz"
intel_archive="$DIST/scrollsplit-${TAG}-${INTEL_TARGET}.tar.gz"
rm -f "$arm_archive" "$intel_archive"
tar -C "target/$ARM_TARGET/release" -czf "$arm_archive" scrollsplit
tar -C "target/$INTEL_TARGET/release" -czf "$intel_archive" scrollsplit

arm_sha256="$(shasum -a 256 "$arm_archive" | awk '{print $1}')"
intel_sha256="$(shasum -a 256 "$intel_archive" | awk '{print $1}')"

git push origin HEAD
git tag "$TAG"
git push origin "$TAG"
gh release create "$TAG" "$arm_archive" "$intel_archive" \
  --repo "$SOURCE_REPOSITORY" \
  --title "$TAG" \
  --generate-notes \
  --verify-tag

SOURCE_REPOSITORY="$SOURCE_REPOSITORY" ruby scripts/render-homebrew-formula.rb \
  "$VERSION" "$arm_sha256" "$intel_sha256" "$TAP_DIR/Formula/scrollsplit.rb"

git -C "$TAP_DIR" add Formula/scrollsplit.rb
git -C "$TAP_DIR" commit -m "scrollsplit ${VERSION}"
git -C "$TAP_DIR" push origin HEAD

echo "Released scrollsplit $VERSION"
echo "Install with: brew install ${HOMEBREW_TAP_REPOSITORY%/homebrew-*}/${HOMEBREW_TAP_REPOSITORY#*/homebrew-}/scrollsplit"

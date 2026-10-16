#!/usr/bin/env bash
set -euo pipefail

root=$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)
bin=${1:-"$root/target/debug/pristine"}
if [[ "$bin" != /* ]]; then
  bin="$PWD/$bin"
fi
if [[ ! -x "$bin" ]]; then
  printf 'Build the binary first: cargo build\n' >&2
  exit 2
fi

repo=$(mktemp -d "${TMPDIR:-/tmp}/pristine-demo.XXXXXX")
trap 'rm -rf "$repo"' EXIT
git -C "$repo" init -q
git -C "$repo" config user.name "pristine demo"
git -C "$repo" config user.email "demo@example.invalid"
printf '# Demo\n' > "$repo/README.md"
git -C "$repo" add README.md
git -C "$repo" commit -qm baseline
base=$(git -C "$repo" rev-parse HEAD)
mkdir -p "$repo/docs"
printf 'Guide\n' > "$repo/docs/guide.md"
git -C "$repo" add docs/guide.md
git -C "$repo" commit -qm 'docs: add guide'
(cd "$repo" && "$bin" check --base "$base" --title 'docs: add guide')

mkdir -p "$repo/src"
printf 'fn main() {}\n' > "$repo/src/main.rs"
git -C "$repo" add src/main.rs
git -C "$repo" commit -qm 'docs: add source too'
if output=$(cd "$repo" && "$bin" check --base "$base" --title 'docs: add source too' 2>&1); then
  status=0
else
  status=$?
fi
if [[ $status -ne 1 || "$output" != *"src/main.rs"* ]]; then
  printf 'Expected exit 1 and src/main.rs finding; got exit %s:\n%s\n' "$status" "$output" >&2
  exit 1
fi
printf 'Expected advisory verified: src/main.rs is outside documentation.\n'

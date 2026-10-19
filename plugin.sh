#!/bin/sh
here="$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)"
sys="$(uname -s)"
case "$sys" in
MINGW* | MSYS* | CYGWIN*) rel="x86_64-pc-windows-msvc/teams.exe" ;;
Darwin*)
    case "$(uname -m)" in
    arm64) rel="aarch64-apple-darwin/teams" ;;
    *) rel="x86_64-apple-darwin/teams" ;;
    esac ;;
*)
    case "$(uname -m)" in
    aarch64 | arm64) rel="aarch64-unknown-linux-musl/teams" ;;
    *) rel="x86_64-unknown-linux-musl/teams" ;;
    esac ;;
esac
if [ -x "$here/$rel" ]; then exec "$here/$rel" "$@"; fi
for cand in "$here/target/release/teams.exe" "$here/target/release/teams" "$here/target/debug/teams.exe" "$here/target/debug/teams"; do
    if [ -x "$cand" ]; then exec "$cand" "$@"; fi
done
echo "teams binary not found in $here" >&2
exit 1

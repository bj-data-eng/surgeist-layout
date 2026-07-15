#!/usr/bin/env bash
set -euo pipefail

script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
repo_root="$(cd "${script_dir}/.." && pwd)"
cd "${repo_root}"

usage() {
    printf 'Usage: %s <fmt|fmt-check|check|test|clippy|generator-check|generator-test|generator-clippy>\n' "${0##*/}" >&2
}

if [ "$#" -ne 1 ]; then
    usage
    exit 64
fi

task="$1"

case "${task}" in
    fmt)
        exec cargo fmt
        ;;
    fmt-check)
        exec cargo fmt --check
        ;;
    check)
        exec cargo check --locked -p surgeist-layout
        ;;
    test)
        exec cargo test --locked -p surgeist-layout
        ;;
    clippy)
        exec cargo clippy --locked -p surgeist-layout --all-targets -- -F unsafe-code -D warnings
        ;;
    generator-check)
        exec cargo check --locked -p surgeist-layout --all-targets --features layout-golden-generate
        ;;
    generator-test)
        exec cargo test --locked -p surgeist-layout --all-targets --features layout-golden-generate
        ;;
    generator-clippy)
        exec cargo clippy --locked -p surgeist-layout --all-targets --features layout-golden-generate -- -F unsafe-code -D warnings
        ;;
    *)
        printf 'Unknown task: %s\n' "${task}" >&2
        usage
        exit 64
        ;;
esac

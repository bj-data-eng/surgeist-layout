#!/usr/bin/env bash
set -euo pipefail

script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
repo_root="$(cd "${script_dir}/.." && pwd)"
cd "${repo_root}"

usage() {
    printf 'Usage: %s <parity-all|corpus-check|taffy-check>\n' "${0##*/}" >&2
}

if [ "$#" -ne 1 ]; then
    usage
    exit 64
fi

task="$1"

case "${task}" in
    parity-all)
        unset SURGEIST_PARITY_FILTER
        exec cargo test --locked -p surgeist-layout --test layout runs_all_checked_in_browser_parity_xml -- --ignored
        ;;
    corpus-check)
        unset SURGEIST_LAYOUT_BROWSER_PARITY_ROOT
        unset SURGEIST_LAYOUT_GENERATE_FILTER
        unset SURGEIST_BROWSER_PATH
        unset SURGEIST_BROWSER_CACHE
        unset SURGEIST_BROWSER_VERSION
        exec cargo run --locked -p surgeist-layout --features layout-golden-generate --bin surgeist-layout-generate -- check-corpus
        ;;
    taffy-check)
        unset SURGEIST_LAYOUT_BROWSER_PARITY_ROOT
        unset SURGEIST_LAYOUT_GENERATE_FILTER
        unset SURGEIST_BROWSER_PATH
        unset SURGEIST_BROWSER_CACHE
        unset SURGEIST_BROWSER_VERSION
        exec cargo run --locked -p surgeist-layout --features layout-golden-generate --bin surgeist-layout-generate -- check-taffy-corpus
        ;;
    *)
        printf 'Unknown task: %s\n' "${task}" >&2
        usage
        exit 64
        ;;
esac

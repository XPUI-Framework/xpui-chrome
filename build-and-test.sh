#!/usr/bin/env bash

# Everything CI checks in this repository, in one command.
#
#   ./build-and-test.sh          format, lint, test, and every documented snippet
#   ./build-and-test.sh check    the same thing; the name CI uses
#   ./build-and-test.sh fix      format Rust in place first
#
# **Half of what runs is in `bin/gate-common.sh`**, of which every repository
# in the organisation carries a byte-identical copy. This file is what this
# repository configures and the order the checks run in. `xpui-dev` compares
# the nine copies and runs all nine gates.

set -euo pipefail

PROJECT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "${PROJECT_DIR}"

# One crate, at the root. `.` rather than a named directory, and `file_sizes`
# prunes `target` so an extracted package's `src/` is not read as ours.
SOURCE_ROOTS=(.)

# No feature is needed to test this crate: it paints through `xpui`'s traits
# and its tests use the fake host, which `xpui/testing` supplies as a dev
# dependency rather than a feature of this build.
TEST_FEATURES=""

# This crate runs on every device `xpui` does, so it is linted for both
# bare-metal architectures over the same crate list. Neither has atomic
# compare-and-swap; the second is not the stricter run, it is the second
# architecture. The `?` says Cortex-M0+ may skip when the target is not
# installed rather than failing a gate somebody cannot fix without a download.
HOST_WORKSPACE=1
LINT_TARGETS=("riscv32imc-unknown-none-elf" "thumbv6m-none-eabi?")
LINT_TARGET_CRATES=(-p xpui-chrome)

. bin/gate-common.sh

gates() {
  file_sizes
  every_check_runs
  readmes_warn
  prose_is_compiled
  doc_paths
  cpp_snippets_compile
  lint
  test_suite
  doc_tests
  doc_links
}

case "${1:-check}" in
  check)
    rust_format_check
    cpp_format_check
    gates
    printf '\nChecks passed.\n'
    ;;
  fix)
    rust_format_fix
    cpp_format_fix
    gates
    printf '\nFormatted and checked.\n'
    ;;
  *)
    echo "usage: ./build-and-test.sh [check|fix]" >&2
    exit 2
    ;;
esac

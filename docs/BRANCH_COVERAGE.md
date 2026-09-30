# Branch Coverage Map

The current Linux development host is not sufficient to claim cross-platform
100% branch coverage. Use native CI runners for Windows, macOS, FreeBSD, and
OpenBSD paths, and keep platform-specific reports separate from the Linux
report.

## Commands

Run the library unit suite with branch instrumentation:

```sh
cargo +nightly llvm-cov --lib --branch --summary-only
```

Run all targets, including the CLI integration test:

```sh
cargo +nightly llvm-cov --all-targets --branch --summary-only
```

Generate machine-readable details for mapping uncovered outcomes:

```sh
cargo +nightly llvm-cov report --json --output-path target/ai-sandbox-coverage.json
```

## Current Linux Checkpoint

Measured during the coverage work in this checkout:

- Library-only: 307/328 branch outcomes covered (93.60%).
- All targets: 312/362 branch outcomes covered (86.19%).
- All-target tests: 158 library tests and 4 CLI integration tests passed.
- BSD pledge serialization and the Bubblewrap builder/finder seams are covered
  on Linux; a duplicate directory restriction check in the allow-rule pass was
  removed after the earlier deny-rule pass was confirmed to enforce it.

These are a checkpoint, not a 100% result. Coverage can change as tests or
compiler instrumentation change; rerun the commands above before updating this
table. The current all-target report has 50 uncovered branch outcomes, with
several associated with platform-gated code or host/kernel-dependent paths.

## Remaining Platform Work

- Linux: add tests for the unexecuted Bubblewrap capability-probe outcomes and
  remaining builder/policy combinations; the host has `bwrap`, but namespace
  support may be unavailable.
- macOS: exercise Seatbelt proxy parsing edge outcomes and real
  `sandbox-exec` integration on a native runner.
- Windows: cover protected-directory filtering and Windows process/token/ACL
  code on a Windows runner.
- FreeBSD/OpenBSD: test Capsicum/pledge native enforcement APIs on their
  respective runners. Linux can only cover their policy serialization and
  unsupported-platform adapters.

Do not force coverage through fabricated error paths where a native platform
API or kernel capability is required. Prefer injectable adapters for behavior
that can be meaningfully simulated, and native CI for OS-specific enforcement.
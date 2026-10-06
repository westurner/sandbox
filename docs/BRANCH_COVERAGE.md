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

Measured after the current Linux coverage work:

- Library-only: 447/478 branch outcomes covered (93.51%).
- All targets: 452/486 branch outcomes covered (93.00%).
- All-target tests: 180 library tests and 4 CLI integration tests passed.
- Bubblewrap tests cover unsupported Proxy/Localhost policies, read-only root
  validation, workspace/system-root skips, duplicate roots and shared parents,
  sibling roots beneath the current home directory, and non-UTF-8 mount paths.
- Execution-policy tests cover absolute executable aliases, directory bypass
  inputs, invalid path roots, network matching, and wildcard-rule fallback.
- Portable Seatbelt policy generation and Windows allow/deny path planning each
  reach 100% branch coverage in this Linux report; this does not replace native
  enforcement tests.
- OpenBSD pledge now performs the exec handoff with `execpromises` and omits the
  removed `tmppath` promise from its default policy.

These are a checkpoint, not a 100% result. Coverage can change as tests or
compiler instrumentation change; rerun the commands above before updating this
table. The current all-target report has 34 uncovered branch outcomes, with
several associated with platform-gated code, host filesystem layout, or
kernel-dependent paths.

## Remaining Platform Work

- Linux: builder and policy coverage is expanded. Kernel-dependent Bubblewrap
  behavior still depends on host namespace support and must not be inferred
  from argument-builder tests.
- macOS: a native `sandbox-exec` test now verifies that a read-only policy
  denies file creation; it runs on the existing macOS CI runner.
- Windows: native tests exercise restricted-token process creation and verify
  unsupported ACL/network policies fail closed. Filesystem ACL and network
  enforcement are not implemented, so this does not claim those controls are
  enforced.
- FreeBSD/OpenBSD: native tests verify a real child cannot create a file under
  Capsicum/pledge. Dedicated VM jobs were added; they are the runtime checks
  because this Linux host cannot execute either kernel API.

Cross-platform runtime results must be recorded from their native CI jobs; the
Linux checkpoint above only reports locally executed tests.

Do not force coverage through fabricated error paths where a native platform
API or kernel capability is required. Prefer injectable adapters for behavior
that can be meaningfully simulated, and native CI for OS-specific enforcement.
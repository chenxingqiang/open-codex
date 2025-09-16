# icodex-core

This crate implements the business logic for Codex. It is designed to be used by the various Codex UIs written in Rust.

## Dependencies

Note that `icodex-core` makes some assumptions about certain helper utilities being available in the environment. Currently, this

### macOS

Expects `/usr/bin/sandbox-exec` to be present.

### Linux

Expects the binary containing `icodex-core` to run the equivalent of `icodex debug landlock` when `arg0` is `icodex-linux-sandbox`. See the `icodex-arg0` crate for details.

### All Platforms

Expects the binary containing `icodex-core` to simulate the virtual `apply_patch` CLI when `arg1` is `--icodex-run-as-apply-patch`. See the `icodex-arg0` crate for details.

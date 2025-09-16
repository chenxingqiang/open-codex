# icodex-linux-sandbox

This crate is responsible for producing:

- a `icodex-linux-sandbox` standalone executable for Linux that is bundled with the Node.js version of the Codex CLI
- a lib crate that exposes the business logic of the executable as `run_main()` so that
  - the `icodex-exec` CLI can check if its arg0 is `icodex-linux-sandbox` and, if so, execute as if it were `icodex-linux-sandbox`
  - this should also be true of the `icodex` multitool CLI

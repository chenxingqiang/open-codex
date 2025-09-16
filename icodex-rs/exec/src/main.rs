//! Entry-point for the `icodex-exec` binary.
//!
//! When this CLI is invoked normally, it parses the standard `icodex-exec` CLI
//! options and launches the non-interactive Codex agent. However, if it is
//! invoked with arg0 as `icodex-linux-sandbox`, we instead treat the invocation
//! as a request to run the logic for the standalone `icodex-linux-sandbox`
//! executable (i.e., parse any -s args and then run a *sandboxed* command under
//! Landlock + seccomp.
//!
//! This allows us to ship a completely separate set of functionality as part
//! of the `icodex-exec` binary.
use clap::Parser;
use icodex_arg0::arg0_dispatch_or_else;
use icodex_common::CliConfigOverrides;
use icodex_exec::Cli;
use icodex_exec::run_main;

#[derive(Parser, Debug)]
struct TopCli {
    #[clap(flatten)]
    config_overrides: CliConfigOverrides,

    #[clap(flatten)]
    inner: Cli,
}

fn main() -> anyhow::Result<()> {
    arg0_dispatch_or_else(|icodex_linux_sandbox_exe| async move {
        let top_cli = TopCli::parse();
        // Merge root-level overrides into inner CLI struct so downstream logic remains unchanged.
        let mut inner = top_cli.inner;
        inner
            .config_overrides
            .raw_overrides
            .splice(0..0, top_cli.config_overrides.raw_overrides);

        run_main(inner, icodex_linux_sandbox_exe).await?;
        Ok(())
    })
}

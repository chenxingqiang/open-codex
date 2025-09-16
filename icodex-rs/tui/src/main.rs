use clap::Parser;
use icodex_arg0::arg0_dispatch_or_else;
use icodex_common::CliConfigOverrides;
use icodex_tui::Cli;
use icodex_tui::run_main;

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
        let mut inner = top_cli.inner;
        inner
            .config_overrides
            .raw_overrides
            .splice(0..0, top_cli.config_overrides.raw_overrides);
        let usage = run_main(inner, icodex_linux_sandbox_exe).await?;
        if !usage.is_zero() {
            println!("{}", icodex_core::protocol::FinalOutput::from(usage));
        }
        Ok(())
    })
}

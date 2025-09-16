use icodex_arg0::arg0_dispatch_or_else;
use icodex_common::CliConfigOverrides;
use icodex_mcp_server::run_main;

fn main() -> anyhow::Result<()> {
    arg0_dispatch_or_else(|icodex_linux_sandbox_exe| async move {
        run_main(icodex_linux_sandbox_exe, CliConfigOverrides::default()).await?;
        Ok(())
    })
}

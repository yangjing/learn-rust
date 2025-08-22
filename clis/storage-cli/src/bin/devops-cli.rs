use anyhow::{Ok, Result};
use clap::Parser;
use log::debug;
use storage_cli::{cmd::DevopsCmd, conf::DevopsConf, operators::get_operator};

#[tokio::main]
async fn main() -> Result<()> {
  logforth::stdout().apply();

  let cmd = DevopsCmd::parse();
  debug!("args is {:?}", cmd);

  let conf = DevopsConf::from_devops_cmd(&cmd)?;

  if let Some(file_op) = cmd.file_op {
    let op = get_operator(&conf).await?;
    file_op.execute(&op).await?;
  }

  Ok(())
}

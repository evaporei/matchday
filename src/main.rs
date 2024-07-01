use clap::Parser;
use dotenv::dotenv;

use matchday::cmd::Cmd;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenv().ok();

    let cmd = Cmd::parse();

    cmd.run().await?;

    Ok(())
}

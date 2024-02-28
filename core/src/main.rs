use wind_core::application::Application;
use wind_core::cli::CLI;

use anyhow::Result;
use clap::Parser;

#[tokio::main]
async fn main() -> Result<()> {
    let cli = CLI::parse();

    let mut app = Application::new(cli)?;

    app.run().await?;

    Ok(())
}

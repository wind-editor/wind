use wind_core::application::Application;
use wind_core::cli::Arguments;

use anyhow::Result;
use clap::Parser;

#[tokio::main]
async fn main() -> Result<()> {
    let args = Arguments::parse();

    let mut app = Application::new(args)?;

    app.run().await?;

    Ok(())
}

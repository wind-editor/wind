use wind::application::Application;
use wind::cli::Arguments;

use anyhow::Result;
use clap::Parser;

#[tokio::main]
async fn main() -> Result<()> {
    let args = Arguments::parse();

    let mut app = Application::new(args)?;

    app.run().await?;

    Ok(())
}

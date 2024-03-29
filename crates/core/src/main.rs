use wind_core::app::App;
use wind_core::cli::CLI;

use anyhow::Result;
use clap::Parser;

#[tokio::main]
async fn main() -> Result<()> {
    let cli = CLI::parse();

    let mut app = App::new(cli)?;

    app.run().await
}

use wind::cli::Arguments;
use wind_view::editor::Editor;

use anyhow::Result;
use clap::Parser;

fn main() -> Result<()> {
    let args = Arguments::parse();

    let editor = Editor::new();

    editor.run()?;

    Ok(())
}

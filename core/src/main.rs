use wind::cli::Arguments;
use wind_view::editor::Editor;

use anyhow::Result;
use clap::Parser;

fn main() -> Result<()> {
    let _args = Arguments::parse();

    let mut editor = Editor::default();

    editor.run()?;

    Ok(())
}

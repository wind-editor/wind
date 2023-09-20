use wind::cli::Arguments;
use wind_view::editor::Editor;

use anyhow::Result;
use clap::Parser;

fn main() -> Result<()> {
    let args = Arguments::parse();

    let mut editor = Editor::new(args.file);

    editor.run()?;

    Ok(())
}

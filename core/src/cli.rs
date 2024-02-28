use clap::Parser;

use std::path::PathBuf;

#[derive(Parser)]
pub struct CLI {
    pub file_path: Option<PathBuf>,
}

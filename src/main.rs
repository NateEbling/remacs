mod editor;
mod input;
mod buffer;
use crate::editor::Editor;

use std::io;

fn main() -> io::Result<()> {
    let mut ed = Editor::new();
    ed.start()
}

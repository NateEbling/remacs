mod editor;
mod input;
mod buffer;
use crate::editor::Editor;

use std::io;

fn main() -> io::Result<()> {
    let args: Vec<String> = std::env::args().collect();
    let mut editor = if args.len() > 1 {
        let filename = &args[1];
        if std::path::Path::new(filename).exists() {
            let buf = buffer::load_buffer(filename)?;
            Editor::from_file(filename.to_string(), buf)
        } else {
            Editor::new_with_filename(filename.to_string())
        }
    } else {
        Editor::new()
    };
    
    editor.start()
}

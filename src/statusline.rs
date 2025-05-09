use crossterm::{
    terminal::{self},
    execute,
    cursor,
    style::{Attribute, SetAttribute},
};
use std::io::{self, stdout, Write};
use std::path::Path;

use crate::editor::Editor;
use crate::editor::VERSION;

pub fn create_statusline(editor: &mut Editor) -> io::Result<()> {
    let mut stdout = stdout();

    let (term_width, term_height) = terminal::size()?;
    let term_height = term_height as usize;

    for (i, _line) in editor.buf.iter().enumerate() {
        if i >= term_height - 2 { break; }
    }

    let mod_marker = if editor.modified { "-*" } else { "--" };
    
    let filename = if editor.filename.is_empty() {
        "[No name]"
    } else {
        &editor.filename
    };

    // Make this detected later
    let encoding = "utf-8";

    let rel_path = if editor.filename.is_empty() {
        "".to_string()
    } else {
        let path = Path::new(&editor.filename);
        match std::env::current_dir() {
            Ok(current_dir) => {
                match path.strip_prefix(&current_dir) {
                    Ok(rel) => rel.to_string_lossy().to_string(),
                    Err(_) => path.to_string_lossy().to_string(),
                }
            }
            Err(_) => path.to_string_lossy().to_string(),
        }
    };

    let statusline = format!(
        "{} Remacs {}: {} ({}) {} ",
        mod_marker,
        VERSION,
        filename,
        encoding,
        rel_path,
    );

    let statusline = if statusline.len() > term_width as usize {
        statusline[..term_width as usize].to_string()
    } else {
        let dash_count = term_width as usize - statusline.len();
        format!("{}{}", statusline, "-".repeat(dash_count))
    };

    execute!(
        stdout, 
        cursor::MoveTo(0, (term_height - 2) as u16),
        SetAttribute(Attribute::Reverse),
    )?;
    write!(stdout, "{}", statusline)?;
    execute!(stdout, SetAttribute(Attribute::Reset))?;

    return Ok(());
}

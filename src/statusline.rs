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

    let viewport_lines = term_height - 2;
    let total_lines = editor.buf.len();
    let last_visible_line = editor.row_offset + viewport_lines;

    let pos_marker = if total_lines <= viewport_lines {
        "All"
    } else if editor.row_offset == 0 {
        "Top"
    } else if last_visible_line >= total_lines {
        "Bot"
    } else {
        let percent = ((editor.row_offset as f64 / (total_lines as f64 - viewport_lines as f64)) * 100.0).round() as usize;
        &format!("{percent}%")
    };

    let left = format!(
        "{} Remacs {}: {} ({}) {} ",
        mod_marker,
        VERSION,
        filename,
        encoding,
        rel_path,
    );

    let right = format!(" {} --", pos_marker);
    let dash_count = term_width.saturating_sub((left.len() + right.len()).try_into().unwrap());
    let filler = "-".repeat(dash_count.into());

    let statusline = format!("{}{}{}", left, filler, right);

    execute!(
        stdout, 
        cursor::MoveTo(0, (term_height - 2) as u16),
        SetAttribute(Attribute::Reverse),
    )?;
    write!(stdout, "{}", statusline)?;
    execute!(stdout, SetAttribute(Attribute::NoReverse))?;

    return Ok(());
}

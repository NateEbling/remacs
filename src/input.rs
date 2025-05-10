use crossterm::{
    event::{self, Event, KeyCode, KeyModifiers, KeyEvent},
};

use crate::editor::Editor;
use crate::editor::Command;
use crate::editor::EditorMode;
use crate::buffer::save_buffer;

macro_rules! ctrl {
    ($ch:expr, $key_event:expr) => {
        $key_event.code == KeyCode::Char($ch) && $key_event.modifiers.contains(KeyModifiers::CONTROL)
    };
}

macro_rules! alt {
    ($ch:expr, $key_event:expr) => {
        $key_event.code == KeyCode::Char($ch) && $key_event.modifiers.contains(KeyModifiers::ALT)
    };
}

macro_rules! alt_ctrl {
    ($ch:expr, $key_event:expr) => {
        $key_event.code == KeyCode::Char($ch)
            && $key_event.modifiers.contains(KeyModifiers::ALT)
            && $key_event.modifiers.contains(KeyModifiers::CONTROL)
    };
}

pub fn get_inputs(editor: &mut Editor) -> Result<bool, std::io::Error> {
    let mut check = false;
    if let Event::Key(key_event) = event::read()? {
        match editor.mode {
            EditorMode::SaveFile => {
                check = check_keys_s(editor, key_event);
            }
            EditorMode::Normal => {
                check = check_keys_n(editor, key_event);
            }
            EditorMode::PromptQuit => {
                match key_event.code {
                    KeyCode::Char('y') => return Ok(true),
                    KeyCode::Char('n') | KeyCode::Esc => {
                        editor.mode = EditorMode::Normal;
                    }
                    _ => {}
                }
            }
        }
    }
    Ok(check)
}

fn check_keys_s(editor: &mut Editor, key_event: KeyEvent) -> bool {
    match key_event.code {
        KeyCode::Enter => {
            if editor.filename.is_empty() {
                // Nothing has been entered, so continue to wait for user to 
                // enter a filename.
            } else {
                match save_buffer(&editor.buf, &editor.filename) {
                    Ok(0) => {

                    }
                    Ok(n) => {
                        editor.modified = false;
                        editor.message = Some(format!("(Wrote {} line{})", n, if n == 1 { "" } else { "s" }));
                    }
                    Err(e) => {
                        eprintln!("Error saving file: {e}");
                    }
                }
                editor.mode = EditorMode::Normal;
            }
        }

        KeyCode::Esc => {
            editor.filename.clear();
            editor.mode = EditorMode::Normal;
        }

        KeyCode::Backspace => {
            editor.filename.pop();
        }

        KeyCode::Char(c) => {
            editor.filename.push(c);
        }
        
        _ => {}
    }
    false
}

fn check_keys_n(editor: &mut Editor, key_event: KeyEvent) -> bool {
    match key_event.code {
        _ if ctrl!('x', key_event) => {
            editor.cmd = Command::CtrlX;             
        }

        // Kill to end of line
        _ if ctrl!('k', key_event) => {
            if let Some(line) = editor.buf.get_mut(editor.cur_y) {
                line.truncate(editor.cur_x);
                editor.modified = true;
            }
        }

        // Delete previous character
        _ if ctrl!('h', key_event) => {
            if editor.cur_x > 0 {
                editor.buf[editor.cur_y].remove(editor.cur_x - 1);
                editor.cur_x -= 1;
                editor.modified = true;
            } else if editor.cur_y > 0 {
                let prev_line_len = editor.buf[editor.cur_y - 1].len();
                let current_line = editor.buf.remove(editor.cur_y);
                editor.cur_y -= 1;
                editor.cur_x = prev_line_len;
                editor.buf[editor.cur_y].push_str(&current_line);
                editor.modified = true;
            }
        }

        // Delete next character
        _ if ctrl!('d', key_event) => {
            if editor.cur_x < editor.buf[editor.cur_y].len() {
                editor.buf[editor.cur_y].remove(editor.cur_x);
                editor.modified = true;
            } else if editor.cur_y + 1 < editor.buf.len() {
                let next_line = editor.buf.remove(editor.cur_y + 1);
                editor.buf[editor.cur_y].push_str(&next_line);
                editor.modified = true;
            }
        }

        _ if alt!('d', key_event) => {
            if let Some(line) = editor.buf.get_mut(editor.cur_y) {
                let rest = &line[editor.cur_x..];
                if let Some(pos) = rest.find(char::is_whitespace) {
                    let mut end = editor.cur_x + pos;
                    // Skip over following whitespace
                    while end < line.len() && line.as_bytes()[end].is_ascii_whitespace() {
                        end += 1;
                    }
                    line.drain(editor.cur_x..end);
                } else {
                    line.truncate(editor.cur_x);
                }
                editor.modified = true;
            }
        }

        // Delete previous word (Alt + Ctrl + h)
        _ if alt_ctrl!('h', key_event) => {
            if let Some(line) = editor.buf.get_mut(editor.cur_y) {
                if editor.cur_x == 0 && editor.cur_y > 0 {
                    // Merge with previous line
                    let current_line = editor.buf.remove(editor.cur_y);
                    editor.cur_y -= 1;
                    editor.cur_x = editor.buf[editor.cur_y].len();
                    editor.buf[editor.cur_y].push_str(&current_line);
                    editor.modified = true;
                } else if editor.cur_x > 0 {
                    let before = &line[..editor.cur_x];
                    let idx = before.trim_end_matches(char::is_whitespace).rfind(char::is_whitespace)
                        .map(|i| i + 1)
                        .unwrap_or(0);
                    line.drain(idx..editor.cur_x);
                    editor.cur_x = idx;
                    editor.modified = true;
                }
            }
        }

        // Save file
        KeyCode::Char('d') => {
            if editor.cmd == Command::CtrlX {
                editor.cmd = Command::None;
                if editor.filename_given && !editor.filename.is_empty() {
                    match save_buffer(&editor.buf, &editor.filename) {
                        Ok(_) => {
                            editor.modified = false;
                            let count = editor.buf.len();
                            editor.message = Some(format!("(Wrote {} lines)", count));
                        } 
                        Err(e) => {
                            eprintln!("Error saving file {e}");
                        }
                    }
                } else {
                    editor.mode = EditorMode::SaveFile;
                }
            } else {
                if editor.cur_y >= editor.buf.len() {
                    editor.buf.push(String::new());
                }
                editor.buf[editor.cur_y].insert(editor.cur_x, 'd');
                editor.cur_x += 1;
            }
        }
        
        // Quit
        KeyCode::Char('c') => {
            if editor.cmd == Command::CtrlX {
                if !editor.modified {
                    return true;
                } else {
                    editor.mode = EditorMode::PromptQuit;
                }
            } else {
                if editor.cur_y >= editor.buf.len() {
                    editor.buf.push(String::new());
                }
                editor.buf[editor.cur_y].insert(editor.cur_x, 'c');
                editor.cur_x += 1;
            }
        }

        // Write file
        KeyCode::Char('w') => {
            if editor.cmd == Command::CtrlX {
                editor.mode = EditorMode::SaveFile;
                editor.cmd = Command::None;
            } else {
                if editor.cur_y >= editor.buf.len() {
                    editor.buf.push(String::new());
                }
                editor.buf[editor.cur_y].insert(editor.cur_x, 'w');
                editor.cur_x += 1;
            }
        }

        KeyCode::Char(c) => {
            if editor.cmd == Command::CtrlX {
                editor.message = Some(format!("(Key not bound)"));
                editor.cmd = Command::None;
            } else {
                if editor.cur_y >= editor.buf.len() {
                    editor.buf.push(String::new());
                }
                editor.buf[editor.cur_y].insert(editor.cur_x, c);
                editor.cur_x += 1;
                editor.modified = true;
            }
        }

        KeyCode::Enter => {
            let current_line = editor.buf.get_mut(editor.cur_y).unwrap();
            let new_line = current_line.split_off(editor.cur_x);
            editor.buf.insert(editor.cur_y + 1, new_line);
            editor.cur_y += 1;
            editor.cur_x = 0;
            editor.modified = true;
        }

        KeyCode::Backspace => {
            if editor.cur_x > 0 {
                editor.buf[editor.cur_y].remove(editor.cur_x - 1);
                editor.cur_x -= 1;
                editor.modified = true;
            } else if editor.cur_y > 0 {
                let prev_line_len = editor.buf[editor.cur_y - 1].len();
                let current_line = editor.buf.remove(editor.cur_y);
                editor.cur_y -= 1;
                editor.cur_x = prev_line_len;
                editor.buf[editor.cur_y].push_str(&current_line);
                editor.modified = true;
            }
        }

        KeyCode::Left => {
            if editor.cur_x > 0 {
                editor.cur_x -= 1;
            } else if editor.cur_y > 0 {
                editor.cur_y -= 1;
                editor.cur_x = editor.buf[editor.cur_y].len();
            }
        }

        KeyCode::Right => {
            if editor.cur_x < editor.buf[editor.cur_y].len() {
                editor.cur_x += 1;
            } else if editor.cur_y + 1 < editor.buf.len() {
                editor.cur_y += 1;
                editor.cur_x = 0;
            }
        }

        KeyCode::Up => {
            if editor.cur_y > 0 {
                editor.cur_y -= 1;
                editor.cur_x = editor.cur_x.min(editor.buf[editor.cur_y].len());
            }
        }
        
        KeyCode::Down => {
            if editor.cur_y + 1 < editor.buf.len() {
                editor.cur_y += 1;
                editor.cur_x = editor.cur_x.min(editor.buf[editor.cur_y].len());
            }
        }
        _ => {
            editor.cmd = Command::None;
        }
    }
    false
}

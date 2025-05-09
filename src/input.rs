use crossterm::{
    event::{self, Event, KeyCode, KeyModifiers, KeyEvent},
};

use crate::editor::Editor;
use crate::editor::Command;
use crate::editor::EditorMode;
use crate::buffer::save_buffer;

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
                    Ok(_) => {
                        editor.modified = false;
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
        KeyCode::Char('x') if key_event.modifiers.contains(KeyModifiers::CONTROL) => {
            editor.cmd = Command::CtrlX;             
        }
        
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
            if editor.cur_y >= editor.buf.len() {
                editor.buf.push(String::new());
            }
            editor.buf[editor.cur_y].insert(editor.cur_x, c);
            editor.cur_x += 1;
            editor.modified = true;
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

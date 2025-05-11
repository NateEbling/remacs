use crossterm::{
    event::{self, Event, KeyCode, KeyModifiers, KeyEvent},
};

use crate::editor::Editor;
use crate::editor::Command;
use crate::editor::EditorMode;
use crate::buffer::save_buffer;
use crate::ctrl;
use crate::alt;
use crate::alt_ctrl;

pub fn get_inputs(editor: &mut Editor) -> Result<bool, std::io::Error> {
    let mut check = false;
    if let Event::Key(key_event) = event::read()? {
        match editor.mode {
            EditorMode::SaveFile => {
                check = check_keys_save(editor, key_event);
            }
            EditorMode::Normal => {
                check = check_keys_normal(editor, key_event);
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

fn check_keys_save(editor: &mut Editor, key_event: KeyEvent) -> bool {
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

fn check_keys_normal(editor: &mut Editor, key_event: KeyEvent) -> bool {
    editor.message = None;
    match key_event.code {
        _ if ctrl!('x', key_event) => {
            editor.cmd = Command::CtrlX;             
        }

        // New line
        _ if ctrl!('m', key_event) => {

        }

        _ if ctrl!('a', key_event) => {
            editor.move_to_line_start(); 
        }

        _ if ctrl!('e', key_event) => {
            editor.move_to_line_end();
        }

        _ if ctrl!('n', key_event) => {
            editor.move_next_line();
        }

        _ if ctrl!('p', key_event) => {
            editor.move_prev_line();
        }

        _ if ctrl!('z', key_event) => {
            editor.move_prev_page();
        }

        _ if ctrl!('v', key_event) => {
            editor.move_next_page();
        }

        _ if ctrl!('k', key_event) => {
            editor.kill_to_eol();
        }

        _ if ctrl!('h', key_event) => {
            editor.del_prev_char();
        }

        _ if ctrl!('d', key_event) => {
            editor.del_next_char();
        }

        _ if alt!('d', key_event) => {
            editor.del_next_word();
        }

        _ if alt_ctrl!('h', key_event) => {
            editor.del_prev_word();
        }

        // Quick exit (save + quit)
        _ if alt!('z', key_event) => {
            return editor.quick_exit();
        }

        // Save file
        KeyCode::Char('d') => {
            if editor.cmd == Command::CtrlX {
                editor.save_file();
            } else {
                editor.insert_char('d');
            }
        }
        
        // Quit
        KeyCode::Char('c') => {
            if editor.cmd == Command::CtrlX {
                return editor.quit();
            } else {
                editor.insert_char('c');
            }
        }

        // Write file
        KeyCode::Char('w') => {
            if editor.cmd == Command::CtrlX {
                editor.write_buffer();
            } else {
                editor.insert_char('w');
            }
        }

        // Incremental search
        KeyCode::Char('s') => {
            if editor.cmd == Command::CtrlX {
                
            } else {
                editor.insert_char('s');
            }
        }

        KeyCode::Tab => {
            editor.insert_tab();
        }

        KeyCode::Char(c) => {
            if editor.cmd == Command::CtrlX {
                editor.message = Some(format!("(Key not bound)"));
                editor.cmd = Command::None;
            } else {
                editor.insert_char(c);
            }
        }

        KeyCode::Enter => {
            let current_line = editor.buf.get_mut(editor.cur_y).unwrap();
            let new_line = current_line.split_off(editor.cur_x);
            editor.buf.insert(editor.cur_y + 1, new_line);
            editor.cur_y += 1;
            editor.cur_x = 0;
            editor.update_modified();
        }

        KeyCode::Backspace => {
            if editor.cur_x > 0 {
                editor.buf[editor.cur_y].remove(editor.cur_x - 1);
                editor.cur_x -= 1;
                editor.update_modified();
            } else if editor.cur_y > 0 {
                let prev_line_len = editor.buf[editor.cur_y - 1].len();
                let current_line = editor.buf.remove(editor.cur_y);
                editor.cur_y -= 1;
                editor.cur_x = prev_line_len;
                editor.buf[editor.cur_y].push_str(&current_line);
                editor.update_modified();
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

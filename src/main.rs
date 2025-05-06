use crossterm::{
    cursor,
    event::{self, Event, KeyCode, KeyModifiers},
    execute, 
    terminal::{self, ClearType},
};
use std::io::{self, stdout, Write, BufWriter};
use std::fs::File;

#[derive(Debug, PartialEq)]
enum PendingCommand {
    None,
    CtrlX,
}

#[derive(Debug, PartialEq)]
enum EditorMode {
    Normal,
    Input,
}

fn main() -> io::Result<()> {
    let mut stdout = stdout();

    let mut mode = EditorMode::Normal;

    let mut pending_command = PendingCommand::None;

    let mut filename_input = String::new();

    terminal::enable_raw_mode()?;
    execute!(stdout, terminal::EnterAlternateScreen, cursor::Show)?;

    let mut buffer: Vec<String> = vec![String::new()];

    let mut cursor_x = 0;
    let mut cursor_y = 0;

    loop {
        execute!(
            stdout, 
            terminal::Clear(ClearType::All), 
            cursor::MoveTo(0, 0)
        )?;

        for line in &buffer {
            write!(stdout, "{}\r\n", line)?;
        }

        if mode == EditorMode::Input {
            write!(stdout, "\r\nWrite file: {filename_input}")?;
        }

        execute!(stdout, cursor::MoveTo(cursor_x as u16, cursor_y as u16))?;
        stdout.flush()?;

        if let Event::Key(key_event) = event::read()? {
            match mode {
                EditorMode::Input => {
                    match key_event.code {
                        KeyCode::Enter => {
                            if !filename_input.is_empty() {
                                if let Err(e) = save_buffer(&buffer, &filename_input) {
                                    eprintln!("Error saving file {e}");
                                }
                            }
                            filename_input.clear();
                            mode = EditorMode::Normal;
                        }
                        KeyCode::Esc => {
                            filename_input.clear();
                            mode = EditorMode::Normal;
                        }
                        KeyCode::Backspace => {
                            filename_input.pop();
                        }
                        KeyCode::Char(c) => {
                            filename_input.push(c);
                        }
                        _ => {}
                    }
                }
                EditorMode::Normal => {
                    match key_event.code {
                        KeyCode::Char('x') if key_event.modifiers.contains(KeyModifiers::CONTROL) => {
                            pending_command = PendingCommand::CtrlX;
                        }
                
                        KeyCode::Char('c') => {
                            if pending_command == PendingCommand::CtrlX {
                                break;
                            } else {
                                if cursor_y >= buffer.len() {
                                    buffer.push(String::new());
                                }
                                buffer[cursor_y].insert(cursor_x, 'c');
                                cursor_x += 1;
                            }
                        }
                    
                        KeyCode::Char('w') => {
                            if pending_command == PendingCommand::CtrlX {
                                mode = EditorMode::Input;
                                pending_command = PendingCommand::None;
                            } else {
                                if cursor_y >= buffer.len() {
                                    buffer.push(String::new());
                                }
                                buffer[cursor_y].insert(cursor_x, 'w');
                                cursor_x += 1;
                            }
                        }

                        KeyCode::Char(c) => {
                            if cursor_y >= buffer.len() {
                                buffer.push(String::new());
                            }
                            buffer[cursor_y].insert(cursor_x, c);
                            cursor_x += 1;
                        }

                        KeyCode::Enter => {
                            let current_line = buffer.get_mut(cursor_y).unwrap();
                            let new_line = current_line.split_off(cursor_x);
                            buffer.insert(cursor_y + 1, new_line);
                            cursor_y += 1;
                            cursor_x = 0;
                        }

                        KeyCode::Backspace => {
                            if cursor_x > 0 {
                                buffer[cursor_y].remove(cursor_x - 1);
                                cursor_x -= 1;
                            } else if cursor_y > 0 {
                                let prev_line_len = buffer[cursor_y - 1].len();
                                let current_line = buffer.remove(cursor_y);
                                cursor_y -= 1;
                                cursor_x = prev_line_len;
                                buffer[cursor_y].push_str(&current_line);
                            }
                        }

                        KeyCode::Left => {
                            if cursor_x > 0 {
                                cursor_x -= 1;
                            } else if cursor_y > 0 {
                                cursor_y -= 1;
                                cursor_x = buffer[cursor_y].len();
                            }
                        }

                        KeyCode::Right => {
                            if cursor_x < buffer[cursor_y].len() {
                                cursor_x += 1;
                            } else if cursor_y + 1 < buffer.len() {
                                cursor_y += 1;
                                cursor_x = 0;
                            }
                        }

                        KeyCode::Up => {
                            if cursor_y > 0 {
                                cursor_y -= 1;
                                cursor_x = cursor_x.min(buffer[cursor_y].len());
                            }
                        }
                        
                        KeyCode::Down => {
                            if cursor_y + 1 < buffer.len() {
                                cursor_y += 1;
                                cursor_x = cursor_x.min(buffer[cursor_y].len());
                            }
                        }
                        _ => {
                            pending_command = PendingCommand::None;
                        }
                    }
                }
            }
        }
    }

    execute!(stdout, terminal::LeaveAlternateScreen, cursor::Show)?;
    terminal::disable_raw_mode()
}

fn save_buffer(buffer: &[String], filename: &str) -> io::Result<()> {
    let file = File::create(filename)?;
    let mut writer = BufWriter::new(file);

    for line in buffer {
        writeln!(writer, "{line}")?;
    }

    writer.flush()
}

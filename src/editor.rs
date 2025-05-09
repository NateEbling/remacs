use crossterm::{
    cursor,
    execute, 
    terminal::{self, ClearType},
};

use std::io::{self, stdout, Write};

use crate::input::get_inputs;
use crate::statusline::create_statusline;

pub const VERSION: &str = "0.1.0";

#[derive(Debug, PartialEq)]
pub enum Command {
    None,
    CtrlX,
}

#[derive(Debug, PartialEq)]
pub enum EditorMode {
    Normal,
    SaveFile,
    PromptQuit,
}

pub struct Editor {
    pub mode: EditorMode,
    pub cmd: Command,
    pub filename: String,
    pub cur_x: usize,
    pub cur_y: usize,
    pub buf: Vec<String>,
    pub modified: bool,
    pub row_offset: usize,
    pub col_offset: usize,
}

impl Editor {
    pub fn new() -> Self {
        Self {
            mode: EditorMode::Normal,
            cmd: Command::None,
            filename: String::new(),
            cur_x: 0,
            cur_y: 0,
            buf: vec![String::new()],
            modified: false,
            row_offset: 0,
            col_offset: 0,
        }
    }
    pub fn from_file(filename: String, buf: Vec<String>) -> Self {
        Self {
            mode: EditorMode::Normal,
            cmd: Command::None,
            filename,
            cur_x: 0,
            cur_y: 0,
            buf,
            modified: false,
            row_offset: 0,
            col_offset: 0,
        }
    }

    pub fn new_with_filename(filename: String) -> Self {
        Self {
            mode: EditorMode::Normal,
            cmd: Command::None,
            filename,
            cur_x: 0,
            cur_y: 0,
            buf: vec![String::new()],
            modified: false,
            row_offset: 0,
            col_offset: 0,
        }
    }

    pub fn start(&mut self) -> io::Result<()> {
        let mut stdout = stdout();

        let (_term_width, term_height) = terminal::size()?;
        let term_height = term_height as usize;
        let max_lines = (term_height - 2) as usize;

        terminal::enable_raw_mode()?;
        execute!(stdout, terminal::EnterAlternateScreen, cursor::Show)?;

        loop {
            execute!(
                stdout,
                terminal::Clear(ClearType::FromCursorDown),
                cursor::MoveTo(0, 0)
            )?;

            let screen_lines = (term_height - 2) as usize;

            if self.cur_y < self.row_offset {
                self.row_offset = self.cur_y;
            } else if self.cur_y >= self.row_offset + screen_lines {
                self.row_offset = self.cur_y - screen_lines + 1;
            }

            for i in 0..max_lines {
                let buff_line = self.row_offset + i;
                if buff_line >= self.buf.len() {
                    write!(stdout, "~\r\n")?;
                } else {
                    write!(stdout, "{}\r\n", self.buf[buff_line])?;
                }
            }

            if self.cur_y >= self.row_offset + screen_lines {
                self.row_offset = self.cur_y - screen_lines + 1;
            }

            let _ = create_statusline(self);

            execute!(stdout, cursor::MoveTo(0, (term_height - 1) as u16))?;

            let max_y = (term_height - 3) as usize;

            let mut cur_x = self.cur_x.saturating_sub(self.col_offset) as u16;
            let screen_y = self.cur_y.saturating_sub(self.row_offset);

            let mut cur_y = if screen_y >= screen_lines {
                (screen_lines - 1) as u16
            } else {
                screen_y as u16
            };

            if screen_y >= screen_lines {
                cur_y = (screen_lines - 1) as u16;
            } else {
                cur_y = screen_y as u16;
            }

            let prompt_y = (term_height - 1) as u16;
            match self.mode {
                EditorMode::SaveFile => {
                    let tmp = "Write file: ".to_string();
                    write!(stdout, "{}{}", tmp, self.filename)?;
                    cur_x = (tmp.len() + self.filename.len()) as u16;
                    cur_y = prompt_y;
                }
                EditorMode::PromptQuit => {
                    let tmp = "Modified buffers exist. Leave anyway (y/n)? ".to_string();
                    write!(stdout, "{tmp}")?;
                    cur_x = tmp.len() as u16;
                    cur_y = prompt_y;
                }
                _ => {
                    cur_x = self.cur_x.saturating_sub(self.col_offset) as u16;
                    cur_y = self.cur_y.saturating_sub(self.row_offset) as u16;
                }
            }

            execute!(stdout, cursor::MoveTo(cur_x, cur_y))?;
            stdout.flush()?;

            match get_inputs(self) {
                Ok(true) => break,
                Ok(false) => continue,
                Err(e) => {
                    eprintln!("Error: {e}");
                    break;
                }
            }
        }

        execute!(stdout, terminal::LeaveAlternateScreen, cursor::Show)?;
        terminal::disable_raw_mode()
    }
}

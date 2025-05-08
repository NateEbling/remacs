use crossterm::{
    cursor,
    execute, 
    terminal::{self, ClearType},
    style::{Attribute, SetAttribute},
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
        }
    }

    pub fn start(&mut self) -> io::Result<()> {
        let mut stdout = stdout();

        let (term_width, term_height) = terminal::size()?;
        let term_height = term_height as usize;

        terminal::enable_raw_mode()?;
        execute!(stdout, terminal::EnterAlternateScreen, cursor::Show)?;

        loop {
            execute!(
                stdout,
                terminal::Clear(ClearType::All),
                cursor::MoveTo(0, 0)
            )?;

            create_statusline(self);

            execute!(stdout, cursor::MoveTo(0, (term_height - 1) as u16))?;

            let (mut cur_x, mut cur_y) = (self.cur_x as u16, self.cur_y as u16);

            match self.mode {
                EditorMode::SaveFile => {
                    let tmp = "Write file: ".to_string();
                    write!(stdout, "{}{}", tmp, self.filename)?;
                    cur_x = (tmp.len() + self.filename.len()) as u16;
                    cur_y = (term_height - 1) as u16;
                }
                EditorMode::PromptQuit => {
                    let tmp = "Modified buffers exist. Leave anyway (y/n)? ".to_string();
                    write!(stdout, "{tmp}")?;
                    cur_x = tmp.len() as u16;
                    cur_y = (term_height - 1) as u16;
                }
                _ => {}
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

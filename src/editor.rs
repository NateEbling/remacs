use crossterm::{
    cursor,
    execute, 
    terminal::{self, ClearType},
};

use std::io::{self, stdout, Write};

use crate::input::get_inputs;

#[derive(Debug, PartialEq)]
pub enum Command {
    None,
    CtrlX,
}

#[derive(Debug, PartialEq)]
pub enum EditorMode {
    Normal,
    SaveFile,
}

pub struct Editor {
    pub mode: EditorMode,
    pub cmd: Command,
    pub filename: String,
    pub cur_x: usize,
    pub cur_y: usize,
    pub buf: Vec<String>,
}

impl Editor {
    pub fn new() -> Self {
        let buf: Vec<String> = vec![String::new()];

        let mode = EditorMode::Normal;

        let cmd = Command::None;

        let filename = String::new();
    
        Self {
            mode,
            cmd,
            filename,
            cur_x: 0,
            cur_y: 0,
            buf,
        }
    }

    pub fn start(&mut self) -> io::Result<()> {
        let mut stdout = stdout();
        terminal::enable_raw_mode()?;
        execute!(stdout, terminal::EnterAlternateScreen, cursor::Show)?;

        loop {
            execute!(
                stdout,
                terminal::Clear(ClearType::All),
                cursor::MoveTo(0, 0)
            )?;

            for line in &self.buf {
                write!(stdout, "{line}\r\n")?;
            }
            
            if self.mode == EditorMode::SaveFile {
                write!(stdout, "\r\nWrite file: {}", self.filename)?;
            }

            execute!(stdout, cursor::MoveTo(self.cur_x as u16, self.cur_y as u16))?;
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

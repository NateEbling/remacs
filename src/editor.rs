use crossterm::{
    cursor,
    terminal::{self, ClearType},
    queue,
};

use std::io::{self, stdout, Write};

use crate::input::get_inputs;
use crate::statusline::create_statusline;

pub const VERSION: &str = "0.2.0";

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
    pub last_frame: Vec<String>,
    pub filename_given: bool,
    pub message: Option<String>,
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
            last_frame: vec![String::new()],
            filename_given: false,
            message: None,
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
            last_frame: vec![String::new()],
            filename_given: true,
            message: None,
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
            last_frame: vec![String::new()],
            filename_given: true,
            message: None,
        }
    }

    pub fn start(&mut self) -> io::Result<()> {
        let mut stdout = stdout();

        terminal::enable_raw_mode()?;
        queue!(
            stdout, 
            terminal::EnterAlternateScreen, 
            cursor::EnableBlinking,
            cursor::Show,
        )?;

        loop {
            self.render(&mut stdout)?;

            match get_inputs(self) {
                Ok(true) => break,
                Ok(false) => continue,
                Err(e) => {
                    eprintln!("Error: {e}");
                    break;
                }
            }
        }

        queue!(stdout, terminal::LeaveAlternateScreen, cursor::Show)?;
        terminal::disable_raw_mode()
    }

    fn render(&mut self, stdout: &mut io::Stdout) -> io::Result<()> {
        let (_term_width, term_height) = terminal::size()?;
        let term_height = term_height as usize;
        let max_lines = (term_height - 2) as usize;

        queue!(stdout, cursor::MoveTo(0, 0))?;
        
        let screen_lines = (term_height - 2) as usize;

        if self.cur_y < self.row_offset {
            self.row_offset = self.cur_y;
        } else if self.cur_y >= self.row_offset + screen_lines {
            self.row_offset = self.cur_y - screen_lines + 1;
        }

        self.last_frame.truncate(max_lines);

        for i in 0..max_lines {
            let buff_line = self.row_offset + i;
            let screen_y = i as u16;

            let new_line = if buff_line >= self.buf.len() {
                "~".to_string()
            } else {
                self.buf[buff_line].clone()
            };

            if self.last_frame.get(i) != Some(&new_line) {
                queue!(
                    stdout,
                    cursor::MoveTo(0, screen_y),
                    terminal::Clear(ClearType::CurrentLine),
                )?;
                write!(stdout, "{new_line}")?;

                if i < self.last_frame.len() {
                    self.last_frame[i] = new_line.clone();
                } else {
                    self.last_frame.push(new_line.clone());
                }
            }
        }

        if self.cur_y >= self.row_offset + screen_lines {
            self.row_offset = self.cur_y - screen_lines + 1;
        }

        let _ = create_statusline(self);

        queue!(stdout, cursor::MoveTo(0, (term_height - 1) as u16))?;

        let cur_x;
        let cur_y;

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

        if self.mode == EditorMode::Normal {
            if let Some(ref msg) = self.message {
                queue!(stdout, cursor::MoveTo(0, prompt_y), terminal::Clear(ClearType::CurrentLine))?;
                queue!(stdout, crossterm::style::Print(msg))?;
            }
        }

        queue!(stdout, cursor::MoveTo(cur_x, cur_y))?;
        stdout.flush()?;
        self.message = None;
        Ok(())
    }
} 

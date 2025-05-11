use crossterm::{
    cursor,
    terminal::{self, ClearType},
    queue,
};

use std::io::{self, stdout, Write};

use crate::input::get_inputs;
use crate::statusline::create_statusline;
use crate::buffer::save_buffer;

pub const VERSION: &str = "0.2.0";
pub const TAB_WIDTH: usize = 4;

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
    pub original_buf: Vec<String>,
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
            original_buf: vec![String::new()],
        }
    }

    pub fn from_file(filename: String, buf: Vec<String>) -> Self {
        let original_buf = buf.clone();
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
            original_buf,
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
            original_buf: vec![String::new()],
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

    pub fn move_to_line_start(&mut self) {
        self.cur_x = 0;
    }

    pub fn move_to_line_end(&mut self) {
        if let Some(line) = self.buf.get(self.cur_y) {
            self.cur_x = line.len();
        }
    }

    pub fn move_next_line(&mut self) {
        if self.cur_y + 1 < self.buf.len() {
            self.cur_y += 1;
            let len = self.buf[self.cur_y].len();
            self.cur_x = self.cur_x.min(len);
        }
    }

    pub fn move_prev_line(&mut self) {
        if self.cur_y > 0 {
            self.cur_y -= 1;
            let len = self.buf[self.cur_y].len();
            self.cur_x = self.cur_x.min(len);
        }
    }

    pub fn move_next_page(&mut self) {
        let (_, term_height) = terminal::size().unwrap_or((0, 0));
        let lines_per_page = term_height.saturating_sub(2) as usize;

        if self.cur_y + 1 < self.buf.len() {
            self.cur_y = (self.cur_y + lines_per_page).min(self.buf.len() - 1);
            let len = self.buf[self.cur_y].len();
            self.cur_x = self.cur_x.min(len);
        }
    }

    pub fn move_prev_page(&mut self) {
        let (_, term_height) = terminal::size().unwrap_or((0, 0));              
        let lines_per_page = term_height.saturating_sub(2) as usize;            
                                                                                
        if self.cur_y > 0 {                                    
            self.cur_y = self.cur_y.saturating_sub(lines_per_page);
            let len = self.buf[self.cur_y].len();                               
            self.cur_x = self.cur_x.min(len);                                   
        }   
    }

    pub fn kill_to_eol(&mut self) {
        if let Some(line) = self.buf.get_mut(self.cur_y) {
            line.truncate(self.cur_x);
            self.update_modified();
        }
    }

    pub fn del_prev_char(&mut self) {
        if self.cur_x > 0 {
            self.buf[self.cur_y].remove(self.cur_x - 1);
            self.cur_x -= 1;
            self.update_modified();
        } else if self.cur_y > 0 {
            let prev_line_len = self.buf[self.cur_y - 1].len();
            let current_line = self.buf.remove(self.cur_y);
            self.cur_y -= 1;
            self.cur_x = prev_line_len;
            self.buf[self.cur_y].push_str(&current_line);
            self.update_modified();
        }
    }

    pub fn del_next_char(&mut self) {
        if self.cur_x < self.buf[self.cur_y].len() {
            self.buf[self.cur_y].remove(self.cur_x);
            self.update_modified();
        } else if self.cur_y + 1 < self.buf.len() {
            let next_line = self.buf.remove(self.cur_y + 1);
            self.buf[self.cur_y].push_str(&next_line);
            self.update_modified();
        }
    }

    pub fn del_next_word(&mut self) {
        if let Some(line) = self.buf.get_mut(self.cur_y) {
            let rest = &line[self.cur_x..];
            if let Some(pos) = rest.find(char::is_whitespace) {
                let mut end = self.cur_x + pos;
                // Skip over following whitespace
                while end < line.len() && line.as_bytes()[end].is_ascii_whitespace() {
                    end += 1;
                }
                line.drain(self.cur_x..end);
            } else {
                line.truncate(self.cur_x);
            }
            self.update_modified();
        }
    }

    // TODO: FIX, NOT WORKING
    pub fn del_prev_word(&mut self) {
        if let Some(line) = self.buf.get_mut(self.cur_y) {
            if self.cur_x == 0 && self.cur_y > 0 {
                // Merge with previous line
                let current_line = self.buf.remove(self.cur_y);
                self.cur_y -= 1;
                self.cur_x = self.buf[self.cur_y].len();
                self.buf[self.cur_y].push_str(&current_line);
                self.update_modified();
            } else if self.cur_x > 0 {
                let before = &line[..self.cur_x];
                let idx = before.trim_end_matches(char::is_whitespace).rfind(char::is_whitespace)
                    .map(|i| i + 1)
                    .unwrap_or(0);
                line.drain(idx..self.cur_x);
                self.cur_x = idx;
                self.update_modified();
            }
        }
    }

    pub fn quick_exit(&mut self) -> bool {
        match save_buffer(&self.buf, &self.filename) {
            Ok(_) => {
                return true;
            }
            Err(e) => {
                self.message = Some(format!("Error saving file, cannot exit: {e}"));
                self.mode = EditorMode::Normal;
                return false;
            }
        }
    }

    pub fn insert_char(&mut self, c: char) {
        if self.cur_y >= self.buf.len() {
            self.buf.push(String::new());
        }
        self.buf[self.cur_y].insert(self.cur_x, c);
        self.cur_x += 1;
        self.update_modified();
    }

    pub fn save_file(&mut self) {
        self.cmd = Command::None;
        if self.filename_given && !self.filename.is_empty() {
            match save_buffer(&self.buf, &self.filename) {
                Ok(_) => {
                    self.original_buf = self.buf.clone();
                    self.modified = false;
                    let count = self.buf.len();
                    self.message = Some(format!("(Wrote {} lines)", count));
                } 
                Err(e) => {
                    eprintln!("Error saving file {e}");
                }
            }
        } else {
            self.mode = EditorMode::SaveFile;
        }
    }

    pub fn quit(&mut self) -> bool {
        if !self.modified {
            return true;
        } else {
            self.mode = EditorMode::PromptQuit;
        }
        false
    }

    pub fn write_buffer(&mut self) {
        self.mode = EditorMode::SaveFile;
        self.cmd = Command::None;
    }

    pub fn update_modified(&mut self) {
        self.modified = self.buf != self.original_buf;
    }

    pub fn insert_tab(&mut self) {
        for _ in 0..TAB_WIDTH {
            self.insert_char(' ');
        }
    }
} 

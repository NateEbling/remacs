use std::io::{self, Write};
use std::fs::{self, File};
use std::path::Path;

pub fn save_buffer(buf: &[String], filename: &str) -> io::Result<usize> {
    let path = Path::new(filename);

    // Read existing lines from file if it exists
    let existing_lines: Vec<String> = if path.exists() {
        match fs::read_to_string(path) {
            Ok(content) => content.lines().map(|s| s.to_string()).collect(),
            Err(_) => vec![],
        }
    } else {
        vec![]
    };

    // Compare buffers line by line
    let mut changed_lines = 0;
    let max_len = buf.len().max(existing_lines.len());

    for i in 0..max_len {
        let new = buf.get(i).map(|s| s.trim_end());
        let old = existing_lines.get(i).map(|s| s.trim_end());

        if new != old {
            changed_lines += 1;
        }
    }

    // If no lines changed, skip writing
    if changed_lines == 0 {
        return Ok(0);
    }

    // Write updated buffer to file
    let mut file = File::create(path)?;
    for line in buf {
        writeln!(file, "{line}")?;
    }

    Ok(changed_lines)
}

pub fn load_buffer(filename: &str) -> io::Result<Vec<String>> {
    let content = std::fs::read_to_string(filename)?;
    Ok(content.lines().map(|s| s.to_string()).collect())
}

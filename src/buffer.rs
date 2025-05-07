use std::io::{self, BufWriter, Write};
use std::fs::File;

pub fn save_buffer(buffer: &[String], filename: &str) -> io::Result<()> {
    let file = File::create(filename)?;
    let mut writer = BufWriter::new(file);

    for line in buffer {
        writeln!(writer, "{line}")?;
    }

    writer.flush()
}

pub fn load_buffer(filename: &str) -> io::Result<Vec<String>> {
    let content = std::fs::read_to_string(filename)?;
    Ok(content.lines().map(|s| s.to_string()).collect())
}

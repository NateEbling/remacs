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

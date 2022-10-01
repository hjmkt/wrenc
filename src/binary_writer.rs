use std::fs::File;
use std::io::{self, Write};

pub struct BinaryWriter<'a> {
    output: Box<dyn Write + 'a>,
    buf: u8,
    index: usize,
}

impl<'a> BinaryWriter<'a> {
    pub fn standard(stdout: &'a io::Stdout) -> BinaryWriter<'a> {
        BinaryWriter {
            output: Box::new(stdout.lock()),
            buf: 0,
            index: 0,
        }
    }

    pub fn file(path: String) -> io::Result<BinaryWriter<'a>> {
        File::create(path).map(|file| BinaryWriter {
            output: Box::new(io::BufWriter::new(file)),
            buf: 0,
            index: 0,
        })
    }

    pub fn write_bit(&mut self, bit: bool) {
        self.buf = (self.buf << 1) | bit as u8;
        self.index += 1;
        if self.index == 8 {
            self.index = 0;
            let tmp = [self.buf];
            if let Err(e) = self.write(&tmp) {
                panic!("{e}");
            }
            self.buf = 0;
        }
    }

    pub fn write_bits(&mut self, bits: &Vec<bool>) {
        for bit in bits {
            self.write_bit(*bit);
        }
    }

    pub fn _byte_align(&mut self) {
        let rem = if self.index > 0 { 8 - self.index } else { 0 };
        for _ in 0..rem {
            self.write_bit(false);
        }
    }
}

impl<'a> Write for BinaryWriter<'a> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.output.write(buf)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.output.flush()
    }
}

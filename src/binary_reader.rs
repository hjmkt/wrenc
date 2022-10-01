#![allow(dead_code)]
use debug_print::*;
use std::fs::File;
use std::io::{self, BufRead, Read};

pub struct BinaryReader<'a> {
    input: Box<dyn BufRead + 'a>,
    buffer: u8,
    bit_offset: usize,
}

impl<'a> BinaryReader<'a> {
    pub fn standard(stdin: &'a io::Stdin) -> BinaryReader<'a> {
        BinaryReader {
            input: Box::new(stdin.lock()),
            buffer: 0,
            bit_offset: 0,
        }
    }

    pub fn file(path: String) -> io::Result<BinaryReader<'a>> {
        File::open(path).map(|file| BinaryReader {
            input: Box::new(io::BufReader::new(file)),
            buffer: 0,
            bit_offset: 0,
        })
    }

    pub fn vec(v: &'a [u8]) -> io::Result<BinaryReader<'a>> {
        Ok(BinaryReader {
            input: Box::new(v),
            buffer: 0,
            bit_offset: 0,
        })
    }

    pub fn read_bit(&mut self) -> bool {
        //println!("read bit");
        if self.bit_offset > 0 {
            let bit = (self.buffer >> (7 - self.bit_offset)) & 1 > 0;
            self.bit_offset = (self.bit_offset + 1) % 8;
            bit
        } else {
            let mut tmp: Vec<u8> = vec![0; 1];
            match self.input.read(&mut tmp[..]) {
                Ok(_) => {
                    self.buffer = tmp[0];
                    let bit = (self.buffer >> 7) & 1 > 0;
                    self.bit_offset = 1;
                    bit
                }
                Err(_) => panic!(),
            }
        }
    }

    // FIXME speedup
    pub fn read_bits(&mut self, n_bits: usize) -> Vec<bool> {
        let mut bits = vec![];
        for _ in 0..n_bits {
            bits.push(self.read_bit());
        }
        bits
    }

    pub fn read_to_vec<T: From<u8>>(&mut self, v: &mut Vec<T>) -> io::Result<usize> {
        let len = v.len();
        debug_eprintln!("len = {}", len);
        let mut tmp: Vec<u8> = vec![0; len];
        let mut read_bytes = 0;
        while read_bytes < len {
            match self.input.read(&mut tmp[read_bytes..]) {
                Ok(s) => {
                    for i in 0..s {
                        v[read_bytes + i] = T::from(tmp[read_bytes + i]);
                    }
                    read_bytes += s;
                }
                Err(e) => return Err(e),
            }
        }
        Ok(len)
    }
}

impl<'a> Read for BinaryReader<'a> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.input.read(buf)
    }
}

impl<'a> BufRead for BinaryReader<'a> {
    fn fill_buf(&mut self) -> io::Result<&[u8]> {
        self.input.fill_buf()
    }

    fn consume(&mut self, amt: usize) {
        self.input.consume(amt);
    }
}

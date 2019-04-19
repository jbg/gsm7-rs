#![feature(proc_macro_hygiene)]

use std::io;

use bitstream_io::{BigEndian, BitReader, BitWriter};
use phf::phf_ordered_set;


const ESC: u8 = 0x1B;

static GSM7_CHARSET: phf::OrderedSet<char> = phf_ordered_set! {
    '@', '£', '$', '¥', 'è', 'é', 'ù', 'ì',  'ò', 'Ç', '\n', 'Ø',    'ø', '\r', 'Å', 'å',
    'Δ', '_', 'Φ', 'Γ', 'Λ', 'Ω', 'Π', 'Ψ',  'Σ', 'Θ', 'Ξ',  '\x1B', 'Æ', 'æ',  'ß', 'É',
    ' ', '!', '"', '#', '¤', '%', '&', '\'', '(', ')', '*',  '+',    ',', '-',  '.', '/',
    '0', '1', '2', '3', '4', '5', '6', '7', '8',  '9', ':',  ';',    '<', '=',  '>', '?',
    '¡', 'A', 'B', 'C', 'D', 'E', 'F', 'G', 'H',  'I', 'J',  'K',    'L', 'M',  'N', 'O',
    'P', 'Q', 'R', 'S', 'T', 'U', 'V', 'W', 'X',  'Y', 'Z',  'Ä',    'Ö', 'Ñ',  'Ü', '§',
    '¿', 'a', 'b', 'c', 'd', 'e', 'f', 'g', 'h',  'i', 'j',  'k',    'l', 'm',  'n', 'o',
    'p', 'q', 'r', 's', 't', 'u', 'v', 'w', 'x',  'y', 'z',  'ä',    'ö', 'ñ',  'ü', 'à',
};

pub struct Gsm7Reader<R: io::Read> {
    reader: BitReader<R, BigEndian>
}

impl<R: io::Read> Gsm7Reader<R> {
    pub fn new(reader: R) -> Self {
        Self { reader: BitReader::new(reader) }
    }

    pub fn read_char(&mut self) -> io::Result<Option<char>> {
        let septet: u8 = match self.reader.read(7) {
            Ok(s) => s,
            Err(ref e) if e.kind() == io::ErrorKind::UnexpectedEof => return Ok(None),
            Err(e) => return Err(e),
        };

        if septet == ESC {
            let septet: u8 = self.reader.read(7)?;
            Ok(Some(match septet {
                0x0A => '\x0C',
                0x14 => '^',
                0x28 => '{',
                0x29 => '}',
                0x2F => '\\',
                0x3C => '[',
                0x3D => '~',
                0x3E => ']',
                0x40 => '|',
                0x65 => '€',
                _ => return Err(io::ErrorKind::InvalidData.into())
            }))
        }
        else {
            if let Some(c) = GSM7_CHARSET.index(septet as usize) {
                Ok(Some(*c))
            }
            else {
                Err(io::ErrorKind::InvalidData.into())
            }
        }
    }
}

impl<R: io::Read> Iterator for Gsm7Reader<R> {
    type Item = io::Result<char>;

    fn next(&mut self) -> Option<Self::Item> {
        self.read_char().transpose()
    }
}

pub struct Gsm7Writer<W: io::Write> {
    writer: BitWriter<W, BigEndian>,
    counter: usize,
}

impl<W: io::Write> Gsm7Writer<W> {
    pub fn new(writer: W) -> Self {
        Self { writer: BitWriter::new(writer), counter: 0 }
    }

    pub fn write_char(&mut self, c: char) -> io::Result<()> {
        match c {
            '\x0C' => self.write_ext(0x0A)?,
            '^' => self.write_ext(0x14)?,
            '{' => self.write_ext(0x28)?,
            '}' => self.write_ext(0x29)?,
            '\\' => self.write_ext(0x2F)?,
            '[' => self.write_ext(0x3C)?,
            '~' => self.write_ext(0x3D)?,
            ']' => self.write_ext(0x3E)?,
            '|' => self.write_ext(0x40)?,
            '€' => self.write_ext(0x65)?,
            _ => if let Some(b) = GSM7_CHARSET.get_index(&c) {
                self.writer.write(7, b as u8)?;
            }
            else {
                return Err(io::ErrorKind::InvalidData.into());
            }
        }
        self.counter += 7;
        Ok(())
    }

    pub fn into_writer(mut self) -> io::Result<W> {
        let remainder = self.counter % 8;
        if remainder == 7 {
            self.writer.write(7, 0x0D)?;
        }
        else if remainder != 0 {
            self.writer.byte_align()?;
        }
        Ok(self.writer.into_writer())
    }

    fn write_ext(&mut self, b: u8) -> io::Result<()> {
        self.writer.write(7, 0x1B)?;
        self.writer.write(7, b)?;
        Ok(())
    }
}


#[cfg(test)]
mod tests {
    use std::io;

    use crate::{Gsm7Reader, Gsm7Writer};


    #[test]
    fn it_works() {
        let v = Vec::new();

        let mut writer = Gsm7Writer::new(v);
        writer.write_char('H').unwrap();
        writer.write_char('e').unwrap();
        writer.write_char('l').unwrap();
        writer.write_char('l').unwrap();
        writer.write_char('o').unwrap();

        let v = writer.into_writer().unwrap();
        eprintln!("v: {:?}", v);

        let mut reader = Gsm7Reader::new(io::Cursor::new(&v));
        assert_eq!(reader.read_char().unwrap(), Some('H'));
        assert_eq!(reader.read_char().unwrap(), Some('e'));
        assert_eq!(reader.read_char().unwrap(), Some('l'));
        assert_eq!(reader.read_char().unwrap(), Some('l'));
        assert_eq!(reader.read_char().unwrap(), Some('o'));
    }

    #[test]
    fn iteration_works() {
        let v = Vec::new();

        let mut writer = Gsm7Writer::new(v);
        writer.write_char('H').unwrap();
        writer.write_char('e').unwrap();
        writer.write_char('l').unwrap();
        writer.write_char('l').unwrap();
        writer.write_char('o').unwrap();

        let v = writer.into_writer().unwrap();
        eprintln!("v: {:?}", v);

        let reader = Gsm7Reader::new(io::Cursor::new(&v));
        let s: String = reader.collect::<io::Result<_>>().unwrap();
        assert_eq!(&s, "Hello");
    }
}

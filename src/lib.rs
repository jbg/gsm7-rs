use std::io;

use bitstream_io::{BitReader, BitWriter, LittleEndian, Numeric};

type Endianness = LittleEndian;

const ESC: u8 = 0x1B;

static GSM7_CHARSET: [char; 128] = [
    '@', '£', '$', '¥', 'è', 'é', 'ù', 'ì',  'ò', 'Ç', '\n', 'Ø',    'ø', '\r', 'Å', 'å',
    'Δ', '_', 'Φ', 'Γ', 'Λ', 'Ω', 'Π', 'Ψ',  'Σ', 'Θ', 'Ξ',  '\x1B', 'Æ', 'æ',  'ß', 'É',
    ' ', '!', '"', '#', '¤', '%', '&', '\'', '(', ')', '*',  '+',    ',', '-',  '.', '/',
    '0', '1', '2', '3', '4', '5', '6', '7', '8',  '9', ':',  ';',    '<', '=',  '>', '?',
    '¡', 'A', 'B', 'C', 'D', 'E', 'F', 'G', 'H',  'I', 'J',  'K',    'L', 'M',  'N', 'O',
    'P', 'Q', 'R', 'S', 'T', 'U', 'V', 'W', 'X',  'Y', 'Z',  'Ä',    'Ö', 'Ñ',  'Ü', '§',
    '¿', 'a', 'b', 'c', 'd', 'e', 'f', 'g', 'h',  'i', 'j',  'k',    'l', 'm',  'n', 'o',
    'p', 'q', 'r', 's', 't', 'u', 'v', 'w', 'x',  'y', 'z',  'ä',    'ö', 'ñ',  'ü', 'à',
];

pub struct Gsm7Reader<R: io::Read> {
    reader: BitReader<R, Endianness>,
}

impl<R: io::Read> Gsm7Reader<R> {
    pub fn new(reader: R) -> Self {
        Self { reader: BitReader::new(reader) }
    }
}

impl<R: io::Read> From<BitReader<R, Endianness>> for Gsm7Reader<R> {
    fn from(reader: BitReader<R, Endianness>) -> Self {
        Self { reader }
    }
}

impl<R: io::Read> Iterator for Gsm7Reader<R> {
    type Item = io::Result<char>;

    fn next(&mut self) -> Option<Self::Item> {
        let septet: u8 = match self.reader.read(7) {
            Ok(s) => s,
            Err(ref e) if e.kind() == io::ErrorKind::UnexpectedEof => return None,
            Err(e) => return Some(Err(e)),
        };

        if septet == ESC {
            let septet: u8 = match self.reader.read(7) {
                Ok(s) => s,
                Err(e) => return Some(Err(e)),
            };
            Some(Ok(match septet {
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
                _ => return Some(Err(io::ErrorKind::InvalidData.into())),
            }))
        }
        else {
            if let Some(c) = GSM7_CHARSET.get(septet as usize) {
                Some(Ok(*c))
            }
            else {
                Some(Err(io::ErrorKind::InvalidData.into()))
            }
        }
    }
}

pub struct Gsm7Writer<W: io::Write> {
    writer: BitWriter<W, Endianness>,
    counter: usize,
}

impl<W: io::Write> Gsm7Writer<W> {
    pub fn new(writer: W) -> Self {
        Self { writer: BitWriter::new(writer), counter: 0 }
    }

    pub fn write_bit(&mut self, bit: bool) -> io::Result<()> {
        self.writer.write_bit(bit)?;
        self.counter += 1;
        Ok(())
    }

    pub fn write<U>(&mut self, bits: u32, value: U) -> io::Result<()>
    where
        U: Numeric
    {
        self.writer.write(bits, value)?;
        self.counter += bits as usize;
        Ok(())
    }

    pub fn write_bytes(&mut self, buf: &[u8]) -> io::Result<()> {
        self.writer.write_bytes(buf)
    }

    pub fn write_str(&mut self, s: &str) -> io::Result<()> {
        for c in s.chars() {
            self.write_char(c)?;
        }
        Ok(())
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
            _ => if let Some(b) = GSM7_CHARSET.iter().position(|&v| v == c) {
                self.writer.write(7, b as u8)?;
                self.counter += 7;
            }
            else {
                return Err(io::ErrorKind::InvalidData.into());
            }
        }
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
        self.counter += 14;
        Ok(())
    }
}

impl<W: io::Write> From<BitWriter<W, Endianness>> for Gsm7Writer<W> {
    fn from(writer: BitWriter<W, Endianness>) -> Self {
        Self { writer, counter: 0 }
    }
}

#[cfg(test)]
mod tests {
    use std::io;

    use crate::{Gsm7Reader, Gsm7Writer};

    #[test]
    fn it_works() -> io::Result<()> {
        let v = Vec::new();

        let mut writer = Gsm7Writer::new(v);
        writer.write_str("Hello")?;

        let v = writer.into_writer()?;
        eprintln!("v: {:?}", v);

        let reader = Gsm7Reader::new(io::Cursor::new(&v));
        let s = reader.collect::<io::Result<String>>()?;
        assert_eq!(s.as_str(), "Hello");

        Ok(())
    }

    #[test]
    fn it_works_correctly() -> io::Result<()> {
        let v: Vec<_> = vec![84, 58, 157, 14].into_iter().collect();
        let reader = Gsm7Reader::new(io::Cursor::new(&v));
        let s = reader.collect::<io::Result<String>>()?;
        assert_eq!(&s, "Tttt");

        let v = vec![0xD4, 0xF2, 0x9C, 0x0E];
        let reader = Gsm7Reader::new(io::Cursor::new(&v));
        let s: String = reader.collect::<io::Result<String>>()?;
        assert_eq!(&s, "Test");

        Ok(())
    }
}

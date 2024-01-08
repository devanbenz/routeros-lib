use std::io::{Read, Write};
use std::net::TcpStream;
use std::str;

pub struct Word {
    pub content: String,
}

impl Word {
    pub fn get_length(&self) -> Vec<u8> {
        /*
        0 <= len <= 0x7F	            1	len, lowest byte
        0x80 <= len <= 0x3FFF	        2	len |= 0x8000, two lower bytes
        0x4000 <= len <= 0x1FFFFF	    3	len |= 0xC00000, three lower bytes
        0x200000 <= len <= 0xFFFFFFF	4	len |= 0xE0000000
        len >= 0x10000000	            5  	0xF0 and len as four bytes
         */
        match self.content.as_bytes().len() {
            len if (0..=0x7f).contains(&len) => {
                let first_byte: u8;
                if let Some(&lowest_byte) = self.content.len().to_le_bytes().first() {
                    first_byte = lowest_byte
                } else {
                    panic!("String is empty!");
                };

                return vec![first_byte];
            }
            len if (0x80..=0x3fff).contains(&len) => {
                let length = len | 0x8000;
                let first_byte: u8;
                let second_byte: u8;
                if let Some(&lowest_byte) = length.to_le_bytes().first() {
                    first_byte = lowest_byte;
                } else {
                    panic!("string is empty!");
                };
                if let Some(&second_lowest_byte) = length.to_le_bytes().get(1) {
                    second_byte = second_lowest_byte;
                } else {
                    panic!("string is empty!");
                };
                return vec![first_byte, second_byte];
            }
            _ => unreachable!(),
        }
    }
}

struct Sentence {
    words: Vec<Word>,
}

impl Sentence {
    fn read_words(&self) -> Vec<u8> {
        let mut sentence: Vec<u8> = Vec::new();

        for word in self.words.iter() {
            for len in word.get_length() {
                sentence.push(len);
            }
            for byte in word.content.as_bytes() {
                sentence.push(*byte);
            }
        }

        sentence
    }
}

pub struct RouterOsClient {
    stream: TcpStream,
    sentence: Sentence,
}

impl RouterOsClient {
    pub fn new(addr: &str) -> Self {
        RouterOsClient {
            stream: TcpStream::connect(addr).expect("error connecting to tcp stream"),
            sentence: Sentence { words: vec![] },
        }
    }

    pub fn write_api_data(&mut self, sentence: Vec<String>) {
        let mut words: Vec<Word> = Vec::new();
        for word in sentence {
            words.push(Word { content: word })
        }
        self.sentence = Sentence { words };

        let raw_data = self.sentence.read_words();
        let data_written = self.write(&raw_data).expect("error writing stream");
        if data_written < raw_data.len() {
            panic!("partial write!");
        }
        self.flush().expect("error flushing stream");
    }

    pub fn read_api_data(&mut self) {
        let mut buffer = [0; 1024];
        let bytes_read = self
            .stream
            .read(&mut buffer)
            .expect("error reading from stream");
        let res = str::from_utf8(&buffer[..bytes_read]).expect("error parsing to string");
        println!("{}", res);
    }
}

impl Write for RouterOsClient {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.stream.write(buf)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.stream.flush()
    }
}

impl Read for RouterOsClient {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize, std::io::Error> {
        self.stream.read(buf)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_length_first_order() {
        let word: Word = Word {
            content: "hi".to_owned(),
        };

        let want: Vec<u8> = vec![2];
        assert_eq!(want, word.get_length());
    }

    #[test]
    fn test_length_second_order() {
        let byte_size = 0x3FFF;
        let word: Word = Word {
            content: "a".to_owned().repeat(byte_size),
        };

        let want: Vec<u8> = vec![255, 191];
        assert_eq!(want, word.get_length());
    }

    #[test]
    fn test_read_sentence() {
        let command = Word {
            content: "/test".to_owned(),
        };
        let attribute = Word {
            content: "=test=true".to_owned(),
        };
        let sentence = Sentence {
            words: vec![command, attribute],
        };

        let want: Vec<u8> = vec![
            5, 47, 116, 101, 115, 116, 10, 61, 116, 101, 115, 116, 61, 116, 114, 117, 101,
        ];
        assert_eq!(want, sentence.read_words());
    }
}

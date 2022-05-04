use std::fs::read_to_string;

const ZERO: Bit = false;
const ONE: Bit = true;

type Byte = u8;
type Bit = bool;

pub struct Bstream {
    stream: Vec<Byte>,
    count: u8,
}

impl Bstream {
    pub(self) fn new_breader(b: Vec<Byte>) ->Self{
        Bstream {stream: b, count: 8}
    }

    pub(self) fn new_bwriter() -> Self{
        Bstream {stream: Vec::new(), count: 0}
    }

    pub(self) fn bytes(self)->Vec<Byte>{
        self.stream
    }

    pub(self) fn write_bit(&mut self, bit: Bit){
        if self.count == 0{
            self.stream.push(0);
            self.count = 8
        }
        let mut i = self.stream.len() - 1;
        if bit == true {
            self.stream[i] |= 1 << (self.count - 1)
        }
        self.count -= 1;
    }

    // write_byte write a Byte into stream
    pub(self) fn write_byte(&mut self, byt: u8){
        if self.count == 0{
            self.stream.push(0);
            self.count = 8;
        }
        let mut i = self.stream.len();
        self.stream[i] |= byt >> (8-self.count);
        i += 1;
        self.stream[i] = byt << self.count;
    }

    pub(self) fn write_bits(&mut self, mut u: u64,mut nbits: u8){
        u <<= 64 - nbits;
        while nbits >= 8 {
            let byt = (u >> 56) as u8;
            self.write_byte(byt);
            u <<= 8;
            nbits -= 8;
        }
        while nbits > 0 {
            self.write_bit((u >> 63) == 1);
            u <<= 1;
            nbits -= 1;
        }
    }

    pub(self) fn read_bit(&mut self) -> Option<Bit>{
        if self.stream.len() == 0 {
            return None;
        }
        if self.count == 0 {
            self.stream.remove(0);
            if self.stream.len() == 0 {
                return None;
            }
            self.count = 8;
        }
        self.count -= 1;
        let d  = self.stream[0] & 0x80;
        self.stream[0] <<= 1;
        Some(d != 0)
    }


    pub(self) fn read_byte(&mut self) -> Option<Byte>{
        if self.stream.len() == 0 {
            return None;
        }
        if self.count == 0 {
            self.stream.remove(0);
            if self.stream.len() == 0 {
                return None;
            }
            self.count = 8;
        }
        if self.count == 8{
            self.count = 0;
            return Some(self.stream.remove(0) as Byte)
        }
        let mut byt = self.stream.remove(0);
        if self.stream.len() == 0 {
            return None
        }
        byt |= self.stream[0] >> self.count;
        self.stream[0] <<= 8-self.count;
        Some(byt as Byte)
    }

    pub(self) fn read_bits(&mut self,mut nbits: u8)-> Option<u64>{
        let mut u: u64 = 0;
        while nbits >= 8 {
            self.read_byte().map(|v|{
                u = (u << 8) | v as u64;
                nbits -= 8;
            })?;
        }
        if nbits == 0 {
            return Some(u);
        }
        if nbits > self.count as u8 {
            u = (u << self.count as u64) | (self.stream[0] >> (8-self.count)) as u64;
            nbits -= self.count;
            self.stream.remove(0);
            if self.stream.len() == 0{
                return None;
            }
            self.count = 8;
        }

        u = ( u << nbits as u64) | (self.stream[0] >> (8-nbits)) as u64;
        self.stream[0] <<= nbits;
        self.count -= nbits;
        Some(u)
    }

    pub(self) fn reset(&mut self) {
        self.stream.clear();
        self.count = 0;
    }
}

#[cfg(test)]
mod test{
    use super::{Bstream, ONE, ZERO};
    #[test]
    fn test_bastream_handler(){
        let mut stream = Bstream::new_bwriter();
        for bit in vec![ZERO, ONE] {
            stream.write_bit(bit);
        }

        let mut reader = Bstream::new_breader(stream.bytes());
        for bit in vec![ZERO, ONE]{
            assert_eq!(reader.read_bit(), Some(bit))
        }
    }
}

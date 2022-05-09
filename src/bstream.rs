use std::fs::read_to_string;

pub const ZERO: Bit = false;
pub const ONE: Bit = true;

type Bit = bool;


pub struct BStream {
    stream: Vec<u8>,
    count: u8,
}

impl BStream {
    pub fn new_breader(b: Vec<u8>) ->Self{
        BStream {stream: b, count: 8}
    }

    pub fn new_bwriter() -> Self{
        BStream {stream: Vec::new(), count: 0}
    }

    pub fn clone(&self)-> Vec<u8>{
        self.stream.clone()
    }

    pub fn bytes(self)->Vec<u8>{
        self.stream
    }

    pub fn write_bit(&mut self, bit: Bit){
        if self.count == 0{
            self.stream.push(0);
            self.count = 8
        }
        let i = self.stream.len() - 1;
        if bit == true {
            self.stream[i] |= (1 as u8).checked_shl(self.count as u32 - 1).unwrap_or(0);
        }
        self.count -= 1;
    }

    // write_byte write a Byte into stream
    pub fn write_byte(&mut self, byt: u8){
        if self.count == 0{
            self.stream.push(0);
            self.count = 8;
        }
        let mut i = self.stream.len()-1;
        self.stream[i] |= byt.checked_shr(8-self.count as u32).unwrap_or(0);
        self.stream.push(0);
        i += 1;
        self.stream[i] = byt.checked_shl(self.count as u32).unwrap_or(0);
    }

    pub fn write_bits(&mut self, mut u: u64,mut nbits: u8){
        u <<= 64 - nbits;

        while nbits >= 8 {
            let byt = u.checked_shr(56).unwrap_or(0) as u8;

            self.write_byte(byt);
            u = u.checked_shl(8).unwrap_or(0);
            nbits -= 8;
        }
        while nbits > 0 {
            self.write_bit(u.checked_shr(63).unwrap_or(0) == 1);
            u = u.checked_shl(1).unwrap_or(0);
            nbits -= 1;
        }
    }

    pub fn read_bit(&mut self) -> Option<Bit>{
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
        self.stream[0] = self.stream[0].checked_shl(1).unwrap_or(0);
        Some(d != 0)
    }


    pub fn read_byte(&mut self) -> Option<u8>{
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
            return Some(self.stream.remove(0) as u8)
        }
        let mut byt = self.stream.remove(0);
        if self.stream.len() == 0 {
            return None
        }
        byt |= self.stream[0].checked_shr(self.count as u32).unwrap_or(0);

        self.stream[0] = self.stream[0].checked_shl(8-self.count as u32).unwrap_or(0);
        Some(byt)
    }

    pub fn read_bits(&mut self,mut nbits: u8)-> Option<u64>{

        let mut u: u64 = 0;
        while nbits >= 8 {
            self.read_byte().map(|v|{
                u = u.checked_shl(8).unwrap_or(0) | v as u64;
                nbits -= 8;
            })?;
        }
        if nbits == 0 {
            return Some(u);
        }
        if nbits > self.count as u8 {
            u = (u.checked_shl(self.count as u32)).unwrap_or(0)as u64 | self.stream[0].checked_shr(8- self.count as u32).unwrap_or(0) as u64;
            nbits -= self.count;
            self.stream.remove(0);
            if self.stream.len() == 0{
                return None;
            }
            self.count = 8;
        }
        u = u.checked_shl(nbits as u32).unwrap_or(0)| self.stream[0].checked_shr(8-nbits as u32).unwrap_or(0) as u64;
        self.stream[0] = self.stream[0].checked_shl(nbits as u32).unwrap_or(0);
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
    use super::{BStream, ONE, ZERO};
    use std::io::Write;


    #[test]
    fn test_bstream_readbit(){
        // empty read
        let mut reader = BStream::new_breader(BStream::new_bwriter().bytes());
        assert_eq!(reader.read_bit(), None);

        // write one bit,read one bit, should ok
        let mut writer = BStream::new_bwriter();
        writer.write_bit(ONE);
        assert_eq!(BStream::new_breader(writer.clone()).read_bit(), Some(ONE));

        // write one bit ,read 2 bit, should ok
        writer.reset();
        writer.write_bit(ONE);
        let mut reader = BStream::new_breader(writer.clone());
        assert_eq!(reader.read_bit(), Some(ONE));
        assert_eq!(reader.read_bit(), Some(ZERO));

        // write 8 bit, read 8bit
        let mut cases = vec![ONE, ZERO, ONE, ZERO, ONE, ZERO, ONE, ZERO];
        writer.reset();
        for v in cases.clone(){
            writer.write_bit(v);
        }
        let mut reader= BStream::new_breader(writer.clone());
        for v in cases.clone(){
            assert_eq!(reader.read_bit(), Some(v));
        }
        //  write 8 bit ,read 9bit
        assert_eq!(reader.read_bit(), None);

    }
    #[test]
    fn test_bstream_readbyte(){
        // empty reader
        let mut empty_read = BStream::new_breader(BStream::new_bwriter().bytes());
        assert_eq!(empty_read.read_bit(), None);
        // only one byte, but has been readed
        let mut one_byte_read = BStream::new_breader(vec![1]);
        assert_eq!(one_byte_read.read_byte(), Some(1));
        assert_eq!(one_byte_read.read_byte(), None);
        //has read one block, next block is enough
        let mut case2_read = BStream{
            stream: vec![0, 9],
            count: 0,
        };
        assert_eq!(case2_read.read_byte(), Some(9));

        // next block is not enough
        let mut case3_read = BStream{stream: vec![0], count:1};
        assert_eq!(case3_read.read_byte(), None);

        // next block is enough
        let mut case4_read = BStream{stream: vec![128, 0], count:7};
        assert_eq!(case4_read.read_byte(), Some(128));
    }

    // #[test]
    fn test_bstream_readbits(){
        // empty read
        let mut reader1 = BStream::new_breader(BStream::new_bwriter().bytes());
        assert_eq!(reader1.read_bits(64), None);
        // read 64 bits
        let mut reader2 = BStream{
            stream: vec![0, 0 ,0 ,0 ,0,0,0,1],
            count: 8
        };
        assert_eq!(reader2.read_bits(64), Some(1 as u64));
    }
    #[test]
    fn test_bstream_handler(){
        let mut writer = BStream::new_bwriter();

        for _i in 0..10{
            writer.write_bit((_i/2) == 0);
        }
        for _i in 0..128{
            writer.write_byte(_i);
        }
        for _i in 0..64{
            writer.write_bits(_i, 7);
        }


        let mut reader = BStream::new_breader(writer.clone());

        for _i in 0..10{
            assert_eq!(reader.read_bit(), Some((_i/2) == 0));
        }
        for _i in 0..128{
            assert_eq!(reader.read_byte(), Some(_i));
        }
        for _i in 0..64{
            assert_eq!(reader.read_bits(7), Some(_i));
        }




    }
}

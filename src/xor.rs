use crate::bstream;
use crate::bstream::{ZERO, BStream};
use std::ops::{BitXor, BitAnd, BitOr};
use std::convert::TryFrom;
use std::fs::{File};
use std::io::{Read, Write};
use std::borrow::Borrow;


pub struct XORChunk<'a> {
    t0: i64,
    t: i64,
    val : f64,
    b_stream:  bstream::BStream,
    leading : u8,
    trailing: u8,
    writer: &'a File,

    t_delta: u32,
}
impl <'a> XORChunk<'a>{
    fn new(t0: i64 , writer : &'a File)->XORChunk{
        XORChunk{
            t0,
            writer: writer,
            t: 0, val: 0.0, b_stream : bstream::BStream::new_bwriter(), leading:0, trailing: 0, t_delta: 0
        }
    }
    //
    fn push(&mut self, t:i64, v: f64){
        if self.t == 0{
            self.t = t;
            self.val = v;
            self.t_delta = (t - self.t0) as u32;
            self.b_stream.write_bits(self.t_delta as u64, 14);
            self.b_stream.write_bits(v.to_bits(), 64);
            return;
        }
        // compression time stamp
        let t_delta = u32::try_from(t-self.t).unwrap();

        let dod = i32::try_from(1 as u32).unwrap();

        if dod == 0 {
            self.b_stream.write_bit(ZERO);
        }else if -63 <= dod && dod <= 64{
            self.b_stream.write_bits(0x02, 2);
            self.b_stream.write_bits(dod as u64, 7);
        }else if -255 <= dod && dod <= 256{
            self.b_stream.write_bits(0x06, 3);
            self.b_stream.write_bits(dod as u64, 9);
        }else if -2047 <= dod && dod <= 2047{
            self.b_stream.write_bits(0x0e, 4);
            self.b_stream.write_bits(dod as u64, 12);
        }else {
            self.b_stream.write_bits(0x0f, 4);
            self.b_stream.write_bits(dod as u64, 32);
        }

        // compression value
        let v_delta = v.to_bits().bitxor(self.val.to_bits());

        if v_delta == 0{
            // if xor with the previous is zero(means they are same value)
            self.b_stream.write_bit(bstream::ZERO);
        }else {
            self.b_stream.write_bit(bstream::ONE);
            let mut leading = v_delta.leading_zeros();
            let mut trailing = v_delta.trailing_zeros();
            if leading >= 32 {
                leading = 31;
            }

            if self.leading != 0xff && leading>= self.leading as u32 && trailing >=self.trailing as u32{
                self.b_stream.write_bit(bstream::ZERO);
                self.b_stream.write_bits(v_delta >> self.trailing, 64-self.leading- self.trailing);
            }else {
                self.leading = leading as u8;
                self.trailing = trailing as u8;
                self.b_stream.write_bit(bstream::ONE);
                self.b_stream.write_bits(self.leading as u64, 5);
                let sigbits = 64 - self.leading - self.trailing;
                self.b_stream.write_bits(sigbits as u64, 6);
                self.b_stream.write_bits(v_delta >> self.trailing, sigbits as u8);
            }
        }
        self.t_delta = t_delta;
        self.t = t;
        self.val = v;
        return;

    }

    fn commit(mut self) -> Result<(), String>{
        self.writer.write_all(self.b_stream.bytes().as_slice()).expect("write stream to file failed ");
        Ok(())
    }
}


struct XORIterator<'a> {
    t0: i64,
    t: i64,
    val: f64,
    file: &'a File,

    b_stream: bstream::BStream,
    leading : u8,
    trailing : u8,
    finished: bool,
    t_delta: i32,
}



#[cfg(test)]
mod test{
    // use super::GorillaEncoder;
    use super::XORChunk;
    use std::fs::File;

    #[test]
    fn test_push(){
        let fp = File::create("tmp").expect("create file failed");
        let mut gorrila_encoder = XORChunk::new(10, &fp);
        gorrila_encoder.push(11, 0.1);
        gorrila_encoder.push(12, 0.1);
        gorrila_encoder.push(13, 0.1);
        gorrila_encoder.push(14, 0.1);
        gorrila_encoder.push(14, 0.1);
        gorrila_encoder.push(15, 0.1);
        gorrila_encoder.commit().expect("sync data filed");
    }
}


pub trait SeriesEncoder {
    fn encode_sample(k:i64, v: f64)-> Result<(), String>;
    fn flush()->Result<(), String>;
}


pub trait SeriesDecoder {
    fn decoder_sample()->Result<(i64, f64), String>;
}

pub struct GorillaEncoder{

}


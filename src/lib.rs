#![feature(in_band_lifetimes)]

mod bstream;
mod xor;
mod tsdb;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
}

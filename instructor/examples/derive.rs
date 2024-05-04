use instructor::{Buffer, LittleEndian};
use instructor_derive::Deserialize;

fn main() {
    let mut data: &[u8] = &[0x01, 0x00, 0x00, 0x00, 0x0f];
    println!("{:#?}", data.read::<Header2, LittleEndian>().unwrap())
}

#[derive(Debug, Deserialize)]
struct Header {
    length: u32,
    re: u8
}

#[derive(Debug, Deserialize)]
struct Header2(u16, u16);
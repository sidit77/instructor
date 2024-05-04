use instructor::{Buffer, Unpack};
use instructor::utils::Length;

fn main() {
    let mut data: &[u8] = &[0x06, 0x00, 0x01, 0x00, 0x0a, 0x02, 0x02, 0x00, 0x02, 0x00];
    let l2cap: L2capHeader = data.read().unwrap();
    println!("{:?}", l2cap);
}


#[derive(Debug, Unpack)]
#[instructor(endian = "little")]
struct L2capHeader {
    len: Length<u16, 2>,
    cid: u16
}




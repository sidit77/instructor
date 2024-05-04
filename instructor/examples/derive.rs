use instructor::{Buffer, Unpack};
use instructor::utils::Length;

fn main() {
    let mut data: &[u8] = &[0x06, 0x00, 0x01, 0x00, 0x0a, 0x02, 0x02, 0x00, 0x02, 0x00];
    let l2cap: L2capHeader = data.read().unwrap();
    println!("{:?}", l2cap);
    let signaling: SignalingHeader = data.read().unwrap();
    println!("{:?}", signaling);
    println!("{:X?}", data);
}


#[derive(Debug, Unpack)]
#[instructor(endian = "little")]
struct L2capHeader {
    len: Length<u16, 2>,
    cid: u16
}

#[derive(Debug, Unpack)]
#[instructor(endian = "little")]
struct SignalingHeader {
    code: SignalingCodes,
    id: u8,
    length: Length<u16, 0>
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Unpack)]
#[repr(u8)]
enum SignalingCodes {
    CommandReject = 0x01,
    ConnectionRequest = 0x02,
    ConnectionResponse = 0x03,
    ConfigureRequest = 0x04,
    ConfigureResponse = 0x05,
    DisconnectionRequest = 0x06,
    DisconnectionResponse = 0x07,
    EchoRequest = 0x08,
    EchoResponse = 0x09,
    InformationRequest = 0x0A,
    InformationResponse = 0x0B,
    ConnectionParameterUpdateRequest = 0x12,
    ConnectionParameterUpdateResponse = 0x13,
    LECreditBasedConnectionRequest = 0x14,
    LECreditBasedConnectionResponse = 0x15,
    FlowControlCreditIndex = 0x16,
    CreditBasedConnectionRequest = 0x17,
    CreditBasedConnectionResponse = 0x18,
    CreditBasedReconfigurationRequest = 0x19,
    CreditBasedReconfigurationResponse = 0x1A,
}
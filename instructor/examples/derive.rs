use bytes::{Buf, BufMut, Bytes, BytesMut};
use instructor::{Buffer, Instruct, Exstruct, DoubleEndedBufferMut};
use instructor::utils::Length;

fn main() {
    let btpacket = &[0x00, 0x28, 0x0a, 0x00, 0x06, 0x00, 0x01, 0x00, 0x0a, 0x02, 0x02, 0x00, 0x02, 0x00];
    let mut data = Bytes::from_static(btpacket);
    let acl: AclHeader = data.read().unwrap();
    println!("{:?}", acl);
    let l2cap: L2capHeader = data.read().unwrap();
    println!("{:?}", l2cap);
    let signaling: SignalingHeader = data.read().unwrap();
    println!("{:?}", signaling);
    println!("{:?}", data);

    let mut test = BytesMut::new();
    test.put(data.clone());
    test.write_front(dbg!(&SignalingHeader {
        code: SignalingCodes::InformationRequest,
        id: 2,
        length: Length::new(test.len()).unwrap(),
    }));
    test.write_front(dbg!(&L2capHeader {
        len: Length::new(test.len()).unwrap(),
        cid: 1,
    }));
    test.write_front(dbg!(&AclHeader {
        handle: 2048,
        pb: BoundaryFlag::FirstAutomaticallyFlushable,
        bc: BroadcastFlag::PointToPoint,
        length: Length::new(test.len()).unwrap(),
    }));



    println!("{:02x?}", test.chunk());
    assert_eq!(test.chunk(), btpacket.as_slice());

    //let mut test2 = BytesMut::new();
    //test2.put(data);
    //test2.write_front(&signaling);
    //test2.write_front(&l2cap);
    //assert_eq!(test.chunk(), test2.chunk());
}

#[derive(Debug, Exstruct, Instruct)]
#[instructor(endian = "little")]
struct AclHeader {
    #[instructor(bitfield(u16))]
    #[instructor(bits(0..12))]
    handle: u16,
    #[instructor(bits(12..14))]
    pb: BoundaryFlag,
    #[instructor(bits(14..16))]
    bc: BroadcastFlag,
    length: Length<u16, 0>
}

#[derive(Debug, Exstruct, Instruct)]
#[instructor(endian = "little")]
struct L2capHeader {
    len: Length<u16, 2>,
    cid: u16
}

#[derive(Debug, Exstruct, Instruct)]
#[instructor(endian = "little")]
struct SignalingHeader {
    code: SignalingCodes,
    id: u8,
    length: Length<u16, 0>
}

#[derive(Debug, Exstruct, Instruct)]
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

#[derive(Debug, Exstruct, Instruct)]
#[repr(u8)]
enum BoundaryFlag {
    FirstNonAutomaticallyFlushable = 0b00,
    Continuing = 0b01,
    FirstAutomaticallyFlushable = 0b10,
}

#[derive(Debug, Exstruct, Instruct)]
#[repr(u8)]
enum BroadcastFlag {
    PointToPoint = 0b00,
    BrEdrBroadcast = 0b01,
}

#[derive(Debug, Instruct)]
#[instructor(endian = "little")]
enum Headers {
    Acl(AclHeader),
    L2cap(L2capHeader),
    Signaling(SignalingHeader),
}
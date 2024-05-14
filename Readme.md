# Instructor

A small library to extract or write structs and enums from or to byte steams.
This library also has explicit support for endianess.

## Usage

This library is focused around two traits:
- `Exstruct` for extracting structs from byte streams
- `Instruct` for writing structs to byte streams

For most cases the required traits should be automatically derivable.

```rust
// Derive traits for int-like enums
#[derive(Debug, Exstruct, Instruct)]
#[repr(u8)] // for int-like enums an explicit representation is required
enum SignalingCodes {
    // unless a default element is defined, 
    // an error will be thrown when encountering an element which is out of range
    #[instructor(default)] 
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

#[derive(Debug, Exstruct, Instruct)]
// explicitly specify endianess to only derive the trait for little endian
//  otherwise the traits will be generic over endianess
#[instructor(endian = "little")] 
struct SignalingHeader {
    code: SignalingCodes,
    id: u8,
    length: Length<u16, 0> // A special type that automatically checks the size of the remaining buffer
}

#[derive(Debug, Exstruct, Instruct)]
#[instructor(endian = "little")]
struct L2capHeader {
    len: Length<u16, 2>,
    cid: u16
}

#[derive(Debug, Exstruct, Instruct)]
#[instructor(endian = "little")]
struct AclHeader {
    // Bitfields are also supported
    #[instructor(bitfield(u16))]
    #[instructor(bits(0..12))]
    handle: u16,
    #[instructor(bits(12..14))]
    pb: BoundaryFlag,
    #[instructor(bits(14..16))]
    bc: BroadcastFlag,
    length: Length<u16, 0>
}

// Enums with data only support deriving the Instruct trait 
#[derive(Debug, Instruct)]
#[instructor(endian = "little")]
enum Headers {
    Acl(AclHeader),
    L2cap(L2capHeader),
    Signaling(SignalingHeader),
}

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
}
```

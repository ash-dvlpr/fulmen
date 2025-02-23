use blob::*;
use std::collections::HashMap;

use std::net::Ipv4Addr;
use std::str::FromStr;

#[allow(dead_code)]
#[derive(Debug)]
struct MyStruct {
    field: u32,
}

// --- Setups ---

fn localhost_ip() -> Ipv4Addr {
    Ipv4Addr::from_str("127.0.0.1").unwrap()
}

// -- Tests --
// cargo +nightly miri test

#[test]
fn storage_and_retrieval_test() {
    let mut blobs: HashMap<&'static str, Blob> = HashMap::new();
    blobs.insert("IPv4", Blob::from(localhost_ip()));
    blobs.insert("My", Blob::from(MyStruct { field: 30}));
    blobs.insert("ZST", Blob::new::<()>());

    // Assert allocatino
    assert_eq!(true, blobs["IPv4"].has_value());
    assert_eq!(true, blobs["My"].has_value());
    assert_eq!(false, blobs["ZST"].has_value());

    // IPv4
    let data = blobs["IPv4"].downcast_ref::<Ipv4Addr>().unwrap();
    assert_eq!(data.octets(), localhost_ip().octets());

    // MyStruct
    let data = blobs["My"].downcast_ref::<MyStruct>().unwrap();
    assert_eq!(data.field, 30);

    // ZST
    let data = blobs["ZST"].downcast_ref::<()>();
    assert_eq!(Err(Error::None), data);
}

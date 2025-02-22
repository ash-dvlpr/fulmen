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
    let mut blobs: HashMap<&'static str, Option<Blob>> = HashMap::new();
    blobs.insert("IPv4", Blob::from(localhost_ip()).ok());
    blobs.insert("My", Blob::from(MyStruct { field: 30}).ok());
    blobs.insert("ZST", Blob::new::<()>().ok());

    assert!(blobs["IPv4"].is_some());
    assert!(blobs["My"].is_some());
    assert!(blobs["ZST"].is_none());

    // Extract values
    let data = blobs["IPv4"].as_ref().unwrap().downcast_ref::<Ipv4Addr>().unwrap();
    assert_eq!(data.octets(), localhost_ip().octets());

    // Extract values
    let data = blobs["My"].as_ref().unwrap().downcast_ref::<MyStruct>().unwrap();
    assert_eq!(data.field, 30);
}

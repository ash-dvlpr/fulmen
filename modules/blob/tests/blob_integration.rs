use blob::*;
use std::collections::HashMap;

use core::alloc::Layout;
use std::net::Ipv4Addr;
use std::str::FromStr;

#[allow(dead_code)]
#[derive(Debug)]
struct MyDroppable {
    field: u32,
}

impl Drop for MyDroppable {
    fn drop(&mut self) {
        println!("{:?}", self)
    }
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
    blobs.insert("Drop", Blob::from(MyDroppable { field: 30}).ok());
    blobs.insert("ZST", Blob::new::<()>().ok());

    assert!(blobs["IPv4"].is_some());
    assert!(blobs["Drop"].is_none());
    assert!(blobs["ZST"].is_none());

    // Extract values
    
}

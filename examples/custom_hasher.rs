extern crate hash_ring;

use hash_ring::HashRing;
use std::fmt;
use std::fmt::Display;
use std::fmt::Formatter;
use std::hash::BuildHasherDefault;
use std::hash::Hasher;

// This is a hasher that always returns the same number
// no matter the input. This is meant as an example and
// should never be used in production code as all keys go
// to the same node.
#[derive(Default)]
struct ConstantHasher;

impl Hasher for ConstantHasher {
    fn write(&mut self, _bytes: &[u8]) {
        // Do nothing
    }

    fn finish(&self) -> u64 {
        1
    }
}

type ConstantBuildHasher = BuildHasherDefault<ConstantHasher>;

#[derive(Clone, Debug, PartialEq, Hash)]
struct NodeInfo {
    host: &'static str,
    port: u16,
}

impl Display for NodeInfo {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{}:{}", self.host, self.port)
    }
}

fn main() {
    let nodes = vec![
        NodeInfo {
            host: "localhost",
            port: 15324,
        },
        NodeInfo {
            host: "localhost",
            port: 15325,
        },
        NodeInfo {
            host: "localhost",
            port: 15326,
        },
        NodeInfo {
            host: "localhost",
            port: 15327,
        },
        NodeInfo {
            host: "localhost",
            port: 15328,
        },
    ];

    let hash_ring: HashRing<NodeInfo, ConstantBuildHasher> =
        HashRing::with_hasher(nodes, 10, ConstantBuildHasher::default());

    for key in &["hello", "dude", "martian", "tardis"] {
        println!(
            "Key: '{}', Node: {}",
            key,
            hash_ring.get_node(&key).unwrap()
        );
    }
}

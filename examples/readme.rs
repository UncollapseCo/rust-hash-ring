extern crate hash_ring;
use hash_ring::HashRing;
use std::fmt;
use std::fmt::Display;
use std::fmt::Formatter;
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
        NodeInfo {
            host: "localhost",
            port: 15329,
        },
    ];

    let mut hash_ring: HashRing<NodeInfo> = HashRing::new(nodes, 10);

    println!(
        "Key: '{}', Node: {}",
        "hello",
        hash_ring.get_node(&"hello").unwrap()
    );

    println!(
        "Key: '{}', Node: {}",
        "dude",
        hash_ring.get_node(&"dude").unwrap()
    );

    println!(
        "Key: '{}', Node: {}",
        "martian",
        hash_ring.get_node(&"martian").unwrap()
    );

    println!(
        "Key: '{}', Node: {}",
        "tardis",
        hash_ring.get_node(&"tardis").unwrap()
    );

    hash_ring.remove_node(&NodeInfo {
        host: "localhost",
        port: 15329,
    });

    println!(
        "Key: '{}', Node: {}",
        "hello",
        hash_ring.get_node(&"hello").unwrap()
    );

    hash_ring.add_node(&NodeInfo {
        host: "localhost",
        port: 15329,
    });

    println!(
        "Key: '{}', Node: {}",
        "hello",
        hash_ring.get_node(&"hello").unwrap()
    );
}

use std::collections::BinaryHeap;
use std::collections::HashMap;
use std::hash::BuildHasher;
use std::hash::BuildHasherDefault;
use std::hash::Hash;
use std::hash::Hasher;
use std::mem;
use twox_hash::XxHash64;

type XxHash64Hasher = BuildHasherDefault<XxHash64>;

/// HashRing
pub struct HashRing<T, S = XxHash64Hasher> {
    replicas: usize,
    ring: HashMap<u64, T>,
    sorted_keys: Vec<u64>,
    hash_builder: S,
}

impl<T: Hash + Clone> HashRing<T, XxHash64Hasher> {
    /// Creates a new hash ring with the specified nodes.
    /// Replicas is the number of virtual nodes each node has to make a better distribution.
    pub fn new(nodes: Vec<T>, replicas: usize) -> HashRing<T, XxHash64Hasher> {
        HashRing::with_hasher(nodes, replicas, XxHash64Hasher::default())
    }
}

impl<T, S> HashRing<T, S>
where
    T: Hash + Clone,
    S: BuildHasher,
{
    pub fn with_hasher(nodes: Vec<T>, replicas: usize, hash_builder: S) -> HashRing<T, S> {
        let mut new_hash_ring: HashRing<T, S> = HashRing {
            replicas,
            ring: HashMap::new(),
            sorted_keys: Vec::new(),
            hash_builder,
        };

        for n in &nodes {
            new_hash_ring.add_node(n);
        }
        new_hash_ring
    }

    /// Adds a node to the hash ring
    pub fn add_node(&mut self, node: &T) {
        let mut temp = vec![];

        mem::swap(&mut temp, &mut self.sorted_keys);

        let mut heap = BinaryHeap::from(temp);

        for i in 0..self.replicas {
            let key = self.gen_key((node, i));
            self.ring.insert(key, (*node).clone());
            heap.push(key);
        }

        self.sorted_keys = heap.into_sorted_vec();
    }

    /// Deletes a node from the hash ring
    pub fn remove_node(&mut self, node: &T) {
        for i in 0..self.replicas {
            let key = self.gen_key((node, i));

            if self.ring.remove(&key).is_none() {
                return;
            }

            if let Ok(found) = self.sorted_keys.binary_search(&key) {
                self.sorted_keys.remove(found);
            }
        }
    }

    /// Gets the node a specific key belongs to
    pub fn get_node<K: Hash>(&self, key: &K) -> Option<&T> {
        if self.sorted_keys.is_empty() {
            return None;
        }

        let generated_key = self.hash_builder.hash_one(key);

        // let nodes = self.sorted_keys.clone();

        // for node in &nodes {
        //     if generated_key <= *node {
        //         return Some(self.ring.get(node).unwrap());
        //     }
        // }

        // let node = &nodes[0];

        // returns either index of found hash, or index where it should be inserted as Err
        // say ring has 2 5 9 12 15 and we look for 10, it should return index 3 (value 12)
        // if theres an exact match, say we look for 9, it should return index 2 (value 9)
        let node_index = self
            .sorted_keys
            .binary_search(&generated_key)
            .unwrap_or_else(|x| x);

        let node = self.sorted_keys[node_index];

        return Some(self.ring.get(&node).unwrap());
    }

    /// Generates a key from a string value
    fn gen_key(&self, key: (&T, usize)) -> u64 {
        let mut hasher = self.hash_builder.build_hasher();
        key.0.hash(&mut hasher);
        key.1.hash(&mut hasher);
        hasher.finish()
    }
}

#[cfg(test)]
mod test {
    use std::hash::BuildHasherDefault;
    use std::hash::Hasher;

    use crate::HashRing;

    #[derive(Clone, Debug, PartialEq, Hash)]
    struct NodeInfo {
        host: &'static str,
        port: u16,
    }

    // Defines a NodeInfo for a localhost address with a given port.
    fn node(port: u16) -> NodeInfo {
        NodeInfo {
            host: "localhost",
            port,
        }
    }

    #[test]
    fn test_empty_ring() {
        let hash_ring: HashRing<NodeInfo> = HashRing::new(vec![], 10);
        assert_eq!(None, hash_ring.get_node(&"hello"));
    }

    #[test]
    fn test_default_nodes() {
        let mut nodes: Vec<NodeInfo> = Vec::new();
        nodes.push(node(15324));
        nodes.push(node(15325));
        nodes.push(node(15326));
        nodes.push(node(15327));
        nodes.push(node(15328));
        nodes.push(node(15329));

        let mut hash_ring: HashRing<NodeInfo> = HashRing::new(nodes, 10);

        assert_eq!(Some(&node(15324)), hash_ring.get_node(&"two"));
        assert_eq!(Some(&node(15325)), hash_ring.get_node(&"seven"));
        assert_eq!(Some(&node(15326)), hash_ring.get_node(&"hello"));
        assert_eq!(Some(&node(15327)), hash_ring.get_node(&"dude"));
        assert_eq!(Some(&node(15328)), hash_ring.get_node(&"fourteen"));
        assert_eq!(Some(&node(15329)), hash_ring.get_node(&"five"));

        hash_ring.remove_node(&node(15329));
        assert_eq!(Some(&node(15326)), hash_ring.get_node(&"hello"));

        hash_ring.add_node(&node(15329));
        assert_eq!(Some(&node(15326)), hash_ring.get_node(&"hello"));
    }

    #[derive(Clone, Hash, Debug)]
    struct CustomNodeInfo {
        pub host: &'static str,
        pub port: u16,
    }

    impl ToString for CustomNodeInfo {
        fn to_string(&self) -> String {
            format!("{}:{}", self.host, self.port)
        }
    }

    #[test]
    fn test_custom_nodes() {
        let mut nodes: Vec<CustomNodeInfo> = Vec::new();
        nodes.push(CustomNodeInfo {
            host: "localhost",
            port: 15324,
        });
        nodes.push(CustomNodeInfo {
            host: "localhost",
            port: 15325,
        });
        nodes.push(CustomNodeInfo {
            host: "localhost",
            port: 15326,
        });
        nodes.push(CustomNodeInfo {
            host: "localhost",
            port: 15327,
        });
        nodes.push(CustomNodeInfo {
            host: "localhost",
            port: 15328,
        });
        nodes.push(CustomNodeInfo {
            host: "localhost",
            port: 15329,
        });

        let mut hash_ring: HashRing<CustomNodeInfo> = HashRing::new(nodes, 10);

        assert_eq!(
            Some("localhost:15326".to_string()),
            hash_ring.get_node(&"hello").map(|x| x.to_string(),)
        );
        assert_eq!(
            Some("localhost:15327".to_string()),
            hash_ring.get_node(&"dude").map(|x| x.to_string(),)
        );

        hash_ring.remove_node(&CustomNodeInfo {
            host: "localhost",
            port: 15329,
        });
        assert_eq!(
            Some("localhost:15326".to_string()),
            hash_ring.get_node(&"hello").map(|x| x.to_string(),)
        );

        hash_ring.add_node(&CustomNodeInfo {
            host: "localhost",
            port: 15329,
        });
        assert_eq!(
            Some("localhost:15326".to_string()),
            hash_ring.get_node(&"hello").map(|x| x.to_string(),)
        );
    }

    #[test]
    fn test_remove_actual_node() {
        let mut nodes: Vec<NodeInfo> = Vec::new();
        nodes.push(node(15324));
        nodes.push(node(15325));
        nodes.push(node(15326));
        nodes.push(node(15327));
        nodes.push(node(15328));
        nodes.push(node(15329));

        let mut hash_ring: HashRing<NodeInfo> = HashRing::new(nodes, 10);

        // This should be num nodes * num replicas
        assert_eq!(60, hash_ring.sorted_keys.len());
        assert_eq!(60, hash_ring.ring.len());

        hash_ring.remove_node(&node(15326));

        // This should be num nodes * num replicas
        assert_eq!(50, hash_ring.sorted_keys.len());
        assert_eq!(50, hash_ring.ring.len());
    }

    #[test]
    fn test_remove_non_existent_node() {
        let mut nodes: Vec<NodeInfo> = Vec::new();
        nodes.push(node(15324));
        nodes.push(node(15325));
        nodes.push(node(15326));
        nodes.push(node(15327));
        nodes.push(node(15328));
        nodes.push(node(15329));

        let mut hash_ring: HashRing<NodeInfo> = HashRing::new(nodes, 10);

        hash_ring.remove_node(&node(15330));

        // This should be num nodes * num replicas
        assert_eq!(60, hash_ring.sorted_keys.len());
        assert_eq!(60, hash_ring.ring.len());
    }

    #[test]
    fn test_custom_hasher() {
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

        let mut nodes: Vec<NodeInfo> = Vec::new();
        nodes.push(node(15324));
        nodes.push(node(15325));
        nodes.push(node(15326));
        nodes.push(node(15327));
        nodes.push(node(15328));
        nodes.push(node(15329));

        let hash_ring: HashRing<NodeInfo, ConstantBuildHasher> =
            HashRing::with_hasher(nodes, 10, ConstantBuildHasher::default());

        assert_eq!(Some(&node(15329)), hash_ring.get_node(&"hello"));
        assert_eq!(Some(&node(15329)), hash_ring.get_node(&"dude"));
        assert_eq!(Some(&node(15329)), hash_ring.get_node(&"two"));
    }
}

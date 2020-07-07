use colored::*;
#[macro_use]
use hex_literal::hex;
use hash_db::{AsHashDB, HashDB, HashDBRef, Hasher as HashDBHasher, Prefix};
use rocksdb::{IteratorMode, Options, DB};
use sp_std;
use trie::node_codec::NodeCodec;
use trie_db::{
    node::{Node, NodeHandle, NodePlan},
    TrieDB, TrieDBNodeIterator, TrieLayout,
};

pub mod blake2 {
    use hash_db::Hasher;
    use std::hash::Hasher as StdHasherTrait;

    #[derive(Default)]
    pub struct StdHasher {}
    impl StdHasherTrait for StdHasher {
        #[inline]
        fn finish(&self) -> u64 {
            unimplemented!("finish of StdHasherTrait unimplement");
        }

        #[inline]
        #[allow(unused_assignments)]
        fn write(&mut self, bytes: &[u8]) {
            unimplemented!("write of StdHasherTrait unimplement");
        }
    }

    #[derive(Debug)]
    pub struct Blake2Hasher;

    impl Hasher for Blake2Hasher {
        type Out = [u8; 32];
        type StdHasher = StdHasher;
        const LENGTH: usize = 32;

        fn hash(x: &[u8]) -> Self::Out {
            unimplemented!("hash of Hasher unimplement");
        }
    }
}

type Hash = [u8; 32];
type Hasher = crate::blake2::Blake2Hasher;

pub struct SimpleTrie<'a> {
    pub db: DB,
    pub cfs: Vec<&'a str>,
}

impl<'a> AsHashDB<Hasher, Vec<u8>> for SimpleTrie<'a> {
    fn as_hash_db(&self) -> &dyn hash_db::HashDB<Hasher, Vec<u8>> {
        &*self
    }

    fn as_hash_db_mut<'b>(&'b mut self) -> &'b mut (dyn HashDB<Hasher, Vec<u8>> + 'b) {
        &mut *self
    }
}

impl<'a> HashDB<Hasher, Vec<u8>> for SimpleTrie<'a> {
    fn get(&self, key: &Hash, prefix: Prefix) -> Option<Vec<u8>> {
        // TODO: handle Prefix
        for cf in self.cfs.clone() {
            let h = self.db.cf_handle(cf).unwrap();
            if let Some(v) = self.db.get_cf(h, &key).unwrap() {
                return Some(v);
            }
        }
        None
    }

    fn contains(&self, hash: &Hash, prefix: Prefix) -> bool {
        // TODO: handle Prefix
        for cf in self.cfs.clone() {
            let h = self.db.cf_handle(cf).unwrap();
            if self.db.get_cf(h, &hash).unwrap().is_some() {
                return true;
            }
        }
        false
    }

    fn insert(&mut self, prefix: Prefix, value: &[u8]) -> Hash {
        unimplemented!("insert of HashDB unimplement");
    }

    fn emplace(&mut self, key: Hash, prefix: Prefix, value: Vec<u8>) {
        unimplemented!("emplace of HashDB unimplement");
    }

    fn remove(&mut self, key: &Hash, prefix: Prefix) {
        unimplemented!("remove of HashDB unimplement");
    }
}

impl<'a> HashDBRef<Hasher, Vec<u8>> for SimpleTrie<'a> {
    fn get(&self, key: &Hash, prefix: Prefix) -> Option<Vec<u8>> {
        // TODO: handle Prefix
        println!("get prefix: {:?} key({}): {:?}", prefix, key.len(), key);
        for cf in self.cfs.clone() {
            let h = self.db.cf_handle(cf).unwrap();
            if let Some(v) = self.db.get_cf(h, &key).unwrap() {
                return Some(v);
            }
        }
        None
    }

    fn contains(&self, hash: &Hash, prefix: Prefix) -> bool {
        // TODO: handle Prefix
        println!(
            "contains prefix: {:?} key({}): {:?}",
            prefix,
            hash.len(),
            hash
        );
        for cf in self.cfs.clone() {
            let h = self.db.cf_handle(cf).unwrap();
            if self.db.get_cf(h, &hash).unwrap().is_some() {
                return true;
            }
        }
        false
    }
}

pub struct Layout<H>(sp_std::marker::PhantomData<H>);

impl<H: HashDBHasher> TrieLayout for Layout<H> {
    const USE_EXTENSION: bool = false;
    type Hash = H;
    type Codec = NodeCodec<Self::Hash>;
}
fn setup_db_connection() -> (DB, Vec<&'static str>) {
    let opts = Options::default();
    let cfs = vec![
        "default", "col0", "col1", "col2", "col3", "col4", "col5", "col6", "col7", "col8", "col9",
        "col10",
    ];
    let db = DB::open_cf(&opts, "./db", cfs.clone()).unwrap();
    (db, cfs)
}

fn main() {
    let state_root = &hex!("09c0a468b841682c4cf29297408fadba23329fb7c0c5c81163c40f28f5caa5cd");
    {
        println!(
            "{}",
            "We have state root, and the db, let's pull this node out".cyan()
        );
        let (db, cfs) = setup_db_connection();
        // [9, 192, 164, 104, 184, 65, 104, 44, 76, 242, 146, 151, 64, 143, 173, 186, 35, 50, 159, 183, 192, 197, 200, 17, 99, 196, 15, 40, 245, 202, 165, 205]

        let extrinsics_root =
            &hex!("b8946898950fe86044251bf4b9e0a71c0304d119fdee180ab6129059c698dbd1");

        for cf in cfs.iter() {
            let h = db.cf_handle(cf).unwrap();
            let iter = db.iterator_cf(h, IteratorMode::Start);
            for (key, value) in iter {
                if *key == state_root[..] {
                    println!("Got state root hash in column family {}", cf);
                    println!("State Root {:?} \n {:?}", key, value);
                };

                // 128 is the seperator and contain 6 hash
                // [
                // 128,
                // 44, 152,
                // 128,
                // 195, 162, 207, 123, 55, 18, 222, 77, 1, 22, 17, 199, 219, 213, 251, 6, 219, 238, 93, 205, 228, 57, 200, 241, 174, 67, 28, 169, 56, 67, 133, 227,
                // 128,
                // 163, 1, 165, 139, 133, 248, 193, 151, 249, 13, 162, 27, 86, 16, 212, 200, 254, 39, 224, 160, 190, 105, 225, 221, 19, 7, 246, 109, 6, 202, 107, 194,
                // 128,
                // 183, 207, 190, 0, 154, 221, 23, 122, 61, 23, 200, 77, 4, 206, 177, 67, 75, 70, 146, 247, 160, 60, 44, 1, 193, 112, 28, 200, 207, 30, 252, 13,
                // 128,
                // 169, 211, 94, 35, 16, 145, 205, 137, 159, 42, 39, 155, 8, 205, 23, 49, 194, 6, 0, 48, 123, 252, 28, 183, 111, 148, 14, 163, 184, 197, 136, 197,
                // 128,
                // 129, 101, 66, 162, 246, 165, 217, 92, 249, 6, 252, 250, 25, 216, 48, 38, 83, 144, 124, 58, 177, 32, 205, 61, 138, 219, 219, 23, 15, 86, 157, 170,
                // 128,
                // 14, 118, 239, 35, 99, 149, 106, 122, 22, 52, 150, 153, 107, 252, 84, 152, 196, 146, 67, 247, 231, 69, 60, 230, 252, 236, 175, 161, 113, 48, 23, 153
                // ]

                if *key == extrinsics_root[..] {
                    println!("Got extrinsic root hash in column family {}", cf);
                    println!("Extrinsic Root {:?} \n {:?}", key, value);
                };
            }
        }
    }
    println!("==================================================================================");
    {
        println!("{}", "Decode the state node".cyan());
        let (db, cfs) = setup_db_connection();
        let simple_trie = SimpleTrie {
            db,
            cfs: cfs.clone(),
        };

        let trie = TrieDB::<Layout<Hasher>>::new(&simple_trie, state_root).unwrap();
        let mut iter = TrieDBNodeIterator::new(&trie).unwrap();

        let node = iter.next();
        // Following code show the root node, and Hash structure use range to get the data as real hash
        if let Some(n) = node {
            let n = n.unwrap();
            println!("= Nibble Vec: {:?}", n.0);
            println!("= NodeKey: {:?}", n.1); // Exactly the Hash for the node
            let owned_node = n.2.node();
            println!("= owned_node: {:?}", owned_node);
            let node_plan = n.2.node_plan();
            match node_plan {
                NodePlan::Branch { children, value } => {
                    println!("== Branch");
                    println!("== value: {:?}", value);
                    for c in children {
                        if let Some(h) = c {
                            println!("== child: {:?}", h);
                        }
                    }
                }
                NodePlan::NibbledBranch {
                    partial,
                    children,
                    value,
                } => {
                    println!("== NibbleBranch");
                    println!("== partial: {:?}", partial);
                    println!("== value: {:?}", value);
                    for c in children {
                        if let Some(h) = c {
                            println!("== child: {:?}", h);
                        }
                    }
                }
                _ => {}
            }
        }
        // Here is the decoded output, there are 6 child node have data
        // [None, None, Hash, Hash, None, Hash, None, None, None, None, None, Hash, Hash, None, None, Hash]
        // == child: Hash(4..36)		Hash([195, 162, 207, 123, 55, 18, 222, 77, 1, 22, 17, 199, 219, 213, 251, 6, 219, 238, 93, 205, 228, 57, 200, 241, 174, 67, 28, 169, 56, 67, 133, 227]),
        //								c3a2cf7b3712de4d011611c7dbd5fb06dbee5dcde439c8f1ae431ca9384385e3
        //
        // == child: Hash(37..69)		Hash([163, 1, 165, 139, 133, 248, 193, 151, 249, 13, 162, 27, 86, 16, 212, 200, 254, 39, 224, 160, 190, 105, 225, 221, 19, 7, 246, 109, 6, 202, 107, 194])
        //								a301a58b85f8c197f90da21b5610d4c8fe27e0a0be69e1dd1307f66d06ca6bc2
        //
        // == child: Hash(70..102)		Hash([183, 207, 190, 0, 154, 221, 23, 122, 61, 23, 200, 77, 4, 206, 177, 67, 75, 70, 146, 247, 160, 60, 44, 1, 193, 112, 28, 200, 207, 30, 252, 13])
        //								b7cfbe009add177a3d17c84d04ceb1434b4692f7a03c2c01c1701cc8cf1efc0d
        //
        // == child: Hash(103..135)		Hash([169, 211, 94, 35, 16, 145, 205, 137, 159, 42, 39, 155, 8, 205, 23, 49, 194, 6, 0, 48, 123, 252, 28, 183, 111, 148, 14, 163, 184, 197, 136, 197])
        //								a9d35e231091cd899f2a279b08cd1731c20600307bfc1cb76f940ea3b8c588c5
        //
        // == child: Hash(136..168)		Hash([129, 101, 66, 162, 246, 165, 217, 92, 249, 6, 252, 250, 25, 216, 48, 38, 83, 144, 124, 58, 177, 32, 205, 61, 138, 219, 219, 23, 15, 86, 157, 170])
        //								816542a2f6a5d95cf906fcfa19d8302653907c3ab120cd3d8adbdb170f569daa
        //
        // == child: Hash(169..201)		Hash([14, 118, 239, 35, 99, 149, 106, 122, 22, 52, 150, 153, 107, 252, 84, 152, 196, 146, 67, 247, 231, 69, 60, 230, 252, 236, 175, 161, 113, 48, 23, 153])
        //								0e76ef2363956a7a163496996bfc5498c49243f7e7453ce6fcecafa171301799
    }
    println!("==================================================================================");
    {
        println!("{}", "Find out the children".cyan());
        let (db, cfs) = setup_db_connection();
        let children = vec![
            &hex!("c3a2cf7b3712de4d011611c7dbd5fb06dbee5dcde439c8f1ae431ca9384385e3"),
            &hex!("a301a58b85f8c197f90da21b5610d4c8fe27e0a0be69e1dd1307f66d06ca6bc2"),
            &hex!("b7cfbe009add177a3d17c84d04ceb1434b4692f7a03c2c01c1701cc8cf1efc0d"),
            &hex!("a9d35e231091cd899f2a279b08cd1731c20600307bfc1cb76f940ea3b8c588c5"),
            &hex!("816542a2f6a5d95cf906fcfa19d8302653907c3ab120cd3d8adbdb170f569daa"),
            &hex!("0e76ef2363956a7a163496996bfc5498c49243f7e7453ce6fcecafa171301799"),
        ];
        for cf in cfs.iter() {
            let h = db.cf_handle(cf).unwrap();
            let iter = db.iterator_cf(h, IteratorMode::Start);
            for (key, value) in iter {
                for child in children.iter() {
                    if *key == child[..] {
                        println!("Got child hash in column family {}", cf);
                        println!("child {:?} \n {:?}", key, value);
                    };
                }
            }
        }
    }
}

// Some materials to decode things from pallet
//
//  CurrentIndex get(fn current_index): SessionIndex;
//	"cec5070d609dd3497f72bde07fc96ba072763800a36a99fdfc7c10f6415f6ee6|Session|CurrentIndex",
//	 --------------------------------++++++++++++++++++++++++++++++++
//	  32 hex len -> 16 bytes
//
//	QueuedChanged: bool;
//	"cec5070d609dd3497f72bde07fc96ba09450bfa4b96a3fa7a3c8f40da6bf32e1|Session|QueuedChanged",
//	 --------------------------------
//
//	pub Now get(fn now) build(|_| 0.into()): T::Moment;
//	"f0c365c3cf59d671eb72da0e7a4113c49f1f0515f462cdcf84e0f1d6045dfcbb|Timestamp|Now",
//	 --------------------------------
//
//	DidUpdate: bool;
//	"f0c365c3cf59d671eb72da0e7a4113c4bbd108c4899964f707fdaffb82636065|Timestamp|DidUpdate",
//	 --------------------------------
//
//	 state root hash is "0x09c0a468b841682c4cf29297408fadba23329fb7c0c5c81163c40f28f5caa5cd"
//	 extrinsics root hash is "0xb8946898950fe86044251bf4b9e0a71c0304d119fdee180ab6129059c698dbd1"

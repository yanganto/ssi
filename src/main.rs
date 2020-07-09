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

//	 #5 state root hash is "0x940a55c41ce61b2d771e82f8a6c6f4939a712a644502f5efa7c59afea0a3a67e"
//	 #5 extrinsics root hash is "0xc1f78e951f26fe2c55e10f32b7bc97a227ee59274fabff18e5eabb6bee70c494"
//	 #50 state root hash is "0x3b559d574c4a9f13e55d0256655f0f71a70a703766226f1080f80022e39c057d"
//	 #50 extrinsics root hash is "0x2772dcca7b706ca5c9692cb02e373d667ab269ea9085eb55e6900584b7c2c682"
fn main() {
    let state_root_5 = &hex!("940a55c41ce61b2d771e82f8a6c6f4939a712a644502f5efa7c59afea0a3a67e");
    // [148, 10, 85, 196, 28, 230, 27, 45, 119, 30, 130, 248, 166, 198, 244, 147, 154, 113, 42, 100, 69, 2, 245, 239, 167, 197, 154, 254, 160, 163, 166, 126]
    let state_root_50 = &hex!("3b559d574c4a9f13e55d0256655f0f71a70a703766226f1080f80022e39c057d");
    // [59, 85, 157, 87, 76, 74, 159, 19, 229, 93, 2, 86, 101, 95, 15, 113, 167, 10, 112, 55, 102, 34, 111, 16, 128, 248, 0, 34, 227, 156, 5, 125]

    let extrinsics_root_5 =
        &hex!("c1f78e951f26fe2c55e10f32b7bc97a227ee59274fabff18e5eabb6bee70c494");
    let extrinsics_root_50 =
        &hex!("2772dcca7b706ca5c9692cb02e373d667ab269ea9085eb55e6900584b7c2c682");
    {
        println!(
            "{}",
            "We have state root, and the db, let's pull this node out".cyan()
        );
        let (db, cfs) = setup_db_connection();

        for cf in cfs.iter() {
            let h = db.cf_handle(cf).unwrap();
            let iter = db.iterator_cf(h, IteratorMode::Start);
            for (key, value) in iter {
                if *key == state_root_5[..] {
                    println!("Got state root hash #5 in column family {}", cf);
                    println!("State Root #5 {:?} \n {:?}", key, value);
                };

                // 128 is the seperator and contain 6 hash
                //  [
                //  128,
                //  44, 152,
                //  128,
                //  85, 114, 90, 184, 81, 124, 23, 235, 182, 170, 184, 133, 223, 93, 124, 251, 173, 165, 190, 146, 234, 244, 217, 33, 222, 72, 35, 215, 116, 144, 19, 36,
                //  128,
                //  254, 64, 112, 194, 32, 25, 207, 40, 249, 31, 246, 143, 125, 54, 211, 81, 35, 61, 4, 132, 72, 140, 41, 106, 142, 111, 56, 156, 57, 89, 131, 20,
                //  128,
                //  192, 98, 139, 161, 117, 76, 9, 117, 55, 27, 157, 185, 121, 240, 247, 152, 207, 145, 160, 222, 132, 194, 154, 32, 234, 200, 227, 208, 108, 197, 178, 7,
                //  128,
                //  49, 239, 154, 236, 197, 182, 244, 198, 173, 189, 166, 59, 187, 46, 204, 215, 78, 31, 59, 29, 95, 55, 237, 98, 113, 164, 48, 198, 16, 219, 132, 25,
                //  128,
                //  5, 186, 0, 144, 71, 86, 173, 65, 68, 101, 0, 237, 69, 84, 150, 46, 111, 109, 136, 206, 86, 88, 156, 177, 9, 20, 70, 100, 251, 208, 141, 121,
                //  128,
                //  160, 117, 69, 22, 65, 224, 115, 49, 4, 235, 92, 183, 242, 80, 47, 241, 125, 66, 246, 68, 19, 116, 151, 54, 185, 234, 44, 248, 13, 22, 45, 193
                //  ]
                if *key == state_root_50[..] {
                    println!("Got state root hash #50 in column family {}", cf);
                    println!("State Root #50 {:?} \n {:?}", key, value);
                };
                // [
                // 128,
                // 44, 152,
                // 128,
                // 94, 160, 29, 150, 47, 206, 102, 184, 44, 189, 162, 219, 238, 99, 92, 175, 155, 113, 164, 89, 83, 126, 3, 28, 142, 179, 123, 67, 223, 54, 236, 110,
                // 128,
                // 254, 64, 112, 194, 32, 25, 207, 40, 249, 31, 246, 143, 125, 54, 211, 81, 35, 61, 4, 132, 72, 140, 41, 106, 142, 111, 56, 156, 57, 89, 131, 20,
                // 128,
                // 186, 239, 159, 204, 255, 176, 65, 173, 46, 89, 234, 229, 224, 173, 99, 171, 160, 247, 64, 36, 224, 190, 171, 44, 238, 243, 103, 85, 219, 220, 88, 231,
                // 128,
                // 25, 0, 97, 20, 123, 121, 139, 175, 63, 105, 174, 89, 206, 29, 35, 156, 173, 24, 48, 50, 136, 242, 207, 8, 72, 46, 144, 52, 18, 133, 232, 112,
                // 128,
                // 129, 101, 66, 162, 246, 165, 217, 92, 249, 6, 252, 250, 25, 216, 48, 38, 83, 144, 124, 58, 177, 32, 205, 61, 138, 219, 219, 23, 15, 86, 157, 170,
                // 128,
                // 86, 19, 59, 211, 244,
                // 128,
                // 21, 23, 250, 142, 131, 23, 178, 43, 217, 200, 158, 181, 223, 190, 119, 217, 26, 9, 179, 225, 71, 97, 160, 254, 75, 108]

                if *key == extrinsics_root_5[..] {
                    println!("Got extrinsic root hash #5 in column family {}", cf);
                    println!("Extrinsic Root #5 {:?} \n {:?}", key, value);
                };
                // not found
                if *key == extrinsics_root_50[..] {
                    println!("Got extrinsic root hash #50 in column family {}", cf);
                    println!("Extrinsic Root #50 {:?} \n {:?}", key, value);
                };
                // not found
            }
        }
    }
    println!("==================================================================================");
    {
        println!("{}", "Decode the state node for #5".cyan());
        let (db, cfs) = setup_db_connection();
        let simple_trie = SimpleTrie {
            db,
            cfs: cfs.clone(),
        };

        let trie = TrieDB::<Layout<Hasher>>::new(&simple_trie, state_root_5).unwrap();
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
        //
        // [None, None, Hash, Hash, None, Hash, None, None, None, None, None, Hash, Hash, None, None, Hash]
        //    0     1     2*    3*    4     5*    6     7     8     9     a     b*    c*    d     e     f*
        //
        // == NibbleBranch
        // == partial: NibbleSlicePlan { bytes: 1..1, offset: 0 }
        // == value: None
        // == child: Hash(4..36)    [94, 160, 29, 150, 47, 206, 102, 184, 44, 189, 162, 219, 238, 99, 92, 175, 155, 113, 164, 89, 83, 126, 3, 28, 142, 179, 123, 67, 223, 54, 236, 110]
        // == child: Hash(37..69)   [254, 64, 112, 194, 32, 25, 207, 40, 249, 31, 246, 143, 125, 54, 211, 81, 35, 61, 4, 132, 72, 140, 41, 106, 142, 111, 56, 156, 57, 89, 131, 20]
        // == child: Hash(70..102)  [186, 239, 159, 204, 255, 176, 65, 173, 46, 89, 234, 229, 224, 173, 99, 171, 160, 247, 64, 36, 224, 190, 171, 44, 238, 243, 103, 85, 219, 220, 88, 231]
        // == child: Hash(103..135) [25, 0, 97, 20, 123, 121, 139, 175, 63, 105, 174, 89, 206, 29, 35, 156, 173, 24, 48, 50, 136, 242, 207, 8, 72, 46, 144, 52, 18, 133, 232, 112]
        // == child: Hash(136..168) [129, 101, 66, 162, 246, 165, 217, 92, 249, 6, 252, 250, 25, 216, 48, 38, 83, 144, 124, 58, 177, 32, 205, 61, 138, 219, 219, 23, 15, 86, 157, 170]
        // == child: Hash(169..201) [86, 19, 59, 211, 244, 128, 21, 23, 250, 142, 131, 23, 178, 43, 217, 200, 158, 181, 223, 190, 119, 217, 26, 9, 179, 225, 71, 97, 160, 254, 75, 108]
    }
    {
        println!("{}", "Decode the state node for #50".cyan());
        let (db, cfs) = setup_db_connection();
        let simple_trie = SimpleTrie {
            db,
            cfs: cfs.clone(),
        };

        let trie = TrieDB::<Layout<Hasher>>::new(&simple_trie, state_root_50).unwrap();
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
        //
        // [None, None, Hash, Hash, None, Hash, None, None, None, None, None, Hash, Hash, None, None, Hash]
        //    0     1     2*    3*    4     5*    6     7     8     9     a     b*    c*    d     e     f*
        // == NibbleBranch
        // == partial: NibbleSlicePlan { bytes: 1..1, offset: 0 }
        // == value: None
        // == child: Hash(4..36)    [85, 114, 90, 184, 81, 124, 23, 235, 182, 170, 184, 133, 223, 93, 124, 251, 173, 165, 190, 146, 234, 244, 217, 33, 222, 72, 35, 215, 116, 144, 19, 36]
        // == child: Hash(37..69)   [254, 64, 112, 194, 32, 25, 207, 40, 249, 31, 246, 143, 125, 54, 211, 81, 35, 61, 4, 132, 72, 140, 41, 106, 142, 111, 56, 156, 57, 89, 131, 20] (no change from #5 - #50)
        // == child: Hash(70..102)  [192, 98, 139, 161, 117, 76, 9, 117, 55, 27, 157, 185, 121, 240, 247, 152, 207, 145, 160, 222, 132, 194, 154, 32, 234, 200, 227, 208, 108, 197, 178, 7]
        // == child: Hash(103..135) [49, 239, 154, 236, 197, 182, 244, 198, 173, 189, 166, 59, 187, 46, 204, 215, 78, 31, 59, 29, 95, 55, 237, 98, 113, 164, 48, 198, 16, 219, 132, 25]
        // == child: Hash(136..168) [5, 186, 0, 144, 71, 86, 173, 65, 68, 101, 0, 237, 69, 84, 150, 46, 111, 109, 136, 206, 86, 88, 156, 177, 9, 20, 70, 100, 251, 208, 141, 121]
        // == child: Hash(169..201) [160, 117, 69, 22, 65, 224, 115, 49, 4, 235, 92, 183, 242, 80, 47, 241, 125, 66, 246, 68, 19, 116, 151, 54, 185, 234, 44, 248, 13, 22, 45, 193]
    }
    println!("==================================================================================");
    {
        println!("{}", "Find out the children".cyan());
        let (db, cfs) = setup_db_connection();
        let children = vec![
            //#5
            &[
                94, 160, 29, 150, 47, 206, 102, 184, 44, 189, 162, 219, 238, 99, 92, 175, 155, 113,
                164, 89, 83, 126, 3, 28, 142, 179, 123, 67, 223, 54, 236, 110,
            ],
            &[
                254, 64, 112, 194, 32, 25, 207, 40, 249, 31, 246, 143, 125, 54, 211, 81, 35, 61, 4,
                132, 72, 140, 41, 106, 142, 111, 56, 156, 57, 89, 131, 20,
            ],
            &[
                186, 239, 159, 204, 255, 176, 65, 173, 46, 89, 234, 229, 224, 173, 99, 171, 160,
                247, 64, 36, 224, 190, 171, 44, 238, 243, 103, 85, 219, 220, 88, 231,
            ],
            &[
                25, 0, 97, 20, 123, 121, 139, 175, 63, 105, 174, 89, 206, 29, 35, 156, 173, 24, 48,
                50, 136, 242, 207, 8, 72, 46, 144, 52, 18, 133, 232, 112,
            ],
            &[
                129, 101, 66, 162, 246, 165, 217, 92, 249, 6, 252, 250, 25, 216, 48, 38, 83, 144,
                124, 58, 177, 32, 205, 61, 138, 219, 219, 23, 15, 86, 157, 170,
            ],
            &[
                86, 19, 59, 211, 244, 128, 21, 23, 250, 142, 131, 23, 178, 43, 217, 200, 158, 181,
                223, 190, 119, 217, 26, 9, 179, 225, 71, 97, 160, 254, 75, 108,
            ],
            // #50
            &[
                85, 114, 90, 184, 81, 124, 23, 235, 182, 170, 184, 133, 223, 93, 124, 251, 173,
                165, 190, 146, 234, 244, 217, 33, 222, 72, 35, 215, 116, 144, 19, 36,
            ],
            // &[254, 64, 112, 194, 32, 25, 207, 40, 249, 31, 246, 143, 125, 54, 211, 81, 35, 61, 4, 132, 72, 140, 41, 106, 142, 111, 56, 156, 57, 89, 131, 20]
            &[
                192, 98, 139, 161, 117, 76, 9, 117, 55, 27, 157, 185, 121, 240, 247, 152, 207, 145,
                160, 222, 132, 194, 154, 32, 234, 200, 227, 208, 108, 197, 178, 7,
            ],
            &[
                49, 239, 154, 236, 197, 182, 244, 198, 173, 189, 166, 59, 187, 46, 204, 215, 78,
                31, 59, 29, 95, 55, 237, 98, 113, 164, 48, 198, 16, 219, 132, 25,
            ],
            &[
                5, 186, 0, 144, 71, 86, 173, 65, 68, 101, 0, 237, 69, 84, 150, 46, 111, 109, 136,
                206, 86, 88, 156, 177, 9, 20, 70, 100, 251, 208, 141, 121,
            ],
            &[
                160, 117, 69, 22, 65, 224, 115, 49, 4, 235, 92, 183, 242, 80, 47, 241, 125, 66,
                246, 68, 19, 116, 151, 54, 185, 234, 44, 248, 13, 22, 45, 193,
            ],
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

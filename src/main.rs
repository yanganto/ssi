#[macro_use]
use hex_literal::hex;
use hash_db::{AsHashDB, HashDB, HashDBRef, Prefix};
use rocksdb::{IteratorMode, Options, DB};
use std::sync::Arc;
use trie_db::{TrieDB, TrieDBNodeIterator, TrieLayout};

pub mod blake2 {
    use hash_db::Hasher;
    use std::hash::Hasher as StdHasherTrait;

    #[derive(Default)]
    pub struct StdHasher {}
    impl StdHasherTrait for StdHasher {
        #[inline]
        fn finish(&self) -> u64 {
            unimplemented!();
        }

        #[inline]
        #[allow(unused_assignments)]
        fn write(&mut self, bytes: &[u8]) {
            unimplemented!();
        }
    }

    /// Concrete implementation of Hasher using Blake2b 256-bit hashes
    #[derive(Debug)]
    pub struct Blake2Hasher;

    impl Hasher for Blake2Hasher {
        type Out = [u8; 32];
        type StdHasher = StdHasher;
        const LENGTH: usize = 32;

        fn hash(x: &[u8]) -> Self::Out {
            unimplemented!();
        }
    }
}

type Hash = [u8; 32];
type Hasher = crate::blake2::Blake2Hasher;

pub struct SimpleTrie {
    pub db: Arc<DB>,
}

impl<'a> AsHashDB<Hasher, Vec<u8>> for SimpleTrie {
    fn as_hash_db(&self) -> &dyn hash_db::HashDB<Hasher, Vec<u8>> {
        &*self
    }

    fn as_hash_db_mut<'b>(&'b mut self) -> &'b mut (dyn HashDB<Hasher, Vec<u8>> + 'b) {
        &mut *self
    }
}

impl<'a> HashDB<Hasher, Vec<u8>> for SimpleTrie {
    fn get(&self, key: &Hash, prefix: Prefix) -> Option<Vec<u8>> {
        self.db.get(&key).expect("Database error")
    }

    fn contains(&self, hash: &Hash, prefix: Prefix) -> bool {
        //TODO:
        // self.get(hash, prefix).is_some()
        unimplemented!();
    }

    fn insert(&mut self, prefix: Prefix, value: &[u8]) -> Hash {
        unimplemented!();
    }

    fn emplace(&mut self, key: Hash, prefix: Prefix, value: Vec<u8>) {
        unimplemented!();
    }

    fn remove(&mut self, key: &Hash, prefix: Prefix) {
        unimplemented!();
    }
}

impl HashDBRef<Hasher, Vec<u8>> for SimpleTrie {
    fn get(&self, key: &Hash, prefix: Prefix) -> Option<Vec<u8>> {
        self.db.get(&key).expect("Database error")
    }

    fn contains(&self, hash: &Hash, prefix: Prefix) -> bool {
        //TODO:
        // self.get(hash, prefix).is_some()
        unimplemented!();
    }
}

// dyn hash_db::HashDBRef<_, std::vec::Vec<u8>>
// impl HashDBRef<_, std::vec::Vec<u8>> for DB {}
//  hash_db::HashDBRef<_, std::vec::Vec<u8>>`

fn main() {
    let mut opts = Options::default();
    let cfs = vec![
        "default", "col0", "col1", "col2", "col3", "col4", "col5", "col6", "col7", "col8", "col9",
        "col10",
    ];
    let db = DB::open_cf(&opts, "./db", cfs.clone()).unwrap();

    let state_root = &hex!("09c0a468b841682c4cf29297408fadba23329fb7c0c5c81163c40f28f5caa5cd")[..];
    let extrinsics_root =
        &hex!("b8946898950fe86044251bf4b9e0a71c0304d119fdee180ab6129059c698dbd1")[..];

    for cf in cfs.iter() {
        let h = db.cf_handle(cf).unwrap();
        let iter = db.iterator_cf(h, IteratorMode::Start);
        for (key, value) in iter {
            if *key == *state_root {
                println!("Got state root hash in column family {}", cf);
                println!("Saw {:?} \n {:?}", key, value);
            };
            if *key == *extrinsics_root {
                println!("Got extrinsic root hash in column family {}", cf);
                println!("Saw {:?} \n {:?}", key, value);
            };
        }
    }

    let simple_trie = SimpleTrie { db: Arc::new(db) };

    // TODO Fix TrieLayout
    // let trie = TrieDB::<_, TrieLayout>::new(&simple_trie, &state_root).unwrap();
    // let mut iter = trie_db::TrieDBNodeIterator::new(&trie).unwrap();
}

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

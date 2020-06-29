#[macro_use]
use hex_literal::hex;
use hash_db::{AsHashDB, HashDB, HashDBRef, Hasher as HashDBHasher, Prefix};
use rocksdb::{IteratorMode, Options, DB};
use sp_std;
use trie::node_codec::NodeCodec;
use trie_db::{TrieDB, TrieDBNodeIterator, TrieLayout};

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
}

pub struct Layout<H>(sp_std::marker::PhantomData<H>);

impl<H: HashDBHasher> TrieLayout for Layout<H> {
    const USE_EXTENSION: bool = false;
    type Hash = H;
    type Codec = NodeCodec<Self::Hash>;
}

fn main() {
    let opts = Options::default();
    let cfs = vec![
        "default", "col0", "col1", "col2", "col3", "col4", "col5", "col6", "col7", "col8", "col9",
        "col10",
    ];
    let db = DB::open_cf(&opts, "./db", cfs.clone()).unwrap();

    let state_root = &hex!("09c0a468b841682c4cf29297408fadba23329fb7c0c5c81163c40f28f5caa5cd");
    let extrinsics_root = &hex!("b8946898950fe86044251bf4b9e0a71c0304d119fdee180ab6129059c698dbd1");

    for cf in cfs.iter() {
        let h = db.cf_handle(cf).unwrap();
        let iter = db.iterator_cf(h, IteratorMode::Start);
        for (key, value) in iter {
            if *key == state_root[..] {
                println!("Got state root hash in column family {}", cf);
                println!("Saw {:?} \n {:?}", key, value);
            };
            if *key == extrinsics_root[..] {
                println!("Got extrinsic root hash in column family {}", cf);
                println!("Saw {:?} \n {:?}", key, value);
            };
        }
    }

    let simple_trie = SimpleTrie { db, cfs };

    let trie = TrieDB::<Layout<Hasher>>::new(&simple_trie, state_root).unwrap();
    let mut iter = TrieDBNodeIterator::new(&trie).unwrap();
    loop {
        let item = iter.next();
        if item.is_none() {
            break;
        }
        println!("{:?}", item);
    }
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

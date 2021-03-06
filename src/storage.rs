/// Storage
/// implement import trait to read the storage,
/// such that this tool can ultilize the TireNodeIterate
use hash_db::{AsHashDB, HashDB, HashDBRef, Hasher as HashDBHasher, Prefix};
use rocksdb::{IteratorMode, Options, DB};
use sp_trie::node_codec::NodeCodec;
use trie_db::TrieLayout;

use crate::logger::{debug, trace};

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
        fn write(&mut self, _bytes: &[u8]) {
            unimplemented!("write of StdHasherTrait unimplement");
        }
    }

    #[derive(Debug)]
    pub struct Blake2Hasher;

    impl Hasher for Blake2Hasher {
        type Out = [u8; 32];
        type StdHasher = StdHasher;
        const LENGTH: usize = 32;

        fn hash(_x: &[u8]) -> Self::Out {
            unimplemented!("hash of Hasher unimplement");
        }
    }
}

type Hash = [u8; 32];
pub type Hasher = crate::storage::blake2::Blake2Hasher;

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
        trace!("get prefix: {:?}, key({}): {:?}", prefix, key.len(), key);
        let key: Vec<u8> = if !prefix.0.is_empty() || prefix.1.is_some() {
            let mut k = if !prefix.0.is_empty() {
                prefix.0.to_vec()
            } else {
                Vec::<u8>::new()
            };
            if let Some(p) = prefix.1 {
                k.push(p);
            }
            k.append(&mut key.to_vec());
            trace!("Prefixed key: {:?}", k);
            k
        } else {
            trace!("key: {:?}", key);
            key.to_vec()
        };
        for cf in self.cfs.clone() {
            let h = self.db.cf_handle(cf).unwrap();
            if let Some(v) = self.db.get_cf(h, &key).unwrap() {
                return Some(v);
            }
        }
        None
    }

    fn contains(&self, hash: &Hash, prefix: Prefix) -> bool {
        trace!(
            "contains prefix: {:?} key({}): {:?}",
            prefix,
            hash.len(),
            hash
        );
        let key: Vec<u8> = if let Some(p) = prefix.1 {
            let mut k = vec![p];
            k.append(&mut hash.to_vec());
            k
        } else {
            hash.to_vec()
        };
        debug!("key: {:?}", key);
        for cf in self.cfs.clone() {
            let h = self.db.cf_handle(cf).unwrap();
            if self.db.get_cf(h, &hash).unwrap().is_some() {
                return true;
            }
        }
        false
    }

    fn insert(&mut self, _prefix: Prefix, _value: &[u8]) -> Hash {
        unimplemented!("insert of HashDB unimplement");
    }

    fn emplace(&mut self, _key: Hash, _prefix: Prefix, _value: Vec<u8>) {
        unimplemented!("emplace of HashDB unimplement");
    }

    fn remove(&mut self, _key: &Hash, _prefix: Prefix) {
        unimplemented!("remove of HashDB unimplement");
    }
}

impl<'a> HashDBRef<Hasher, Vec<u8>> for SimpleTrie<'a> {
    fn get(&self, key: &Hash, prefix: Prefix) -> Option<Vec<u8>> {
        trace!("get prefix: {:?}, key({}): {:?}", prefix, key.len(), key);
        let key: Vec<u8> = if !prefix.0.is_empty() || prefix.1.is_some() {
            let mut k = if !prefix.0.is_empty() {
                prefix.0.to_vec()
            } else {
                Vec::<u8>::new()
            };
            if let Some(p) = prefix.1 {
                k.push(p);
            }
            k.append(&mut key.to_vec());
            trace!("Prefixed key: {:?}", k);
            k
        } else {
            trace!("key: {:?}", key);
            key.to_vec()
        };
        for cf in self.cfs.clone() {
            let h = self.db.cf_handle(cf).unwrap();
            if let Some(v) = self.db.get_cf(h, &key).unwrap() {
                return Some(v);
            }
        }
        None
    }

    fn contains(&self, hash: &Hash, prefix: Prefix) -> bool {
        trace!(
            "contains prefix: {:?} key({}): {:?}",
            prefix,
            hash.len(),
            hash
        );
        let key: Vec<u8> = if let Some(p) = prefix.1 {
            let mut k = vec![p];
            k.append(&mut hash.to_vec());
            k
        } else {
            hash.to_vec()
        };
        trace!("key: {:?}", key);
        for cf in self.cfs.clone() {
            let h = self.db.cf_handle(cf).unwrap();
            if self.db.get_cf(h, &key).unwrap().is_some() {
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
pub fn setup_db_connection(db_path: &str) -> (DB, Vec<&'static str>) {
    let opts = Options::default();
    let cfs = vec![
        "default", "col0", "col1", "col2", "col3", "col4", "col5", "col6", "col7", "col8", "col9",
        "col10",
    ];

    // TODO handle this unwarp
    let db = DB::open_cf_for_read_only(&opts, db_path, cfs.clone(), false).unwrap();
    (db, cfs)
}

pub fn raw_query(db: &DB, cfs: &[&str], prefix: Prefix, node_key: [u8; 32]) -> Option<Box<[u8]>> {
    let key: Vec<u8> = if !prefix.0.is_empty() || prefix.1.is_some() {
        let mut k = if !prefix.0.is_empty() {
            prefix.0.to_vec()
        } else {
            Vec::<u8>::new()
        };
        if let Some(p) = prefix.1 {
            k.push(p);
        }
        k.append(&mut node_key.to_vec());
        k
    } else {
        node_key.to_vec()
    };
    for cf in cfs.iter() {
        let h = db.cf_handle(cf).unwrap();
        for (k, v) in db.iterator_cf(h, IteratorMode::Start) {
            if *k == key[..] {
                return Some(v);
            }
        }
    }
    None
}

/// Helper function for char to children nodes index
pub fn map_char_to_pos(c: char) -> usize {
    match c {
        '0' | '1' | '2' | '3' | '4' | '5' | '6' | '7' | '8' | '9' => {
            c.to_digit(10).unwrap() as usize
        }
        'a' | 'A' => 10,
        'b' | 'B' => 11,
        'c' | 'C' => 12,
        'd' | 'D' => 13,
        'e' | 'E' => 14,
        'f' | 'F' => 15,
        _ => panic!("hex string uncorrect"),
    }
}

/// Helper function for children nodes index to char
pub fn map_pos_to_char(p: usize) -> char {
    match p {
        0 | 1 | 2 | 3 | 4 | 5 | 6 | 7 | 8 | 9 => format!("{}", p).chars().next().unwrap(),
        10 => 'a',
        11 => 'b',
        12 => 'c',
        13 => 'd',
        14 => 'e',
        15 => 'f',
        _ => panic!("hex string uncorrect"),
    }
}

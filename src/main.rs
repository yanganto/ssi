use array_bytes::hex_bytes_unchecked;
use rocksdb::{IteratorMode, Options, DB};

fn main() {
    let mut opts = Options::default();
    let cfs = vec![
        "default", "col0", "col1", "col2", "col3", "col4", "col5", "col6", "col7", "col8", "col9",
        "col10",
    ];
    let db = DB::open_cf(&opts, "./db", cfs.clone()).unwrap();

    for cf in cfs.iter() {
        let h = db.cf_handle(cf).unwrap();
        let iter = db.iterator_cf(h, IteratorMode::Start);
        for (key, value) in iter {
            if *key
                == hex_bytes_unchecked(
                    "0x09c0a468b841682c4cf29297408fadba23329fb7c0c5c81163c40f28f5caa5cd",
                )[..]
            {
                println!("Got state root hash in column family {}", cf);
                println!("Saw {:?} {:?}", key, value);
            };
            if *key
                == hex_bytes_unchecked(
                    "0xb8946898950fe86044251bf4b9e0a71c0304d119fdee180ab6129059c698dbd1",
                )[..]
            {
                println!("Got extrinsic root hash in column family {}", cf);
                println!("Saw {:?} {:?}", key, value);
            };
        }
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

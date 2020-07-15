use std::env::args_os;

use colored::*;
#[macro_use]
use hex_literal::hex;
use trie_db::{
    node::{Node, NodeHandle, NodeHandlePlan, NodePlan},
    TrieDB, TrieDBNodeIterator, TrieLayout,
};

mod logger;
use logger::{debug, error, info, init_logger, trace, Logger};

mod storage;
use storage::{
    map_char_to_pos, map_pos_to_char, raw_query, setup_db_connection, Hasher, Layout, SimpleTrie,
};

mod cli;
use cli::parse_args;

static LOGGER: Logger = Logger;

//	Hash("System") ++ Hahs("Account") ++ Hash(Account_ID) ++ Account_ID
//	"26aa394eea5630e07c48ae0c9558cef7b99d880ec681799c0cf30e8886371da9" ++ Hash(Account_ID) ++ Account_ID
//	 #5 state root hash is "0x940a55c41ce61b2d771e82f8a6c6f4939a712a644502f5efa7c59afea0a3a67e"
//	 #5 extrinsics root hash is "0xc1f78e951f26fe2c55e10f32b7bc97a227ee59274fabff18e5eabb6bee70c494"
//	 #50 state root hash is "0x3b559d574c4a9f13e55d0256655f0f71a70a703766226f1080f80022e39c057d"
//	 #50 extrinsics root hash is "0x2772dcca7b706ca5c9692cb02e373d667ab269ea9085eb55e6900584b7c2c682"
fn main() {
    let matches = parse_args(args_os());
    init_logger(&LOGGER, matches.value_of("log").unwrap_or("error"));

    // In POC
    // We will get all system aacount info
    // Hash("System") ++ Hahs("Account") ++ Hash(Account_ID) ++ Account_ID
    let storage_key = "26aa394eea5630e07c48ae0c9558cef7b99d880ec681799c0cf30e8886371da9";
    let state_root_hash = &hex!("940a55c41ce61b2d771e82f8a6c6f4939a712a644502f5efa7c59afea0a3a67e"); // #5
                                                                                                     // let state_root_hash = &hex!("3b559d574c4a9f13e55d0256655f0f71a70a703766226f1080f80022e39c057d"); // #50
    let recurcive = true;

    // #50
    // [59, 85, 157, 87, 76, 74, 159, 19, 229, 93, 2, 86, 101, 95, 15, 113, 167, 10, 112, 55, 102, 34, 111, 16, 128, 248, 0, 34, 227, 156, 5, 125]

    info!("State Root Hash: {:?}", state_root_hash);
    info!("Storage Key: {}", storage_key);

    let storage_key: Vec<usize> = storage_key.chars().map(map_char_to_pos).collect();
    debug!("Storage Key Path: {:?}", storage_key);

    {
        let (db, cfs) = setup_db_connection();
        let (db2, _) = setup_db_connection();
        let simple_trie = SimpleTrie {
            db,
            cfs: cfs.clone(),
        };

        // TODO: handle unwarp here
        let trie = TrieDB::<Layout<Hasher>>::new(&simple_trie, state_root_hash).unwrap();
        let mut node_iter = TrieDBNodeIterator::new(&trie).unwrap();
        let mut path_iter = storage_key.iter();

        let mut target_node_key = Some(state_root_hash.to_vec());
        loop {
            let current_node = node_iter.next();
            debug!("current node: {:?}", current_node);
            if current_node.is_none() {
                break;
            }

            // TODO handle the 2nd unwrap
            let n = current_node.unwrap().unwrap();

            if n.1.is_none() {
                // some node not inspect
                let k = n.0.inner();
                debug!("Ignored Key({}): {:?}", k.len(), k);
                continue;
            }
            debug!("Key({}): {:?}", n.1.unwrap().len(), n.1.unwrap());
            if n.1.unwrap() == *target_node_key.clone().unwrap() {
                debug!("find node: {:?}", target_node_key);
                let path = path_iter.next();
                let owned_node = n.2.node();
                let node_plan = n.2.node_plan();

                // TODO refactor this
                let data = raw_query(
                    &db2,
                    &cfs,
                    n.0.as_prefix(),
                    target_node_key.clone().unwrap(),
                );
                debug!("data for {:?}: {:?}", target_node_key, data);
                let data = data.unwrap();

                if let Some(p) = path {
                    match node_plan {
                        NodePlan::NibbledBranch {
                            children,
                            value,
                            partial,
                        } => {
                            info!("Path to \"{}\"({})", map_pos_to_char(*p), p);
                            trace!("children: {:?}", children);
                            trace!("value: {:?}", value);
                            trace!("partial: {:?}", partial);

                            for _ in 0..partial.len() {
                                if let Some(p) = path_iter.next() {
                                    info!("Path to \"{}\"({})\t(partial)", map_pos_to_char(*p), p);
                                }
                            }

                            let c = children
                                .into_iter()
                                .nth(*p)
                                .expect("branch node should have this child");
                            debug!("child: {:?}", c);
                            if let Some(c) = c.clone() {
                                let h = match c {
                                    NodeHandlePlan::Hash(r) | NodeHandlePlan::Inline(r) => {
                                        data[r].to_vec()
                                    }
                                };
                                target_node_key = Some(h);
                                debug!("new target node key: {:?}", target_node_key);
                            } else {
                                error!("Path Error");
                                break;
                            }
                        }
                        _ => panic!("should not here"), //| nodeplan::nibbledbranch { children, value, partial }
                    }
                } else if recurcive {
                } else {
                    break;
                }
            }
            //        println!("= Nibble Vec: {:?}", n.0);
            //        println!("= NodeKey: {:?}", n.1); // Exactly the Hash for the node
            //        let owned_node = n.2.node();
            //        println!("= owned_node: {:?}", owned_node);
            //        let node_plan = n.2.node_plan();

            // if current_node ==
        }
    }

    //println!("==================================================================================");
    //{
    //    println!("{}", "Decode the state node for #5".cyan());
    //    let (db, cfs) = setup_db_connection();
    //    let simple_trie = SimpleTrie {
    //        db,
    //        cfs: cfs.clone(),
    //    };

    //    let trie = TrieDB::<Layout<Hasher>>::new(&simple_trie, state_root_5).unwrap();
    //    let mut iter = TrieDBNodeIterator::new(&trie).unwrap();

    //    let node = iter.next();
    //    // Following code show the root node, and Hash structure use range to get the data as real hash
    //    if let Some(n) = node {
    //        let n = n.unwrap();
    //        println!("= Nibble Vec: {:?}", n.0);
    //        println!("= NodeKey: {:?}", n.1); // Exactly the Hash for the node
    //        let owned_node = n.2.node();
    //        println!("= owned_node: {:?}", owned_node);
    //        let node_plan = n.2.node_plan();
    //        match node_plan {
    //            NodePlan::Branch { children, value } => {
    //                println!("== Branch");
    //                println!("== value: {:?}", value);
    //                for c in children {
    //                    if let Some(h) = c {
    //                        println!("== child: {:?}", h);
    //                    }
    //                }
    //            }
    //            NodePlan::NibbledBranch {
    //                partial,
    //                children,
    //                value,
    //            } => {
    //                println!("== NibbleBranch");
    //                println!("== partial: {:?}", partial);
    //                println!("== value: {:?}", value);
    //                for c in children {
    //                    if let Some(h) = c {
    //                        println!("== child: {:?}", h);
    //                    }
    //                }
    //            }
    //            _ => {}
    //        }
    //    }
    //    // Here is the decoded output, there are 6 child node have data
    //    //
    //    // [None, None, Hash, Hash, None, Hash, None, None, None, None, None, Hash, Hash, None, None, Hash]
    //    //    0     1     2*    3*    4     5*    6     7     8     9     a     b*    c*    d     e     f*
    //    //
    //    // == NibbleBranch
    //    // == partial: NibbleSlicePlan { bytes: 1..1, offset: 0 }
    //    // == value: None
    //    // == child: Hash(4..36)    [94, 160, 29, 150, 47, 206, 102, 184, 44, 189, 162, 219, 238, 99, 92, 175, 155, 113, 164, 89, 83, 126, 3, 28, 142, 179, 123, 67, 223, 54, 236, 110]
    //    // == child: Hash(37..69)   [254, 64, 112, 194, 32, 25, 207, 40, 249, 31, 246, 143, 125, 54, 211, 81, 35, 61, 4, 132, 72, 140, 41, 106, 142, 111, 56, 156, 57, 89, 131, 20]
    //    // == child: Hash(70..102)  [186, 239, 159, 204, 255, 176, 65, 173, 46, 89, 234, 229, 224, 173, 99, 171, 160, 247, 64, 36, 224, 190, 171, 44, 238, 243, 103, 85, 219, 220, 88, 231]
    //    // == child: Hash(103..135) [25, 0, 97, 20, 123, 121, 139, 175, 63, 105, 174, 89, 206, 29, 35, 156, 173, 24, 48, 50, 136, 242, 207, 8, 72, 46, 144, 52, 18, 133, 232, 112]
    //    // == child: Hash(136..168) [129, 101, 66, 162, 246, 165, 217, 92, 249, 6, 252, 250, 25, 216, 48, 38, 83, 144, 124, 58, 177, 32, 205, 61, 138, 219, 219, 23, 15, 86, 157, 170]
    //    // == child: Hash(169..201) [86, 19, 59, 211, 244, 128, 21, 23, 250, 142, 131, 23, 178, 43, 217, 200, 158, 181, 223, 190, 119, 217, 26, 9, 179, 225, 71, 97, 160, 254, 75, 108]
    //    let node = iter.next();
    //    println!("get child: {:?}", node);
    //}
    //{
    //    println!("{}", "Decode the state node for #50".cyan());
    //    let (db, cfs) = setup_db_connection();
    //    let simple_trie = SimpleTrie {
    //        db,
    //        cfs: cfs.clone(),
    //    };

    //    let trie = TrieDB::<Layout<Hasher>>::new(&simple_trie, state_root_50).unwrap();
    //    let mut iter = TrieDBNodeIterator::new(&trie).unwrap();

    //    let node = iter.next();
    //    // Following code show the root node, and Hash structure use range to get the data as real hash
    //    if let Some(n) = node {
    //        let n = n.unwrap();
    //        println!("= Nibble Vec: {:?}", n.0);
    //        println!("= NodeKey: {:?}", n.1); // Exactly the Hash for the node
    //        let owned_node = n.2.node();
    //        println!("= owned_node: {:?}", owned_node);
    //        let node_plan = n.2.node_plan();
    //        match node_plan {
    //            NodePlan::Branch { children, value } => {
    //                println!("== Branch");
    //                println!("== value: {:?}", value);
    //                for c in children {
    //                    if let Some(h) = c {
    //                        println!("== child: {:?}", h);
    //                    }
    //                }
    //            }
    //            NodePlan::NibbledBranch {
    //                partial,
    //                children,
    //                value,
    //            } => {
    //                println!("== NibbleBranch");
    //                println!("== partial: {:?}", partial);
    //                println!("== value: {:?}", value);
    //                for c in children {
    //                    if let Some(h) = c {
    //                        println!("== child: {:?}", h);
    //                    }
    //                }
    //            }
    //            _ => {}
    //        }
    //    }
    //    // Here is the decoded output, there are 6 child node have data
    //    //
    //    // [None, None, Hash, Hash, None, Hash, None, None, None, None, None, Hash, Hash, None, None, Hash]
    //    //    0     1     2*    3*    4     5*    6     7     8     9     a     b*    c*    d     e     f*
    //    // == NibbleBranch
    //    // == partial: NibbleSlicePlan { bytes: 1..1, offset: 0 }
    //    // == value: None
    //    // == child: Hash(4..36)    32 [85, 114, 90, 184, 81, 124, 23, 235, 182, 170, 184, 133, 223, 93, 124, 251, 173, 165, 190, 146, 234, 244, 217, 33, 222, 72, 35, 215, 116, 144, 19, 36]
    //    //
    //    // == child: Hash(37..69)   48 [254, 64, 112, 194, 32, 25, 207, 40, 249, 31, 246, 143, 125, 54, 211, 81, 35, 61, 4, 132, 72, 140, 41, 106, 142, 111, 56, 156, 57, 89, 131, 20]
    //    // (no change from #5 - #50)
    //    //
    //    // == child: Hash(70..102)  80 [192, 98, 139, 161, 117, 76, 9, 117, 55, 27, 157, 185, 121, 240, 247, 152, 207, 145, 160, 222, 132, 194, 154, 32, 234, 200, 227, 208, 108, 197, 178, 7]
    //    // == child: Hash(103..135) 176 [49, 239, 154, 236, 197, 182, 244, 198, 173, 189, 166, 59, 187, 46, 204, 215, 78, 31, 59, 29, 95, 55, 237, 98, 113, 164, 48, 198, 16, 219, 132, 25]
    //    // == child: Hash(136..168) 192 [5, 186, 0, 144, 71, 86, 173, 65, 68, 101, 0, 237, 69, 84, 150, 46, 111, 109, 136, 206, 86, 88, 156, 177, 9, 20, 70, 100, 251, 208, 141, 121]
    //    // == child: Hash(169..201) 240 [160, 117, 69, 22, 65, 224, 115, 49, 4, 235, 92, 183, 242, 80, 47, 241, 125, 66, 246, 68, 19, 116, 151, 54, 185, 234, 44, 248, 13, 22, 45, 193]
    //}
    //println!("==================================================================================");
    //{
    //    let mut data: Vec<u8> = vec![
    //        94, 160, 29, 150, 47, 206, 102, 184, 44, 189, 162, 219, 238, 99, 92, 175, 155, 113,
    //        164, 89, 83, 126, 3, 28, 142, 179, 123, 67, 223, 54, 236, 110,
    //    ];
    //    println!("{:?}", blake2b(32, &[], &data[..]).as_bytes());
    //}
    //{
    //    println!("{}", "Find out the children".cyan());
    //    let (db, cfs) = setup_db_connection();
    //    let children = vec![
    //        // hash key again
    //        &[
    //            200, 114, 120, 200, 68, 53, 157, 194, 16, 49, 255, 146, 69, 65, 236, 93, 42, 122,
    //            177, 25, 6, 185, 229, 19, 55, 193, 91, 187, 49, 21, 236, 150,
    //        ],
    //        //#5
    //        &[
    //            94, 160, 29, 150, 47, 206, 102, 184, 44, 189, 162, 219, 238, 99, 92, 175, 155, 113,
    //            164, 89, 83, 126, 3, 28, 142, 179, 123, 67, 223, 54, 236, 110,
    //        ],
    //        &[
    //            254, 64, 112, 194, 32, 25, 207, 40, 249, 31, 246, 143, 125, 54, 211, 81, 35, 61, 4,
    //            132, 72, 140, 41, 106, 142, 111, 56, 156, 57, 89, 131, 20,
    //        ],
    //        &[
    //            186, 239, 159, 204, 255, 176, 65, 173, 46, 89, 234, 229, 224, 173, 99, 171, 160,
    //            247, 64, 36, 224, 190, 171, 44, 238, 243, 103, 85, 219, 220, 88, 231,
    //        ],
    //        &[
    //            25, 0, 97, 20, 123, 121, 139, 175, 63, 105, 174, 89, 206, 29, 35, 156, 173, 24, 48,
    //            50, 136, 242, 207, 8, 72, 46, 144, 52, 18, 133, 232, 112,
    //        ],
    //        &[
    //            129, 101, 66, 162, 246, 165, 217, 92, 249, 6, 252, 250, 25, 216, 48, 38, 83, 144,
    //            124, 58, 177, 32, 205, 61, 138, 219, 219, 23, 15, 86, 157, 170,
    //        ],
    //        &[
    //            86, 19, 59, 211, 244, 128, 21, 23, 250, 142, 131, 23, 178, 43, 217, 200, 158, 181,
    //            223, 190, 119, 217, 26, 9, 179, 225, 71, 97, 160, 254, 75, 108,
    //        ],
    //        // #50
    //        &[
    //            85, 114, 90, 184, 81, 124, 23, 235, 182, 170, 184, 133, 223, 93, 124, 251, 173,
    //            165, 190, 146, 234, 244, 217, 33, 222, 72, 35, 215, 116, 144, 19, 36,
    //        ],
    //        // &[254, 64, 112, 194, 32, 25, 207, 40, 249, 31, 246, 143, 125, 54, 211, 81, 35, 61, 4, 132, 72, 140, 41, 106, 142, 111, 56, 156, 57, 89, 131, 20]
    //        &[
    //            192, 98, 139, 161, 117, 76, 9, 117, 55, 27, 157, 185, 121, 240, 247, 152, 207, 145,
    //            160, 222, 132, 194, 154, 32, 234, 200, 227, 208, 108, 197, 178, 7,
    //        ],
    //        &[
    //            49, 239, 154, 236, 197, 182, 244, 198, 173, 189, 166, 59, 187, 46, 204, 215, 78,
    //            31, 59, 29, 95, 55, 237, 98, 113, 164, 48, 198, 16, 219, 132, 25,
    //        ],
    //        &[
    //            5, 186, 0, 144, 71, 86, 173, 65, 68, 101, 0, 237, 69, 84, 150, 46, 111, 109, 136,
    //            206, 86, 88, 156, 177, 9, 20, 70, 100, 251, 208, 141, 121,
    //        ],
    //        &[
    //            160, 117, 69, 22, 65, 224, 115, 49, 4, 235, 92, 183, 242, 80, 47, 241, 125, 66,
    //            246, 68, 19, 116, 151, 54, 185, 234, 44, 248, 13, 22, 45, 193,
    //        ],
    //    ];
    //    for cf in cfs.iter() {
    //        let h = db.cf_handle(cf).unwrap();
    //        let iter = db.iterator_cf(h, IteratorMode::Start);
    //        for (key, value) in iter {
    //            for child in children.iter() {
    //                if *key == child[..] {
    //                    println!("Got child hash in column family {}", cf);
    //                    println!("child {:?} \n {:?}", key, value);
    //                };
    //            }
    //        }
    //    }
    //}
}

use std::collections::HashMap;
use std::env::args_os;
use std::ops::Range;

// use sp_core::hashing::twox_128;
use trie_db::{
    node::{NodeHandlePlan, NodePlan},
    TrieDB, TrieDBNodeIterator,
};

mod logger;
use logger::{debug, error, info, init_logger, trace, warn, Logger};

mod storage;
use storage::{
    map_char_to_pos, map_pos_to_char, raw_query, setup_db_connection, Hasher, Layout, SimpleTrie,
};

mod cli;
use cli::parse_args;

mod errors;
use errors::Error;

static LOGGER: Logger = Logger;

type Data = Vec<u8>;

fn parse_child_hash(c: NodeHandlePlan, data: &[u8]) -> Vec<u8> {
    match c {
        NodeHandlePlan::Hash(r) | NodeHandlePlan::Inline(r) => data[r].to_vec(),
    }
}

fn parse_value(r: Option<Range<usize>>, data: &[u8]) -> Vec<u8> {
    if let Some(r) = r {
        data[r].to_vec()
    } else {
        vec![]
    }
}

fn pretty_print(prefix: &str, map: HashMap<Vec<u8>, Vec<usize>>) -> String {
    let mut out = String::from("[\n");
    for (k, v) in map.iter() {
        out.push_str(&format!(
            "\t[{}..{}]({:?}): \t{}\n",
            k[0],
            k.last().unwrap_or(&0),
            k.len(),
            v.iter().fold(format!("0x{}", prefix), |mut acc, x| {
                acc.push(map_pos_to_char(*x));
                acc
            })
        ));
    }
    out.push(']');
    out
}

fn json_output(output: Vec<(String, Data)>) -> String {
    let mut out = String::from("[");
    let output_last_idx = output.len() - 1;
    for (idx, (k, v)) in output.iter().enumerate() {
        out.push_str(&format!(r#"{{"{}":{:?}}}"#, k, v));
        if idx < output_last_idx {
            out.push(',');
        }
    }
    out.push(']');
    out
}

fn app() -> Result<(), Error> {
    let mut output: Vec<(String, Data)> = Vec::new();
    let matches = parse_args(args_os());
    init_logger(&LOGGER, matches.value_of("log").unwrap_or("error"));
    let including_children = !matches.is_present("exactly");
    let leaf_only = !matches.is_present("all node");
    let storage_key_hash = matches
        .value_of("storage key")
        .expect("shorage key is required");
    let raw_state_root_hash = matches
        .value_of("root hash")
        .expect("root hash is required");
    let db_path = matches.value_of("db path").expect("db path is required");

    let mut state_root_hash: [u8; 32] = Default::default();
    if raw_state_root_hash.starts_with("0x") {
        let tmp = hex::decode(raw_state_root_hash.strip_prefix("0x").unwrap())?;
        if tmp.len() == 32 {
            state_root_hash.copy_from_slice(&tmp[..]);
        } else {
            return Err(Error::OptionValueIncorrect(
                "state root hash".to_string(),
                "size is not correct".to_string(),
            ));
        }
    } else {
        return Err(Error::OptionValueIncorrect(
            "state root hash".to_string(),
            "0x prefix is not exist".to_string(),
        ));
    };

    info!("SSI Version: {}", env!("CARGO_PKG_VERSION"));
    info!("db path: {}", db_path);
    info!("including_children: {}", including_children);
    info!("leaf only: {}", leaf_only);
    info!("State Root Hash: {:?}", state_root_hash);
    info!("Storage Key Hash: {}", storage_key_hash);

    let storage_key: Vec<usize> = storage_key_hash.chars().map(map_char_to_pos).collect();
    debug!("Storage Key Path: {:?}", storage_key);

    let (db, cfs) = setup_db_connection(db_path);
    let (db2, _) = setup_db_connection(db_path);
    let simple_trie = SimpleTrie {
        db,
        cfs: cfs.clone(),
    };

    // TODO: handle unwarp here
    let trie = TrieDB::<Layout<Hasher>>::new(&simple_trie, &state_root_hash).unwrap();
    let mut node_iter = TrieDBNodeIterator::new(&trie).unwrap();
    let mut path_iter = storage_key.iter();

    let mut target_node_key = Some(state_root_hash.to_vec());
    // child tree, hash -> path
    let mut children_hash_to_path: HashMap<Vec<u8>, Vec<usize>> = HashMap::new();

    loop {
        let current_node = node_iter.next();
        trace!("current node: {:?}", current_node);
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
        let node_key = n.1.unwrap();
        let node_plan = n.2.node_plan();
        debug!("Key({}): {:?}", node_key.len(), node_key);
        if node_key == *target_node_key.clone().unwrap() {
            debug!("find node: {:?}", target_node_key);
            let path = path_iter.next();

            // TODO refactor this
            let data = raw_query(&db2, &cfs, n.0.as_prefix(), node_key);
            trace!("prefix: {:?}, node key {:?}", n.0.as_prefix(), node_key);
            let data =
                data.expect("node key error, open trace log for futher for finding the root cause");
            debug!("data for {:?}: {} length bytes", node_key, data.len());

            if let Some(p) = path {
                let child = match node_plan {
                    NodePlan::NibbledBranch {
                        children,
                        value,
                        partial,
                    } => {
                        debug!(
                            "Find path to a nibbleBranch \"{}\"({})",
                            map_pos_to_char(*p),
                            p
                        );
                        trace!("children: {:?}", children);
                        trace!("value: {:?}", value);
                        trace!("partial: {:?}", partial);

                        for _ in 0..partial.len() {
                            if let Some(p) = path_iter.next() {
                                debug!(
                                    "Find path to \"{}\"({})\t(partial)",
                                    map_pos_to_char(*p),
                                    p
                                );
                            }
                        }

                        children
                            .get(*p)
                            .expect("branch node should have this child")
                            .clone()
                    }
                    NodePlan::Branch { children, value } => {
                        debug!("Find path to a branch \"{}\"({})", map_pos_to_char(*p), p);
                        trace!("children: {:?}", children);
                        trace!("value: {:?}", value);

                        children
                            .get(*p)
                            .expect("branch node should have this child")
                            .clone()
                    }
                    NodePlan::Extension { child, partial } => {
                        debug!(
                            "Find path to an extension \"{}\"({})",
                            map_pos_to_char(*p),
                            p
                        );
                        trace!("partial: {:?}", partial);

                        for _ in 0..partial.len() {
                            if let Some(p) = path_iter.next() {
                                debug!("Path to \"{}\"({})\t(partial)", map_pos_to_char(*p), p);
                            }
                        }
                        Some(child.clone())
                    }
                    _ => panic!("should not here"),
                };
                debug!("child: {:?}", child);
                if let Some(c) = child {
                    let h = parse_child_hash(c, &data);
                    target_node_key = Some(h);
                    debug!("new target node key: {:?}", target_node_key);
                } else {
                    error!("Path Error");
                    break;
                }
            } else {
                // This end of path
                match node_plan {
                    NodePlan::Leaf { value, .. } => {
                        info!("Last node is leaf");
                        output.push((
                            format!("0x{}", storage_key_hash),
                            parse_value(Some(value.clone()), &data),
                        ));
                        break;
                    }
                    NodePlan::Branch { children, .. } => {
                        if !leaf_only && !including_children {
                            output.push((format!("0x{}", storage_key_hash), vec![]));
                        }
                        if including_children {
                            info!("Last node is branch");
                            for (idx, child) in children.iter().enumerate() {
                                if let Some(c) = child {
                                    children_hash_to_path
                                        .insert(parse_child_hash(c.clone(), &data), vec![idx]);
                                }
                            }
                        } else {
                            error!("Last node is branch");
                            break;
                        }
                    }
                    NodePlan::NibbledBranch {
                        children,
                        partial,
                        value,
                        ..
                    } => {
                        if !leaf_only {
                            output.push((
                                format!("0x{}", storage_key_hash),
                                parse_value(value.clone(), &data),
                            ));
                        }
                        if including_children {
                            info!("Last node is nibble branch");
                            let partial_path = vec![16; partial.len()];
                            for (idx, child) in children.iter().enumerate() {
                                if let Some(c) = child {
                                    let mut path = partial_path.clone();
                                    path.push(idx);
                                    children_hash_to_path
                                        .insert(parse_child_hash(c.clone(), &data), path);
                                }
                            }
                        } else {
                            warn!("Last node is nibble branch");
                            output.push((format!("0x{}", storage_key_hash), vec![]));
                            break;
                        }
                    }
                    NodePlan::Extension { partial, child } => {
                        if !leaf_only {
                            output.push((format!("0x{}", storage_key_hash), vec![]));
                        }
                        if including_children {
                            debug!("Last node is extension");
                            children_hash_to_path.insert(
                                parse_child_hash(child.clone(), &data),
                                vec![16; partial.len()],
                            );
                        } else {
                            error!("Last node is extension");
                            output.push((format!("0x{}", storage_key_hash), vec![]));
                            break;
                        }
                    }
                    NodePlan::Empty => {
                        output.push((format!("0x{}", storage_key_hash), vec![]));
                        warn!("Last node is empty");
                    }
                };
            }
        } else if including_children && children_hash_to_path.contains_key(&node_key.to_vec()) {
            // TODO refactor this
            let data = raw_query(&db2, &cfs, n.0.as_prefix(), node_key);
            trace!("prefix: {:?}, node key {:?}", n.0.as_prefix(), node_key);
            let data =
                data.expect("node key error, open trace log for futher for finding the root cause");
            debug!("data for {:?}: {} length bytes", node_key, data.len());
            let path_prefix = children_hash_to_path
                .get(&node_key.to_vec())
                .unwrap()
                .clone();
            let trie_key =
                path_prefix
                    .iter()
                    .fold(format!("0x{}", storage_key_hash), |mut acc, x| {
                        acc.push(map_pos_to_char(*x));
                        acc
                    });

            match node_plan {
                NodePlan::Leaf { value, .. } => {
                    info!("Find {} in 0x{} subtrie", trie_key, storage_key_hash);
                    output.push((trie_key, parse_value(Some(value.clone()), &data)));
                }
                NodePlan::Branch { children, .. } => {
                    if including_children {
                        for (idx, child) in children.iter().enumerate() {
                            if let Some(c) = child {
                                let mut path = path_prefix.clone();
                                path.push(idx);
                                children_hash_to_path
                                    .insert(parse_child_hash(c.clone(), &data), path);
                            }
                        }
                    }
                }
                NodePlan::NibbledBranch {
                    value,
                    children,
                    partial,
                    ..
                } => {
                    if !leaf_only {
                        output.push((trie_key, parse_value(value.clone(), &data)));
                    }
                    if including_children {
                        let mut partial_path = vec![16; partial.len()];
                        for (idx, child) in children.iter().enumerate() {
                            if let Some(c) = child {
                                let mut path = path_prefix.clone();
                                path.append(&mut partial_path);
                                path.push(idx);
                                children_hash_to_path
                                    .insert(parse_child_hash(c.clone(), &data), path);
                            }
                        }
                    }
                }
                NodePlan::Extension { partial, child } => {
                    let mut path = path_prefix;
                    path.append(&mut vec![16; partial.len()]);
                    children_hash_to_path.insert(parse_child_hash(child.clone(), &data), path);
                }
                NodePlan::Empty => {
                    warn!("Last node is empty");
                }
            };
        }
    }
    // remove the subtrie root, because it is not leaf
    if leaf_only && including_children && output.len() > 1 {
        output.remove(0);
    }
    trace!(
        "overall nodes in substrie: {}",
        pretty_print(storage_key_hash, children_hash_to_path)
    );
    println!("{}", json_output(output));
    Ok(())
}

fn main() {
    // println!("{:?}", hex::encode(twox_128(b"System")));
    // println!("{:?}", hex::encode(twox_128(b"Account")));
    if let Err(e) = app() {
        println!("{}", e);
    }
}

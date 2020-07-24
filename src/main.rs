use std::collections::HashMap;
use std::env::args_os;
use std::ops::Range;

use sp_core::hashing::{blake2_256, twox_128};
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
use cli::{parse_args, ArgMatches};

mod errors;
use errors::Error;

mod codec;
use codec::{blake2_128_concat_encode, key_semantic_decode, twox_64_concat_encode};

static LOGGER: Logger = Logger;

type Data = (Vec<u8>, bool);

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

fn json_output(output: Vec<(String, Data)>, summary: bool, prefix: &str) -> String {
    let mut out = String::from("[");
    if !output.is_empty() {
        let output_last_idx = output.len() - 1;
        for (idx, (k, v)) in output.iter().enumerate() {
            if summary {
                let hash = if v.0.len() == 32 {
                    hex::encode(&v.0)
                } else {
                    hex::encode(blake2_256(&v.0))
                };
                let semantic_result = key_semantic_decode(k);
                out.push_str(&format!(
                    r#"{{"0x{}":{{"hash":"0x{}","length":{},"leaf":{},"subtrie_path":"{}","pallet":"{}","field":"{}","key":"{}"}}}}"#,
                    k,
                    hash,
                    v.0.len(),
                    v.1,
                    k.strip_prefix(prefix).unwrap(),
                    semantic_result.0,
                    semantic_result.1,
                    semantic_result.2.unwrap_or_default(),
                ));
            } else {
                out.push_str(&format!(r#"{{"{}":{:?}}}"#, k, v.0));
            }
            if idx < output_last_idx {
                out.push(',');
            }
        }
    }
    out.push(']');
    out
}

fn get_storage_key_hash(matches: &ArgMatches) -> Result<String, Error> {
    if matches.is_present("storage key") {
        // TODO valid date storage key here
        Ok(matches.value_of("storage key").unwrap().to_string())
    } else {
        let mut out = String::new();
        let mut first_key = false;
        out.push_str(&hex::encode(twox_128(
            matches
                .value_of("pallet")
                .expect("pallet is the at last parameter to generate the storage key")
                .as_bytes(),
        )));
        if matches.is_present("field") {
            out.push_str(&hex::encode(twox_128(
                matches.value_of("field").unwrap().as_bytes(),
            )));
        }

        if matches.is_present("twox 64 concat") {
            if !matches.is_present("field") {
                return Err(Error::OptionValueIncorrect(
                    "field".to_string(),
                    "field name is required when genereate a key in that field".to_string(),
                ));
            }
            out.push_str(&twox_64_concat_encode(
                &matches.value_of("twox 64 concat").unwrap(),
            ));
            first_key = true;
        }
        if matches.is_present("black2 128 concat") {
            if !matches.is_present("field") {
                return Err(Error::OptionValueIncorrect(
                    "field".to_string(),
                    "field name is required when genereate a key in that field".to_string(),
                ));
            }
            out.push_str(&blake2_128_concat_encode(
                &matches.value_of("black2 128 concat").unwrap(),
            ));
            first_key = true;
        }
        if matches.is_present("identity") {
            if !matches.is_present("field") {
                return Err(Error::OptionValueIncorrect(
                    "field".to_string(),
                    "field name is required when genereate a key in that field".to_string(),
                ));
            }
            out.push_str(&hex::encode(
                matches.value_of("identity").unwrap().as_bytes(),
            ));
            first_key = true;
        }
        if matches.is_present("twox 64 concat 2nd") {
            if !first_key {
                return Err(Error::OptionValueIncorrect(
                    "twox 64 concat/black2 128 concat/identity".to_string(),
                    "one of aformentioned option is required when genereate a secondary key for double map".to_string(),
                ));
            }
            out.push_str(&twox_64_concat_encode(
                &matches.value_of("twox 64 concat 2nd").unwrap(),
            ));
        }
        if matches.is_present("black2 128 concat 2nd") {
            if !first_key {
                return Err(Error::OptionValueIncorrect(
                    "twox 64 concat/black2 128 concat/identity".to_string(),
                    "one of aformentioned option is required when genereate a secondary key for double map".to_string(),
                ));
            }
            out.push_str(&blake2_128_concat_encode(
                &matches.value_of("black2 128 concat 2nd").unwrap(),
            ));
        }
        if matches.is_present("identity 2nd") {
            if !first_key {
                return Err(Error::OptionValueIncorrect(
                    "twox 64 concat/black2 128 concat/identity".to_string(),
                    "one of aformentioned option is required when genereate a secondary key for double map".to_string(),
                ));
            }
            out.push_str(&hex::encode(
                matches.value_of("identity 2nd").unwrap().as_bytes(),
            ));
        }
        Ok(out)
    }
}

fn app() -> Result<(), Error> {
    let mut output: Vec<(String, Data)> = Vec::new();
    let matches = parse_args(args_os());
    init_logger(&LOGGER, matches.value_of("log").unwrap_or("error"));
    let including_children = !matches.is_present("exactly");
    let leaf_only = !matches.is_present("all node");
    let storage_key_hash = &get_storage_key_hash(&matches)?;
    let raw_state_root_hash = matches
        .value_of("root hash")
        .expect("root hash is required");
    let db_path = matches.value_of("db path").expect("db path is required");
    let summary = matches.is_present("summarize output");

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
    info!("DB path: {}", db_path);
    info!("Including_children: {}", including_children);
    info!("Leaf only: {}", leaf_only);
    info!("State root hash: {:?}", state_root_hash);
    info!("Storage key hash: {}", storage_key_hash);
    info!("Sumarize data: {}", summary);

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

    let mut node_count = 0;

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
            trace!("Ignored Key({}): {:?}", k.len(), k);
            continue;
        }
        let node_key = n.1.unwrap();
        let node_plan = n.2.node_plan();
        trace!("Key({}): {:?}", node_key.len(), node_key);
        if node_key == *target_node_key.clone().unwrap() {
            info!("Find the {} nodes", node_count);
            node_count += 1;
            debug!("node key: {:?}", target_node_key);
            let path = path_iter.next();

            // TODO refactor this
            let data = raw_query(&db2, &cfs, n.0.as_prefix(), node_key);
            trace!("prefix: {:?}, node key {:?}", n.0.as_prefix(), node_key);
            let data =
                data.expect("node key error, open trace log for futher for finding the root cause");
            debug!("data for {:?}: {} length bytes", node_key, data.len());

            if let Some(mut p) = path {
                match node_plan {
                    NodePlan::Leaf { value, .. } => {
                        warn!("run into leaf before the path drain");
                        let value = parse_value(Some(value.clone()), &data);
                        if value.len() == 32 {
                            target_node_key = Some(value);
                            continue;
                        } else {
                            warn!("Run into leaf node early");
                            output.push((storage_key_hash.to_string(), (value, true)));
                            break;
                        }
                    }
                    NodePlan::NibbledBranch {
                        children,
                        value,
                        partial,
                    } => {
                        let partial_nibble = partial.build(&data);
                        debug!("Get a NibbledBranch, find path to {:?}", partial_nibble);
                        debug!("partial: {:?}", partial_nibble);

                        for (idx, n) in partial_nibble.iter().enumerate() {
                            if idx == 0 {
                                if *p != n as usize {
                                    error!(
                                        "trie path not match path: {}({}), nibble: {}",
                                        map_pos_to_char(*p),
                                        p,
                                        n
                                    );
                                }
                                continue;
                            }
                            if let Some(pp) = path_iter.next() {
                                if *pp != n as usize {
                                    error!(
                                        "trie path not match path: {}({}), nibble: {}",
                                        map_pos_to_char(*p),
                                        pp,
                                        n
                                    );
                                }
                                debug!(
                                    "Find path to \"{}\"({})\t(partial: {:?})",
                                    map_pos_to_char(*pp),
                                    pp,
                                    n
                                );
                                p = pp;
                            }
                        }

                        trace!("value: {:?}", value);
                        if let Some(c) = children.get(*p).expect("trie hash this child").clone() {
                            debug!("children: {:?}", children);
                            debug!("child: {:?}", c);
                            let h = parse_child_hash(c, &data);
                            target_node_key = Some(h);
                            debug!("new target node key: {:?}", target_node_key);
                        } else if including_children {
                            for (idx, child) in children.iter().enumerate() {
                                if let Some(c) = child {
                                    children_hash_to_path
                                        .insert(parse_child_hash(c.clone(), &data), vec![idx]);
                                }
                            }
                        } else {
                            output.push((
                                storage_key_hash.to_string(),
                                (parse_value(value.clone(), &data), true),
                            ));
                            break;
                        }
                    }
                    NodePlan::Branch { children, value } => {
                        debug!(
                            "Get a Branch, find path to \"{}\"({})",
                            map_pos_to_char(*p),
                            p
                        );
                        trace!("value: {:?}", value);

                        if let Some(c) = children.get(*p).expect("trie hash this child").clone() {
                            trace!("children: {:?}", children);
                            debug!("child: {:?}", c);
                            let h = parse_child_hash(c, &data);
                            target_node_key = Some(h);
                            debug!("new target node key: {:?}", target_node_key);
                        } else {
                            error!("children: {:?}", children);
                            error!("Path end at {}, Branch node has no child for here", p);
                            break;
                        }
                    }
                    NodePlan::Extension { child, partial } => {
                        debug!(
                            "Get an Extension, find path to \"{}\"({})",
                            map_pos_to_char(*p),
                            p
                        );
                        let partial_nibble = partial.build(&data);
                        trace!("partial: {:?}", partial_nibble);

                        let mut from_node_check = false;
                        for n in partial_nibble.iter() {
                            if !from_node_check {
                                if *p != n as usize {
                                    error!(
                                        "trie path not match path: {}({}), nibble: {}",
                                        map_pos_to_char(*p),
                                        p,
                                        n
                                    );
                                }
                                from_node_check = true;
                                continue;
                            }
                            if let Some(p) = path_iter.next() {
                                debug!(
                                    "Find path to \"{}\"({})\t(partial: {:?})",
                                    map_pos_to_char(*p),
                                    p,
                                    n
                                );
                            }
                        }
                        debug!("child: {:?}", child);
                        let h = parse_child_hash(child.clone(), &data);
                        target_node_key = Some(h);
                        debug!("new target node key: {:?}", target_node_key);
                    }
                    _ => {
                        error!("Nonexistent, please check your input parameters");
                        return Ok(());
                    }
                };
            } else {
                // This end of path
                match node_plan {
                    NodePlan::Leaf { value, .. } => {
                        let value = parse_value(Some(value.clone()), &data);
                        if value.len() == 32 {
                            info!("Get the last node, and it is child trie");
                            children_hash_to_path.insert(value, vec![]);
                        } else {
                            info!("Get th last node, and it is Leaf");
                            output.push((storage_key_hash.to_string(), (value, true)));
                            break;
                        }
                    }
                    NodePlan::Branch { children, .. } => {
                        info!("Get the last node, and it is Branch");
                        if !leaf_only && !including_children {
                            output.push((storage_key_hash.to_string(), (vec![], false)));
                        }
                        if including_children {
                            for (idx, child) in children.iter().enumerate() {
                                if let Some(c) = child {
                                    children_hash_to_path
                                        .insert(parse_child_hash(c.clone(), &data), vec![idx]);
                                }
                            }
                        } else {
                            error!("Get the last node but it is branch");
                            break;
                        }
                    }
                    NodePlan::NibbledBranch { .. } => {
                        panic!("unreachable by design");
                    }
                    NodePlan::Extension { partial, child } => {
                        info!("Get the last node, and it is extension");
                        if !leaf_only {
                            output.push((storage_key_hash.to_string(), (vec![], false)));
                        }
                        if including_children {
                            let partial_nibble = partial.build(&data);
                            let mut partial_path = vec![];
                            let mut from_node_check = false;
                            for n in partial_nibble.iter() {
                                if !from_node_check {
                                    from_node_check = true;
                                    continue;
                                }
                                partial_path.push(n as usize);
                            }

                            children_hash_to_path
                                .insert(parse_child_hash(child.clone(), &data), partial_path);
                        } else {
                            error!("Get the last node but it is extension");
                            output.push((storage_key_hash.to_string(), (vec![], false)));
                            break;
                        }
                    }
                    NodePlan::Empty => {
                        warn!("Get the last node but it is empty");
                        output.push((storage_key_hash.to_string(), (vec![], false)));
                    }
                };
            }
        } else if including_children && children_hash_to_path.contains_key(&node_key.to_vec()) {
            // TODO refactor this
            let data = raw_query(&db2, &cfs, n.0.as_prefix(), node_key);
            debug!("prefix: {:?}, node key {:?}", n.0.as_prefix(), node_key);
            let data =
                data.expect("node key error, open trace log for futher for finding the root cause");
            debug!("data for {:?}: {} length bytes", node_key, data.len());
            let path_prefix = children_hash_to_path
                .get(&node_key.to_vec())
                .unwrap()
                .clone();
            let trie_key = path_prefix
                .iter()
                .fold(storage_key_hash.to_string(), |mut acc, x| {
                    acc.push(map_pos_to_char(*x));
                    acc
                });

            match node_plan {
                NodePlan::Leaf { value, .. } => {
                    info!("Find 0x{} in 0x{} subtrie", trie_key, storage_key_hash);
                    output.push((trie_key, (parse_value(Some(value.clone()), &data), true)));
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
                        output.push((trie_key, (parse_value(value.clone(), &data), false)));
                    }
                    if including_children {
                        let partial_nibble = partial.build(&data);
                        let mut partial_path = partial_nibble
                            .iter()
                            .map(|b| b as usize)
                            .collect::<Vec<usize>>();
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
    trace!(
        "overall nodes in substrie: {}",
        pretty_print(storage_key_hash, children_hash_to_path)
    );
    println!("{}", json_output(output, summary, storage_key_hash));
    Ok(())
}

fn main() {
    if let Err(e) = app() {
        println!("{}", e);
    }
}

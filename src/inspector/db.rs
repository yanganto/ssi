/// Inspect the TrieNodes in the DB
///
/// Trace the the storage key in the Trie, and list the exactly trie node or the subtrie nodes
use std::collections::HashMap;
use std::ops::Range;

use sp_core::hashing::blake2_256;
use trie_db::{
    node::{NodeHandlePlan, NodePlan},
    TrieDB, TrieDBNodeIterator,
};

use crate::cli::ArgMatches;
use crate::codec::storage_key_semantic_decode;
use crate::errors::Error;
use crate::inspector::get_storage_key_hash;
use crate::logger::{debug, error, info, trace, warn};
use crate::storage::{
    map_char_to_pos, map_pos_to_char, raw_query, setup_db_connection, Hasher, Layout, SimpleTrie,
};

/// the (byte data, is leaf node)
type Data = (Vec<u8>, bool);

#[derive(Debug)]
enum NodeChangeStatus {
    Insert,
    // There is may not realy deletion in KVDB, but the node is not recorded in Tire structure,
    // user can not query the data throught Trie, and the node will be deemed as Deleted
    Delete,
    Modify,
}

/// The (diff data, diff node state)
/// If the data lenght is not the same, the diff data will record all the nore data as positive
/// bytes.
///
/// If the data length is the same,
/// the byte with 0 means the data not change,
/// and the byte with positive value means the data is inserted
/// and the byte with negative value means the data is deleted
type DiffData = (Vec<i16>, NodeChangeStatus);

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

/// Print the output as JSON format
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
                let semantic_result = storage_key_semantic_decode(k, false);
                out.push_str(&format!(
                    r#"{{"0x{}":{{"hash":"0x{}","length":{},"leaf":{},"subtrie_path":"{}","pallet":"{}","field":"{}","key":"{}"}}}}"#,
                    k,
                    hash,
                    v.0.len(),
                    v.1,
                    k.strip_prefix(prefix).unwrap(),
                    semantic_result.0.unwrap_or_default(),
                    semantic_result.1.unwrap_or_default(),
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

/// Print the difference as JSON format
fn json_diff(output: Vec<(String, DiffData)>, summary: bool, prefix: &str) -> String {
    let mut out = String::from("[");
    if !output.is_empty() {
        let output_last_idx = output.len() - 1;
        for (idx, (k, v)) in output.iter().enumerate() {
            if summary {
                let semantic_result = storage_key_semantic_decode(k, false);
                out.push_str(&format!(
                    r#"{{"0x{}":{{"length":{}, "change_length":{},"status":"{:?}","subtrie_path":"{}","pallet":"{}","field":"{}","key":"{}"}}}}"#,
                    k,
                    v.0.len(),
                    v.0.iter().fold(0, |acc, x| if *x !=0 {acc + 1} else {acc}),
                    v.1,
                    k.strip_prefix(prefix).unwrap(),
                    semantic_result.0.unwrap_or_default(),
                    semantic_result.1.unwrap_or_default(),
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

fn get_subtrie_node(
    storage_key_hash: &str,
    db_path: &str,
    state_root_hash: [u8; 32],
    including_children: bool,
    leaf_only: bool,
) -> Result<Vec<(String, Data)>, Error> {
    let mut output: Vec<(String, Data)> = Vec::new();

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
                            error!("Run into leaf node early");
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
                        debug!("partial: {:?}", partial_nibble);
                        debug!("children: {:?}", children);
                        debug!(
                            "Get a NibbledBranch ({}), partial path {:?}",
                            node_count, partial_nibble
                        );
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
                                trace!(
                                    "Find path to \"{}\"({})\t(partial: {:?})",
                                    map_pos_to_char(*pp),
                                    pp,
                                    n
                                );
                            }
                        }

                        let child = if !partial_nibble.is_empty() {
                            if let Some(pp) = path_iter.next() {
                                p = pp;
                                debug!("update path to {}", p);
                                Some(
                                    children
                                        .get(*p)
                                        .unwrap()
                                        .clone()
                                        .expect("trie hash this child")
                                        .clone(),
                                )
                            } else {
                                None
                            }
                        } else {
                            children.get(*p).expect("trie hash this child").clone()
                        };

                        if child.is_none() {
                            debug!("go to the end of path {:?}", partial_nibble);
                        } else {
                            debug!(
                                "skip path to {:?}, and will go to {}",
                                partial_nibble,
                                map_pos_to_char(*p)
                            );
                        }

                        trace!("value: {:?}", value);
                        if let Some(c) = child {
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
                            "Get a Branch ({}), find path to \"{}\"({})",
                            node_count,
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

                        if let Some(p) = path_iter.next() {
                            debug!(
                                "Get an Extension ({}), skip path to {:?}, and will go to {}",
                                node_count,
                                partial_nibble,
                                map_pos_to_char(*p)
                            );
                        } else {
                            debug!(
                                "Get an Extension ({}), go to the end of path {:?}",
                                node_count, partial_nibble
                            );
                        }

                        debug!("child: {:?}", child);
                        let h = parse_child_hash(child.clone(), &data);
                        target_node_key = Some(h);
                        debug!("new target node key: {:?}", target_node_key);
                    }
                    _ => {
                        error!("Nonexistent, please check your input parameters");
                        return Ok(output);
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
            node_count += 1;
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
    Ok(output)
}

fn hex_str_to_state_hash(
    state_root_hash: &mut [u8; 32],
    raw_state_root_hash: &str,
) -> Result<(), Error> {
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
    Ok(())
}

pub fn db_inspect_app(matches: ArgMatches) -> Result<(), Error> {
    let storage_key_hash = &get_storage_key_hash(&matches)?;
    let summary = matches.is_present("summarize output");
    let including_children = !matches.is_present("exactly");
    let leaf_only = !matches.is_present("all node");
    let raw_state_root_hash = matches
        .value_of("root hash")
        .expect("root hash is required");
    let db_path = matches.value_of("path").expect("db path is required");

    let mut state_root_hash: [u8; 32] = Default::default();
    hex_str_to_state_hash(&mut state_root_hash, raw_state_root_hash)?;

    info!("SSI Version: {}", env!("CARGO_PKG_VERSION"));
    info!("DB path: {}", db_path);
    info!("Including_children: {}", including_children);
    info!("Leaf only: {}", leaf_only);
    info!("State root hash: {:?}", state_root_hash);
    info!("Storage key hash: {}", storage_key_hash);
    info!("Sumarize data: {}", summary);

    let output = get_subtrie_node(
        storage_key_hash,
        db_path,
        state_root_hash,
        including_children,
        leaf_only,
    )?;
    println!("{}", json_output(output, summary, storage_key_hash));
    Ok(())
}
pub fn db_diff_app(matches: ArgMatches) -> Result<(), Error> {
    let storage_key_hash = &get_storage_key_hash(&matches)?;
    let summary = matches.is_present("summarize output");
    let including_children = !matches.is_present("exactly");
    let leaf_only = !matches.is_present("all node");
    let raw_state_root_hash = matches
        .value_of("root hash")
        .expect("root hash is required");
    let db_path = matches.value_of("path").expect("db path is required");

    let mut state_root_hash_1: [u8; 32] = Default::default();
    hex_str_to_state_hash(&mut state_root_hash_1, raw_state_root_hash)?;

    let mut state_root_hash_2: [u8; 32] = Default::default();
    hex_str_to_state_hash(&mut state_root_hash_2, raw_state_root_hash)?;

    info!("SSI Version: {}", env!("CARGO_PKG_VERSION"));
    info!("DB path: {}", db_path);
    info!("Including_children: {}", including_children);
    info!("Leaf only: {}", leaf_only);
    info!("State root hash: {:?}", state_root_hash_1);
    info!("State root hash diff: {:?}", state_root_hash_2);
    info!("Storage key hash: {}", storage_key_hash);
    info!("Sumarize data: {}", summary);

    let origin: HashMap<_, _> = get_subtrie_node(
        storage_key_hash,
        db_path,
        state_root_hash_1,
        including_children,
        leaf_only,
    )?
    .into_iter()
    .collect();

    let after: HashMap<_, _> = get_subtrie_node(
        storage_key_hash,
        db_path,
        state_root_hash_2,
        including_children,
        leaf_only,
    )?
    .into_iter()
    .collect();

    let mut output: Vec<(String, DiffData)> = Vec::new();

    for (k, v) in after.iter() {
        // Modify is only the same structure, so we assume it is same length
        if origin.contains_key(k) {
            let origin_value = origin.get(k).unwrap();
            if origin_value.0.len() == v.0.len() {
                let mut has_diff = false;
                let diff = origin_value
                    .0
                    .iter()
                    .zip(v.0.iter())
                    .map(|(b, a)| {
                        if a == b {
                            0
                        } else {
                            has_diff = true;
                            *a as i16
                        }
                    })
                    .collect::<Vec<i16>>();
                if has_diff {
                    output.push((k.clone(), (diff, NodeChangeStatus::Modify)));
                }
                continue;
            }
        }
        output.push((
            k.clone(),
            (
                v.0.iter().map(|b| *b as i16).collect(),
                NodeChangeStatus::Insert,
            ),
        ))
    }

    for (k, v) in origin.iter() {
        if !after.contains_key(k) {
            output.push((
                k.clone(),
                (
                    v.0.iter().map(|b| -(*b as i16)).collect(),
                    NodeChangeStatus::Delete,
                ),
            ))
        }
    }
    println!("{}", json_diff(output, summary, storage_key_hash));
    Ok(())
}

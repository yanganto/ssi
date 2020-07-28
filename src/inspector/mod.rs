use sp_core::hashing::twox_128;

use crate::cli::ArgMatches;
use crate::codec::storage_key_semantic_decode;
use crate::codec::{blake2_128_concat_encode, twox_64_concat_encode};
use crate::errors::Error;

mod db;
pub use db::db_inspect_app;

mod stream;
pub use stream::stream_inspect_app;

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

pub fn decode_storage_key(matches: ArgMatches) -> Result<(), Error> {
    if let Ok(storage_key_hash) = get_storage_key_hash(&matches) {
        let semantic_result = storage_key_semantic_decode(&storage_key_hash, true);
        println!(
            "{} > {} > {}",
            semantic_result.0.unwrap_or_default(),
            semantic_result.1.unwrap_or_default(),
            semantic_result.2.unwrap_or_default()
        );
    }
    Ok(())
}

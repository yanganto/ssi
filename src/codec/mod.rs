use sp_core::hashing::{blake2_128, twox_64};

mod hash_maps;
use hash_maps::{BLAKE2_MAP, XX_MAP};

pub fn twox_64_concat_encode(s: &str) -> String {
    let mut out = hex::encode(twox_64(s.as_bytes()));
    out.push_str(&hex::encode(s.as_bytes()));
    out
}

pub fn blake2_128_concat_encode(s: &str) -> String {
    let mut out = hex::encode(blake2_128(s.as_bytes()));
    out.push_str(&hex::encode(s.as_bytes()));
    out
}

fn twox_64_concat_decode(s: String) -> Option<String> {
    if s.len() > 16 {
        let (first, last) = s.split_at(16);
        let decode_bytes = hex::decode(last).unwrap_or_default();
        let k = std::str::from_utf8(&decode_bytes).unwrap_or_default();
        if twox_64_concat_encode(k).starts_with(first) {
            return Some(k.to_string());
        }
    }
    None
}

fn black2_128_concat_decode(s: String) -> Option<String> {
    if s.len() > 32 {
        let (first, last) = s.split_at(32);
        let decode_bytes = hex::decode(last).unwrap_or_default();
        let k = std::str::from_utf8(&decode_bytes).unwrap_or_default();
        if blake2_128_concat_encode(k).starts_with(first) {
            return Some(k.to_string());
        }
    }
    None
}

fn pallet_decode(s: &str) -> Option<&str> {
    if let Some(p) = XX_MAP.get(s) {
        Some(p)
    } else {
        None
    }
}

fn field_decode(s: &str) -> &str {
    if let Some(p) = XX_MAP.get(s) {
        p
    } else {
        s
    }
}

// TODO: handle the 2nd key
pub fn storage_key_semantic_decode(
    s: &'_ str,
    keep_unsolve: bool,
) -> (Option<&'_ str>, Option<&'_ str>, Option<String>) {
    if s.len() < 32 {
        return (None, None, None);
    }

    let (p, tail) = s.split_at(32);
    let pallet_name = pallet_decode(p);

    if tail.len() < 32 {
        if keep_unsolve {
            return (pallet_name, Some(tail), None);
        } else {
            return (pallet_name, None, None);
        }
    }

    let (f, tail) = tail.split_at(32);
    let field_name = field_decode(f);

    let key = if !tail.is_empty() {
        let mut k = twox_64_concat_decode(tail.to_string());

        if k.is_none() {
            k = black2_128_concat_decode(tail.to_string());
        }

        if k.is_none() && tail.len() >= 64 {
            let (black2_key, tail) = tail.split_at(64);
            k = BLAKE2_MAP
                .get(black2_key)
                .map(|c| format!("{}∥{}", c, tail));
        }

        if k.is_none() && tail.len() >= 32 && tail.len() < 64 {
            let (two_x_key, tail) = tail.split_at(32);
            k = XX_MAP.get(two_x_key).map(|c| format!("{}∥{}", c, tail));
        }

        if k.is_none() && keep_unsolve {
            Some(tail.to_string())
        } else {
            let decode_bytes = hex::decode(tail).unwrap_or_default();
            std::str::from_utf8(&decode_bytes)
                .ok()
                .map(|s| s.to_string())
        }
    } else {
        None
    };

    (pallet_name, Some(field_name), key)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_twox_64_concat_decode() {
        assert_eq!(twox_64_concat_decode("".to_string()), None);
        assert_eq!(
            twox_64_concat_decode("3fe5e3a3f34ce9df2f2f457665".to_string()),
            Some("//Eve".to_string())
        );
    }
    #[test]
    fn test_key_semantic_decode() {
        assert_eq!(key_semantic_decode("26aa394eea5630e07c48ae0c9558cef7b99d880ec681799c0cf30e8886371da93fe5e3a3f34ce9df2f2f457665"), 
			("System", "Account", Some("//Eve".to_string())));
    }
}

use std::fs;
use std::fs::File;
use std::io::Read;
use sha2::{Digest, Sha256};
use crate::structures::HashStore;

pub fn load_hashes() -> HashStore {
    let path = ".ando/hashes";
    if let Ok(data) = fs::read_to_string(path) {
        serde_json::from_str(&data).unwrap_or_default()
    } else {
        HashStore::default()
    }
}

pub fn save_hashes(store: &HashStore) {
    let path = ".ando";
    if !std::path::Path::new(path).exists() {
        fs::create_dir(path).unwrap();
    }
    fs::write(".ando/hashes", serde_json::to_string_pretty(store).unwrap()).unwrap();
}

pub fn hash_file(path: &str) -> Option<String> {
    let mut file = File::open(path).ok()?;
    let mut hasher = Sha256::new();
    let mut buf = [0u8; 4096];

    loop {
        let n = file.read(&mut buf).ok()?;
        if n == 0 { break; }
        hasher.update(&buf[..n]);
    }

    let result = hasher.finalize();
    Some(hex::encode(result))
}

pub fn update_hash(target: &str, deps: &[String], store: &mut HashStore) {
    let mut combined = Sha256::new();
    for dep in deps {
        combined.update(hash_file(dep).unwrap_or_default());
    }
    let new_hash = hex::encode(combined.finalize());
    store.hashes.insert(target.to_string(), new_hash);
}

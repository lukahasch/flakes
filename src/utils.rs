use std::hash::Hasher;
use std::collections::hash_map::DefaultHasher;
use std::hash::Hash;

pub fn hash<T: Hash>(data: &T) -> u64 {
    let mut hasher = DefaultHasher::new();
    data.hash(&mut hasher);
    hasher.finish()
}

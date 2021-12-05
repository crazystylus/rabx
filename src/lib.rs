//! # kvs crate
//! `kvs` is a in-memory key value store for storing, retreving and deleting keys

#![deny(missing_docs)]
use std::collections::{hash_map::RandomState, HashMap};

/// This struct stores the key-value pairs using a HashMap
pub struct KvStore {
    hm: HashMap<String, String, RandomState>,
}

impl Default for KvStore {
    fn default() -> Self {
        Self::new()
    }
}

impl KvStore {
    /// This returns a new key value store
    ///
    /// # Examples
    /// ```
    /// let mut my_kvs = kvs::KvStore::new();
    /// ```
    pub fn new() -> Self {
        Self { hm: HashMap::new() }
    }

    /// Gets the value associated with that respective key
    /// # Examples
    /// ```
    /// let value1 = String::from("value1");
    /// let mut my_kvs = kvs::KvStore::new();
    /// my_kvs.set(String::from("key1"), value1);
    /// assert_eq!(my_kvs.get(String::from("key1")), Some(String::from("value1")));
    /// assert_eq!(my_kvs.get(String::from("key2")), None);
    /// ```
    pub fn get(&self, key: String) -> Option<String> {
        self.hm.get(&key).map(|value| value.to_owned())
    }

    /// Inserts a `key`:`value` pair into the
    /// key value store
    pub fn set(&mut self, key: String, value: String) {
        self.hm.insert(key, value);
    }

    /// Drops the key value pair from key store
    /// # Examples
    /// ```
    /// let value1 = String::from("value1");
    /// let mut my_kvs = kvs::KvStore::new();
    /// my_kvs.set(String::from("key1"), value1);
    /// assert_eq!(my_kvs.get(String::from("key1")), Some(String::from("value1")));
    /// my_kvs.remove( String::from("key1"));
    /// assert_eq!(my_kvs.get(String::from("key1")), None);
    /// ```
    pub fn remove(&mut self, key: String) {
        self.hm.remove(&key);
    }
}

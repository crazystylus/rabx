//! # kvs crate
//! `kvs` is a in-memory key value store for storing, retreving and deleting keys

#![deny(missing_docs)]
mod command;
use anyhow::{anyhow, Context, Result};
use std::collections::{hash_map::RandomState, HashMap};
use std::fs::OpenOptions;
use std::io::{BufReader, BufWriter};

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
    pub fn get(&mut self, key: String) -> Result<String> {
        // Open DB
        let f = OpenOptions::new()
            .read(true)
            .open("db.bson")
            .with_context(|| {
                "Unable to open the database file: \"db.bson\" for reading".to_string()
            })?;
        let mut db_reader = BufReader::new(f);
        // Restore hash_map
        loop {
            match bson::de::from_reader::<_, command::Command>(&mut db_reader) {
                Ok(doc) => match doc {
                    command::Command::Set { key, value } => self.hm.insert(key, value),
                    command::Command::Rm { key } => self.hm.remove(&key),
                },
                Err(_) => break,
            };
        }
        let value = self
            .hm
            .get(&key)
            .map(|value| value.to_owned())
            .ok_or(anyhow!("key: {} not found in store", key))?;
        Ok(value)
    }

    /// Inserts a `key`:`value` pair into the
    /// key value store
    pub fn set(&mut self, key: String, value: String) -> Result<()> {
        let kvs_command = bson::to_document(&command::Command::Set { key, value })?;
        // Open DB
        let f = OpenOptions::new()
            .append(true)
            .create(true)
            .open("db.bson")
            .with_context(|| {
                "Unable to open the database file: \"db.bson\" for appending".to_string()
            })?;
        let mut db_writer = BufWriter::new(f);
        // Append the command at the end
        kvs_command.to_writer(&mut db_writer)?;
        //self.hm.insert(key, value);
        Ok(())
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
    pub fn remove(&mut self, key: String) -> Result<()> {
        let kvs_command = bson::to_document(&command::Command::Rm { key })?;
        // Open DB
        let f = OpenOptions::new()
            .append(true)
            .open("db.bson")
            .with_context(|| {
                "Unable to open the database file: \"db.bson\" for appending".to_string()
            })?;
        let mut db_writer = BufWriter::new(f);
        // Append the command at the end
        kvs_command.to_writer(&mut db_writer)?;
        //self.hm.insert(key, value);
        Ok(())
    }
}

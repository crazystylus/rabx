//! # kvs crate
//! `kvs` is a in-memory key value store for storing, retreving and deleting keys

#![deny(missing_docs)]
mod command;
pub use anyhow::Result;
use anyhow::{anyhow, Context};
use std::collections::{hash_map::RandomState, HashMap};
use std::fs::OpenOptions;
use std::io::{BufReader, BufWriter, Seek, SeekFrom, Write};
use std::path::PathBuf;

/// This struct stores the key-value pairs using a HashMap
pub struct KvStore {
    hm: HashMap<String, u64, RandomState>,
    db_path: PathBuf,
    total: usize,
    actual: usize,
}

impl KvStore {
    /// This opens the key value store from the file
    pub fn open(path: impl Into<PathBuf>) -> Result<Self> {
        let mut kvs = HashMap::new();
        // Open DB
        let mut db_path: PathBuf = path.into();
        db_path.push("db");
        db_path.set_extension("bson");
        if !db_path.is_file() {
            return Ok(Self {
                hm: kvs,
                db_path,
                total: 0,
                actual: 0,
            });
        }
        let f = OpenOptions::new()
            .read(true)
            .open(&db_path)
            .with_context(|| {
                format!(
                    "Unable to open the database file: {} for reading",
                    db_path.to_str().unwrap()
                )
            })?;
        let mut db_reader = BufReader::new(f);
        // Restore hash_map
        let mut total_records: usize = 0;
        loop {
            let curr_pos = db_reader.stream_position().unwrap();
            total_records += 1;
            match bson::de::from_reader::<_, command::Command>(&mut db_reader) {
                Ok(doc) => match doc {
                    command::Command::Set { key, .. } => kvs.insert(key, curr_pos),
                    command::Command::Rm { key } => kvs.remove(&key),
                },
                Err(_) => break,
            };
        }
        let actual_records = kvs.len();
        Ok(KvStore {
            hm: kvs,
            db_path,
            total: total_records,
            actual: actual_records,
        })
    }

    /// Gets the value associated with that respective key
    /// # Examples
    /// ```
    /// use anyhow::{anyhow, Result};
    /// use tempfile::TempDir;
    /// let temp_dir = TempDir::new().expect("unable to create temporary working directory");
    /// let mut my_kvs = kvs::KvStore::open(temp_dir.path()).unwrap();
    /// my_kvs.set(String::from("key1"), String::from("value1")).unwrap();
    /// assert_eq!(my_kvs.get(String::from("key1")).unwrap().unwrap(), String::from("value1"));
    /// assert!(my_kvs.get(String::from("key2")).unwrap().is_none());
    /// ```
    pub fn get(&self, key: String) -> Result<Option<String>> {
        let record_pos = self.hm.get(&key).map(|value| value.to_owned());
        if record_pos.is_none() {
            return Ok(None);
        }
        let record_pos = record_pos.unwrap();
        let f = OpenOptions::new()
            .read(true)
            .append(true)
            .create(true)
            .open(&self.db_path)
            .with_context(|| {
                format!(
                    "Unable to open the database file: {} for reading",
                    self.db_path.to_str().unwrap()
                )
            })?;
        let mut db_reader = BufReader::new(f);
        db_reader.seek(SeekFrom::Start(record_pos))?;
        match bson::de::from_reader::<_, command::Command>(&mut db_reader)? {
            command::Command::Set { value, .. } => Ok(Some(value)),
            _ => Err(anyhow!("Key not found")),
        }
    }

    /// Inserts a `key`:`value` pair into the
    /// key value store
    /// # Examples
    /// ```
    /// use anyhow::{anyhow, Result};
    /// use tempfile::TempDir;
    /// let temp_dir = TempDir::new().expect("unable to create temporary working directory");
    /// let mut my_kvs = kvs::KvStore::open(temp_dir.path()).unwrap();
    /// my_kvs.set(String::from("key1"), String::from("value1")).unwrap();
    /// my_kvs.set(String::from("key2"), String::from("value2")).unwrap();
    /// assert_eq!(my_kvs.get(String::from("key1")).unwrap().unwrap(), String::from("value1"));
    /// assert_eq!(my_kvs.get(String::from("key2")).unwrap().unwrap(), String::from("value2"));
    /// ```
    pub fn set(&mut self, key: String, value: String) -> Result<()> {
        let kvs_command = bson::to_document(&command::Command::Set {
            key: key.clone(),
            value,
        })?;
        // Open DB
        let f = OpenOptions::new()
            .append(true)
            .create(true)
            .open(&self.db_path)
            .with_context(|| {
                format!(
                    "Unable to open the database file: {} for appending",
                    self.db_path.to_str().unwrap()
                )
            })?;
        let mut db_writer = BufWriter::new(f);
        let curr_pos = db_writer.seek(SeekFrom::End(0))?;
        // Append the command at the end
        kvs_command.to_writer(&mut db_writer)?;
        // Add to in-memory DB
        if self.hm.insert(key, curr_pos).is_none() {
            self.actual += 1;
        }
        self.total += 1;
        db_writer.flush()?;
        if self.total / (self.actual + 1) > 2 {
            self.compaction()?;
        }
        Ok(())
    }

    /// Drops the key value pair from key store
    /// # Examples
    /// ```
    /// use tempfile::TempDir;
    /// let temp_dir = TempDir::new().expect("unable to create temporary working directory");
    /// let mut my_kvs = kvs::KvStore::open(temp_dir.path()).unwrap();
    /// my_kvs.set(String::from("key1"), String::from("value1")).unwrap();
    /// assert_eq!(my_kvs.get(String::from("key1")).unwrap().unwrap(), String::from("value1"));
    /// my_kvs.remove( String::from("key1")).unwrap();
    /// assert!(my_kvs.get(String::from("key1")).unwrap().is_none());
    /// ```
    pub fn remove(&mut self, key: String) -> Result<()> {
        self.hm.remove(&key).ok_or(anyhow!("Key not found"))?;
        let kvs_command = bson::to_document(&command::Command::Rm { key })?;
        // Open DB
        let f = OpenOptions::new()
            .append(true)
            .create(true)
            .open(&self.db_path)
            .with_context(|| {
                format!(
                    "Unable to open the database file: {} for appending",
                    self.db_path.to_str().unwrap()
                )
            })?;
        let mut db_writer = BufWriter::new(f);
        // Append the command at the end
        kvs_command.to_writer(&mut db_writer)?;
        db_writer.flush()?;
        self.total += 2;
        if self.total / (self.actual + 1) > 2 {
            self.compaction()?;
        }
        Ok(())
    }
    /// This reduces the size of the DB log via cleanup
    fn compaction(&mut self) -> Result<()> {
        let mut final_db = self.db_path.clone();
        final_db.pop();
        final_db.push("bath.bson");
        // Open DB
        let f = OpenOptions::new()
            .append(true)
            .create(true)
            .open(&final_db)
            .with_context(|| {
                format!(
                    "Unable to open the database file: {} for appending",
                    self.db_path.to_str().unwrap()
                )
            })?;
        let mut db_writer = BufWriter::new(f);
        for key in self.hm.keys() {
            let kvs_command = bson::to_document(&command::Command::Set {
                key: key.clone(),
                value: self.get(key.clone()).unwrap().unwrap(),
            })?;
            // Append the command at the end
            kvs_command.to_writer(&mut db_writer)?;
        }
        db_writer.flush()?;
        std::fs::remove_file(&self.db_path)?;
        std::fs::rename(&final_db, &self.db_path)?;
        let f = OpenOptions::new()
            .read(true)
            .open(&self.db_path)
            .with_context(|| {
                format!(
                    "Unable to open the database file: {} for reading",
                    self.db_path.to_str().unwrap()
                )
            })?;
        let mut db_reader = BufReader::new(f);
        // Restore hash_map
        let mut total_records: usize = 0;
        loop {
            let curr_pos = db_reader.stream_position().unwrap();
            total_records += 1;
            match bson::de::from_reader::<_, command::Command>(&mut db_reader) {
                Ok(doc) => match doc {
                    command::Command::Set { key, .. } => self.hm.insert(key, curr_pos),
                    command::Command::Rm { key } => self.hm.remove(&key),
                },
                Err(_) => break,
            };
        }
        self.actual = self.hm.len();
        self.total = total_records;
        Ok(())
    }
}

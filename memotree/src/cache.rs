//! # Rust Cache Library
//!
//! This is a robust cache library implemented in Rust. It provides a `Cache` struct that allows you to store key-value pairs with a fixed capacity and perform various operations on the cache.
//!
//! ## Usage
//!
//! To use this library, add the following line to your `Cargo.toml` file:
//!
//! ```toml
//! [dependencies]
//! rust_cache = "0.1.0"
//! ```
//!
//! ## Example
//!
//! ```
//! use rust_cache::{Cache, ListProps, Order, Filter, StartAfter};
//!
//! fn main() {
//!     // Create a new cache with a capacity of 100
//!     let mut cache: Cache<i32> = Cache::new(100);
//!
//!     // Insert key-value pairs into the cache
//!     cache.insert_str("key1", 1);
//!     cache.insert_str("key2", 2);
//!     cache.insert_str("key3", 3);
//!
//!     // Get a value from the cache
//!     if let Some(value) = cache.get("key1") {
//!         println!("Value: {}", value);
//!     }
//!
//!     // Remove a key from the cache
//!     cache.remove("key2");
//!
//!     // List all key-value pairs in the cache
//!     let list_props = ListProps::new()
//!         .start_after_key("key1")
//!         .filter(Filter::None)
//!         .order(Order::Asc)
//!         .limit(10);
//!
//!     if let Ok(list) = cache.list(list_props) {
//!         for (key, value) in list {
//!             println!("Key: {}, Value: {}", key, value);
//!         }
//!     }
//! }
//! ```
//!
//! ## Structs
//!
//! ### `Cache<V>`
//!
//! A cache struct that stores key-value pairs.
//!
//! #### Type Parameters
//!
//! - `V`: The type of the values stored in the cache.
//!
//! #### Methods
//!
//! - `new(capacity: usize) -> Cache<V>`: Creates a new cache with the specified capacity.
//! - `insert(&mut self, key: &'static str, value: V)`: Inserts a key-value pair into the cache. If the key already exists, the value is updated.
//! - `insert_if_not_exists(&mut self, key: &'static str, value: V) -> Result<(), Error>`: Inserts a key-value pair into the cache only if the key does not already exist.
//! - `get(&self, key: &str) -> Option<&V>`: Returns a reference to the value associated with the given key, or `None` if the key is not found in the cache.
//! - `get_mut(&mut self, key: &str) -> Option<&mut V>`: Returns a mutable reference to the value associated with the given key, or `None` if the key is not found in the cache.
//! - `capacity(&self) -> usize`: Returns the capacity of the cache.
//! - `set_capacity(&mut self, capacity: usize)`: Sets the capacity of the cache.
//! - `remove(&mut self, key: &str) -> Result<(), Error>`: Removes the key-value pair with the given key from the cache.
//! - `clear(&mut self)`: Removes all key-value pairs from the cache.
//! - `len(&self) -> usize`: Returns the number of key-value pairs in the cache.
//! - `is_empty(&self) -> bool`: Returns `true` if the cache is empty, `false` otherwise.
//! - `contains_key(&self, key: &str) -> bool`: Returns `true` if the cache contains the given key, `false` otherwise.
//! - `list<T>(&self, props: T) -> Result<Vec<(&str, &V)>, Error>`: Returns a list of key-value pairs in the cache based on the provided list properties.
//!
//! ### Enums
//!
//! #### `Error`
//!
//! An enumeration of possible errors that can occur in the cache operations.
//!
//! ### Enums for Filtering and Sorting
//!
//! #### `Filter`
//!
//! An enumeration of filter options for listing key-value pairs in the cache.
//!
//! #### `Order`
//!
//! An enumeration of sorting order options for listing key-value pairs in the cache.
//!
//! #### `StartAfter`
//!
//! An enumeration of the start position options for listing key-value pairs in the cache.
//!
//! ### Structs for Listing Properties
//!
//! #### `ListProps`
//!
//! A struct that holds the properties for listing key-value pairs in the cache.
//!
//! ## License
//!
//! This library is licensed under the MIT License.

use std::collections::HashMap;
use std::fmt::Debug;
use std::fmt::Display;

pub enum Error {
    SortKeyNotFound,
    CacheAlreadyExists,
    SortKeyExists,
    TableAlreadyExists,
    KeyNotFound,
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Error::SortKeyNotFound => write!(f, "Sort key not found"),
            Error::CacheAlreadyExists => write!(f, "Cache already exists"),
            Error::SortKeyExists => write!(f, "Sort key exists"),
            Error::TableAlreadyExists => write!(f, "Table already exists"),
            Error::KeyNotFound => write!(f, "Key not found"),
        }
    }
}

impl Debug for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        Display::fmt(&self, f)
    }
}

#[derive(Debug)]
pub enum Filter {
    StartWith(&'static str),
    EndWith(&'static str),
    StartAndEndWith(&'static str, &'static str),
    None,
}

impl Default for Filter {
    fn default() -> Self {
        Self::None
    }
}

#[derive(Debug, Clone)]
pub enum Order {
    Asc,
    Desc,
}

impl Default for Order {
    fn default() -> Self {
        Self::Asc
    }
}

#[derive(Debug, Clone)]
pub enum StartAfter {
    Key(&'static str),
    None,
}

impl Default for StartAfter {
    fn default() -> Self {
        Self::None
    }
}

#[derive(Default, Debug)]
pub struct ListProps {
    pub start_after_key: StartAfter,
    pub filter: Filter,
    pub order: Order,
    pub limit: usize,
}

impl ListProps {
    fn new() -> Self {
        Self {
            start_after_key: StartAfter::None,
            filter: Filter::None,
            order: Order::Asc,
            limit: 10,
        }
    }

    pub fn start_after_key(mut self, key: &'static str) -> Self {
        self.start_after_key = StartAfter::Key(key);
        self
    }

    pub fn filter(mut self, filter: Filter) -> Self {
        self.filter = filter;
        self
    }

    pub fn order(mut self, order: Order) -> Self {
        self.order = order;
        self
    }
}

impl From<Filter> for ListProps {
    fn from(filter: Filter) -> Self {
        Self {
            start_after_key: StartAfter::None,
            filter,
            order: Order::Asc,
            limit: 10,
        }
    }
}

impl From<Order> for ListProps {
    fn from(order: Order) -> Self {
        Self {
            start_after_key: StartAfter::None,
            filter: Filter::None,
            order,
            limit: 10,
        }
    }
}

impl From<StartAfter> for ListProps {
    fn from(start_after_key: StartAfter) -> Self {
        Self {
            start_after_key,
            filter: Filter::None,
            order: Order::Asc,
            limit: 10,
        }
    }
}

pub type Key = String;

#[derive(Clone, Debug, PartialEq)]
pub struct Cache<V>
where
    V: PartialEq,
{
    map: HashMap<Key, V>,
    list: Vec<Key>,
    capacity: usize,
    _phantom: std::marker::PhantomData<V>,
}

impl<V> Cache<V>
where
    V: PartialEq,
{
    pub fn new(capacity: usize) -> Self {
        Self {
            map: HashMap::new(),
            list: Vec::new(),
            capacity,
            _phantom: std::marker::PhantomData,
        }
    }

    pub fn insert_str(&mut self, key: &'static str, value: V) {
        self.insert(key.to_string(), value);
    }

    pub fn insert(&mut self, key: Key, value: V) {
        if let Some(value) = self.map.get(&key) {
            if value.eq(value) {
                return;
            }
        }

        if self.map.len() != 0 && self.map.len() == self.capacity {
            let first_key = self.list.remove(0);
            self.map.remove(&first_key);
        }

        // sorted insert
        let position = self
            .list
            .iter()
            .position(|k| k > &key)
            .unwrap_or(self.list.len());
        self.list.insert(position, key.to_string());
        self.map.insert(key, value.into());
    }

    pub fn insert_if_not_exists(&mut self, key: Key, value: V) -> Result<(), Error> {
        if self.map.contains_key(&key) {
            return Err(Error::SortKeyExists);
        }

        self.insert(key, value);
        Ok(())
    }

    pub fn get(&self, key: &str) -> Option<&V> {
        self.map.get(key)
    }

    pub fn get_mut(&mut self, key: &str) -> Option<&mut V> {
        self.map.get_mut(key)
    }

    pub fn capacity(&self) -> usize {
        self.capacity
    }

    pub fn set_capacity(&mut self, capacity: usize) {
        self.capacity = capacity;
    }

    pub fn remove(&mut self, key: &str) -> Result<(), Error> {
        match self.list.iter().position(|k| k == &key) {
            Some(position) => {
                self.list.remove(position);
                self.map.remove(key);
                Ok(())
            }
            None => Err(Error::KeyNotFound),
        }
    }

    pub fn clear(&mut self) {
        self.map.clear();
        self.list.clear();
    }

    pub fn len(&self) -> usize {
        self.map.len()
    }

    pub fn is_empty(&self) -> bool {
        self.map.is_empty()
    }

    pub fn contains_key(&self, key: &str) -> bool {
        self.map.contains_key(key)
    }

    pub fn list<T>(&self, props: T) -> Result<Vec<(Key, &V)>, Error>
    where
        T: Into<ListProps>,
    {
        let props = props.into();

        let position = match props.start_after_key {
            StartAfter::Key(key) => {
                self.list
                    .iter()
                    .position(|k| k == &key)
                    .ok_or(Error::SortKeyNotFound)?
                    + 1
            }
            StartAfter::None => 0,
        };

        let mut list = Vec::new();
        let mut count = 0;

        match props.order {
            Order::Asc => {
                let skip_iter = self.list.iter().skip(position);
                for k in skip_iter {
                    let filtered = match props.filter {
                        Filter::StartWith(key) => {
                            if k.starts_with(&key) {
                                Some((k.clone(), self.map.get(k).unwrap()))
                            } else {
                                None
                            }
                        }
                        Filter::EndWith(key) => {
                            if k.ends_with(&key) {
                                Some((k.clone(), self.map.get(k).unwrap()))
                            } else {
                                None
                            }
                        }
                        Filter::StartAndEndWith(start_key, end_key) => {
                            if k.starts_with(&start_key) && k.ends_with(&end_key) {
                                Some((k.clone(), self.map.get(k).unwrap()))
                            } else {
                                None
                            }
                        }
                        Filter::None => Some((k.clone(), self.map.get(k).unwrap())),
                    };

                    if let Some(item) = filtered {
                        list.push(item);
                        count += 1;
                        if count == props.limit {
                            break;
                        }
                    }
                }
            }
            Order::Desc => {
                let skip_iter = self.list.iter().rev().skip(position);
                for k in skip_iter {
                    let filtered = match props.filter {
                        Filter::StartWith(key) => {
                            if k.starts_with(&key) {
                                Some((k.clone(), self.map.get(k).unwrap()))
                            } else {
                                None
                            }
                        }
                        Filter::EndWith(key) => {
                            if k.ends_with(&key) {
                                Some((k.clone(), self.map.get(k).unwrap()))
                            } else {
                                None
                            }
                        }
                        Filter::StartAndEndWith(start_key, end_key) => {
                            if k.starts_with(&start_key) && k.ends_with(&end_key) {
                                Some((k.clone(), self.map.get(k).unwrap()))
                            } else {
                                None
                            }
                        }
                        Filter::None => Some((k.clone(), self.map.get(k).unwrap())),
                    };

                    if let Some(item) = filtered {
                        list.push(item);
                        count += 1;
                        if count == props.limit {
                            break;
                        }
                    }
                }
            }
        };

        Ok(list)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_cache_insert() {
        let mut cache = Cache::new(2);
        cache.insert_str("key1", 1);
        cache.insert_str("key2", 2);
        cache.insert_str("key3", 3);
        assert_eq!(cache.get("key1"), None);
        assert_eq!(cache.get("key2"), Some(&2));
        assert_eq!(cache.get("key3"), Some(&3));
    }

    #[test]
    fn test_cache_remove() {
        let mut cache = Cache::new(2);
        cache.insert_str("key1", 1);
        cache.insert_str("key2", 2);
        cache.remove("key1");
        assert_eq!(cache.get("key1"), None);
        cache.insert_str("key3", 3);
        assert_eq!(cache.get("key3"), Some(&3));
        assert_eq!(cache.get("key2"), Some(&2));
    }

    #[test]
    fn test_cache_clear() {
        let mut cache = Cache::new(2);
        cache.insert_str("key1", 1);
        cache.insert_str("key2", 2);
        cache.clear();
        assert_eq!(cache.len(), 0);
    }

    #[test]
    fn test_cache_list_asc() {
        let mut cache = Cache::new(5);
        cache.insert_str("key2", 2);
        cache.insert_str("key1", 1);
        cache.insert_str("key5", 5);
        cache.insert_str("key4", 4);
        cache.insert_str("key3", 3);

        let result_res = cache.list(StartAfter::Key("key2"));

        assert_eq!(result_res.is_ok(), true);

        let result = match result_res {
            Ok(result) => result,
            Err(_) => panic!("Error"),
        };

        assert_eq!(result.len(), 3);
        assert_eq!(result[0], ("key3".to_string(), &3));
        assert_eq!(result[1], ("key4".to_string(), &4));
        assert_eq!(result[2], ("key5".to_string(), &5));
    }

    #[test]
    fn test_cache_list_desc() {
        let mut cache = Cache::new(5);
        cache.insert_str("key5", 5);
        cache.insert_str("key1", 1);
        cache.insert_str("key3", 3);
        cache.insert_str("key4", 4);
        cache.insert_str("key2", 2);

        let result_res = cache.list(ListProps {
            order: Order::Desc,
            filter: Filter::None,
            start_after_key: StartAfter::Key("key3"),
            limit: 10,
        });

        assert_eq!(result_res.is_ok(), true);

        let result = match result_res {
            Ok(result) => result,
            Err(_) => panic!("Error"),
        };

        assert_eq!(result.len(), 2);
        assert_eq!(result[0], ("key2".to_string(), &2));
        assert_eq!(result[1], ("key1".to_string(), &1));
    }

    #[test]
    fn test_filter_start_with() {
        let mut cache = Cache::new(10);

        cache.insert_str("postmodern", 8);
        cache.insert_str("postpone", 6);
        cache.insert_str("precept", 2);
        cache.insert_str("postmortem", 9);
        cache.insert_str("precaution", 3);
        cache.insert_str("precede", 1);
        cache.insert_str("precognition", 5);
        cache.insert_str("postmark", 10);
        cache.insert_str("postgraduate", 7);
        cache.insert_str("preconceive", 4);

        let result_res = cache.list(Filter::StartWith("postm"));

        assert_eq!(result_res.is_ok(), true);

        let result = match result_res {
            Ok(result) => result,
            Err(_) => panic!("Error"),
        };

        assert_eq!(result.len(), 3);
        assert_eq!(result[0], ("postmark".to_string(), &10));
        assert_eq!(result[1], ("postmodern".to_string(), &8));
        assert_eq!(result[2], ("postmortem".to_string(), &9));
    }

    #[test]
    fn test_filter_ends_with() {
        let mut cache = Cache::new(10);

        cache.insert_str("postmodern", 8);
        cache.insert_str("postpone", 6);
        cache.insert_str("precept", 2);
        cache.insert_str("postmortem", 9);
        cache.insert_str("precaution", 3);
        cache.insert_str("precede", 1);
        cache.insert_str("precognition", 5);
        cache.insert_str("postmark", 10);
        cache.insert_str("postgraduate", 7);
        cache.insert_str("preconceive", 4);

        let result_res = cache.list(Filter::EndWith("tion"));

        assert_eq!(result_res.is_ok(), true);

        let result = match result_res {
            Ok(result) => result,
            Err(_) => panic!("Error"),
        };

        assert_eq!(result.len(), 2);
        assert_eq!(result[0], ("precaution".to_string(), &3));
        assert_eq!(result[1], ("precognition".to_string(), &5));
    }
}

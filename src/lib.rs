//! # SkipList-rs (Educational Implementation)
//!
//! An educational skip list implementation in Rust.
//!
//! **Note**: It is not intended for production use and has not been optimized or thoroughly tested for all edge cases.
//!
//! ## Educational Features Demonstrated
//!
//!
//!
//! ## Quick Example
//!
//! ```rust
//! use skiplist_rs::SkipList;
//!
//! let mut skiplist = SkipList::new();
//!
//! // Insert key-value pairs
//! skiplist.insert(1, "one");
//! skiplist.insert(2, "two");
//! skiplist.insert(3, "three");
//!
//! // Retrieve values
//! assert_eq!(skiplist.get(&2), Some("two"));
//! assert_eq!(skiplist.get(&5), None);
//! ```
//!

use rand::Rng;
use std::sync::{Arc, RwLock};
use std::cmp::Ordering;

type Link<K, V> = Option<Arc<RwLock<Node<K, V>>>>;

/// Internal node structure for the skip list
struct Node<K, V> {
    key: Option<K>,
    val: Option<V>,
    fwd: Vec<Link<K, V>>,
}

impl<K, V> Node<K, V> {
    /// Create a head node with the specified maximum levels
    fn head(max_levels: usize) -> Self {
        Node {
            key: None, 
            val: None, 
            fwd: vec![None; max_levels],
        }
    }

    /// Create a new entry node with the specified level
    fn entry(key: K, val: V, level: usize) -> Self {
        Node {
            key: Some(key),
            val: Some(val),
            fwd: vec![None; level],
        }
    }    
}

/// A thread-safe skip list with dynamic level management.
///
/// Skip lists are probabilistic data structures that maintain elements in sorted order
/// and provide O(log n) average-case performance for search, insertion, and deletion.
///
/// This implementation automatically adjusts its level structure based on the number
/// of elements to maintain optimal performance characteristics.
pub struct SkipList<K, V> {
    head: Arc<RwLock<Node<K, V>>>,
    max: usize,
    len: usize,
    p: f64,
}

impl<K, V> Default for SkipList<K, V>
where
    K: Ord,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<K, V> SkipList<K, V>
where
    K: Ord,
{
    /// Creates a new empty skip list with default parameters.
    ///
    /// Uses probability 0.5 and starts with 4 initial levels.
    ///
    /// # Examples
    ///
    /// ```
    /// use skiplist_rs::SkipList;
    /// 
    /// let mut skiplist: SkipList<i32, String> = SkipList::new();
    /// skiplist.insert(1, "hello".to_string());
    /// ```
    pub fn new() -> Self {
        Self::with_params(4, 0.5)
    }

    /// Creates a new skip list with custom parameters.
    ///
    /// # Arguments
    ///
    /// * `initial_max` - Initial number of levels (will grow automatically)
    /// * `p` - Probability for level generation (typically 0.25 or 0.5)
    ///
    /// # Examples
    ///
    /// ```
    /// use skiplist_rs::SkipList;
    /// 
    /// let mut skiplist: SkipList<i32, String> = SkipList::with_params(8, 0.25);
    /// ```
    pub fn with_params(initial_max: usize, p: f64) -> Self {
        Self {
            head: Arc::new(RwLock::new(Node::head(initial_max))),
            max: initial_max,
            len: 0,
            p,
        }
    }

    /// Returns the number of elements in the skip list.
    ///
    /// # Examples
    ///
    /// ```
    /// use skiplist_rs::SkipList;
    /// 
    /// let mut skiplist = SkipList::new();
    /// assert_eq!(skiplist.len(), 0);
    /// 
    /// skiplist.insert(1, "one");
    /// assert_eq!(skiplist.len(), 1);
    /// ```
    pub fn len(&self) -> usize {
        self.len
    }

    /// Returns true if the skip list is empty.
    ///
    /// # Examples
    ///
    /// ```
    /// use skiplist_rs::SkipList;
    /// 
    /// let skiplist: SkipList<i32, String> = SkipList::new();
    /// assert!(skiplist.is_empty());
    /// ```
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Calculate optimal max level based on current number of records
    fn optimal_levels(&self) -> usize {
        if self.len == 0 {
            return 1;
        }
        let optimal = ((self.len as f64).log2().ceil() as usize) + 2;
        optimal.max(1)
    }

    /// Dynamically adjust max level if needed
    fn resize(&mut self) {
        let optimal = self.optimal_levels();
        if optimal > self.max {
            self.grow(optimal);
        }
    }

    /// Grow the head node to accommodate more levels
    fn grow(&mut self, new_max: usize) {
        let mut head = self.head.write().unwrap();
        while head.fwd.len() < new_max {
            head.fwd.push(None);
        }
        self.max = new_max;
    }

    /// Generate a random level for a new node
    fn random_level(&self) -> usize {
        let mut rng = rand::thread_rng();
        let mut lvl = 1; 
        while rng.gen_bool(self.p) && lvl < self.max {
            lvl += 1;
        }
        lvl
    }
    
    /// Inserts a key-value pair into the skip list.
    ///
    /// If the key already exists, the old value is replaced and returned.
    /// If the key is new, `None` is returned.
    ///
    /// # Arguments
    ///
    /// * `key` - The key to insert
    /// * `value` - The value to associate with the key
    ///
    /// # Returns
    ///
    /// The previous value if the key existed, or `None` if it's a new key.
    ///
    /// # Examples
    ///
    /// ```
    /// use skiplist_rs::SkipList;
    /// 
    /// let mut skiplist = SkipList::new();
    /// 
    /// // Insert new key
    /// assert_eq!(skiplist.insert(1, "one"), None);
    /// 
    /// // Update existing key
    /// assert_eq!(skiplist.insert(1, "ONE"), Some("one"));
    /// ```
    pub fn insert(&mut self, key: K, value: V) -> Option<V>
    where
        V: Clone,
    {
        let mut update = vec![None; self.max];
        let mut curr = Arc::clone(&self.head);

        // Search phase: find predecessors at each level
        for level in (0..self.max).rev() {
            loop {
                let next = {
                    let curr_ref = curr.read().unwrap();
                    curr_ref.fwd[level].clone()
                };

                match next {
                    Some(node) => {
                        let should_advance = {
                            let node_ref = node.read().unwrap();
                            node_ref.key.as_ref().unwrap() < &key
                        };

                        if should_advance {
                            curr = node;
                        } else {
                            break;
                        }
                    }
                    None => break,
                }
            }
            update[level] = Some(Arc::clone(&curr));
        }

        // Check if key already exists
        if let Some(next) = update[0].as_ref().unwrap().read().unwrap().fwd[0].clone() {
            let mut next_ref = next.write().unwrap();
            if let Some(existing_key) = &next_ref.key {
                if existing_key == &key {
                    return next_ref.val.replace(value);
                }
            }
        }

        // Insert phase: create new node and link it in
        let new_level = self.random_level();
        let new_node = Arc::new(RwLock::new(Node::entry(key, value, new_level)));

        for level in 0..new_level {
            let mut prev = update[level].as_ref().unwrap().write().unwrap();
            new_node.write().unwrap().fwd[level] = prev.fwd[level].take();
            prev.fwd[level] = Some(Arc::clone(&new_node));
        }

        self.len += 1;
        self.resize();
        None
    }

    /// Retrieves a value by its key.
    ///
    /// # Arguments
    ///
    /// * `key` - The key to search for
    ///
    /// # Returns
    ///
    /// The value associated with the key, or `None` if the key doesn't exist.
    ///
    /// # Examples
    ///
    /// ```
    /// use skiplist_rs::SkipList;
    /// 
    /// let mut skiplist = SkipList::new();
    /// skiplist.insert(1, "one");
    /// 
    /// assert_eq!(skiplist.get(&1), Some("one"));
    /// assert_eq!(skiplist.get(&2), None);
    /// ```
    pub fn get(&self, key: &K) -> Option<V> 
    where 
        V: Clone 
    {
        let mut curr = Arc::clone(&self.head);

        for level in (0..self.max).rev() {
            loop {
                let next = {
                    let curr_ref = curr.read().unwrap();
                    curr_ref.fwd[level].clone()
                };

                match next {
                    Some(node) => {
                        match {
                            let node_ref = node.read().unwrap();
                            node_ref.key.as_ref().map(|k| k.cmp(key)) 
                        } {
                            Some(Ordering::Less) => {
                                curr = node;
                            }
                            Some(Ordering::Equal) => {
                                return node.read().unwrap().val.clone();
                            }
                            _ => break,
                        }
                    }
                    None => break,
                }
            }
        }
        None
    }

    /// Returns true if the skip list contains the specified key.
    ///
    /// # Examples
    ///
    /// ```
    /// use skiplist_rs::SkipList;
    /// 
    /// let mut skiplist = SkipList::new();
    /// skiplist.insert(1, "one");
    /// 
    /// assert!(skiplist.contains_key(&1));
    /// assert!(!skiplist.contains_key(&2));
    /// ```
    pub fn contains_key(&self, key: &K) -> bool 
    where 
        V: Clone 
    {
        self.get(key).is_some()
    }
} 
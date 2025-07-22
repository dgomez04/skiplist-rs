/*
Level 2:    [A] ----------------------------> [Z]
Level 1:    [A] ------> [M] --------> [T] --> [Z]
Level 0:    [A]->[B]->[C]->...->[M]->...->[R]->[S]->[T]->...->[Z]
*/
use rand::{Rng};

struct Node<K, V> {
    key: Option<K>,
    val: Option<V>,
    level: usize,
    fwd: Vec<*mut Node<K, V>>,
}

impl<K, V> Node<K, V> {
    fn new(key: K, val: V, level: usize) -> Self {
        Self {
            key: Some(key),
            val: Some(val),
            level,
            fwd: vec![std::ptr::null_mut(); level],
        }
    }
    // Header node constructor.
    fn header(max: usize) -> Self {
        Self {
            key: None, 
            val: None, 
            level: max,
            fwd: vec![std::ptr::null_mut(); max],  
        }
    }
}

struct SkipList<K, V> 
{
    head: Box<Node<K, V>>,
    max: usize,
    len: usize,
    p: f64,
}

impl<K: Ord, V> SkipList<K, V>
{
    fn new(max: usize, p: f64) -> Self {
        Self {
            head: Box::new(Node::header(max)),
            max,
            len: 0,
            p,
        }
    }

    fn random_level(&self) -> usize {
        let mut rng = rand::thread_rng();
        let mut lvl = 1; 

        while rng.gen_bool(self.p) && lvl < self.max {
            lvl += 1;
        }

        lvl
    }
    
    fn put(&mut self, _k: K, _v: V) -> Option<V> {
        let mut updates: Vec<*mut Node<K, V>> = vec![std::ptr::null_mut(); self.max];
        let mut curr = self.head.as_mut() as *mut Node<K, V>;

        // Traverse from top to bottom.
        for level in (0..self.max).rev() {
            unsafe {
                while level < (*curr).level && !(*curr).fwd[level].is_null() {
                    let next = (*curr).fwd[level];
                    if let Some(k) = &(*next).key {
                        if k < &_k {
                            curr = next;
                        } else {
                            break;
                        }
                    } else {
                        break;
                    }
                }
                updates[level] = curr;
            }
        }

        // Check if key already exists (update case).
        unsafe {
            if !(*curr).fwd[0].is_null() {
                let next = (*curr).fwd[0];
                if let Some(k) = &(*next).key {
                    if k == &_k {
                        // Key exists - update and return old value
                        return (*next).val.replace(_v);
                    }
                }
            }
        }

        // Create new node and insert.
        let _lvl = self.random_level();
        let new = Box::new(Node::new(_k, _v, _lvl));
        let new_ptr = Box::into_raw(new);

        // Insert at all levels from 0 to _lvl - 1.

        unsafe {
            for level in 0.._lvl.min(self.max) {
                (*new_ptr).fwd[level] = (*updates[level]).fwd[level];
                (*updates[level]).fwd[level] = new_ptr;
            }
        }
        
        self.len += 1;
        None
    }

    fn get(&self, _k: K) -> Option<&V> {
        let mut curr = self.head.as_ref() as *const Node<K, V>;

        // Traverse from top to bottom.
        for level in (0..self.max).rev() {
            unsafe {
                while level < (*curr).level && !(*curr).fwd[level].is_null() {
                    let next = (*curr).fwd[level] as *const Node<K, V>;
                    if let Some(k) = &(*next).key {
                        if k < &_k {
                            curr = next;
                        } else {
                            break;
                        }
                    }
                }
            }
        }

        // Check if next node at level 0 has the key. 
        unsafe {
            if !(*curr).fwd[0].is_null() {
                let next = (*curr).fwd[0] as *const Node<K, V>;
                if let Some(k) = &(*next).key {
                    if k == &_k {
                        return (*next).val.as_ref();
                    }
                }
            }
        }

        None
    }

    fn for_each<F>(&self, mut _f: F)
    where 
        F: FnMut(&K, &V)
    {
        // Get the pointer to the first node. 
        let mut curr = self.head.as_ref() as *const Node<K, V>;

        // Traverse level 0. 
        unsafe {
            // Skip the header node. 
            curr = (*curr).fwd[0];

            while !curr.is_null() {
                if let (Some(k), Some(v)) = (&(*curr).key, &(*curr).val) {
                    _f(k, v);
                }
                curr = (*curr).fwd[0];
            }
        }

    }
    
    fn size(&self) -> usize {
        self.len 
    }
}


impl <K, V> Drop for SkipList<K, V> {
    fn drop(&mut self) {
        // Traverse level 0 and free all nodes.
        let mut curr = self.head.fwd[0];

        while !curr.is_null() {
            unsafe {
                let next = (*curr).fwd[0];

                let _box = Box::from_raw(curr);
    
                curr = next; 
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_skiplist() {
        let list: SkipList<String, i32> = SkipList::new(4, 0.5);
        assert_eq!(list.size(), 0);
        assert_eq!(list.get("key".to_string()), None);
        
        // Test iteration on empty list
        let mut count = 0;
        list.for_each(|_, _| count += 1);
        assert_eq!(count, 0);
    }

    #[test]
    fn test_single_insert_and_get() {
        let mut list = SkipList::new(4, 0.5);
        
        // Insert and verify
        assert_eq!(list.put("hello".to_string(), 42), None);
        assert_eq!(list.size(), 1);
        assert_eq!(list.get("hello".to_string()), Some(&42));
        
        // Non-existent key
        assert_eq!(list.get("world".to_string()), None);
    }

    #[test]
    fn test_multiple_inserts() {
        let mut list = SkipList::new(4, 0.5);
        
        // Insert multiple values
        list.put("apple".to_string(), 1);
        list.put("banana".to_string(), 2);
        list.put("cherry".to_string(), 3);
        
        assert_eq!(list.size(), 3);
        assert_eq!(list.get("apple".to_string()), Some(&1));
        assert_eq!(list.get("banana".to_string()), Some(&2));
        assert_eq!(list.get("cherry".to_string()), Some(&3));
    }

    #[test]
    fn test_update_existing_key() {
        let mut list = SkipList::new(4, 0.5);
        
        // Insert initial value
        assert_eq!(list.put("key".to_string(), 100), None);
        assert_eq!(list.size(), 1);
        
        // Update existing key - should return old value
        assert_eq!(list.put("key".to_string(), 200), Some(100));
        assert_eq!(list.size(), 1); // Size shouldn't change
        assert_eq!(list.get("key".to_string()), Some(&200));
    }

    #[test]
    fn test_ordering() {
        let mut list = SkipList::new(4, 0.5);
        
        // Insert in random order
        list.put("zebra".to_string(), 26);
        list.put("apple".to_string(), 1);
        list.put("monkey".to_string(), 13);
        list.put("bear".to_string(), 2);
        
        // Collect values in iteration order
        let mut collected = Vec::new();
        list.for_each(|key, value| {
            collected.push((key.clone(), *value));
        });
        
        // Should be in alphabetical order
        assert_eq!(collected, vec![
            ("apple".to_string(), 1),
            ("bear".to_string(), 2),
            ("monkey".to_string(), 13),
            ("zebra".to_string(), 26),
        ]);
    }

    #[test]
    fn test_numeric_keys() {
        let mut list = SkipList::new(4, 0.5);
        
        // Test with numeric keys
        list.put(100, "hundred");
        list.put(50, "fifty");
        list.put(200, "two hundred");
        list.put(25, "twenty five");
        
        assert_eq!(list.get(50), Some(&"fifty"));
        assert_eq!(list.get(999), None);
        
        // Check ordering
        let mut keys = Vec::new();
        list.for_each(|key, _| keys.push(*key));
        assert_eq!(keys, vec![25, 50, 100, 200]);
    }

    #[test]
    fn test_large_dataset() {
        let mut list = SkipList::new(8, 0.5);
        
        // Insert many items
        for i in 0..1000 {
            list.put(i, i * 2);
        }
        
        assert_eq!(list.size(), 1000);
        
        // Verify random access
        assert_eq!(list.get(500), Some(&1000));
        assert_eq!(list.get(999), Some(&1998));
        assert_eq!(list.get(1000), None);
        
        // Verify iteration produces correct count
        let mut count = 0;
        list.for_each(|_, _| count += 1);
        assert_eq!(count, 1000);
    }

    #[test]
    fn test_string_values() {
        let mut list = SkipList::new(4, 0.5);
        
        list.put(1, "first".to_string());
        list.put(3, "third".to_string());
        list.put(2, "second".to_string());
        
        assert_eq!(list.get(2), Some(&"second".to_string()));
        
        // Test update with string
        let old = list.put(2, "updated second".to_string());
        assert_eq!(old, Some("second".to_string()));
        assert_eq!(list.get(2), Some(&"updated second".to_string()));
    }

    #[test]
    fn test_for_each_functionality() {
        let mut list = SkipList::new(4, 0.5);
        
        list.put("a", 1);
        list.put("c", 3);
        list.put("b", 2);
        
        let mut sum = 0;
        let mut keys = Vec::new();
        
        list.for_each(|key, value| {
            sum += *value;
            keys.push(key.clone());
        });
        
        assert_eq!(sum, 6);
        assert_eq!(keys, vec!["a", "b", "c"]); // Should be in sorted order
    }

    #[test]
    fn test_lsm_tombstone_pattern() {
        // Simulate LSM tombstone pattern
        #[derive(Debug, PartialEq, Clone)]
        enum Value {
            Data(String),
            Tombstone,
        }
        
        let mut tombstone_list: SkipList<String, Value> = SkipList::new(4, 0.5);
        
        // Insert data
        tombstone_list.put("user1".to_string(), Value::Data("Alice".to_string()));
        tombstone_list.put("user2".to_string(), Value::Data("Bob".to_string()));
        
        // "Delete" by inserting tombstone
        tombstone_list.put("user1".to_string(), Value::Tombstone);
        
        assert_eq!(tombstone_list.get("user1".to_string()), Some(&Value::Tombstone));
        assert_eq!(tombstone_list.get("user2".to_string()), Some(&Value::Data("Bob".to_string())));
        assert_eq!(tombstone_list.size(), 2);
    }

    #[test]
    fn test_edge_cases() {
        let mut list = SkipList::new(4, 0.5);
        
        // Test with single character keys
        list.put("z", 26);
        list.put("a", 1);
        list.put("m", 13);
        
        assert_eq!(list.get("a"), Some(&1));
        assert_eq!(list.get("z"), Some(&26));
        
        // Test empty string key
        list.put("", 0);
        assert_eq!(list.get(""), Some(&0));
        
        let mut keys = Vec::new();
        list.for_each(|k, _| keys.push(k.clone()));
        assert_eq!(keys, vec!["", "a", "m", "z"]);
    }
}
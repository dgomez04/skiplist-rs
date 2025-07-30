/*
Level 2:    [A] ----------------------------> [Z]
Level 1:    [A] ------> [M] --------> [T] --> [Z]
Level 0:    [A]->[B]->[C]->...->[M]->...->[R]->[S]->[T]->...->[Z]
*/
use rand::{Rng};
use std::sync::{Arc, RwLock};
use std::cmp::Ordering;

/*
    Reference counted nodes.  READ MORE ABOUT THIS TO BETTER UNDERSTAND THE WHY. 
    All will be 'max_level' to simplify traversal logic and bounds check, trading off for memory efficiency.   
*/
type Link<K, V> = Option<Arc<RwLock<Node<K, V>>>>;

struct Node<K, V> {
    key: Option<K>,
    val: Option<V>,
    fwd: Vec<Link<K, V>>,
}

impl<K, V> Node<K, V> {
    fn head(_max: usize) -> Self {
        Node {
            key: None, 
            val: None, 
            fwd: vec![None; _max],
        }
    }

    fn entry(_k: K, _v: V, _lvl: usize) -> Self {
        Node {
            key: Some(_k),
            val: Some(_v),
            fwd: vec![None; _lvl],
        }
    }    
}

struct SkipList<K, V> 
{
    head: Arc<RwLock<Node<K, V>>>,
    max: usize,
    len: usize,
    p: f64,
}

impl<K: Ord, V> SkipList<K, V>
{
    fn new(max: usize, p: f64) -> Self {
        Self {
            head: Arc::new(RwLock::new(Node::head(max))),
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
    
    fn put(&mut self, _k: K, _v: V) -> Option<V>
    where
        K: Ord,
        V: Clone,
    {
        let mut update = vec![None; self.max];
        let mut curr = Arc::clone(&self.head);

        for level in (0..self.max).rev() {
            loop {
                let next = {
                    let curr_ref = curr.read().unwrap();
                    curr_ref.fwd[level].clone()
                };

                match next {
                    Some(node) => {
                        let less = {
                            let node_ref = node.read().unwrap();
                            node_ref.key.as_ref().unwrap() < &_k
                        };

                        if less {
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

        if let Some(next) = update[0].as_ref().unwrap().read().unwrap().fwd[0].clone() {
            let mut next_ref = next.write().unwrap();
            if let Some(key) = &next_ref.key {
                if key == &_k {
                    return next_ref.val.replace(_v);
                }
            }
        }

        let _lvl = self.random_level();
        let entry = Arc::new(RwLock::new(Node::entry(_k, _v, _lvl)));

        for level in 0.._lvl {
            let mut prev = update[level].as_ref().unwrap().write().unwrap();
            entry.write().unwrap().fwd[level] = prev.fwd[level].take();
            prev.fwd[level] = Some(Arc::clone(&entry));
        }

        self.len += 1;
        None
    }


    fn get(&self, _k: K) -> Option<V> 
    where 
        K: Ord,
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
                            node_ref.key.as_ref().map(|k| k.cmp(&_k)) 
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
    
    fn size(&self) -> usize {
        self.len 
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::*;
    use std::sync::Arc;
    use std::thread;
    use std::time::Duration;

    #[test]
    fn insertion_test() {
        let mut skiplist = SkipList::new(4, 0.5);
        
        // Insert some key-value pairs
        assert_eq!(skiplist.put(1, "one"), None);
        assert_eq!(skiplist.put(3, "three"), None);
        assert_eq!(skiplist.put(2, "two"), None);
        
        // Verify size
        assert_eq!(skiplist.size(), 3);
        
        // Verify all values can be retrieved
        assert_eq!(skiplist.get(1), Some("one"));
        assert_eq!(skiplist.get(2), Some("two"));
        assert_eq!(skiplist.get(3), Some("three"));
    }

    #[test]
    fn concurrent_writes_test() {
        let list = Arc::new(RwLock::new(SkipList::new(6, 0.5)));

        let mut handles = vec![];

        for i in 0..4 {
            let list_clone = Arc::clone(&list);
            let handle = thread::spawn(move || {
                let start = i * 1000;
                let end = start + 1000;

                for key in start..end {
                    let mut skiplist = list_clone.write().unwrap();
                    skiplist.put(key, format!("val-{key}"));
                }
            });

            handles.push(handle);
        }

        for handle in handles {
            handle.join().expect("Thread panicked");
        }

        // Final check
        let skiplist = list.read().unwrap();
        assert_eq!(skiplist.size(), 4000);
        assert_eq!(skiplist.get(0), Some("val-0".to_string()));
        assert_eq!(skiplist.get(3999), Some("val-3999".to_string()));
    }

    #[test]
    fn concurrent_rw_test() {
        let skiplist = Arc::new(RwLock::new(SkipList::new(6, 0.5)));

        let writer_skiplist = Arc::clone(&skiplist);
        let writer = thread::spawn(move || {
            for i in 0..1000 {
                let mut list = writer_skiplist.write().unwrap();
                list.put(i, format!("val-{i}"));
                // Optional: simulate delay to increase reader collision chances
                if i % 100 == 0 {
                    thread::sleep(Duration::from_millis(1));
                }
            }
        });

        let mut readers = vec![];
        for _ in 0..4 {
            let reader_skiplist = Arc::clone(&skiplist);
            let handle = thread::spawn(move || {
                for _ in 0..500 {
                    let list = reader_skiplist.read().unwrap();
                    let _ = list.get(42); // Arbitrary key, may or may not exist yet
                    let _ = list.get(999); // Reading near the end
                }
            });
            readers.push(handle);
        }

        writer.join().expect("Writer thread panicked");
        for reader in readers {
            reader.join().expect("Reader thread panicked");
        }

        let list = skiplist.read().unwrap();
        assert_eq!(list.size(), 1000);
        assert_eq!(list.get(0), Some("val-0".to_string()));
        assert_eq!(list.get(999), Some("val-999".to_string()));
    }


    #[test]
    fn in_place_update_test() {
        let mut skiplist = SkipList::new(4, 0.5);
        
        // Insert initial value
        assert_eq!(skiplist.put(42, "initial"), None);
        assert_eq!(skiplist.size(), 1);
        
        // Update existing key - should return old value
        assert_eq!(skiplist.put(42, "updated"), Some("initial"));
        assert_eq!(skiplist.size(), 1); // Size should remain the same
        
        // Verify the updated value
        assert_eq!(skiplist.get(42), Some("updated"));
    }

    #[test]
    fn get_test() {
        let mut skiplist = SkipList::new(4, 0.5);
        
        // Insert values in non-sorted order
        skiplist.put(10, "ten");
        skiplist.put(5, "five");
        skiplist.put(15, "fifteen");
        skiplist.put(7, "seven");
        
        // Test getting existing values
        assert_eq!(skiplist.get(5), Some("five"));
        assert_eq!(skiplist.get(7), Some("seven"));
        assert_eq!(skiplist.get(10), Some("ten"));
        assert_eq!(skiplist.get(15), Some("fifteen"));
    }

    #[test]
    fn missing_key_test() {
        let mut skiplist = SkipList::new(4, 0.5);
        
        // Test getting from empty skiplist
        assert_eq!(skiplist.get(1), None);
        
        // Insert some values
        skiplist.put(2, "two");
        skiplist.put(4, "four");
        skiplist.put(6, "six");
        
        // Test getting non-existent keys
        assert_eq!(skiplist.get(1), None); // Before range
        assert_eq!(skiplist.get(3), None); // In middle
        assert_eq!(skiplist.get(5), None); // In middle
        assert_eq!(skiplist.get(7), None); // After range
        
        // Verify existing keys still work
        assert_eq!(skiplist.get(2), Some("two"));
        assert_eq!(skiplist.get(4), Some("four"));
        assert_eq!(skiplist.get(6), Some("six"));
    }
}



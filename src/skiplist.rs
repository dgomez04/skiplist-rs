/*
Level 2:    [A] ----------------------------> [Z]
Level 1:    [A] ------> [M] --------> [T] --> [Z]
Level 0:    [A]->[B]->[C]->...->[M]->...->[R]->[S]->[T]->...->[Z]
*/
use rand::{Rng};

struct Node<K, V> {
    key: K,
    val: V,
    level: usize,
    fwd: Vec<Option<Box<Node<K, V>>>>,
}

impl<K, V> Node<K, V> {
    fn new(key: K, val: V, level: usize) -> Self {
        Self {
            key,
            val,
            level,
            fwd: vec![None; level],
        }
    }


}

struct SkipList<K, V> 
{
    head: Option<Box<Node<K, V>>>,
    max: usize,
    len: usize,
    p: f64,
}

impl<K, V> SkipList<K, V>
{
    fn new(max: usize, p: f64) -> Self {
        Self {
            head: None,
            max,
            len: 0,
            p,
        }
    }

    fn random_level(&self) -> usize {
        let mut rng = rand::thread_rng();
        let mut lvl = 0; 
        
        while rng.gen_bool(self.p) && lvl < self.max {
            lvl += 1;
        }

        lvl
    }
    
    fn put(&mut self, _k: K, _v: V) -> Option<V> {
        return None;
    }

    fn get(&self, _k: K) -> Option<&V> {
        return None;
    }
    
    fn size(&self) -> usize {
        self.len 
    }
}
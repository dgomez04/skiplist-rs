/*
Level 2:    [A] ----------------------------> [Z]
Level 1:    [A] ------> [M] --------> [T] --> [Z]
Level 0:    [A]->[B]->[C]->...->[M]->...->[R]->[S]->[T]->...->[Z]
*/
use rand::{Rng};
use std::rc::Rc;
use std::cell::RefCell;

/*
    Reference counted nodes.  READ MORE ABOUT THIS TO BETTER UNDERSTAND THE WHY. 
    All will be 'max_level' to simplify traversal logic and bounds check, trading off for memory efficiency.   
*/
type Link<K, V> = Option<Rc<RefCell<Node<K, V>>>>;
enum Node<K, V> {
    Head {
        fwd: Vec<Link<K, V>>,
        lvl: usize,
    },
    Entry {
        k: K,
        v: V,
        lvl: usize,
        fwd: Vec<Link<K, V>>,
    },
}

impl<K, V> Node<K, V> {
    fn head(_lvl: usize) -> Self {
        Node::Head {
            fwd: vec![None; _lvl],
            lvl: _lvl,
        }
    }

    fn entry(_k: K, _v: V, _lvl: usize) -> Self {
        Node::Entry {
            k: _k,
            v: _v,
            lvl: _lvl,
            fwd: vec![None; _lvl],
        }
    }

    fn get_fwd(&self) -> &Vec<Link<K, V>> {
        match self {
            Node::Head { fwd, .. } => fwd,
            Node::Entry { fwd, .. } => fwd,
        }
    }

    fn get_fwd_mut(&mut self) -> &mut Vec<Link<K, V>> {
        match self {
            Node::Head { fwd, .. } => fwd,
            Node::Entry { fwd, .. } => fwd,
        }
    }
    
    fn get_lvl(&self) -> usize {
        match self {
            Node::Head { lvl, .. } => *lvl,
            Node::Entry { lvl, .. } => *lvl,
        }
    }

    fn get_key(&self) -> Option<&K> {
        match self {
            Node::Head { .. } => None,
            Node::Entry { k, .. } => Some(k),
        }
    }

    fn get_val(&self) -> Option<&V> {
        match self {
            Node::Head { .. } => None,
            Node::Entry { v, .. } => Some(v),
        }
    }       
}

struct SkipList<K, V> 
{
    head: Rc<RefCell<Node<K, V>>>,
    max: usize,
    len: usize,
    p: f64,
}

impl<K: Ord, V> SkipList<K, V>
{
    fn new(max: usize, p: f64) -> Self {
        Self {
            head: Rc::new(RefCell::new(Node::head(max))),
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
    /*
        Here we're not considering what if the value we're on is the same as the key. This is an optimization.
     */
    fn get(&self, _k: K) -> Option<V> where V: Clone {
        let mut curr = self.head.clone();

        // Traverse from top to bottom.
        for level in (0..self.max).rev() {
            loop {
                let curr_ref = curr.borrow();

                if curr_ref.get_lvl() <= level {
                    break;
                }

                let next_opt = curr_ref.get_fwd()[level].clone();
                drop(curr_ref);

                if let Some(next) = next_opt {
                    let next_ref = next.borrow();
                    if let Some(k) = next_ref.get_key() {
                        if k < &_k {
                            drop(next_ref);
                            curr = next;
                            continue;
                        } else {
                            break;
                        }
                    }
                }
            }
        }

        // Check if next node at level 0 has the key. 
        if let Some(next) = curr.borrow().get_fwd()[0].clone() {
            if let Some(k) = next.borrow().get_key() {
                if k == &_k {
                    return next.borrow().get_val().cloned();
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
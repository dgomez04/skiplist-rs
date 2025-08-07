# skiplist-rs

An educational Skiplist implementation in Rust-a probabilistic data structure.
> **⚠️ Educational Project**: This is a learning implementation created for educational purposes. It is not intended for production use and has not been optimized or thoroughly tested for all edge cases.

## Quick Example

```rust
use skiplist_rs::SkipList;

fn main() {
    let mut skiplist = SkipList::new();

    // Insert key-value pairs
    skiplist.insert(1, "one");
    skiplist.insert(2, "two");
    skiplist.insert(3, "three");

    // Retrieve values
    assert_eq!(skiplist.get(&2), Some("two"));
    assert_eq!(skiplist.get(&5), None);

    // Update existing values
    let old_value = skiplist.insert(2, "TWO");
    assert_eq!(old_value, Some("two"));
}
```

## Concurrent Access Example

```rust
use skiplist_rs::SkipList;
use std::sync::{Arc, RwLock};
use std::thread;

fn main() {
    let skiplist = Arc::new(RwLock::new(SkipList::new()));

    let mut handles = vec![];
    for i in 0..4 {
        let list = Arc::clone(&skiplist);
        let handle = thread::spawn(move || {
            let mut sl = list.write().unwrap();
            sl.insert(i, format!("value_{}", i));
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.join().unwrap();
    }

    let sl = skiplist.read().unwrap();
    println!("Final size: {}", sl.len());
}
```

This implementation shows how Skiplists achieve O(log n) performance through:

| Elements | Levels | Expected Comparisons |
|----------|--------|---------------------|
| 1-4      | 3-4    | ~2-4               |
| 100      | ~9     | ~9                 |
| 1,000    | ~12    | ~12                |
| 10,000   | ~15    | ~15                |


### Core Methods

- `SkipList::new()` - Create a new skip list
- `SkipList::with_params(levels, probability)` - Create with custom parameters
- `insert(key, value)` - Insert or update a key-value pair
- `get(&key)` - Retrieve a value by key
- `contains_key(&key)` - Check if a key exists
- `len()` - Get the number of elements
- `is_empty()` - Check if empty

## Testing

```bash
# Run all tests
cargo test

# Run specific test categories
cargo test basic_tests
cargo test concurrent_tests

```

## How Skip Lists Work

At their core, Skiplists are just linked lists—except each node has multiple forward pointers arranged into “levels.” The bottom level links every node in order, like a standard list, while higher levels act as express lanes, letting us skip over large sections of data. It’s a simple trick that gives us performance close to balanced trees or sorted arrays, without the complexity of maintaining balance.

```
Level 2: [1] ------------------------------> [15]
Level 1: [1] -> [4] --------> [9] ---------> [15]
Level 0: [1] -> [4] -> [6] -> [9] -> [12] -> [15]
```

Each level is a subset of the level below, allowing efficient search by "skipping" elements.

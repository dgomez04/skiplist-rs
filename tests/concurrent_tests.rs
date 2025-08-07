use skiplist_rs::SkipList;
use std::sync::{Arc, RwLock};
use std::thread;
use std::time::Duration;

#[test]
fn test_concurrent_writes() {
    let list = Arc::new(RwLock::new(SkipList::new()));
    let mut handles = vec![];

    // Spawn 4 threads, each inserting different ranges
    for i in 0..4 {
        let list_clone = Arc::clone(&list);
        let handle = thread::spawn(move || {
            let start = i * 1000;
            let end = start + 1000;

            for key in start..end {
                let mut skiplist = list_clone.write().unwrap();
                skiplist.insert(key, format!("val-{}", key));
            }
        });
        handles.push(handle);
    }

    // Wait for all threads to complete
    for handle in handles {
        handle.join().expect("Thread panicked");
    }

    // Final verification
    let skiplist = list.read().unwrap();
    assert_eq!(skiplist.len(), 4000);
    assert_eq!(skiplist.get(&0), Some("val-0".to_string()));
    assert_eq!(skiplist.get(&3999), Some("val-3999".to_string()));
}

#[test]
fn test_concurrent_reads_and_writes() {
    let skiplist = Arc::new(RwLock::new(SkipList::new()));

    // Writer thread
    let writer_skiplist = Arc::clone(&skiplist);
    let writer = thread::spawn(move || {
        for i in 0..1000 {
            let mut list = writer_skiplist.write().unwrap();
            list.insert(i, format!("val-{}", i));
            // Simulate delay to increase reader collision chances
            if i % 100 == 0 {
                drop(list); // Release lock
                thread::sleep(Duration::from_millis(1));
            }
        }
    });

    // Reader threads
    let mut readers = vec![];
    for _ in 0..4 {
        let reader_skiplist = Arc::clone(&skiplist);
        let handle = thread::spawn(move || {
            for _ in 0..500 {
                let list = reader_skiplist.read().unwrap();
                let _ = list.get(&42); // May or may not exist yet
                let _ = list.get(&999); // Reading near the end
            }
        });
        readers.push(handle);
    }

    // Wait for all threads to complete
    writer.join().expect("Writer thread panicked");
    for reader in readers {
        reader.join().expect("Reader thread panicked");
    }

    // Final verification
    let list = skiplist.read().unwrap();
    assert_eq!(list.len(), 1000);
    assert_eq!(list.get(&0), Some("val-0".to_string()));
    assert_eq!(list.get(&999), Some("val-999".to_string()));
}

#[test]
fn test_concurrent_updates() {
    let skiplist = Arc::new(RwLock::new(SkipList::new()));
    
    // Initialize with base values
    {
        let mut sl = skiplist.write().unwrap();
        for i in 0..10 {
            sl.insert(i, format!("initial_{}", i));
        }
    }

    let mut handles = vec![];

    // Multiple threads updating the same keys
    for thread_id in 0..3 {
        let skiplist_clone = Arc::clone(&skiplist);
        let handle = thread::spawn(move || {
            for round in 0..5 {
                let mut sl = skiplist_clone.write().unwrap();
                for key in 0..10 {
                    let old_val = sl.insert(key, format!("thread_{}_round_{}_key_{}", thread_id, round, key));
                    // Verify we got some previous value back
                    assert!(old_val.is_some());
                }
                drop(sl);
                thread::sleep(Duration::from_millis(1));
            }
        });
        handles.push(handle);
    }

    // Wait for all updates to complete
    for handle in handles {
        handle.join().unwrap();
    }

    // Verify structure is still consistent
    let sl = skiplist.read().unwrap();
    assert_eq!(sl.len(), 10); // Size should remain 10

    // All keys should still exist and have some valid value
    for i in 0..10 {
        let value = sl.get(&i);
        assert!(value.is_some());
        let val_str = value.unwrap();
        assert!(val_str.starts_with("thread_") && val_str.contains(&format!("key_{}", i)));
    }
}

#[test]
fn test_thread_safety_stress() {
    let skiplist = Arc::new(RwLock::new(SkipList::new()));
    let mut handles = vec![];

    // High contention test: many threads doing mixed operations
    for thread_id in 0..8 {
        let skiplist_clone = Arc::clone(&skiplist);
        let handle = thread::spawn(move || {
            for i in 0..100 {
                let key = (thread_id * 100 + i) % 50; // Overlap keys to create contention
                
                {
                    let mut sl = skiplist_clone.write().unwrap();
                    sl.insert(key, format!("t{}_i{}", thread_id, i));
                }
                
                // Quick read test
                {
                    let sl = skiplist_clone.read().unwrap();
                    let _ = sl.get(&key); // Just ensure it doesn't panic
                }
            }
        });
        handles.push(handle);
    }

    // Wait for completion
    for handle in handles {
        handle.join().unwrap();
    }

    // Verify final state is consistent
    let sl = skiplist.read().unwrap();
    
    // Should have at most 50 unique keys
    assert!(sl.len() <= 50);
    
    // All keys in range [0, 49] that exist should return valid values
    for i in 0..50 {
        if let Some(value) = sl.get(&i) {
            assert!(value.starts_with("t") && value.contains("_i"));
        }
    }
} 
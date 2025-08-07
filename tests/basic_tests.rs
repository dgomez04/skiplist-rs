use skiplist_rs::SkipList;

#[test]
fn test_insertion() {
    let mut skiplist = SkipList::new();
    
    // Insert some key-value pairs
    assert_eq!(skiplist.insert(1, "one"), None);
    assert_eq!(skiplist.insert(3, "three"), None);
    assert_eq!(skiplist.insert(2, "two"), None);
    
    // Verify size
    assert_eq!(skiplist.len(), 3);
    
    // Verify all values can be retrieved
    assert_eq!(skiplist.get(&1), Some("one"));
    assert_eq!(skiplist.get(&2), Some("two"));
    assert_eq!(skiplist.get(&3), Some("three"));
}

#[test]
fn test_update_existing_key() {
    let mut skiplist = SkipList::new();
    
    // Insert initial value
    assert_eq!(skiplist.insert(42, "initial"), None);
    assert_eq!(skiplist.len(), 1);
    
    // Update existing key - should return old value
    assert_eq!(skiplist.insert(42, "updated"), Some("initial"));
    assert_eq!(skiplist.len(), 1); // Size should remain the same
    
    // Verify the updated value
    assert_eq!(skiplist.get(&42), Some("updated"));
}

#[test]
fn test_basic_get() {
    let mut skiplist = SkipList::new();
    
    // Insert values in non-sorted order
    skiplist.insert(10, "ten");
    skiplist.insert(5, "five");
    skiplist.insert(15, "fifteen");
    skiplist.insert(7, "seven");
    
    // Test getting existing values
    assert_eq!(skiplist.get(&5), Some("five"));
    assert_eq!(skiplist.get(&7), Some("seven"));
    assert_eq!(skiplist.get(&10), Some("ten"));
    assert_eq!(skiplist.get(&15), Some("fifteen"));
}

#[test]
fn test_missing_key() {
    let mut skiplist = SkipList::new();
    
    // Test getting from empty skiplist
    assert_eq!(skiplist.get(&1), None);
    
    // Insert some values
    skiplist.insert(2, "two");
    skiplist.insert(4, "four");
    skiplist.insert(6, "six");
    
    // Test getting non-existent keys
    assert_eq!(skiplist.get(&1), None); // Before range
    assert_eq!(skiplist.get(&3), None); // In middle
    assert_eq!(skiplist.get(&5), None); // In middle
    assert_eq!(skiplist.get(&7), None); // After range
    
    // Verify existing keys still work
    assert_eq!(skiplist.get(&2), Some("two"));
    assert_eq!(skiplist.get(&4), Some("four"));
    assert_eq!(skiplist.get(&6), Some("six"));
}

#[test]
fn test_contains_key() {
    let mut skiplist = SkipList::new();
    
    skiplist.insert(1, "one");
    skiplist.insert(3, "three");
    
    assert!(skiplist.contains_key(&1));
    assert!(skiplist.contains_key(&3));
    assert!(!skiplist.contains_key(&2));
    assert!(!skiplist.contains_key(&4));
}

#[test]
fn test_empty_skiplist() {
    let skiplist: SkipList<i32, String> = SkipList::new();
    
    assert!(skiplist.is_empty());
    assert_eq!(skiplist.len(), 0);
    assert_eq!(skiplist.get(&1), None);
    assert!(!skiplist.contains_key(&1));
}

#[test]
fn test_skiplist_ordering() {
    let mut skiplist = SkipList::new();
    
    // Insert numbers in random order
    let values = vec![50, 20, 80, 10, 30, 70, 90, 5, 15, 25, 35, 60, 75, 85, 95];
    
    // Insert all values
    for &val in &values {
        assert_eq!(skiplist.insert(val, format!("value_{}", val)), None);
    }
    
    // Verify size
    assert_eq!(skiplist.len(), values.len());
    
    // Test that all inserted values can be retrieved correctly
    for &val in &values {
        assert_eq!(skiplist.get(&val), Some(format!("value_{}", val)));
    }
    
    // Test that search correctly handles values between inserted ones
    assert_eq!(skiplist.get(&12), None); // Between 10 and 15
    assert_eq!(skiplist.get(&22), None); // Between 20 and 25
    assert_eq!(skiplist.get(&65), None); // Between 60 and 70
    assert_eq!(skiplist.get(&100), None); // After all values
    assert_eq!(skiplist.get(&1), None); // Before all values
    
    // Test edge cases around existing values
    assert_eq!(skiplist.get(&4), None);   // Just before 5
    assert_eq!(skiplist.get(&6), None);   // Just after 5
    assert_eq!(skiplist.get(&94), None);  // Just before 95
    assert_eq!(skiplist.get(&96), None);  // Just after 95
    
    // Verify boundary values still work
    assert_eq!(skiplist.get(&5), Some("value_5".to_string()));   // Minimum
    assert_eq!(skiplist.get(&95), Some("value_95".to_string())); // Maximum
} 
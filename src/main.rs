use skiplist_rs::SkipList;

fn main() {
    let mut skiplist = SkipList::new();

    println!("Inserting values into skip list...");
    skiplist.insert(5, "five");
    skiplist.insert(1, "one");
    skiplist.insert(3, "three");
    skiplist.insert(7, "seven");
    skiplist.insert(2, "two");

    println!("\nSearching for values:");
    for key in 1..=7 {
        match skiplist.get(&key) {
            Some(value) => println!("Key {}: {}", key, value),
            None => println!("Key {}: Not found", key),
        }
    }

    let old_value = skiplist.insert(3, "THREE");
    println!("\nUpdated key 3. Old value: {:?}", old_value);
    println!("New value: {:?}", skiplist.get(&3));

    println!("\nFinal stats: {} elements", skiplist.len());
}

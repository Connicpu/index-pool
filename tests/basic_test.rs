extern crate index_pool;
use index_pool::IndexPool;

#[test]
fn basic_test() {
    let mut pool = IndexPool::new();

    let a = pool.new_id();
    let b = pool.new_id();
    let c = pool.new_id();

    let mut data = vec![""; pool.maximum()];
    data[a] = "apple";
    data[b] = "banana";
    data[c] = "coconut";

    // Nevermind, no bananas
    pool.return_id(b).unwrap();

    let p = pool.new_id();
    data[p] = "pineapple";

    assert_eq!(data, vec!["apple", "pineapple", "coconut"]);
}

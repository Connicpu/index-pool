extern crate index_pool;
use index_pool::IndexPool;

#[test]
fn basic_test() {
    let mut pool = IndexPool::new();

    assert_eq!(pool.maximum(), 0);
    assert_eq!(pool.in_use(), 0);

    let a = pool.new_id();

    assert_eq!(pool.maximum(), 1);
    assert_eq!(pool.in_use(), 1);

    let b = pool.new_id();

    assert_eq!(pool.maximum(), 2);
    assert_eq!(pool.in_use(), 2);

    let c = pool.new_id();

    assert_eq!(pool.maximum(), 3);
    assert_eq!(pool.in_use(), 3);
    assert_eq!([a, b, c], [0, 1, 2]);

    let mut data = vec![""; pool.maximum()];
    data[a] = "apple";
    data[b] = "banana";
    data[c] = "coconut";

    assert_eq!(data, vec!["apple", "banana", "coconut"]);

    // Nevermind, no bananas
    pool.return_id(b).unwrap();

    assert_eq!(pool.maximum(), 3);
    assert_eq!(pool.in_use(), 2);

    let p = pool.new_id();
    data[p] = "pineapple";

    assert_eq!(pool.maximum(), 3);
    assert_eq!(pool.in_use(), 3);
    assert_eq!([a, c, p], [0, 2, 1]);
    assert_eq!(data, vec!["apple", "pineapple", "coconut"]);
}

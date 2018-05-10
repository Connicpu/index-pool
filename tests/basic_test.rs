extern crate index_pool;
use index_pool::IndexPool;

#[test]
fn basic_test() {
    let mut pool = IndexPool::new();

    assert!(pool.is_free(0));
    assert!(pool.is_free(1));
    assert!(pool.is_free(2));

    assert_eq!(pool.maximum(), 0);
    assert_eq!(pool.in_use(), 0);
    assert!(pool.all_indices().eq(None));

    let a = pool.new_id();

    assert!(!pool.is_free(a));

    assert_eq!(pool.maximum(), 1);
    assert_eq!(pool.in_use(), 1);
    assert!(pool.all_indices().eq(Some(0)));

    let b = pool.new_id();

    assert!(!pool.is_free(a));
    assert!(!pool.is_free(b));

    assert_eq!(pool.maximum(), 2);
    assert_eq!(pool.in_use(), 2);
    assert!(pool.all_indices().eq([0, 1].iter().cloned()));

    let c = pool.new_id();

    assert!(!pool.is_free(a));
    assert!(!pool.is_free(b));
    assert!(!pool.is_free(c));

    assert_eq!(pool.maximum(), 3);
    assert_eq!(pool.in_use(), 3);
    assert_eq!([a, b, c], [0, 1, 2]);
    assert!(pool.all_indices().eq([0, 1, 2].iter().cloned()));

    let mut data = vec![""; pool.maximum()];
    data[a] = "apple";
    data[b] = "banana";
    data[c] = "coconut";

    assert_eq!(data, vec!["apple", "banana", "coconut"]);

    // Nevermind, no bananas
    pool.return_id(b).unwrap();

    assert!(!pool.is_free(a));
    assert!(pool.is_free(b));
    assert!(!pool.is_free(c));

    assert_eq!(pool.maximum(), 3);
    assert_eq!(pool.in_use(), 2);
    assert!(pool.all_indices().eq([0, 2].iter().cloned()));

    let p = pool.new_id();
    data[p] = "pineapple";

    assert!(!pool.is_free(a));
    assert!(!pool.is_free(p));
    assert!(!pool.is_free(c));

    assert_eq!(pool.maximum(), 3);
    assert_eq!(pool.in_use(), 3);
    assert_eq!([a, c, p], [0, 2, 1]);
    assert_eq!(data, vec!["apple", "pineapple", "coconut"]);
    assert!(pool.all_indices().eq([0, 1, 2].iter().cloned()));
}

#[test]
fn allocate_specific_values() {
    let mut pool = IndexPool::new();
    pool.new_id();

    assert!(pool.is_free(1));
    assert!(pool.is_free(5));

    assert_eq!(pool.request_id(1), Ok(()));

    assert!(!pool.is_free(1));
    assert!(pool.is_free(5));

    assert_eq!(pool.request_id(5), Ok(()));

    assert!(!pool.is_free(1));
    assert!(!pool.is_free(5));

    assert_eq!(pool.return_id(1), Ok(()));

    assert!(pool.is_free(1));
    assert!(!pool.is_free(5));

    assert_eq!(pool.return_id(5), Ok(()));

    assert!(pool.is_free(1));
    assert!(pool.is_free(5));
}

#[test]
fn all_indices() {
    let mut pool = IndexPool::new();

    pool.new_id();
    pool.new_id();
    pool.new_id();

    let mut iter = pool.all_indices();
    assert_eq!(iter.next(), Some(0));
    assert_eq!(iter.next(), Some(1));
    assert_eq!(iter.next(), Some(2));
    assert_eq!(iter.next(), None);
}

#[test]
fn iter_after() {
    let mut pool = IndexPool::new();

    pool.new_id();
    pool.new_id();
    pool.new_id();
    pool.new_id();
    pool.new_id();

    assert_eq!(pool.request_id(7), Ok(()));

    let mut iter = pool.all_indices_after(2);
    assert_eq!(iter.next(), Some(2));
    assert_eq!(iter.next(), Some(3));
    assert_eq!(iter.next(), Some(4));
    assert_eq!(iter.next(), Some(7));
    assert_eq!(iter.next(), None);

    let mut iter = pool.all_indices_after(5);
    assert_eq!(iter.next(), Some(7));
    assert_eq!(iter.next(), None);
}

#[test]
fn request_sequential() {
    let mut pool = IndexPool::new();

    assert_eq!(pool.request_id(5), Ok(()));
    assert!(!pool.is_free(5));
    assert_eq!(pool.request_id(6), Ok(()));
    assert!(!pool.is_free(6));
    assert_eq!(pool.request_id(7), Ok(()));
    assert!(!pool.is_free(7));
    assert_eq!(pool.request_id(8), Ok(()));
    assert!(!pool.is_free(8));
    assert_eq!(pool.request_id(9), Ok(()));
    assert!(!pool.is_free(9));
}

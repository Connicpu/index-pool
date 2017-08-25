# index-pool [![index-pool on crates.io](https://img.shields.io/crates/v/index-pool.svg)](https://crates.io/crates/index-pool) [![Build Status](https://travis-ci.org/Connicpu/index-pool.svg?branch=master)](https://travis-ci.org/Connicpu/index-pool) [![Build status](https://ci.appveyor.com/api/projects/status/oj0v6lw804y0hjv7?svg=true)](https://ci.appveyor.com/project/Connicpu/index-pool)

A pool which manages allocation of unique indices. Acts like a psuedo-memory allocator.

```toml
[dependencies]
index-pool = "1.0"
```

# Example

```rust
extern crate index_pool;
use index_pool::IndexPool;

fn main() {
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
```

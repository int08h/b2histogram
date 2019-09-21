b2histogram
===========

[![crates.io](https://img.shields.io/crates/v/b2histogram.svg?style=flat-square)](https://crates.io/crates/b2histogram)
[![Build Status](https://img.shields.io/travis/int08h/b2histogram/master.svg?style=flat-square)](https://travis-ci.org/int08h/b2histogram)
[![Apache License 2](https://img.shields.io/badge/license-ASF2-blue.svg?style=flat-square)](https://www.apache.org/licenses/LICENSE-2.0.txt)

A fast and efficient 64-bit integer histogram with power-of-2 spaced buckets.

* Fixed memory footprint (520 bytes) with no dynamic allocations
* Constant time record and retrieve operations that compile down to a few instructions
* `no_std` support
* Work in progress: Compact binary serialization

- [Documentation](https://docs.rs/b2histogram)
- [Release Notes](https://github.com/int08h/b2histogram/releases)

## Usage

Add this to your `Cargo.toml`:

```toml
[dependencies]
b2histogram = "1.0"
```

and this to your crate root:

```rust
#[macro_use]
extern crate b2histogram;
```

## Quick Example

```rust
extern crate b2histogram;

use b2histogram::Base2Histogram;

fn main() {
  let mut hist = Base2Histogram::new();

  hist.record(0); // Record a single observation of '0'
  hist.record(11); //
  hist.record(11); // Two observations of '11'
  hist.record_n(300_000, 6); // Six observations of 300,000

  // Retrieve counts directly
  println!("Observations for 300,000: {}", hist.observations(300_000));

  // Retrieve the `Bucket` covering a given value
  println!("Bucket corresponding to '11': {:?}", hist.bucket_for(11));

  // Iterate buckets that have observations
  for bucket in hist.iter().filter(|b| b.count > 0) {
      println!("({:5}, {:5}): {}", bucket.begin, bucket.end, bucket.count);
  }
}
```

See the [documentation](https://docs.rs/b2histogram) for more.


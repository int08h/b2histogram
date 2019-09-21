// Copyright 2019 int08h LLC
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! A fast and efficient 64-bit integer histogram with power-of-2 spaced buckets.
//!
//! * Fixed memory footprint (520 bytes) with no dynamic allocations
//! * Constant time record and retrieve operations that compile down to a few instructions
//! * `no_std` support
//! * Work in progress: Compact binary serialization
//!
//! # Example
//!
//! ```rust
//! extern crate b2histogram;
//!
//! use b2histogram::Base2Histogram;
//!
//! fn main() {
//!   let mut hist = Base2Histogram::new();
//!
//!   hist.record(0); // Record a single observation of '0'
//!   hist.record(11); //
//!   hist.record(11); // Two observations of '11'
//!   hist.record_n(300_000, 6); // Six observations of 300,000
//!
//!   // Retrieve counts directly
//!   println!("Observations for 300,000: {}", hist.observations(300_000));
//!
//!   // Retrieve the `Bucket` covering a given value
//!   println!("Bucket corresponding to '11': {:?}", hist.bucket_for(11));
//!
//!   // Iterate buckets that have observations
//!   for bucket in hist.iter().filter(|b| b.count > 0) {
//!       println!("({:5}, {:5}): {}", bucket.start, bucket.end, bucket.count);
//!   }
//! }
//! ```
//!
//! # Recording Observations
//!
//! Use the `record()` and `record_n()` methods to add observations to the histogram.
//!
//! * [`record(value: u64)`](struct.Base2Histogram.html#method.record) to record a single
//!   observation of `value`
//! * [`record_n(value: u64, count: u64)`](struct.Base2Histogram.html#method.record_n) to record
//!   `count` observations of the same `value`
//!
//! # Retrieving Observations
//!
//! [`observations(value: u64)`](struct.Base2Histogram.html#method.observations) returns the
//! current count for the bucket that covers `value`.
//!
//! [`bucket_for(value: u64)`](struct.Base2Histogram.html#method.bucket_for) will return a
//! [`Bucket`](struct.Bucket.html) struct for the bucket corresponding to `value`. A
//! [`Bucket`](struct.Bucket.html) provides fields to access the lower and upper bounds of the
//! bucket along with the current count.
//!
//! # Bucket Ranges
//!
//! Buckets cover the range `[2^n, 2^(n+1)-1]` including their start and end values (open-open)
//! for all powers of 2 from 0 through 2^62. The bottom-most bucket records observations of
//! zero and the top-most bucket covers `[2^62, +infinity]`.
//!
//! # Overflow Behavior
//!
//! Bucket counts **saturate** (reach maximum value and stay there) instead of overflowing or
//! wrapping around.
//!
//! # Implementation Details
//!
//! ## Bucket Mask
//!
//! The `mask` field is a 64-bit bitmask with each bit corresponding to one of the 64 buckets.
//! If the bit is 1 then that bucket has one or more observations while a 0 value means the
//! bucket is has no observations.
//!
//! The bit position at index `i` (from least-significant-bit (LSB) to most-significant
//! (MSB)) corresponds to bucket `[2^(i-1), (2^i)-1)]`. In diagram form:
//!
//! ```text
//!               MSB <------------------------ LSB
//!               +--+--+--+-------+--+--+--+--+--+
//!               |63|62|61| . . . | 4| 3| 2| 1| 0|
//!               +--+--+--+-------+--+--+--+--+--+
//!                 ^  ^             ^  ^  ^  ^  ^
//!                 |  |      +------+  |  |  |  +-------+
//!     Values      | ++      |    +----+  |  +----+     |
//!  (2^62, +inf) --+ |    Values  |       |       |  Values
//!                   |   (8, 15)  |       |       |  (0, 0)
//!                Values       Values  Values  Values
//!            (2^61, 2^62-1)   (4, 7)  (2, 3)  (1, 1)
//! ```
//!
//! ## Counts Field
//!
//! The `counts` field is a simple `u64` array of 64 elements. There is one array entry for
//! each bucket and array index `i` corresponds to mask bit `i`.

#![no_std]

///
/// A compact and efficient integer histogram with fixed memory footprint,
/// constant runtime performance, and very compact binary serialization.
///
pub struct Base2Histogram {
    counts: [u64; 64],
    mask: u64,
}

/// A bucket maintains a `count` of observations between its `begin` and `end` endpoints.
///
/// Buckets include their endpoint values (known as a "closed-closed" interval). Each
/// bucket covers `[2^n, (2^(n+1))-1]` with the exception of the top-most bucket which
/// covers `[2^62, +infinity]`.
///
#[derive(Debug)]
pub struct Bucket {
    /// Beginning of the range, inclusive
    pub start: u64,
    /// Maximum value of the range, inclusive
    pub end: u64,
    /// Number of observations in the bucket
    pub count: u64,
}

impl Base2Histogram {
    /// Create a new `Base2Histogram` instance
    pub fn new() -> Self {
        Base2Histogram {
            counts: [0u64; 64],
            mask: 0u64,
        }
    }

    /// Record a single observation of `value`
    #[inline]
    pub fn record(&mut self, value: u64) {
        self.record_n(value, 1);
    }

    /// Record `count` observations of `value`
    #[inline]
    pub fn record_n(&mut self, value: u64, count: u64) {
        let idx = self.index_of(value);

        self.counts[idx] = self.counts[idx].saturating_add(count);
        self.mask |= 1 << (idx as u64);
    }

    /// Returns the number of observations recorded by the bucket containing `value`
    ///
    /// To retrieve the number of observations along with its bucket bounds, see
    /// [`bucket_for()`](struct.Base2Histogram.html#method.bucket_for).
    #[inline]
    pub fn observations(&self, value: u64) -> u64 {
        let idx = self.index_of(value);
        self.counts[idx]
    }

    /// Return the `Bucket` to which `value` belongs.
    ///
    /// To retrieve only the number of observations see
    /// [`observations()`](struct.Base2Histogram.html#method.observations).
    #[inline]
    pub fn bucket_for(&self, value: u64) -> Bucket {
        let idx = self.index_of(value);
        self.bucket_at(idx)
    }

    /// Returns the number of buckets with one or more observations
    #[inline]
    pub fn nonzero_buckets(&self) -> u32 {
        u64::count_ones(self.mask)
    }

    /// Returns `true` if the bucket count corresponding to `value` is non-zero
    #[inline]
    pub fn has_counts(&self, value: u64) -> bool {
        let idx = self.index_of(value) as u64;
        self.mask & (1 << idx) != 0
    }

    /// Iterate through all 64 buckets of the histogram in order (0..63)
    pub fn iter(&self) -> impl Iterator<Item=Bucket> + '_ {
        let mut idx = 0;

        core::iter::from_fn(move || {
            if idx < 64 {
                let bucket = self.bucket_at(idx);
                idx += 1;
                Some(bucket)
            } else {
                None
            }
        })
    }

    /// Returns the bucket index into `self.counts` for the `value`
    #[inline]
    fn index_of(&self, value: u64) -> usize {
        match u64::leading_zeros(value) {
            0 => 63 as usize,
            clz => (64 - clz) as usize
        }
    }

    /// Return the `Bucket` at the provided index (index values 0..63)
    fn bucket_at(&self, idx: usize) -> Bucket {
        if idx == 0 {
            Bucket { start: 0, end: 0, count: self.counts[0] }
        } else {
            let shift = (idx - 1) as u32;
            let begin = u64::saturating_pow(2, shift);
            let end = u64::saturating_mul(begin, 2) - 1;
            let count = self.counts[idx];

            Bucket { start: begin, end, count }
        }
    }
}

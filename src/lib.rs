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

//! An efficient histogram with power-of-2 spaced buckets. Modest memory footprint with
//! no dynamic allocations, O(1) operation complexity, and a very compact binary serialization.
//!
//! ## Bucket Ranges
//!
//! Buckets cover the open-open range `[2^n, (2^(n+1))-1]` for all powers of 2 from 0 through 2^61
//! with the top-most bucket covering `[2^62, +infinity]`.
//!
//! ## Overflow Behavior
//!
//! Buckets counts saturate instead of overflowing.

///
/// A compact and efficient histogram with modest memory footprint,
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
    pub begin: u64,
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
    #[inline]
    pub fn observations(&self, value: u64) -> u64 {
        let idx = self.index_of(value);
        self.counts[idx]
    }

    /// Return the `Bucket` to which `value` belongs
    #[inline]
    pub fn bucket_for(&self, value: u64) -> Bucket {
        let idx = self.index_of(value);
        self.bucket_at(idx)
    }

    /// Returns the number of buckets with one or more observations
    #[inline]
    pub fn bucket_count(&self) -> u32 {
        u64::count_ones(self.mask)
    }

    #[inline]
    pub fn has_counts(&self, value: u64) -> bool {
        let idx = self.index_of(value) as u64;
        self.mask & (1 << idx) != 0
    }

    /// Iterate through all 64 buckets of the histogram in order (0..63)
    pub fn iter(&self) -> impl Iterator<Item=Bucket> + '_ {
        let mut idx = 0;

        std::iter::from_fn(move || {
            if idx < 64 {
                let bucket = self.bucket_at(idx);
                idx += 1;
                Some(bucket)
            } else {
                None
            }
        })
    }

    /// Returns the bucket index in `self.counts` for the `value`
    #[inline]
    fn index_of(&self, value: u64) -> usize {
        match u64::leading_zeros(value) {
            0 => 63 as usize,
            clz => (64 - clz) as usize
        }
    }

    /// Return the bucket for the provided index (index values 0..63)
    fn bucket_at(&self, idx: usize) -> Bucket {
        if idx == 0 {
            return Bucket { begin: 0, end: 0, count: self.counts[0] };
        }

        let shift = (idx - 1) as u32;
        let begin = u64::saturating_pow(2, shift);
        let end = u64::saturating_mul(begin, 2) - 1;
        let count = self.counts[idx];

        Bucket { begin, end, count }
    }
}

#[cfg(test)]
mod test {
    use crate::Base2Histogram;

    #[test]
    fn same_value_doesnt_change_bucket_count() {
        // Starts at zero
        let mut hist = Base2Histogram::new();
        assert_eq!(hist.bucket_count(), 0);

        // Zero has its own bucket
        hist.record(0);
        assert_eq!(hist.bucket_count(), 1);

        // Multiple observations of same value don't change bucket count
        for _ in 1..10 {
            hist.record(1);
            assert_eq!(hist.bucket_count(), 2);
        }
    }

    #[test]
    fn buckets_with_counts_are_identified() {
        let mut hist = Base2Histogram::new();
        let seq: Vec<u64> = [2, 8, 14, 20, 34, 50, 62, 1, 3, 12, 19, 31, 49, 61].iter()
            .map(|i: &u32| u64::pow(2, *i))
            .collect();

        for x in seq {
            assert_eq!(hist.has_counts(x), false, "false x={}, {:?}", x, hist.bucket_for(x));
            hist.record(x);
            assert_eq!(hist.has_counts(x), true, "true x={}, {:?}", x, hist.bucket_for(x));
        }
    }

    #[test]
    fn observation_counts_are_cumulative() {
        let mut hist = Base2Histogram::new();
        let value = u32::max_value() as u64;

        hist.record(value);
        assert_eq!(hist.observations(value), 1);

        hist.record_n(value, 9);
        assert_eq!(hist.observations(value), 10);

        hist.record_n(value, u16::max_value() as u64);
        assert_eq!(hist.observations(value), u16::max_value() as u64 + 10);
    }

    #[test]
    fn zero_has_its_own_bucket() {
        let mut hist = Base2Histogram::new();
        hist.record_n(0, 888);

        let bucket = hist.bucket_for(0);
        assert_eq!(bucket.begin, 0);
        assert_eq!(bucket.end, 0);
        assert_eq!(bucket.count, 888);
    }

    #[test]
    fn one_has_its_own_bucket() {
        let mut hist = Base2Histogram::new();
        hist.record_n(1, 123456789);

        let bucket = hist.bucket_for(1);
        assert_eq!(bucket.begin, 1);
        assert_eq!(bucket.end, 1);
        assert_eq!(bucket.count, 123456789);
    }

    #[test]
    fn values_equal_to_bucket_begin() {
        let hist = Base2Histogram::new();

        for i in 0..65 {
            let val = u64::saturating_pow(2, i);
            let b = hist.bucket_for(val);

            println!("i {}, val {}, {:?}", i, val, b);

            let n = if i < 63 { i } else { 62 };
            assert_eq!(b.begin, u64::saturating_pow(2, n));
        }
    }

    #[test]
    fn values_equal_to_bucket_begin_plus_one() {
        let hist = Base2Histogram::new();

        for i in 1..65 {
            let val = u64::saturating_pow(2, i).saturating_add(1);
            let b = hist.bucket_for(val);

            println!("i {}, val {}, {:?}", i, val, b);

            let n = if i < 63 { i } else { 62 };
            assert_eq!(b.begin, u64::saturating_pow(2, n));
        }
    }

    #[test]
    fn values_equal_to_bucket_end() {
        let mut hist = Base2Histogram::new();

        for i in 0..65 {
            let val = u64::saturating_pow(2, i).saturating_mul(2) - 1;
            hist.record(val);
            let b = hist.bucket_for(val);

            println!("i {}, val {}, {:?}", i, val, b);

            if i < 63 {
                assert_eq!(b.end, val);
                assert_eq!(b.count, 1);
            } else {
                // Values over 2^62 (4611686018427387904) accumulate into the same bucket
                assert_eq!(b.end, u64::saturating_pow(2, 63) - 1);
                assert_eq!(b.count, i as u64 - 61);
            }
        }
    }

    #[test]
    fn handle_u64_max_value() {
        let mut hist = Base2Histogram::new();

        hist.record(u64::max_value());
        let b = hist.bucket_for(u64::max_value());

        assert_eq!(b.begin, u64::pow(2, 62));
        assert_eq!(b.end, u64::pow(2, 63) - 1);
        assert_eq!(b.count, 1);
    }

    #[test]
    fn iterating_buckets_is_successful() {
        let mut hist = Base2Histogram::new();

        hist.record_n(0, 100);
        for i in 0..63 {
            hist.record_n(u64::pow(2, i), 100);
        };

        let mut n = 0;
        for b in hist.iter() {
            println!("n={} -> {:?}", n, b);
            if n == 0 {
                assert_eq!(b.begin, 0);
                assert_eq!(b.end, 0);
                assert_eq!(b.count, 100);
            } else if n == 1 {
                assert_eq!(b.begin, 1);
                assert_eq!(b.end, 1);
                assert_eq!(b.count, 100);
            } else {
                let begin = u64::pow(2, n - 1);
                let end = begin.saturating_mul(2) - 1;
                assert_eq!(b.begin, begin);
                assert_eq!(b.end, end);
                assert_eq!(b.count, 100);
            }
            n += 1;
        }
    }
}


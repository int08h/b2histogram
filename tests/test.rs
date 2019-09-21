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

/// Unit tests for Base2Histogram. Not in `lib.rs` because of `no-std` and these tests
/// use `println` for diagnostic output.

#[cfg(test)]
mod test {
    extern crate b2histogram;

    use b2histogram::Base2Histogram;

    #[test]
    fn same_value_doesnt_change_bucket_count() {
        // Starts at zero
        let mut hist = Base2Histogram::new();
        assert_eq!(hist.nonzero_buckets(), 0);

        // Zero has its own bucket
        hist.record(0);
        assert_eq!(hist.nonzero_buckets(), 1);

        // Multiple observations of same value don't change bucket count
        for _ in 1..10 {
            hist.record(1);
            assert_eq!(hist.nonzero_buckets(), 2);
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
        assert_eq!(bucket.start, 0);
        assert_eq!(bucket.end, 0);
        assert_eq!(bucket.count, 888);
    }

    #[test]
    fn one_has_its_own_bucket() {
        let mut hist = Base2Histogram::new();
        hist.record_n(1, 123456789);

        let bucket = hist.bucket_for(1);
        assert_eq!(bucket.start, 1);
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
            assert_eq!(b.start, u64::saturating_pow(2, n));
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
            assert_eq!(b.start, u64::saturating_pow(2, n));
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

        assert_eq!(b.start, u64::pow(2, 62));
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
            if n == 0 || n == 1 {
                assert_eq!(b.start, n);
                assert_eq!(b.end, n);
                assert_eq!(b.count, 100);
            } else {
                let begin = u64::pow(2, n as u32 - 1);
                let end = begin.saturating_mul(2) - 1;
                assert_eq!(b.start, begin);
                assert_eq!(b.end, end);
                assert_eq!(b.count, 100);
            }
            n += 1;
        }
    }
}

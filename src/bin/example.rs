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

use b2histogram::Base2Histogram;

fn main() {
    let mut hist = Base2Histogram::new();

    hist.record(0); // Record a single observation of '0'
    hist.record(11); // Two observations of '11'
    hist.record(11); //
    hist.record_n(300_000, 6); // Six observations of 300,000

    // Retrieve counts directly
    println!("Observations for 300,000: {}", hist.observations(300_000));

    // Retrieve the `Bucket` for a given value
    println!("Bucket corresponding to '11': {:?}", hist.bucket_for(11));

    // Iterate buckets that have observations
    println!(" start     end   count");
    for bucket in hist.iter().filter(|b| b.count > 0) {
        println!("{:6}, {:6}: {:6}", bucket.start, bucket.end, bucket.count);
    }
}

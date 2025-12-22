use std::ops::RangeInclusive;

use example_quests::{QuestContext, quest_context_generator};
use rand::Rng;

const MIN_VALUE: u32 = 0;
const MAX_VALUE: u32 = i32::MAX as u32;
const VALUE_RANGE: std::ops::RangeInclusive<u32> = MIN_VALUE..=MAX_VALUE;

const THRESHOLD_AVERAGE: u32 = 1_800_000_000;
const THRESHOLD_SPREAD: u32 = 100_000_000;
const THRESHOLD_RANGE: RangeInclusive<u32> =
    (THRESHOLD_AVERAGE - THRESHOLD_SPREAD)..=(THRESHOLD_AVERAGE + THRESHOLD_SPREAD);

const MIN_PACKET_LENGTH: usize = 10;
const MAX_PACKET_LENGTH: usize = 20;
const PACKET_LENGTH_RANGE: RangeInclusive<usize> = MIN_PACKET_LENGTH..=MAX_PACKET_LENGTH;

const MIN_PACKET_COUNT: usize = 500;
const MAX_PACKET_COUNT: usize = 600;
const PACKET_COUNT_RANGE: RangeInclusive<usize> = MIN_PACKET_COUNT..=MAX_PACKET_COUNT;

fn main() {
    quest_context_generator(|rng| {
        // Generate threshold T
        let t: u32 = rng.random_range(THRESHOLD_RANGE);

        // Number of packets
        let packet_count: usize = rng.random_range(PACKET_COUNT_RANGE);

        let mut input = format!("{}\n", t);

        let mut unstable_count = 0;
        for _ in 0..packet_count {
            // Length of this packet
            let len: usize = rng.random_range(PACKET_LENGTH_RANGE);

            let mut values = Vec::with_capacity(len as usize);
            for _ in 0..len {
                let value: u32 = rng.random_range(VALUE_RANGE);
                values.push(value);
            }

            // Check stability
            let mut unstable = false;
            for i in 1..values.len() {
                if values[i].abs_diff(values[i - 1]) > t {
                    unstable = true;
                    break;
                }
            }

            if unstable {
                unstable_count += 1;
            }

            let line = values
                .iter()
                .map(|value| value.to_string())
                .collect::<Vec<String>>()
                .join(" ");
            input.push_str(&line);
            input.push('\n');
        }

        let answer = unstable_count.to_string();

        QuestContext::new(input, answer)
    });
}

use crate::numeric::{ICP, WTN};
use crate::E8S;

pub const EXPECTED_INITIAL_BALANCE: u64 = 4_480_000 * E8S;

const TIERS: [(u64, u64); 7] = [
    (80_000, 8 * E8S),
    (160_000, 4 * E8S),
    (320_000, 2 * E8S),
    (640_000, E8S),
    (1_280_000, 50_000_000),
    (2_560_000, 25_000_000),
    (5_120_000, 12_500_000),
];

// Returns how many tokens should be distributed
pub fn compute_rewards(already_distributed: ICP, converting: ICP) -> WTN {
    let mut total_rewards = WTN::ZERO;
    let mut amount_to_distribute = converting.0 / E8S;
    let mut remaining_threshold = already_distributed.0 / E8S;

    for &(threshold, rate) in TIERS.iter() {
        if remaining_threshold >= threshold {
            remaining_threshold -= threshold;
            continue;
        }

        let tier_available = threshold - remaining_threshold;
        remaining_threshold = 0;
        let amount_in_this_tier = if amount_to_distribute > tier_available {
            tier_available
        } else {
            amount_to_distribute
        };

        total_rewards += WTN::from_e8s(amount_in_this_tier * rate);
        amount_to_distribute -= amount_in_this_tier;

        if amount_to_distribute == 0 {
            break;
        }
    }

    total_rewards
}
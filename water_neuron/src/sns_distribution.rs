use crate::E8S;
use crate::numeric::{ICP, WTN};

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

#[test]
fn should_return_expected_rewards() {
    let total_thresholds = TIERS.iter().map(|&(threshold, _)| threshold).sum();
    assert_eq!(
        compute_rewards(ICP::from_unscaled(20_000), ICP::from_unscaled(100)),
        WTN::from_unscaled(800)
    );
    assert_eq!(
        compute_rewards(ICP::ZERO, ICP::from_unscaled(10)),
        WTN::from_unscaled(80)
    );
    assert_eq!(
        compute_rewards(ICP::from_unscaled(total_thresholds), ICP::ONE),
        WTN::ZERO
    );
    assert_eq!(
        ICP::from_unscaled(total_thresholds),
        ICP::from_unscaled(10_160_000)
    );
}

#[test]
fn tiers_should_add_up_to_expected() {
    assert_eq!(
        TIERS
            .iter()
            .map(|(tranch, amount)| tranch * amount)
            .sum::<u64>(),
        EXPECTED_INITIAL_BALANCE
    );
}

#[test]
fn should_distribute_all_wtn() {
    assert_eq!(
        compute_rewards(ICP::ZERO, ICP::MAX),
        WTN::from_e8s(EXPECTED_INITIAL_BALANCE)
    );
    assert_eq!(
        compute_rewards(ICP::ZERO, ICP::from_unscaled(2 * TIERS.last().unwrap().0)),
        WTN::from_e8s(EXPECTED_INITIAL_BALANCE)
    );
}

#[test]
fn should_distribute_wtn() {
    assert_eq!(
        compute_rewards(ICP::from_unscaled(110_000), ICP::from_unscaled(180_000)),
        WTN::from_unscaled(620 * 1_000)
    );
}

#[test]
fn should_distribute_wtn_with_decimal() {
    assert_eq!(
        compute_rewards(ICP::ZERO, ICP::from_e8s(123_456_789)),
        WTN::from_unscaled(8)
    );
}

#[cfg(test)]
pub mod test {
    use crate::E8S;
    use crate::numeric::{ICP, WTN};
    use crate::sns_distribution::compute_rewards;
    use proptest::collection::vec;
    use proptest::prelude::*;

    fn generate_vec_with_sum(
        sum: u64,
        min_elem: u64,
        max_elem: u64,
    ) -> impl Strategy<Value = Vec<u64>> {
        // Ensure the range is valid
        assert!(min_elem <= max_elem);

        let vec_length_strategy = 1..=1000;

        vec(min_elem..=max_elem, vec_length_strategy).prop_map(move |mut v| {
            let current_sum: u64 = v.iter().sum();
            if current_sum == 0 {
                return vec![sum];
            }
            let scale = sum as f64 / current_sum as f64;
            v.iter_mut()
                .for_each(|x| *x = (*x as f64 * scale).round() as u64);

            let mut actual_sum: u64 = v.iter().sum();
            while actual_sum != sum {
                for x in &mut v {
                    if actual_sum == sum {
                        break;
                    } else if actual_sum < sum {
                        *x += 1;
                        actual_sum += 1;
                    } else {
                        if *x > 0 {
                            *x -= 1;
                            actual_sum -= 1;
                        }
                    }
                }
            }

            v
        })
    }

    proptest! {
        #[test]
        fn should_distribute_all_rewards(vec in generate_vec_with_sum(10_161_100 * E8S, E8S, 1_000_000 * E8S)) {
            let sum: u64 = vec.iter().sum();
            prop_assert_eq!(sum, 10_161_100 * E8S);

            let mut total_rewards = WTN::ZERO;
            let mut already_distributed = ICP::ZERO;

            for e in vec {
                let deposited = ICP::from_e8s(e);
                let additional_rewards = compute_rewards(already_distributed, deposited);
                already_distributed += deposited;
                total_rewards += additional_rewards;
            }

            prop_assert_eq!(already_distributed, ICP::from_unscaled(10_161_100));
            prop_assert_eq!(compute_rewards(already_distributed, ICP::ONE), WTN::ZERO);
            prop_assert!(total_rewards <= WTN::from_unscaled(4_480_000));
        }
    }
}

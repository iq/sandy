pub struct AmountOut {
    pub amount_out: u64,
    pub new_reserve_a: u64,
    pub new_reserve_b: u64,
}

// k = y * x
fn get_amount_out(amount_in: u64, reserve_a: u64, reserve_b: u64) -> AmountOut {
    let amount_in_128 = u128::from(amount_in);
    let reserve_a_128 = u128::from(reserve_a);
    let reserve_b_128 = u128::from(reserve_b);

    let amount_in_after_fee = amount_in_128 * (10000 - 25) / 10000;

    let numerator = amount_in_after_fee * reserve_b_128;
    let denominator = reserve_a_128 + amount_in_after_fee;

    let amount_out = (numerator / denominator) as u64;

    AmountOut {
        amount_out,
        new_reserve_a: reserve_a + amount_in,
        new_reserve_b: reserve_b - amount_out,
    }
}

pub fn calculate_swap_amount_in(
    lb: u64,
    ub: u64,
    user_amount_in: u64,
    user_minimum_amount_out: u64,
    reserve_a: u64,
    reserve_b: u64,
) -> u64 {
    let base = 10_000u64;
    let tolerance = 1u64;

    if ub - lb > (tolerance * ((ub + lb) / 2)) / base {
        let mid = (lb + ub) / 2;

        let frontrun_state = get_amount_out(mid, reserve_a, reserve_b);
        let victim_state = get_amount_out(
            user_amount_in,
            frontrun_state.new_reserve_a,
            frontrun_state.new_reserve_b,
        );

        let out = victim_state.amount_out;

        if out >= user_minimum_amount_out {
            return calculate_swap_amount_in(
                mid,
                ub,
                user_amount_in,
                user_minimum_amount_out,
                reserve_a,
                reserve_b,
            );
        }
        return calculate_swap_amount_in(
            lb,
            mid,
            user_amount_in,
            user_minimum_amount_out,
            reserve_a,
            reserve_b,
        );
    }

    (ub + lb) / 2
}

pub fn from_bps(input: u32) -> f64 {
    input as f64 / 10000.0
}

pub fn to_bps(input: f64) -> u32 {
    (input * 10000.0) as u32
}

pub fn panic_with_wrong_capital_split(capital_split_f: [f64; 2]) {
    let mut sum: f64 = 0.0;
    for x in capital_split_f {
        sum += x;
    }

    if sum != 1.0 {
        panic!("wrong capital split, sum must be 1")
    }
}

pub fn get_quantites_with_capital_split(quantity: u64, capital_split_f: [f64; 2]) -> [u64; 2] {
    panic_with_wrong_capital_split(capital_split_f);
    capital_split_f.map(|x| (quantity as f64 * x) as u64)
}
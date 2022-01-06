// pub fn from_bps(input: u64) -> f64 {
//     return input as f64 / 10000.0;
// }

pub fn from_bps(input: u32) -> f64 {
    input as f64 / 10000.0
}

pub fn to_bps(input: f64) -> u32 {
    (input * 10000.0) as u32
}

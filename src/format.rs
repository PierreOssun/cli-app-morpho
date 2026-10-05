use alloy::primitives::U256;

/// A token amount as "1,234,567.89 SYMBOL".
pub fn amount(raw: U256, decimals: u8, unit: &str) -> String {
    let scale = U256::from(10).pow(U256::from(decimals));
    let (whole, cents) = (raw / scale, raw % scale * U256::from(100) / scale);
    let digits = whole.to_string();
    let mut grouped = String::new();
    for (i, digit) in digits.chars().enumerate() {
        if i > 0 && (digits.len() - i) % 3 == 0 {
            grouped.push(',');
        }
        grouped.push(digit);
    }
    format!("{grouped}.{cents:02} {unit}")
}

/// A ratio as "12.34%".
pub fn pct(ratio: f64) -> String {
    format!("{:.2}%", ratio * 100.0)
}

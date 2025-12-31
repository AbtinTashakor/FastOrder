pub fn normalize_phone(raw: &str) -> Option<String> {
    // 1) remove everything except digits
    let mut digits: String = raw.chars().filter(|c| c.is_ascii_digit()).collect();

    // 2) normalize prefixes
    if digits.starts_with("0098") {
        digits = digits.trim_start_matches("0098").to_string();
    } else if digits.starts_with("98") {
        digits = digits.trim_start_matches("98").to_string();
    } else if digits.starts_with("0") {
        digits = digits.trim_start_matches('0').to_string();
    }

    // 3) after normalization we expect exactly 10 digits (9XXXXXXXXX)
    if digits.len() != 10 {
        return None;
    }

    // 4) final canonical form
    Some(format!("+98{}", digits))
}
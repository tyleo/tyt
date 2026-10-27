/// `value` rounded to six decimals with trailing zeros dropped and `-0` read as
/// `0`.
pub fn meters(value: f64) -> String {
    let text = format!("{value:.6}");
    let text = text.trim_end_matches('0').trim_end_matches('.');

    match text {
        "-0" => "0".to_owned(),
        _ => text.to_owned(),
    }
}

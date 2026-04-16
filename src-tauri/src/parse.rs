pub fn parse_progress_line(line: &str) -> Option<f64> {
    let rest = line.strip_prefix("download:")?;
    let pct: f64 = rest.trim().parse().ok()?;
    if !(0.0..=100.0).contains(&pct) {
        return None;
    }
    Some(pct / 100.0)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn mid() {
        assert_eq!(parse_progress_line("download: 63.2"), Some(0.632));
    }
    #[test]
    fn full() {
        assert_eq!(parse_progress_line("download:100.0"), Some(1.0));
    }
    #[test]
    fn spaces() {
        assert_eq!(parse_progress_line("download:   7.5 "), Some(0.075));
    }
    #[test]
    fn non_progress() {
        assert_eq!(parse_progress_line("[youtube] extracting..."), None);
    }
    #[test]
    fn non_numeric() {
        assert_eq!(parse_progress_line("download: NA"), None);
    }
}

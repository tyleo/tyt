use std::str::FromStr;

/// One `--select-index` value: a single object index such as `0`, or an
/// inclusive range `a-b` such as `2-5`. Indices are positions into the
/// document's `objects`. The flag repeats, and an object is selected when any
/// value covers its index.
#[derive(Clone, Copy, Debug)]
pub struct SelectIndex {
    start: usize,
    end: usize,
}

impl SelectIndex {
    /// Whether `index` falls in this selector, inclusive of both ends.
    pub fn contains(self, index: usize) -> bool {
        self.start <= index && index <= self.end
    }
}

impl FromStr for SelectIndex {
    type Err = String;

    /// Parses a single index `5` or an inclusive range `2-5`. A range's start
    /// must not exceed its end, and both ends are non-negative integers.
    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value.split_once('-') {
            Some((start, end)) => {
                let start = parse_index(start)?;
                let end = parse_index(end)?;
                if start > end {
                    return Err(format!("range start {start} is greater than its end {end}"));
                }
                Ok(SelectIndex { start, end })
            }
            None => {
                let index = parse_index(value)?;
                Ok(SelectIndex {
                    start: index,
                    end: index,
                })
            }
        }
    }
}

/// Parses one non-negative object index, naming the bad value on failure.
fn parse_index(value: &str) -> Result<usize, String> {
    value
        .parse::<usize>()
        .map_err(|_| format!("`{value}` is not an object index"))
}

#[cfg(test)]
mod tests {
    use crate::SelectIndex;

    #[test]
    fn parses_single_index() {
        let select: SelectIndex = "5".parse().unwrap();
        assert!(select.contains(5));
        assert!(!select.contains(4));
        assert!(!select.contains(6));
    }

    #[test]
    fn parses_inclusive_range() {
        let select: SelectIndex = "2-5".parse().unwrap();
        assert!(!select.contains(1));
        assert!(select.contains(2));
        assert!(select.contains(5));
        assert!(!select.contains(6));
    }

    #[test]
    fn rejects_reversed_range() {
        assert!("5-2".parse::<SelectIndex>().is_err());
    }

    #[test]
    fn rejects_non_integer_or_negative() {
        assert!("x".parse::<SelectIndex>().is_err());
        assert!("-3".parse::<SelectIndex>().is_err());
        assert!("2-".parse::<SelectIndex>().is_err());
        assert!("".parse::<SelectIndex>().is_err());
    }
}

use std::fmt::{Debug, Display};

use crate::{DurationHuman, DurationHumanValidator};

impl Display for DurationHumanValidator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!(
            "must be between {min} and {max}",
            min = self.min,
            max = self.max
        ))
    }
}

impl Debug for DurationHumanValidator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DurationHumanValidator")
            .field("min", &self.min.to_string())
            .field("default", &self.default.to_string())
            .field("max", &self.max.to_string())
            .finish()
    }
}

impl Display for DurationHuman {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut nanos: u64 = self.into();
        if f.alternate() {
            let durations: Vec<String> = [
                (Self::CENTURY, " century", " centuries"),
                (Self::YEAR, " year", " years"),
                (Self::MONTH, " month", " months"),
                (Self::WEEK, " week", " weeks"),
                (Self::DAY, " day", " days"),
                (Self::HOUR, "h", "h"),
                (Self::MINUTE, "min", "min"),
                (Self::SEC, "s", "s"),
                (Self::MILLI_SEC, "ms", "ms"),
                (Self::MICRO_SEC, "μs", "μs"),
                (1, "ns", "ns"),
            ]
            .iter()
            .filter_map(|(part_ms, unit_singular, unit_plural)| {
                let part = nanos / part_ms;
                nanos %= part_ms;
                if part > 0 {
                    Some(format!(
                        "{}{}",
                        part,
                        if part > 1 { unit_plural } else { unit_singular }
                    ))
                } else {
                    None
                }
            })
            .collect();
            f.write_str(durations.join(" ").as_str())
        } else {
            f.write_str(
                match nanos {
                    _ if nanos < Self::MICRO_SEC || nanos % Self::MICRO_SEC != 0 => {
                        format!("{nanos}ns")
                    }
                    _ if nanos < Self::MILLI_SEC || nanos % Self::MILLI_SEC != 0 => {
                        format!("{}μs", nanos / Self::MICRO_SEC)
                    }
                    _ if nanos < Self::SEC || nanos % Self::SEC != 0 => {
                        format!("{}ms", nanos / Self::MILLI_SEC)
                    }
                    _ if nanos < Self::MINUTE || nanos % Self::MINUTE != 0 => {
                        format!("{}s", nanos / Self::SEC)
                    }
                    _ if nanos < Self::HOUR || nanos % Self::HOUR != 0 => {
                        format!("{}min", nanos / Self::MINUTE)
                    }
                    _ if nanos < Self::DAY || nanos % Self::DAY != 0 => {
                        format!("{}h", nanos / Self::HOUR)
                    }
                    _ if nanos < Self::WEEK || nanos % Self::WEEK != 0 => {
                        format!(
                            "{} day{}",
                            nanos / Self::DAY,
                            if nanos / Self::DAY > 1 { "s" } else { "" }
                        )
                    }
                    _ if nanos < Self::MONTH || nanos % Self::MONTH != 0 => {
                        format!(
                            "{} week{}",
                            nanos / Self::WEEK,
                            if nanos / Self::WEEK > 1 { "s" } else { "" }
                        )
                    }
                    _ if nanos < Self::YEAR || nanos % Self::YEAR != 0 => format!(
                        "{} month{}",
                        nanos / Self::MONTH,
                        if nanos / Self::YEAR > 1 { "s" } else { "" }
                    ),
                    _ if nanos < Self::CENTURY || nanos % Self::CENTURY != 0 => {
                        format!(
                            "{} year{}",
                            nanos / Self::YEAR,
                            if nanos / Self::YEAR > 1 { "s" } else { "" }
                        )
                    }
                    _ => format!(
                        "{} centur{}",
                        nanos / Self::CENTURY,
                        if nanos / Self::CENTURY > 1 {
                            "ies"
                        } else {
                            "y"
                        }
                    ),
                }
                .as_str(),
            )
        }
    }
}

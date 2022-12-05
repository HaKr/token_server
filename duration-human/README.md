# Duration wrapper for humans
Wrapper for std::time::Duration to deal with durations from human readable form

## DurationHuman

### Parse and format for human interaction
Main goal is to declare a Duration from, as well as formatting into, a human readable string.

Parsing a string, adds all values with a time unit to the total duration, so parse("1min 2s 1min")
results in a 122s duration.

Formatting as a string uses the unit for which an integral value can be represented, so
a 122s duration will format as 122s, but a 86400s duration will format as 1day.

Formatting as pretty print includes all units that have a non-zero value,
so 122s will pretty print as "2min 2s" and 90060 as "1 day 1h 1m"

```rust
# use duration_human::DurationHuman;
let duration = DurationHuman::try_from("80h").unwrap();
assert_eq!(format!("{:#}", duration), "3 days 8h".to_string());
assert_eq!(format!("{}", duration), "80h".to_string());
let duration = DurationHuman::try_from("72h").unwrap();
assert_eq!(format!("{:#}", duration), "3 days".to_string());
assert_eq!(format!("{}", duration), "3 days".to_string());
let duration = DurationHuman::try_from("18446744073709551615ns").unwrap();
assert_eq!(format!("{:#}", duration), "5 centuries 84 years 11 months 1 week 6 days 23h 34min 33s 709ms 551μs 615ns".to_string());
// precision is nano second
let duration = DurationHuman::try_from("604800μs").unwrap();
assert_eq!(format!("{:#}", duration), "604ms 800μs".to_string());
assert_eq!(duration.to_string(), "604800μs".to_string());
let duration = DurationHuman::try_from("604800ms").unwrap();
assert_eq!(format!("{:#}", duration), "10min 4s 800ms".to_string());
assert_eq!(duration.to_string(), "604800ms".to_string());
let duration = DurationHuman::try_from("604800s").unwrap();
assert_eq!(format!("{:#}", duration), "1 week".to_string());
let duration = DurationHuman::try_from("604800s").unwrap();
assert_eq!(format!("{:#}", duration), "1 week".to_string());
assert_eq!(format!("{}", duration), "1 week".to_string());
let duration = DurationHuman::try_from("608430s").unwrap();
assert_eq!(format!("{:#}", duration), "1 week 1h 30s".to_string());
assert_eq!(format!("{}", duration), "608430s".to_string());
```

### Adding to Instant
std::time::Instant supports Add<Duration>, but we can't touch that here,
so we turn it around:
```rust
use std::time::Instant;
use duration_human::{DurationHuman, DurationError};

let instant = Instant::now();
let duration = DurationHuman::try_from("420s")?;
let after = duration + instant;
let diff = DurationHuman::from(after - instant);
assert_eq!(format!("{}", diff), format!("7min"));
```
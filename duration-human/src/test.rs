use crate::{DurationError, DurationHuman};

#[test]
fn roundtrip() {
    let duration = format!(
        "{:#}",
        DurationHuman::try_from("It will take 2years 1 week 3days 5h 6min and 10s.").unwrap()
    );
    assert_eq!(duration, format!("2 years 1 week 3 days 5h 6min 10s"));

    let duration = format!("{:#}", DurationHuman::try_from(duration.as_str()).unwrap());
    assert_eq!(duration, format!("2 years 1 week 3 days 5h 6min 10s"));
}

#[test]
fn beware_of_unrecognised() -> Result<(), DurationError> {
    let duration = DurationHuman::try_from("It will take 2years 1 week 3 dagen 5h 6min and 10s.")?;
    assert_eq!(
        format!("{:#}", duration),
        format!("2 years 1 week 5h 6min 10s")
    );
    Ok(())
}

#[test]
fn max() -> Result<(), DurationError> {
    let duration = DurationHuman::try_from(
        "5 centuries 84 years 11 months 1 week 6 days 23h 34min 33s 709ms 551μs 615ns",
    )?;
    let pretty = format!("{:#}", duration);
    let duration_from_pretty = DurationHuman::try_from(pretty.as_str())?;
    assert_eq!(duration, duration_from_pretty);
    Ok(())
}

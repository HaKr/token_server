use crate::{DurationError, DurationHuman};

#[test]
fn roundtrip() {
    let duration = format!(
        "{:#}",
        DurationHuman::try_from("2years 1 week 3days 5h 6min 10s").unwrap()
    );
    assert_eq!(duration, format!("2 years 1 week 3 days 5h 6min 10s"));

    let duration = format!("{:#}", DurationHuman::try_from(duration.as_str()).unwrap());
    assert_eq!(duration, format!("2 years 1 week 3 days 5h 6min 10s"));
}

#[test]
fn ignore_blanks_in_input() -> Result<(), DurationError> {
    let duration = DurationHuman::try_from("2year 1 week 3day 5h 6min 10s")?;
    assert_eq!(
        format!("{:#}", duration),
        format!("2 years 1 week 3 days 5h 6min 10s")
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

mod errors {
    use crate::{DurationError, DurationHuman};

    #[test]
    fn overflow() {
        let duration = DurationHuman::try_from("584 year 10 months 5 weeks 7 days 49h");
        let is_err = duration.is_err();
        assert!(is_err);
        if let Err(err) = duration {
            match err {
                DurationError::IntegerOverflowAt { duration } => {
                    assert_eq!(duration, "49h".to_string());
                }
                err => assert!(!is_err, "Did not expect the error: '{}'", err),
            }
        }
    }

    #[test]
    fn overflow_ms() {
        let duration = DurationHuman::try_from("18446744073709551615ms");
        let is_err = duration.is_err();
        assert!(is_err);
        if let Err(err) = duration {
            match err {
                DurationError::IntegerOverflowAt { duration } => assert!(!duration.is_empty()),
                err => assert!(!is_err, "Did not expect the error: '{}'", err),
            }
        }
    }

    #[test]
    fn syntax_error() {
        let result = DurationHuman::try_from("2year 1 week 3dya 5h 6min 10s");
        let contrived = result.iter().count();
        match result {
            Err(DurationError::InvalidSyntax) => (),
            Err(err) => assert_eq!(contrived, 3, "Did not expect this error {}", err),
            Ok(duration) => assert_eq!(
                contrived, 3,
                "Did not expect a valid duration {:#}",
                duration
            ),
        }
    }
}

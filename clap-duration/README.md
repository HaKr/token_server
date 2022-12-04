# Macros for clap argument value_parse

## Example: value_parse
 ```rust
 use clap::Parser;
 use clap_duration::duration_range_value_parse;
 use duration_human::{DurationHuman, DurationHumanValidator};

 #[derive(Parser)]
 struct SampleOptions {

     #[arg(
         long, default_value="666000ms",
         value_parser = duration_range_value_parse!(min: 10min, max: 1h)
     )]
     interval: DurationHuman,
 }

 let opts = SampleOptions::parse();
 assert_eq!(format!("{:#}",opts.interval), format!("11min 6s"));
 assert_eq!(opts.interval.to_string(), "666s".to_string());

```

## Example: Range and default from a constant
```rust
use clap::Parser;
use clap_duration::duration_range_value_parse;
use duration_human::{DurationHuman, DurationHumanValidator};

assign_duration_range_validator!( LIFETIME_RANGE = {default: 2h, min: 333s, max: 60day});

#[derive(Parser)]
struct ServerOptions {
    
    #[arg(
        long,
        help = format!("What lifetime will it have, between {}", LIFETIME_RANGE),
        default_value = LIFETIME_RANGE.default,
        value_parser = {|lifetime: &str|LIFETIME_RANGE.parse_and_validate(lifetime)}
    )]
    lifetime: DurationHuman,
}

 let opts = ServerOptions::parse();
 assert_eq!(format!("{:#}",opts.lifetime), format!("11min 6s"));
 assert_eq!(opts.lifetime.to_string(), "666s".to_string());

```

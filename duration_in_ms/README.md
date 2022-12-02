# Duration in millisecond precision
Wrapper for std::time::Duration to dela with durations from human readable form


## DurationInms
Can be created by specifying a duration in milliseconds.

Can also be created from a human readable string, which is made up from
an integer, immediately followed by day, h, min, s, ms, μs or ns.

It also implements std::fmt:Display, which shows the duration in the closest
integral form e.g., 3600s becomse 1h, 3660s becomes 61min and 3659s remains 3659s


## DurationInmsRangeAndDefault



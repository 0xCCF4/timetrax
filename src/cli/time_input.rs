use std::str::FromStr;
use time::{Duration, Month, Weekday};

/// A signed duration offset parsed from strings like "-5m", "+1h30m", "30s", "-1h15m30s"
#[derive(Debug, Clone, Copy)]
pub struct TimeOffset(pub Duration);

impl FromStr for TimeOffset {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (sign, rest) = if let Some(r) = s.strip_prefix('-') {
            (-1i64, r)
        } else if let Some(r) = s.strip_prefix('+') {
            (1i64, r)
        } else {
            (1i64, s)
        };

        if rest.is_empty() {
            return Err(format!("Empty offset in '{s}'"));
        }

        let mut total_secs = 0i64;
        let mut remaining = rest;

        while !remaining.is_empty() {
            let num_end = remaining
                .find(|c: char| !c.is_ascii_digit())
                .ok_or_else(|| format!("Expected unit (h/m/s) after number in '{s}'"))?;
            if num_end == 0 {
                return Err(format!("Expected number before unit in '{s}'"));
            }
            let num: i64 = remaining[..num_end]
                .parse()
                .map_err(|_| format!("Invalid number in '{s}'"))?;
            let (unit, after) = remaining[num_end..].split_at(1);
            total_secs += match unit {
                "h" => num * 3600,
                "m" => num * 60,
                "s" => num,
                u => return Err(format!("Unknown unit '{u}' in '{s}' - use h, m, or s")),
            };
            remaining = after;
        }

        Ok(TimeOffset(Duration::seconds(sign * total_secs)))
    }
}

/// An explicit wall-clock time parsed from "HH:MM" or "HH:MM:SS"
#[derive(Debug, Clone, Copy)]
pub struct TimeAt(pub time::Time);

impl FromStr for TimeAt {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts: Vec<&str> = s.split(':').collect();
        let (h, m, sec): (&str, &str, &str) = match parts.as_slice() {
            [h, m] => (h, m, "0"),
            [h, m, sec] => (h, m, sec),
            _ => return Err(format!("Invalid time '{s}' - expected HH:MM or HH:MM:SS")),
        };
        let hour: u8 = h
            .parse()
            .map_err(|_| format!("Invalid hour in '{s}'"))?;
        let minute: u8 = m
            .parse()
            .map_err(|_| format!("Invalid minute in '{s}'"))?;
        let second: u8 = sec
            .parse()
            .map_err(|_| format!("Invalid second in '{s}'"))?;
        time::Time::from_hms(hour, minute, second)
            .map(TimeAt)
            .map_err(|e| format!("Invalid time '{s}': {e}"))
    }
}

/// Resolve the effective wall-clock time from `--at` / `--offset` flags.
/// `--at` takes precedence; `--offset` adjusts now; neither defaults to now.
#[must_use]
pub fn resolve_time(at: Option<TimeAt>, offset: Option<TimeOffset>) -> time::Time {
    use log::error;
    let now = time::OffsetDateTime::now_local()
        .unwrap_or_else(|e| {
            error!("Failed to get local time: {e}. Falling back to UTC.");
            time::OffsetDateTime::now_utc()
        })
        .time();

    if let Some(TimeAt(t)) = at {
        t
    } else if let Some(TimeOffset(d)) = offset {
        now + d
    } else {
        now
    }
}

/// A non-negative duration for quota values, parsed like `TimeOffset` but without a sign.
/// Accepts "8h", "8h30m", "45m", "30s", "1h30m30s".
#[derive(Debug, Clone, Copy)]
pub struct CliDuration(pub Duration);

impl FromStr for CliDuration {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let offset = TimeOffset::from_str(s)?;
        if offset.0 < Duration::ZERO {
            return Err(format!("Duration must be non-negative, got '{s}'"));
        }
        Ok(CliDuration(offset.0))
    }
}

/// A weekday parsed from strings like "mon", "monday", "tue", etc. (case-insensitive).
#[derive(Debug, Clone, Copy)]
pub struct WeekdayInput(pub Weekday);

impl FromStr for WeekdayInput {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "mon" | "mo" | "monday" => Ok(WeekdayInput(Weekday::Monday)),
            "tue" | "tu" | "tuesday" => Ok(WeekdayInput(Weekday::Tuesday)),
            "wed" | "we" | "wednesday" => Ok(WeekdayInput(Weekday::Wednesday)),
            "thu" | "th" | "thursday" => Ok(WeekdayInput(Weekday::Thursday)),
            "fri" | "fr" | "friday" => Ok(WeekdayInput(Weekday::Friday)),
            "sat" | "sa" | "saturday" => Ok(WeekdayInput(Weekday::Saturday)),
            "sun" | "su" | "sunday" => Ok(WeekdayInput(Weekday::Sunday)),
            _ => Err(format!("Unknown weekday '{s}' - use mon/tue/wed/thu/fri/sat/sun")),
        }
    }
}

/// A calendar date parsed from "YYYY-MM-DD".
#[derive(Debug, Clone, Copy)]
pub struct DateInput(pub time::Date);

impl FromStr for DateInput {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts: Vec<&str> = s.splitn(3, '-').collect();
        let [y, m, d] = parts.as_slice() else {
            return Err(format!("Invalid date '{s}' - expected YYYY-MM-DD"));
        };
        let year: i32 = y.parse().map_err(|_| format!("Invalid year in '{s}'"))?;
        let month: u8 = m.parse().map_err(|_| format!("Invalid month in '{s}'"))?;
        let day: u8 = d.parse().map_err(|_| format!("Invalid day in '{s}'"))?;
        let month = Month::try_from(month).map_err(|e| format!("Invalid month in '{s}': {e}"))?;
        time::Date::from_calendar_date(year, month, day)
            .map(DateInput)
            .map_err(|e| format!("Invalid date '{s}': {e}"))
    }
}

/// Format a non-negative duration as "Xh Ym" or "Ym".
#[must_use]
pub fn fmt_duration(d: Duration) -> String {
    let secs = d.whole_seconds().unsigned_abs();
    let h = secs / 3600;
    let m = (secs % 3600) / 60;
    if h > 0 {
        format!("{h}h {m:02}m")
    } else {
        format!("{m}m")
    }
}

/// Format a signed duration as "+Xh Ym" or "-Xh Ym".
#[must_use]
pub fn fmt_signed_duration(d: Duration) -> String {
    let sign = if d < Duration::ZERO { "-" } else { "+" };
    format!("{sign}{}", fmt_duration(d))
}

/// Human-readable weekday name.
#[must_use]
pub fn weekday_name(wd: Weekday) -> &'static str {
    match wd {
        Weekday::Monday => "Monday",
        Weekday::Tuesday => "Tuesday",
        Weekday::Wednesday => "Wednesday",
        Weekday::Thursday => "Thursday",
        Weekday::Friday => "Friday",
        Weekday::Saturday => "Saturday",
        Weekday::Sunday => "Sunday",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use time::Time;

    #[test]
    fn offset_plain_minutes() {
        let off: TimeOffset = "5m".parse().unwrap();
        assert_eq!(off.0, Duration::minutes(5));
    }

    #[test]
    fn offset_plus_sign_minutes() {
        let off: TimeOffset = "+30m".parse().unwrap();
        assert_eq!(off.0, Duration::minutes(30));
    }

    #[test]
    fn offset_negative_minutes() {
        let off: TimeOffset = "-15m".parse().unwrap();
        assert_eq!(off.0, Duration::minutes(-15));
    }

    #[test]
    fn offset_hours_only() {
        let off: TimeOffset = "2h".parse().unwrap();
        assert_eq!(off.0, Duration::hours(2));
    }

    #[test]
    fn offset_negative_hours() {
        let off: TimeOffset = "-1h".parse().unwrap();
        assert_eq!(off.0, Duration::hours(-1));
    }

    #[test]
    fn offset_hours_and_minutes() {
        let off: TimeOffset = "1h30m".parse().unwrap();
        assert_eq!(off.0, Duration::hours(1) + Duration::minutes(30));
    }

    #[test]
    fn offset_negative_hours_and_minutes() {
        let off: TimeOffset = "-1h30m".parse().unwrap();
        assert_eq!(off.0, Duration::hours(-1) + Duration::minutes(-30));
    }

    #[test]
    fn offset_seconds_only() {
        let off: TimeOffset = "90s".parse().unwrap();
        assert_eq!(off.0, Duration::seconds(90));
    }

    #[test]
    fn offset_hours_minutes_seconds() {
        let off: TimeOffset = "1h30m45s".parse().unwrap();
        assert_eq!(
            off.0,
            Duration::hours(1) + Duration::minutes(30) + Duration::seconds(45)
        );
    }

    #[test]
    fn offset_empty_is_error() {
        assert!("".parse::<TimeOffset>().is_err());
    }

    #[test]
    fn offset_missing_unit_is_error() {
        assert!("5".parse::<TimeOffset>().is_err());
    }

    #[test]
    fn offset_unknown_unit_is_error() {
        assert!("5x".parse::<TimeOffset>().is_err());
    }

    #[test]
    fn offset_sign_only_is_error() {
        assert!("-".parse::<TimeOffset>().is_err());
    }

    #[test]
    fn time_at_hhmm() {
        let ta: TimeAt = "14:30".parse().unwrap();
        assert_eq!(ta.0, Time::from_hms(14, 30, 0).unwrap());
    }

    #[test]
    fn time_at_hhmmss() {
        let ta: TimeAt = "09:05:30".parse().unwrap();
        assert_eq!(ta.0, Time::from_hms(9, 5, 30).unwrap());
    }

    #[test]
    fn time_at_midnight() {
        let ta: TimeAt = "00:00:00".parse().unwrap();
        assert_eq!(ta.0, Time::MIDNIGHT);
    }

    #[test]
    fn time_at_invalid_hour_is_error() {
        assert!("25:00".parse::<TimeAt>().is_err());
    }

    #[test]
    fn time_at_invalid_minute_is_error() {
        assert!("12:60".parse::<TimeAt>().is_err());
    }

    #[test]
    fn time_at_garbage_is_error() {
        assert!("not-a-time".parse::<TimeAt>().is_err());
    }

    #[test]
    fn time_at_too_many_parts_is_error() {
        assert!("12:00:00:00".parse::<TimeAt>().is_err());
    }

    #[test]
    fn cli_duration_positive() {
        let d: CliDuration = "8h".parse().unwrap();
        assert_eq!(d.0, Duration::hours(8));
    }

    #[test]
    fn cli_duration_compound() {
        let d: CliDuration = "8h30m".parse().unwrap();
        assert_eq!(d.0, Duration::hours(8) + Duration::minutes(30));
    }

    #[test]
    fn cli_duration_negative_is_error() {
        assert!("-5m".parse::<CliDuration>().is_err());
    }

    #[test]
    fn cli_duration_zero_ok() {
        let d: CliDuration = "0m".parse().unwrap();
        assert_eq!(d.0, Duration::ZERO);
    }

    #[test]
    fn weekday_short_lowercase() {
        assert_eq!("mon".parse::<WeekdayInput>().unwrap().0, Weekday::Monday);
        assert_eq!("tue".parse::<WeekdayInput>().unwrap().0, Weekday::Tuesday);
        assert_eq!("wed".parse::<WeekdayInput>().unwrap().0, Weekday::Wednesday);
        assert_eq!("thu".parse::<WeekdayInput>().unwrap().0, Weekday::Thursday);
        assert_eq!("fri".parse::<WeekdayInput>().unwrap().0, Weekday::Friday);
        assert_eq!("sat".parse::<WeekdayInput>().unwrap().0, Weekday::Saturday);
        assert_eq!("sun".parse::<WeekdayInput>().unwrap().0, Weekday::Sunday);
    }

    #[test]
    fn weekday_two_letter() {
        assert_eq!("mo".parse::<WeekdayInput>().unwrap().0, Weekday::Monday);
        assert_eq!("tu".parse::<WeekdayInput>().unwrap().0, Weekday::Tuesday);
        assert_eq!("we".parse::<WeekdayInput>().unwrap().0, Weekday::Wednesday);
        assert_eq!("th".parse::<WeekdayInput>().unwrap().0, Weekday::Thursday);
        assert_eq!("fr".parse::<WeekdayInput>().unwrap().0, Weekday::Friday);
        assert_eq!("sa".parse::<WeekdayInput>().unwrap().0, Weekday::Saturday);
        assert_eq!("su".parse::<WeekdayInput>().unwrap().0, Weekday::Sunday);
    }

    #[test]
    fn weekday_full_name_case_insensitive() {
        assert_eq!("Monday".parse::<WeekdayInput>().unwrap().0, Weekday::Monday);
        assert_eq!("FRIDAY".parse::<WeekdayInput>().unwrap().0, Weekday::Friday);
        assert_eq!("sunday".parse::<WeekdayInput>().unwrap().0, Weekday::Sunday);
    }

    #[test]
    fn weekday_invalid_is_error() {
        assert!("xyz".parse::<WeekdayInput>().is_err());
        assert!("".parse::<WeekdayInput>().is_err());
    }

    #[test]
    fn date_input_valid() {
        let d: DateInput = "2024-03-15".parse().unwrap();
        assert_eq!(d.0, time::Date::from_calendar_date(2024, time::Month::March, 15).unwrap());
    }

    #[test]
    fn date_input_year_boundaries() {
        assert!("2000-01-01".parse::<DateInput>().is_ok());
        assert!("1970-12-31".parse::<DateInput>().is_ok());
    }

    #[test]
    fn date_input_invalid_month_is_error() {
        assert!("2024-13-01".parse::<DateInput>().is_err());
    }

    #[test]
    fn date_input_invalid_day_is_error() {
        assert!("2024-02-30".parse::<DateInput>().is_err());
    }

    #[test]
    fn date_input_wrong_format_is_error() {
        assert!("15-03-2024".parse::<DateInput>().is_err());
        assert!("2024/03/15".parse::<DateInput>().is_err());
        assert!("not-a-date".parse::<DateInput>().is_err());
    }

    #[test]
    fn fmt_duration_zero() {
        assert_eq!(fmt_duration(Duration::ZERO), "0m");
    }

    #[test]
    fn fmt_duration_minutes_only() {
        assert_eq!(fmt_duration(Duration::minutes(45)), "45m");
    }

    #[test]
    fn fmt_duration_hours_only() {
        assert_eq!(fmt_duration(Duration::hours(8)), "8h 00m");
    }

    #[test]
    fn fmt_duration_hours_and_minutes() {
        assert_eq!(fmt_duration(Duration::hours(1) + Duration::minutes(30)), "1h 30m");
    }

    #[test]
    fn fmt_duration_negative_shows_absolute() {
        // fmt_duration strips sign - callers use fmt_signed_duration for signed output
        assert_eq!(fmt_duration(Duration::hours(-2)), "2h 00m");
    }

    #[test]
    fn fmt_signed_duration_positive() {
        assert_eq!(fmt_signed_duration(Duration::hours(1)), "+1h 00m");
    }

    #[test]
    fn fmt_signed_duration_negative() {
        assert_eq!(fmt_signed_duration(Duration::minutes(-30)), "-30m");
    }

    #[test]
    fn fmt_signed_duration_zero() {
        assert_eq!(fmt_signed_duration(Duration::ZERO), "+0m");
    }

    #[test]
    fn weekday_name_all_variants() {
        assert_eq!(weekday_name(Weekday::Monday), "Monday");
        assert_eq!(weekday_name(Weekday::Tuesday), "Tuesday");
        assert_eq!(weekday_name(Weekday::Wednesday), "Wednesday");
        assert_eq!(weekday_name(Weekday::Thursday), "Thursday");
        assert_eq!(weekday_name(Weekday::Friday), "Friday");
        assert_eq!(weekday_name(Weekday::Saturday), "Saturday");
        assert_eq!(weekday_name(Weekday::Sunday), "Sunday");
    }
}

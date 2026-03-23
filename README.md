# rs-date-utils

[![CI](https://github.com/philiprehberger/rs-date-utils/actions/workflows/ci.yml/badge.svg)](https://github.com/philiprehberger/rs-date-utils/actions/workflows/ci.yml)
[![Crates.io](https://img.shields.io/crates/v/philiprehberger-date-utils.svg)](https://crates.io/crates/philiprehberger-date-utils)
[![License](https://img.shields.io/github/license/philiprehberger/rs-date-utils)](LICENSE)

Date and time utilities — business days, date ranges, holiday calendars, and formatting shortcuts

## Installation

```toml
[dependencies]
philiprehberger-date-utils = "0.1.2"
```

## Usage

```rust
use chrono::NaiveDate;
use philiprehberger_date_utils::{
    add_business_days, business_days_between, is_business_day,
    next_business_day, USFederalCalendar, NoHolidayCalendar,
};

let cal = USFederalCalendar;
let date = NaiveDate::from_ymd_opt(2026, 3, 19).unwrap(); // Thursday

// Add 5 business days (skips weekends and holidays)
let future = add_business_days(date, 5, &cal);

// Count business days between two dates
let count = business_days_between(
    NaiveDate::from_ymd_opt(2026, 3, 16).unwrap(),
    NaiveDate::from_ymd_opt(2026, 3, 23).unwrap(),
    &cal,
);

// Check if a date is a business day
assert!(is_business_day(date, &cal));

// Get the next business day
let next = next_business_day(date, &cal);
```

### Holiday Calendars

```rust
use chrono::NaiveDate;
use philiprehberger_date_utils::{HolidayCalendar, USFederalCalendar};

let cal = USFederalCalendar;

// Check if a date is a holiday
let christmas = NaiveDate::from_ymd_opt(2026, 12, 25).unwrap();
assert!(cal.is_holiday(christmas));

// Get all holidays in a year
let holidays = cal.holidays_in_year(2026);
assert_eq!(holidays.len(), 11);
```

### Date Ranges

```rust
use chrono::NaiveDate;
use philiprehberger_date_utils::{DateRange, NoHolidayCalendar};

let start = NaiveDate::from_ymd_opt(2026, 3, 1).unwrap();
let end = NaiveDate::from_ymd_opt(2026, 3, 31).unwrap();
let range = DateRange::new(start, end);

// Iterate over days
for date in range.iter_days() {
    // ...
}

// Iterate over weeks
for week_start in range.iter_weeks() {
    // ...
}

// Check containment and overlap
assert!(range.contains(NaiveDate::from_ymd_opt(2026, 3, 15).unwrap()));
assert_eq!(range.days_count(), 31);

let cal = NoHolidayCalendar;
let biz_days = range.business_days_count(&cal);
```

### Formatting

```rust
use chrono::NaiveDate;
use philiprehberger_date_utils::{format_long, format_short, format_iso};

let date = NaiveDate::from_ymd_opt(2026, 3, 19).unwrap();

assert_eq!(format_long(date), "March 19, 2026");
assert_eq!(format_short(date), "Mar 19, 2026");
assert_eq!(format_iso(date), "2026-03-19");
```

### Quarter and Fiscal Year

```rust
use chrono::NaiveDate;
use philiprehberger_date_utils::{quarter, fiscal_year, years_between};

let date = NaiveDate::from_ymd_opt(2026, 3, 19).unwrap();

assert_eq!(quarter(date), 1);
assert_eq!(fiscal_year(date, 10), 2025); // October fiscal year start

let birth = NaiveDate::from_ymd_opt(2000, 3, 19).unwrap();
assert_eq!(years_between(birth, date), 26);
```

### Month and Quarter Boundaries

```rust
use chrono::NaiveDate;
use philiprehberger_date_utils::{
    start_of_month, end_of_month, start_of_quarter, end_of_quarter,
};

let date = NaiveDate::from_ymd_opt(2026, 3, 19).unwrap();

assert_eq!(start_of_month(date), NaiveDate::from_ymd_opt(2026, 3, 1).unwrap());
assert_eq!(end_of_month(date), NaiveDate::from_ymd_opt(2026, 3, 31).unwrap());
assert_eq!(start_of_quarter(date), NaiveDate::from_ymd_opt(2026, 1, 1).unwrap());
assert_eq!(end_of_quarter(date), NaiveDate::from_ymd_opt(2026, 3, 31).unwrap());
```

## API

| Function / Type | Description |
|---|---|
| `HolidayCalendar` | Trait for holiday calendars with `is_holiday()` and `holidays_in_year()` |
| `USFederalCalendar` | US federal holidays with weekend adjustment rules |
| `NoHolidayCalendar` | Empty calendar — no holidays |
| `add_business_days(date, days, cal)` | Add or subtract business days, skipping weekends and holidays |
| `business_days_between(start, end, cal)` | Count business days between two dates (end exclusive) |
| `is_business_day(date, cal)` | Check if a date is a business day |
| `next_business_day(date, cal)` | Get the next business day after a date |
| `DateRange::new(start, end)` | Create an inclusive date range |
| `range.iter_days()` | Iterate over every day in the range |
| `range.iter_weeks()` | Iterate over week-start dates in the range |
| `range.contains(date)` | Check if a date is within the range |
| `range.overlaps(other)` | Check if two ranges overlap |
| `range.days_count()` | Total days in the range (inclusive) |
| `range.business_days_count(cal)` | Business days in the range (inclusive) |
| `quarter(date)` | Get the quarter (1-4) for a date |
| `fiscal_year(date, start_month)` | Get the fiscal year based on a custom start month |
| `years_between(from, to)` | Full years between two dates (age calculation) |
| `format_long(date)` | Format as "March 19, 2026" |
| `format_short(date)` | Format as "Mar 19, 2026" |
| `format_iso(date)` | Format as "2026-03-19" |
| `start_of_month(date)` | First day of the month |
| `end_of_month(date)` | Last day of the month |
| `start_of_quarter(date)` | First day of the quarter |
| `end_of_quarter(date)` | Last day of the quarter |

## Development

```bash
cargo test
cargo clippy -- -D warnings
```

## License

MIT

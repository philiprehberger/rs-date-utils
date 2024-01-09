# Changelog

## 0.1.0 (2026-03-19)

- Initial release
- `HolidayCalendar` trait with `is_holiday()` and `holidays_in_year()`
- `USFederalCalendar` with all 11 federal holidays and weekend adjustments
- `NoHolidayCalendar` for weekday-only business day calculations
- `add_business_days()` — add or subtract business days skipping weekends and holidays
- `business_days_between()` — count business days between two dates
- `is_business_day()` and `next_business_day()` helpers
- `DateRange` struct with day/week iteration, contains, overlaps, and business day counting
- `quarter()`, `fiscal_year()`, `years_between()` utility functions
- `format_long()`, `format_short()`, `format_iso()` date formatting
- `start_of_month()`, `end_of_month()`, `start_of_quarter()`, `end_of_quarter()`

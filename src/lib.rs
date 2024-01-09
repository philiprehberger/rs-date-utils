use chrono::{Datelike, Duration, NaiveDate, Weekday};

// ---------------------------------------------------------------------------
// HolidayCalendar trait
// ---------------------------------------------------------------------------

/// A calendar that can report which dates are holidays.
pub trait HolidayCalendar {
    /// Returns `true` if the given date is a holiday.
    fn is_holiday(&self, date: NaiveDate) -> bool;

    /// Returns all holidays observed in the given year.
    fn holidays_in_year(&self, year: i32) -> Vec<NaiveDate>;
}

// ---------------------------------------------------------------------------
// NoHolidayCalendar
// ---------------------------------------------------------------------------

/// A calendar with no holidays — only weekends are non-business days.
pub struct NoHolidayCalendar;

impl HolidayCalendar for NoHolidayCalendar {
    fn is_holiday(&self, _date: NaiveDate) -> bool {
        false
    }

    fn holidays_in_year(&self, _year: i32) -> Vec<NaiveDate> {
        Vec::new()
    }
}

// ---------------------------------------------------------------------------
// USFederalCalendar
// ---------------------------------------------------------------------------

/// US Federal holiday calendar with weekend adjustment rules.
///
/// If a holiday falls on Saturday it is observed on the preceding Friday.
/// If a holiday falls on Sunday it is observed on the following Monday.
pub struct USFederalCalendar;

impl USFederalCalendar {
    /// Return the nth occurrence of a given weekday in (year, month).
    fn nth_weekday(year: i32, month: u32, weekday: Weekday, n: u32) -> NaiveDate {
        let first = NaiveDate::from_ymd_opt(year, month, 1).unwrap();
        let first_wd = first.weekday();
        let days_ahead = (weekday.num_days_from_monday() as i64
            - first_wd.num_days_from_monday() as i64
            + 7)
            % 7;
        first + Duration::days(days_ahead + 7 * (n as i64 - 1))
    }

    /// Return the last occurrence of a given weekday in (year, month).
    fn last_weekday(year: i32, month: u32, weekday: Weekday) -> NaiveDate {
        let last_day = end_of_month(NaiveDate::from_ymd_opt(year, month, 1).unwrap());
        let last_wd = last_day.weekday();
        let days_back = (last_wd.num_days_from_monday() as i64
            - weekday.num_days_from_monday() as i64
            + 7)
            % 7;
        last_day - Duration::days(days_back)
    }

    /// Apply weekend adjustment: Saturday → Friday, Sunday → Monday.
    fn observe(date: NaiveDate) -> NaiveDate {
        match date.weekday() {
            Weekday::Sat => date - Duration::days(1),
            Weekday::Sun => date + Duration::days(1),
            _ => date,
        }
    }

    /// Raw (unadjusted) federal holidays for a given year.
    fn raw_holidays(year: i32) -> Vec<NaiveDate> {
        vec![
            // New Year's Day
            NaiveDate::from_ymd_opt(year, 1, 1).unwrap(),
            // MLK Day — 3rd Monday in January
            Self::nth_weekday(year, 1, Weekday::Mon, 3),
            // Presidents' Day — 3rd Monday in February
            Self::nth_weekday(year, 2, Weekday::Mon, 3),
            // Memorial Day — last Monday in May
            Self::last_weekday(year, 5, Weekday::Mon),
            // Juneteenth
            NaiveDate::from_ymd_opt(year, 6, 19).unwrap(),
            // Independence Day
            NaiveDate::from_ymd_opt(year, 7, 4).unwrap(),
            // Labor Day — 1st Monday in September
            Self::nth_weekday(year, 9, Weekday::Mon, 1),
            // Columbus Day — 2nd Monday in October
            Self::nth_weekday(year, 10, Weekday::Mon, 2),
            // Veterans Day
            NaiveDate::from_ymd_opt(year, 11, 11).unwrap(),
            // Thanksgiving — 4th Thursday in November
            Self::nth_weekday(year, 11, Weekday::Thu, 4),
            // Christmas
            NaiveDate::from_ymd_opt(year, 12, 25).unwrap(),
        ]
    }
}

impl HolidayCalendar for USFederalCalendar {
    fn is_holiday(&self, date: NaiveDate) -> bool {
        self.holidays_in_year(date.year())
            .contains(&date)
    }

    fn holidays_in_year(&self, year: i32) -> Vec<NaiveDate> {
        let mut holidays: Vec<NaiveDate> = Self::raw_holidays(year)
            .into_iter()
            .map(Self::observe)
            .collect();
        holidays.sort();
        holidays.dedup();
        holidays
    }
}

// ---------------------------------------------------------------------------
// Business day functions
// ---------------------------------------------------------------------------

/// Returns `true` if the date is a business day (not a weekend, not a holiday).
pub fn is_business_day(date: NaiveDate, calendar: &impl HolidayCalendar) -> bool {
    let wd = date.weekday();
    wd != Weekday::Sat && wd != Weekday::Sun && !calendar.is_holiday(date)
}

/// Returns the next business day strictly after `date`.
pub fn next_business_day(date: NaiveDate, calendar: &impl HolidayCalendar) -> NaiveDate {
    let mut d = date + Duration::days(1);
    while !is_business_day(d, calendar) {
        d += Duration::days(1);
    }
    d
}

/// Add (or subtract) business days to a date, skipping weekends and holidays.
///
/// - Positive `days` moves forward.
/// - Negative `days` moves backward.
/// - Zero returns the date itself (even if it is not a business day).
pub fn add_business_days(
    date: NaiveDate,
    days: i32,
    calendar: &impl HolidayCalendar,
) -> NaiveDate {
    if days == 0 {
        return date;
    }
    let step = if days > 0 { 1i64 } else { -1i64 };
    let mut remaining = days.unsigned_abs();
    let mut current = date;
    while remaining > 0 {
        current += Duration::days(step);
        if is_business_day(current, calendar) {
            remaining -= 1;
        }
    }
    current
}

/// Count the number of business days between `start` (inclusive) and `end` (exclusive).
///
/// If `end < start` the result is negative.
pub fn business_days_between(
    start: NaiveDate,
    end: NaiveDate,
    calendar: &impl HolidayCalendar,
) -> i32 {
    if start == end {
        return 0;
    }
    let (from, to, sign) = if start < end {
        (start, end, 1)
    } else {
        (end, start, -1)
    };
    let mut count = 0i32;
    let mut d = from;
    while d < to {
        if is_business_day(d, calendar) {
            count += 1;
        }
        d += Duration::days(1);
    }
    count * sign
}

// ---------------------------------------------------------------------------
// DateRange
// ---------------------------------------------------------------------------

/// An inclusive date range from `start` to `end`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DateRange {
    start: NaiveDate,
    end: NaiveDate,
}

impl DateRange {
    /// Create a new `DateRange`. Panics if `start > end`.
    pub fn new(start: NaiveDate, end: NaiveDate) -> Self {
        assert!(start <= end, "DateRange: start must be <= end");
        Self { start, end }
    }

    /// Start date of the range.
    pub fn start(&self) -> NaiveDate {
        self.start
    }

    /// End date of the range.
    pub fn end(&self) -> NaiveDate {
        self.end
    }

    /// Iterate over every day in the range (inclusive).
    pub fn iter_days(&self) -> impl Iterator<Item = NaiveDate> {
        let start = self.start;
        let end = self.end;
        let mut current = start;
        std::iter::from_fn(move || {
            if current <= end {
                let d = current;
                current += Duration::days(1);
                Some(d)
            } else {
                None
            }
        })
    }

    /// Iterate over week-start dates within the range.
    ///
    /// The first yielded date is `start`, then every 7 days after that
    /// while still within the range.
    pub fn iter_weeks(&self) -> impl Iterator<Item = NaiveDate> {
        let start = self.start;
        let end = self.end;
        let mut current = start;
        std::iter::from_fn(move || {
            if current <= end {
                let d = current;
                current += Duration::weeks(1);
                Some(d)
            } else {
                None
            }
        })
    }

    /// Returns `true` if `date` falls within the range (inclusive).
    pub fn contains(&self, date: NaiveDate) -> bool {
        date >= self.start && date <= self.end
    }

    /// Returns `true` if this range overlaps with `other`.
    pub fn overlaps(&self, other: &DateRange) -> bool {
        self.start <= other.end && other.start <= self.end
    }

    /// Number of days in the range (inclusive).
    pub fn days_count(&self) -> i64 {
        (self.end - self.start).num_days() + 1
    }

    /// Number of business days in the range (inclusive of both endpoints).
    pub fn business_days_count(&self, calendar: &impl HolidayCalendar) -> i32 {
        // business_days_between is start-inclusive, end-exclusive, so add one day to end
        business_days_between(self.start, self.end + Duration::days(1), calendar)
    }
}

// ---------------------------------------------------------------------------
// Utility functions
// ---------------------------------------------------------------------------

/// Returns the quarter (1-4) for the given date.
pub fn quarter(date: NaiveDate) -> u8 {
    ((date.month() - 1) / 3 + 1) as u8
}

/// Returns the fiscal year for a given date with a custom fiscal-year start month.
///
/// If the date's month is >= `start_month`, the fiscal year equals the calendar year.
/// Otherwise it equals the previous calendar year.
///
/// For example, with `start_month = 10` (October), September 2026 → FY 2025,
/// October 2026 → FY 2026.
pub fn fiscal_year(date: NaiveDate, start_month: u32) -> i32 {
    if start_month <= 1 || date.month() >= start_month {
        date.year()
    } else {
        date.year() - 1
    }
}

/// Returns the number of full years between two dates (age-style calculation).
///
/// If `to < from` the result is negative.
pub fn years_between(from: NaiveDate, to: NaiveDate) -> i32 {
    if from <= to {
        let mut years = to.year() - from.year();
        if (to.month(), to.day()) < (from.month(), from.day()) {
            years -= 1;
        }
        years
    } else {
        -years_between(to, from)
    }
}

/// Format a date as "March 19, 2026".
pub fn format_long(date: NaiveDate) -> String {
    date.format("%B %-d, %Y").to_string()
}

/// Format a date as "Mar 19, 2026".
pub fn format_short(date: NaiveDate) -> String {
    date.format("%b %-d, %Y").to_string()
}

/// Format a date as "2026-03-19".
pub fn format_iso(date: NaiveDate) -> String {
    date.format("%Y-%m-%d").to_string()
}

/// Returns the first day of the month containing `date`.
pub fn start_of_month(date: NaiveDate) -> NaiveDate {
    NaiveDate::from_ymd_opt(date.year(), date.month(), 1).unwrap()
}

/// Returns the last day of the month containing `date`.
pub fn end_of_month(date: NaiveDate) -> NaiveDate {
    let (y, m) = if date.month() == 12 {
        (date.year() + 1, 1)
    } else {
        (date.year(), date.month() + 1)
    };
    NaiveDate::from_ymd_opt(y, m, 1).unwrap() - Duration::days(1)
}

/// Returns the first day of the quarter containing `date`.
pub fn start_of_quarter(date: NaiveDate) -> NaiveDate {
    let q = quarter(date);
    let month = (q as u32 - 1) * 3 + 1;
    NaiveDate::from_ymd_opt(date.year(), month, 1).unwrap()
}

/// Returns the last day of the quarter containing `date`.
pub fn end_of_quarter(date: NaiveDate) -> NaiveDate {
    let q = quarter(date);
    let last_month = q as u32 * 3;
    end_of_month(NaiveDate::from_ymd_opt(date.year(), last_month, 1).unwrap())
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn d(year: i32, month: u32, day: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(year, month, day).unwrap()
    }

    // -- US Federal Holiday tests --

    #[test]
    fn us_holidays_2026_known_dates() {
        let cal = USFederalCalendar;
        let holidays = cal.holidays_in_year(2026);

        // New Year's Day 2026 = Thursday
        assert!(holidays.contains(&d(2026, 1, 1)));
        // MLK Day 2026 = 3rd Monday Jan = Jan 19
        assert!(holidays.contains(&d(2026, 1, 19)));
        // Presidents' Day = 3rd Monday Feb = Feb 16
        assert!(holidays.contains(&d(2026, 2, 16)));
        // Memorial Day = last Monday May = May 25
        assert!(holidays.contains(&d(2026, 5, 25)));
        // Juneteenth = June 19, 2026 is Friday
        assert!(holidays.contains(&d(2026, 6, 19)));
        // Independence Day = July 4, 2026 is Saturday → observed Friday July 3
        assert!(holidays.contains(&d(2026, 7, 3)));
        assert!(!holidays.contains(&d(2026, 7, 4)));
        // Labor Day = 1st Monday Sep = Sep 7
        assert!(holidays.contains(&d(2026, 9, 7)));
        // Columbus Day = 2nd Monday Oct = Oct 12
        assert!(holidays.contains(&d(2026, 10, 12)));
        // Veterans Day = Nov 11, 2026 is Wednesday
        assert!(holidays.contains(&d(2026, 11, 11)));
        // Thanksgiving = 4th Thursday Nov = Nov 26
        assert!(holidays.contains(&d(2026, 11, 26)));
        // Christmas = Dec 25, 2026 is Friday
        assert!(holidays.contains(&d(2026, 12, 25)));
    }

    #[test]
    fn us_holiday_weekend_adjustment_saturday() {
        let cal = USFederalCalendar;
        // July 4, 2026 is Saturday → observed Friday July 3
        assert!(cal.is_holiday(d(2026, 7, 3)));
        assert!(!cal.is_holiday(d(2026, 7, 4)));
    }

    #[test]
    fn us_holiday_weekend_adjustment_sunday() {
        let cal = USFederalCalendar;
        // Juneteenth 2021: June 19 is Saturday → observed Friday June 18
        assert!(cal.is_holiday(d(2021, 6, 18)));
        // Christmas 2022: Dec 25 is Sunday → observed Monday Dec 26
        assert!(cal.is_holiday(d(2022, 12, 26)));
    }

    #[test]
    fn no_holiday_calendar() {
        let cal = NoHolidayCalendar;
        assert!(!cal.is_holiday(d(2026, 12, 25)));
        assert!(cal.holidays_in_year(2026).is_empty());
    }

    // -- Business day tests --

    #[test]
    fn is_business_day_weekday() {
        let cal = NoHolidayCalendar;
        // Wednesday
        assert!(is_business_day(d(2026, 3, 18), &cal));
        // Saturday
        assert!(!is_business_day(d(2026, 3, 14), &cal));
        // Sunday
        assert!(!is_business_day(d(2026, 3, 15), &cal));
    }

    #[test]
    fn is_business_day_with_holiday() {
        let cal = USFederalCalendar;
        // Christmas 2026 is Thursday Dec 25
        assert!(!is_business_day(d(2026, 12, 25), &cal));
        // Dec 24 is Wednesday, not a holiday
        assert!(is_business_day(d(2026, 12, 24), &cal));
    }

    #[test]
    fn add_business_days_forward() {
        let cal = NoHolidayCalendar;
        // Wednesday + 3 business days = Monday (skip weekend)
        assert_eq!(add_business_days(d(2026, 3, 18), 3, &cal), d(2026, 3, 23));
    }

    #[test]
    fn add_business_days_backward() {
        let cal = NoHolidayCalendar;
        // Monday - 1 business day = Friday
        assert_eq!(add_business_days(d(2026, 3, 23), -1, &cal), d(2026, 3, 20));
    }

    #[test]
    fn add_business_days_zero() {
        let cal = NoHolidayCalendar;
        let saturday = d(2026, 3, 14);
        assert_eq!(add_business_days(saturday, 0, &cal), saturday);
    }

    #[test]
    fn add_business_days_with_holiday() {
        let cal = USFederalCalendar;
        // Dec 24 (Thu) + 1 business day should skip Christmas (Fri Dec 25) → Mon Dec 28
        assert_eq!(add_business_days(d(2026, 12, 24), 1, &cal), d(2026, 12, 28));
    }

    #[test]
    fn business_days_between_same_day() {
        let cal = NoHolidayCalendar;
        assert_eq!(business_days_between(d(2026, 3, 18), d(2026, 3, 18), &cal), 0);
    }

    #[test]
    fn business_days_between_one_week() {
        let cal = NoHolidayCalendar;
        // Mon to next Mon = 5 business days
        assert_eq!(
            business_days_between(d(2026, 3, 16), d(2026, 3, 23), &cal),
            5
        );
    }

    #[test]
    fn business_days_between_reversed() {
        let cal = NoHolidayCalendar;
        assert_eq!(
            business_days_between(d(2026, 3, 23), d(2026, 3, 16), &cal),
            -5
        );
    }

    #[test]
    fn business_days_between_with_holiday() {
        let cal = USFederalCalendar;
        // Week of Dec 21-26, 2026: Christmas (Dec 25 Thu) is holiday
        // Mon-Fri = 5, minus 1 holiday = 4
        assert_eq!(
            business_days_between(d(2026, 12, 21), d(2026, 12, 26), &cal),
            4
        );
    }

    #[test]
    fn next_business_day_from_friday() {
        let cal = NoHolidayCalendar;
        // Friday → Monday
        assert_eq!(next_business_day(d(2026, 3, 20), &cal), d(2026, 3, 23));
    }

    #[test]
    fn next_business_day_from_saturday() {
        let cal = NoHolidayCalendar;
        // Saturday → Monday
        assert_eq!(next_business_day(d(2026, 3, 21), &cal), d(2026, 3, 23));
    }

    // -- DateRange tests --

    #[test]
    fn date_range_days_count() {
        let range = DateRange::new(d(2026, 3, 1), d(2026, 3, 31));
        assert_eq!(range.days_count(), 31);
    }

    #[test]
    fn date_range_iter_days() {
        let range = DateRange::new(d(2026, 3, 1), d(2026, 3, 3));
        let days: Vec<_> = range.iter_days().collect();
        assert_eq!(days, vec![d(2026, 3, 1), d(2026, 3, 2), d(2026, 3, 3)]);
    }

    #[test]
    fn date_range_iter_weeks() {
        let range = DateRange::new(d(2026, 3, 1), d(2026, 3, 20));
        let weeks: Vec<_> = range.iter_weeks().collect();
        assert_eq!(
            weeks,
            vec![d(2026, 3, 1), d(2026, 3, 8), d(2026, 3, 15)]
        );
    }

    #[test]
    fn date_range_contains() {
        let range = DateRange::new(d(2026, 3, 10), d(2026, 3, 20));
        assert!(range.contains(d(2026, 3, 10)));
        assert!(range.contains(d(2026, 3, 15)));
        assert!(range.contains(d(2026, 3, 20)));
        assert!(!range.contains(d(2026, 3, 9)));
        assert!(!range.contains(d(2026, 3, 21)));
    }

    #[test]
    fn date_range_overlaps() {
        let a = DateRange::new(d(2026, 3, 1), d(2026, 3, 15));
        let b = DateRange::new(d(2026, 3, 10), d(2026, 3, 25));
        let c = DateRange::new(d(2026, 3, 16), d(2026, 3, 20));
        assert!(a.overlaps(&b));
        assert!(b.overlaps(&a));
        assert!(!a.overlaps(&c));
    }

    #[test]
    fn date_range_business_days_count() {
        let cal = NoHolidayCalendar;
        // March 16 (Mon) to March 20 (Fri) = 5 business days
        let range = DateRange::new(d(2026, 3, 16), d(2026, 3, 20));
        assert_eq!(range.business_days_count(&cal), 5);
    }

    #[test]
    #[should_panic(expected = "start must be <= end")]
    fn date_range_invalid() {
        DateRange::new(d(2026, 3, 20), d(2026, 3, 10));
    }

    // -- Utility function tests --

    #[test]
    fn test_quarter() {
        assert_eq!(quarter(d(2026, 1, 15)), 1);
        assert_eq!(quarter(d(2026, 3, 31)), 1);
        assert_eq!(quarter(d(2026, 4, 1)), 2);
        assert_eq!(quarter(d(2026, 6, 30)), 2);
        assert_eq!(quarter(d(2026, 7, 1)), 3);
        assert_eq!(quarter(d(2026, 9, 30)), 3);
        assert_eq!(quarter(d(2026, 10, 1)), 4);
        assert_eq!(quarter(d(2026, 12, 31)), 4);
    }

    #[test]
    fn test_fiscal_year() {
        // October start (US government)
        assert_eq!(fiscal_year(d(2026, 9, 30), 10), 2025);
        assert_eq!(fiscal_year(d(2026, 10, 1), 10), 2026);
        assert_eq!(fiscal_year(d(2026, 12, 31), 10), 2026);
        // January start = calendar year
        assert_eq!(fiscal_year(d(2026, 6, 15), 1), 2026);
    }

    #[test]
    fn test_years_between() {
        assert_eq!(years_between(d(2000, 3, 19), d(2026, 3, 19)), 26);
        assert_eq!(years_between(d(2000, 3, 19), d(2026, 3, 18)), 25);
        assert_eq!(years_between(d(2026, 3, 19), d(2000, 3, 19)), -26);
    }

    #[test]
    fn test_years_between_leap_year() {
        // Born on leap day
        assert_eq!(years_between(d(2000, 2, 29), d(2026, 2, 28)), 25);
        assert_eq!(years_between(d(2000, 2, 29), d(2026, 3, 1)), 26);
    }

    #[test]
    fn test_format_long() {
        assert_eq!(format_long(d(2026, 3, 19)), "March 19, 2026");
    }

    #[test]
    fn test_format_short() {
        assert_eq!(format_short(d(2026, 3, 19)), "Mar 19, 2026");
    }

    #[test]
    fn test_format_iso() {
        assert_eq!(format_iso(d(2026, 3, 19)), "2026-03-19");
    }

    #[test]
    fn test_start_of_month() {
        assert_eq!(start_of_month(d(2026, 3, 19)), d(2026, 3, 1));
    }

    #[test]
    fn test_end_of_month() {
        assert_eq!(end_of_month(d(2026, 3, 19)), d(2026, 3, 31));
        // February non-leap
        assert_eq!(end_of_month(d(2026, 2, 10)), d(2026, 2, 28));
        // February leap
        assert_eq!(end_of_month(d(2024, 2, 10)), d(2024, 2, 29));
        // December
        assert_eq!(end_of_month(d(2026, 12, 5)), d(2026, 12, 31));
    }

    #[test]
    fn test_start_of_quarter() {
        assert_eq!(start_of_quarter(d(2026, 3, 19)), d(2026, 1, 1));
        assert_eq!(start_of_quarter(d(2026, 5, 10)), d(2026, 4, 1));
        assert_eq!(start_of_quarter(d(2026, 8, 20)), d(2026, 7, 1));
        assert_eq!(start_of_quarter(d(2026, 11, 5)), d(2026, 10, 1));
    }

    #[test]
    fn test_end_of_quarter() {
        assert_eq!(end_of_quarter(d(2026, 2, 15)), d(2026, 3, 31));
        assert_eq!(end_of_quarter(d(2026, 5, 10)), d(2026, 6, 30));
        assert_eq!(end_of_quarter(d(2026, 8, 20)), d(2026, 9, 30));
        assert_eq!(end_of_quarter(d(2026, 11, 5)), d(2026, 12, 31));
    }

    #[test]
    fn add_business_days_across_year_boundary() {
        let cal = USFederalCalendar;
        // Dec 31, 2025 is Wednesday. Jan 1 2026 is holiday (Thursday).
        // +1 business day from Dec 31 → Jan 2 (Friday)
        assert_eq!(add_business_days(d(2025, 12, 31), 1, &cal), d(2026, 1, 2));
    }

    #[test]
    fn business_days_between_across_year_boundary() {
        let cal = USFederalCalendar;
        // Dec 29 (Mon) to Jan 2 (Fri): Dec 29, 30, 31 are biz days, Jan 1 holiday
        // So 3 business days
        assert_eq!(
            business_days_between(d(2025, 12, 29), d(2026, 1, 2), &cal),
            3
        );
    }
}

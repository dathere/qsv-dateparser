//! A rust library for parsing date strings in commonly used formats. Parsed date will be returned
//! as `chrono`'s `DateTime<Utc>`.
//!
//! # Quick Start
//!
//!
//! Use `str`'s `parse` method:
//!
//! ```
//! use chrono::prelude::*;
//! use qsv_dateparser::DateTimeUtc;
//! use std::error::Error;
//!
//! fn main() -> Result<(), Box<dyn Error>> {
//!     assert_eq!(
//!         "2021-05-14 18:51 PDT".parse::<DateTimeUtc>()?.0,
//!         Utc.ymd(2021, 5, 15).and_hms(1, 51, 0),
//!     );
//!     Ok(())
//! }
//! ```
//!
//! ## Accepted date formats
//!
//! ```
//! use qsv_dateparser::DateTimeUtc;
//!
//! let accepted = vec![
//!     // unix timestamp
//!     "1511648546",
//!     "1620021848429",
//!     "1620024872717915000",
//!     "0",
//!     "-770172300",
//!     "1671673426.123456789",
//!     // rfc3339
//!     "2021-05-01T01:17:02.604456Z",
//!     "2017-11-25T22:34:50Z",
//!     // rfc2822
//!     "Wed, 02 Jun 2021 06:31:39 GMT",
//!     // yyyy-mm-dd hh:mm:ss
//!     "2014-04-26 05:24:37 PM",
//!     "2021-04-30 21:14",
//!     "2021-04-30 21:14:10",
//!     "2021-04-30 21:14:10.052282",
//!     "2014-04-26 17:24:37.123",
//!     "2014-04-26 17:24:37.3186369",
//!     "2012-08-03 18:31:59.257000000",
//!     // yyyy-mm-dd hh:mm:ss z
//!     "2017-11-25 13:31:15 PST",
//!     "2017-11-25 13:31 PST",
//!     "2014-12-16 06:20:00 UTC",
//!     "2014-12-16 06:20:00 GMT",
//!     "2014-04-26 13:13:43 +0800",
//!     "2014-04-26 13:13:44 +09:00",
//!     "2012-08-03 18:31:59.257000000 +0000",
//!     "2015-09-30 18:48:56.35272715 UTC",
//!     // yyyy-mm-dd
//!     "2021-02-21",
//!     // yyyy-mm-dd z
//!     "2021-02-21 PST",
//!     "2021-02-21 UTC",
//!     "2020-07-20+08:00",
//!     // Mon dd, yyyy, hh:mm:ss
//!     "May 8, 2009 5:57:51 PM",
//!     "September 17, 2012 10:09am",
//!     "September 17, 2012, 10:10:09",
//!     // Mon dd, yyyy hh:mm:ss z
//!     "May 02, 2021 15:51:31 UTC",
//!     "May 02, 2021 15:51 UTC",
//!     "May 26, 2021, 12:49 AM PDT",
//!     "September 17, 2012 at 10:09am PST",
//!     // yyyy-mon-dd
//!     "2021-Feb-21",
//!     // Mon dd, yyyy
//!     "May 25, 2021",
//!     "oct 7, 1970",
//!     "oct 7, 70",
//!     "oct. 7, 1970",
//!     "oct. 7, 70",
//!     "October 7, 1970",
//!     // dd Mon yyyy hh:mm:ss
//!     "12 Feb 2006, 19:17",
//!     "12 Feb 2006 19:17",
//!     "14 May 2019 19:11:40.164",
//!     // dd Mon yyyy
//!     "7 oct 70",
//!     "7 oct 1970",
//!     "03 February 2013",
//!     "1 July 2013",
//!     // mm/dd/yyyy hh:mm:ss
//!     "4/8/2014 22:05",
//!     "04/08/2014 22:05",
//!     "4/8/14 22:05",
//!     "04/2/2014 03:00:51",
//!     "8/8/1965 12:00:00 AM",
//!     "8/8/1965 01:00:01 PM",
//!     "8/8/1965 01:00 PM",
//!     "8/8/1965 1:00 PM",
//!     "8/8/1965 12:00 AM",
//!     "4/02/2014 03:00:51",
//!     "03/19/2012 10:11:59",
//!     "03/19/2012 10:11:59.3186369",
//!     // mm/dd/yyyy
//!     "3/31/2014",
//!     "03/31/2014",
//!     "08/21/71",
//!     "8/1/71",
//!     // yyyy/mm/dd hh:mm:ss
//!     "2014/4/8 22:05",
//!     "2014/04/08 22:05",
//!     "2014/04/2 03:00:51",
//!     "2014/4/02 03:00:51",
//!     "2012/03/19 10:11:59",
//!     "2012/03/19 10:11:59.3186369",
//!     // yyyy/mm/dd
//!     "2014/3/31",
//!     "2014/03/31",
//! ];
//!
//! for date_str in accepted {
//!     let result = date_str.parse::<DateTimeUtc>();
//!     assert!(result.is_ok())
//! }
//! ```
//!
//! ### DMY Format
//!
//! It also accepts dates in DMY format with `parse_with_preference`,
//! and the `prefer_dmy` parameter set to true.
//!
//! ```
//! use qsv_dateparser::parse_with_preference;
//!
//! let accepted = vec![
//!     // dd/mm/yyyy
//!     "31/12/2020",
//!     "12/10/2019",
//!     "03/06/2018",
//!     "27/06/68",
//!     // dd/mm/yyyy hh:mm:ss
//!     "4/8/2014 22:05",
//!     "04/08/2014 22:05",
//!     "4/8/14 22:05",
//!     "04/2/2014 03:00:51",
//!     "8/8/1965 12:00:00 AM",
//!     "8/8/1965 01:00:01 PM",
//!     "8/8/1965 01:00 PM",
//!     "31/12/22 15:00"
//! ];
//!
//! for date_str in accepted {
//!     let result = parse_with_preference(date_str, true);
//!     assert!(result.is_ok());
//! }
//! ```

/// Datetime string parser
///
/// ```
/// use chrono::prelude::*;
/// use qsv_dateparser::datetime::Parse;
/// use std::error::Error;
///
/// fn main() -> Result<(), Box<dyn Error>> {
///     let utc_now_time = Utc::now().time();
///     let parse_with_local = Parse::new(&Local, utc_now_time);
///     assert_eq!(
///         parse_with_local.parse("2021-06-05 06:19 PM")?,
///         Local.ymd(2021, 6, 5).and_hms(18, 19, 0).with_timezone(&Utc),
///     );
///
///     let parse_with_utc = Parse::new(&Utc, utc_now_time);
///     assert_eq!(
///         parse_with_utc.parse("2021-06-05 06:19 PM")?,
///         Utc.ymd(2021, 6, 5).and_hms(18, 19, 0),
///     );
///
///     Ok(())
/// }
/// ```
pub mod datetime;

/// Timezone offset string parser
///
/// ```
/// use chrono::prelude::*;
/// use qsv_dateparser::timezone::parse;
/// use std::error::Error;
///
/// fn main() -> Result<(), Box<dyn Error>> {
///     assert_eq!(parse("-0800")?, FixedOffset::west(8 * 3600));
///     assert_eq!(parse("+10:00")?, FixedOffset::east(10 * 3600));
///     assert_eq!(parse("PST")?, FixedOffset::west(8 * 3600));
///     assert_eq!(parse("PDT")?, FixedOffset::west(7 * 3600));
///     assert_eq!(parse("UTC")?, FixedOffset::west(0));
///     assert_eq!(parse("GMT")?, FixedOffset::west(0));
///
///     Ok(())
/// }
/// ```
pub mod timezone;

use crate::datetime::Parse;
use anyhow::{Error, Result};
use chrono::prelude::*;
use std::sync::OnceLock;

/// `DateTimeUtc` is an alias for `chrono`'s `DateTime<UTC>`. It implements `std::str::FromStr`'s
/// `from_str` method, and it makes `str`'s `parse` method to understand the accepted date formats
/// from this crate.
///
/// ```
/// use qsv_dateparser::DateTimeUtc;
///
/// // parsed is DateTimeUTC and parsed.0 is chrono's DateTime<Utc>
/// match "May 02, 2021 15:51:31 UTC".parse::<DateTimeUtc>() {
///     Ok(parsed) => println!("PARSED into UTC datetime {:?}", parsed.0),
///     Err(err) => println!("ERROR from parsing datetime string: {}", err)
/// }
/// ```
pub struct DateTimeUtc(pub DateTime<Utc>);

impl std::str::FromStr for DateTimeUtc {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        parse(s).map(DateTimeUtc)
    }
}

static MIDNIGHT: OnceLock<chrono::NaiveTime> = OnceLock::new();

/// This function tries to recognize the input datetime string with a list of accepted formats.
/// When timezone is not provided, this function assumes it's a [`chrono::Local`] datetime. For
/// custom timezone, use [`parse_with_timezone()`] instead.If all options are exhausted,
/// [`parse()`] will return an error to let the caller know that no formats were matched.
#[inline]
pub fn parse(input: &str) -> Result<DateTime<Utc>> {
    Parse::new(&Local, Utc::now().time()).parse(input)
}

/// Similar to [`parse()`], this function takes a datetime string and a boolean `dmy_preference`.
/// When `dmy_preference` is `true`, it will parse strings using the DMY format. Otherwise, it
/// parses them using an MDY format.
#[inline]
pub fn parse_with_preference(input: &str, dmy_preference: bool) -> Result<DateTime<Utc>> {
    let midnight = MIDNIGHT.get_or_init(|| NaiveTime::from_hms_opt(0, 0, 0).unwrap());
    Parse::new_with_preference(&Utc, *midnight, dmy_preference).parse(input)
}

/// Similar to [`parse()`], this function takes a datetime string and a custom [`chrono::TimeZone`],
/// and tries to parse the datetime string. When timezone is not given in the string, this function
/// will assume and parse the datetime by the custom timezone provided in this function's arguments.
///
pub fn parse_with_timezone<Tz2: TimeZone>(input: &str, tz: &Tz2) -> Result<DateTime<Utc>> {
    Parse::new(tz, Utc::now().time()).parse(input)
}

/// Similar to [`parse()`], this function takes a datetime string and a boolean `dmy_preference`
/// and a timezone. When timezone is not given in the input string, this function will
/// assume and parse the datetime by the custom timezone provided in this function's arguments.
/// When `dmy_preference` is `true`, it will parse strings using the DMY format. Otherwise, it
/// parses them using an MDY format.
#[inline]
pub fn parse_with_preference_and_timezone<Tz2: TimeZone>(
    input: &str,
    dmy_preference: bool,
    tz: &Tz2,
) -> Result<DateTime<Utc>> {
    let midnight = MIDNIGHT.get_or_init(|| NaiveTime::from_hms_opt(0, 0, 0).unwrap());
    Parse::new_with_preference(tz, *midnight, dmy_preference).parse(input)
}

/// Similar to [`parse()`] and [`parse_with_timezone()`], this function takes a datetime string, a
/// custom [`chrono::TimeZone`] and a default naive time. In addition to assuming timezone when
/// it's not given in datetime string, this function also use provided default naive time in parsed
/// [`chrono::DateTime`].
///
pub fn parse_with<Tz2: TimeZone>(
    input: &str,
    tz: &Tz2,
    default_time: NaiveTime,
) -> Result<DateTime<Utc>> {
    Parse::new(tz, default_time).parse(input)
}

#[cfg(test)]
#[allow(deprecated)]
mod tests {
    use super::*;

    #[derive(Clone, Copy)]
    enum Trunc {
        Seconds,
        None,
    }

    #[test]
    fn parse_in_local() {
        let test_cases = vec![
            (
                "rfc3339",
                "2017-11-25T22:34:50Z",
                Utc.ymd(2017, 11, 25).and_hms(22, 34, 50),
                Trunc::None,
            ),
            (
                "rfc2822",
                "Wed, 02 Jun 2021 06:31:39 GMT",
                Utc.ymd(2021, 6, 2).and_hms(6, 31, 39),
                Trunc::None,
            ),
            (
                "ymd_hms",
                "2021-04-30 21:14:10",
                Local
                    .ymd(2021, 4, 30)
                    .and_hms(21, 14, 10)
                    .with_timezone(&Utc),
                Trunc::None,
            ),
            (
                "ymd_hms_z",
                "2017-11-25 13:31:15 PST",
                Utc.ymd(2017, 11, 25).and_hms(21, 31, 15),
                Trunc::None,
            ),
            (
                "ymd",
                "2021-02-21",
                Local
                    .ymd(2021, 2, 21)
                    .and_time(Local::now().time())
                    .unwrap()
                    .with_timezone(&Utc),
                Trunc::Seconds,
            ),
            (
                "ymd_z",
                "2021-02-21 PST",
                FixedOffset::west(8 * 3600)
                    .ymd(2021, 2, 21)
                    .and_time(
                        Utc::now()
                            .with_timezone(&FixedOffset::west(8 * 3600))
                            .time(),
                    )
                    .unwrap()
                    .with_timezone(&Utc),
                Trunc::Seconds,
            ),
            (
                "month_ymd",
                "2021-Feb-21",
                Local
                    .ymd(2021, 2, 21)
                    .and_time(Local::now().time())
                    .unwrap()
                    .with_timezone(&Utc),
                Trunc::Seconds,
            ),
            (
                "month_mdy_hms",
                "May 8, 2009 5:57:51 PM",
                Local
                    .ymd(2009, 5, 8)
                    .and_hms(17, 57, 51)
                    .with_timezone(&Utc),
                Trunc::None,
            ),
            (
                "month_mdy_hms_z",
                "May 02, 2021 15:51 UTC",
                Utc.ymd(2021, 5, 2).and_hms(15, 51, 0),
                Trunc::None,
            ),
            (
                "month_mdy",
                "May 25, 2021",
                Local
                    .ymd(2021, 5, 25)
                    .and_time(Local::now().time())
                    .unwrap()
                    .with_timezone(&Utc),
                Trunc::Seconds,
            ),
            (
                "month_dmy_hms",
                "14 May 2019 19:11:40.164",
                Local
                    .ymd(2019, 5, 14)
                    .and_hms_milli(19, 11, 40, 164)
                    .with_timezone(&Utc),
                Trunc::None,
            ),
            (
                "month_dmy",
                "1 July 2013",
                Local
                    .ymd(2013, 7, 1)
                    .and_time(Local::now().time())
                    .unwrap()
                    .with_timezone(&Utc),
                Trunc::Seconds,
            ),
            (
                "slash_mdy_hms",
                "03/19/2012 10:11:59",
                Local
                    .ymd(2012, 3, 19)
                    .and_hms(10, 11, 59)
                    .with_timezone(&Utc),
                Trunc::None,
            ),
            (
                "slash_mdy",
                "08/21/71",
                Local
                    .ymd(1971, 8, 21)
                    .and_time(Local::now().time())
                    .unwrap()
                    .with_timezone(&Utc),
                Trunc::Seconds,
            ),
            (
                "slash_ymd_hms",
                "2012/03/19 10:11:59",
                Local
                    .ymd(2012, 3, 19)
                    .and_hms(10, 11, 59)
                    .with_timezone(&Utc),
                Trunc::None,
            ),
            (
                "slash_ymd",
                "2014/3/31",
                Local
                    .ymd(2014, 3, 31)
                    .and_time(Local::now().time())
                    .unwrap()
                    .with_timezone(&Utc),
                Trunc::Seconds,
            ),
        ];

        for &(test, input, want, trunc) in test_cases.iter() {
            match trunc {
                Trunc::None => {
                    assert_eq!(
                        super::parse(input).unwrap(),
                        want,
                        "parse_in_local/{}/{}",
                        test,
                        input
                    )
                }
                Trunc::Seconds => assert_eq!(
                    super::parse(input)
                        .unwrap()
                        .trunc_subsecs(0)
                        .with_second(0)
                        .unwrap(),
                    want.trunc_subsecs(0).with_second(0).unwrap(),
                    "parse_in_local/{}/{}",
                    test,
                    input
                ),
            };
        }
    }

    #[test]
    fn parse_with_timezone_in_utc() {
        let test_cases = vec![
            (
                "rfc3339",
                "2017-11-25T22:34:50Z",
                Utc.ymd(2017, 11, 25).and_hms(22, 34, 50),
                Trunc::None,
            ),
            (
                "rfc2822",
                "Wed, 02 Jun 2021 06:31:39 GMT",
                Utc.ymd(2021, 6, 2).and_hms(6, 31, 39),
                Trunc::None,
            ),
            (
                "ymd_hms",
                "2021-04-30 21:14:10",
                Utc.ymd(2021, 4, 30).and_hms(21, 14, 10),
                Trunc::None,
            ),
            (
                "ymd_hms_z",
                "2017-11-25 13:31:15 PST",
                Utc.ymd(2017, 11, 25).and_hms(21, 31, 15),
                Trunc::None,
            ),
            (
                "ymd",
                "2021-02-21",
                Utc.ymd(2021, 2, 21).and_time(Utc::now().time()).unwrap(),
                Trunc::Seconds,
            ),
            (
                "ymd_z",
                "2021-02-21 PST",
                FixedOffset::west(8 * 3600)
                    .ymd(2021, 2, 21)
                    .and_time(
                        Utc::now()
                            .with_timezone(&FixedOffset::west(8 * 3600))
                            .time(),
                    )
                    .unwrap()
                    .with_timezone(&Utc),
                Trunc::Seconds,
            ),
            (
                "month_ymd",
                "2021-Feb-21",
                Utc.ymd(2021, 2, 21).and_time(Utc::now().time()).unwrap(),
                Trunc::Seconds,
            ),
            (
                "month_mdy_hms",
                "May 8, 2009 5:57:51 PM",
                Utc.ymd(2009, 5, 8).and_hms(17, 57, 51),
                Trunc::None,
            ),
            (
                "month_mdy_hms_z",
                "May 02, 2021 15:51 UTC",
                Utc.ymd(2021, 5, 2).and_hms(15, 51, 0),
                Trunc::None,
            ),
            (
                "month_mdy",
                "May 25, 2021",
                Utc.ymd(2021, 5, 25).and_time(Utc::now().time()).unwrap(),
                Trunc::Seconds,
            ),
            (
                "month_dmy_hms",
                "14 May 2019 19:11:40.164",
                Utc.ymd(2019, 5, 14).and_hms_milli(19, 11, 40, 164),
                Trunc::None,
            ),
            (
                "month_dmy",
                "1 July 2013",
                Utc.ymd(2013, 7, 1).and_time(Utc::now().time()).unwrap(),
                Trunc::Seconds,
            ),
            (
                "slash_mdy_hms",
                "03/19/2012 10:11:59",
                Utc.ymd(2012, 3, 19).and_hms(10, 11, 59),
                Trunc::None,
            ),
            (
                "slash_mdy",
                "08/21/71",
                Utc.ymd(1971, 8, 21).and_time(Utc::now().time()).unwrap(),
                Trunc::Seconds,
            ),
            (
                "slash_ymd_hms",
                "2012/03/19 10:11:59",
                Utc.ymd(2012, 3, 19).and_hms(10, 11, 59),
                Trunc::None,
            ),
            (
                "slash_ymd",
                "2014/3/31",
                Utc.ymd(2014, 3, 31).and_time(Utc::now().time()).unwrap(),
                Trunc::Seconds,
            ),
        ];

        for &(test, input, want, trunc) in test_cases.iter() {
            match trunc {
                Trunc::None => {
                    assert_eq!(
                        super::parse_with_timezone(input, &Utc).unwrap(),
                        want,
                        "parse_with_timezone_in_utc/{}/{}",
                        test,
                        input
                    )
                }
                Trunc::Seconds => assert_eq!(
                    super::parse_with_timezone(input, &Utc)
                        .unwrap()
                        .trunc_subsecs(0)
                        .with_second(0)
                        .unwrap(),
                    want.trunc_subsecs(0).with_second(0).unwrap(),
                    "parse_with_timezone_in_utc/{}/{}",
                    test,
                    input
                ),
            };
        }
    }

    #[test]
    fn parse_with_preference_and_timezone_in_utc() {
        let current_time = Utc::now().time();
        let current_hour = current_time.hour();
        let current_minute = current_time.minute();
        // let current_second = current_time.second();
        let test_cases = vec![
            (
                "rfc3339",
                "2017-11-25T22:34:50Z",
                Utc.ymd(2017, 11, 25).and_hms(22, 34, 50),
                Trunc::None,
            ),
            (
                "rfc2822",
                "Wed, 02 Jun 2021 06:31:39 GMT",
                Utc.ymd(2021, 6, 2).and_hms(6, 31, 39),
                Trunc::None,
            ),
            // we currently do not parse dmy format using hyphens,
            // so the following tests are commented out
            // (
            //     "dmy_hms",
            //     "30-04-2021 21:14:10",
            //     Utc.ymd(2021, 4, 30).and_hms(21, 14, 10),
            //     Trunc::None,
            // ),
            // (
            //     "dmy_hms_z",
            //     "25-11-2017 13:31:15 PST",
            //     Utc.ymd(2017, 11, 25).and_hms(21, 31, 15),
            //     Trunc::None,
            // ),
            // (
            //     "dmy",
            //     "21-02-2021",
            //     // Utc.ymd(2021, 2, 21).and_time(Utc::now().time()).unwrap(),
            //     Utc.with_ymd_and_hms(2021, 2, 21, current_hour, current_minute, current_second)
            //         .unwrap(),
            //     Trunc::Seconds,
            // ),
            // (
            //     "dmy_z",
            //     "21-02-2021 PST",
            //     FixedOffset::west(8 * 3600)
            //         .ymd(2021, 2, 21)
            //         .and_time(
            //             Utc::now()
            //                 .with_timezone(&FixedOffset::west(8 * 3600))
            //                 .time(),
            //         )
            //         .unwrap()
            //         .with_timezone(&Utc),
            //     Trunc::Seconds,
            // ),
            // (
            //     "month_dmy",
            //     "21-Feb-2021",
            //     Utc.ymd(2021, 2, 21).and_time(Utc::now().time()).unwrap(),
            //     Trunc::Seconds,
            // ),
            (
                "month_mdy_hms",
                "May 8, 2009 5:57:51 PM",
                Utc.ymd(2009, 5, 8).and_hms(17, 57, 51),
                Trunc::None,
            ),
            (
                "month_mdy_hms_z",
                "May 02, 2021 15:51 UTC",
                Utc.ymd(2021, 5, 2).and_hms(15, 51, 0),
                Trunc::None,
            ),
            (
                "month_mdy",
                "May 25, 2021",
                Utc.ymd(2021, 5, 25).and_time(Utc::now().time()).unwrap(),
                Trunc::Seconds,
            ),
            (
                "month_dmy_hms",
                "14 May 2019 19:11:40.164",
                Utc.ymd(2019, 5, 14).and_hms_milli(19, 11, 40, 164),
                Trunc::None,
            ),
            (
                "month_dmy",
                "1 July 2013",
                Utc.ymd(2013, 7, 1).and_time(Utc::now().time()).unwrap(),
                Trunc::Seconds,
            ),
            (
                "slash_dmy_hms",
                "19/03/2012 10:11:59",
                Utc.ymd(2012, 3, 19).and_hms(10, 11, 59),
                Trunc::None,
            ),
            (
                "slash_dmy",
                "21/08/71",
                Utc.ymd(1971, 8, 21).and_time(Utc::now().time()).unwrap(),
                Trunc::Seconds,
            ),
            (
                "slash_dmy_hms",
                "19/03/2012 10:11:59",
                Utc.ymd(2012, 3, 19).and_hms(10, 11, 59),
                Trunc::None,
            ),
            (
                "slash_dmy",
                "31/3/2014",
                Utc.ymd(2014, 3, 31).and_time(Utc::now().time()).unwrap(),
                Trunc::Seconds,
            ),
        ];

        for &(test, input, want, trunc) in test_cases.iter() {
            match trunc {
                Trunc::None => {
                    assert_eq!(
                        super::parse_with_preference_and_timezone(input, true, &Utc).unwrap(),
                        want,
                        "parse_with_preference_and_timezone_in_utc/{}/{}",
                        test,
                        input
                    )
                }
                Trunc::Seconds => assert_eq!(
                    super::parse_with_preference_and_timezone(input, true, &Utc)
                        .unwrap()
                        .trunc_subsecs(0)
                        .with_hour(current_hour)
                        .unwrap()
                        .with_minute(current_minute)
                        .unwrap()
                        .with_second(0)
                        .unwrap(),
                    want.trunc_subsecs(0).with_second(0).unwrap(),
                    "parse_with_preference_and_timezone_in_utc/{}/{}",
                    test,
                    input
                ),
            };
        }
    }

    #[test]
    fn parse_unambiguous_dmy() {
        assert_eq!(
            super::parse("31/3/22").unwrap().date(),
            Utc.ymd(2022, 3, 31)
        );
        assert_eq!(
            super::parse_with_preference("3/31/22", true)
                .unwrap()
                .date(),
            Utc.ymd(2022, 3, 31)
        );
        assert_eq!(
            super::parse_with_preference("31/07/2021", true)
                .unwrap()
                .date(),
            Utc.ymd(2021, 7, 31)
        );
    }
}

#![allow(deprecated)]
use crate::timezone;
use anyhow::{Result, anyhow};
use chrono::prelude::*;
use regex::Regex;

macro_rules! regex {
    ($re:literal $(,)?) => {{
        static RE: std::sync::OnceLock<regex::Regex> = std::sync::OnceLock::new();
        RE.get_or_init(|| unsafe {
            regex::RegexBuilder::new($re)
                .unicode(false)
                .build()
                .unwrap_unchecked()
        })
    }};
}
/// Parse struct has methods implemented parsers for accepted formats.
pub struct Parse<'z, Tz2> {
    tz: &'z Tz2,
    default_time: NaiveTime,
    prefer_dmy: bool,
}

impl<'z, Tz2> Parse<'z, Tz2>
where
    Tz2: TimeZone,
{
    /// Create a new instance of [`Parse`] with a custom parsing timezone that handles the
    /// datetime string without time offset.
    pub const fn new(tz: &'z Tz2, default_time: NaiveTime) -> Self {
        Self {
            tz,
            default_time,
            prefer_dmy: false,
        }
    }

    pub const fn prefer_dmy(&mut self, yes: bool) -> &Self {
        self.prefer_dmy = yes;
        self
    }

    /// Create a new instance of [`Parse`] with a custom parsing timezone that handles the
    /// datetime string without time offset, and the date parsing preference.
    pub const fn new_with_preference(
        tz: &'z Tz2,
        default_time: NaiveTime,
        prefer_dmy: bool,
    ) -> Self {
        Self {
            tz,
            default_time,
            prefer_dmy,
        }
    }

    /// This method tries to parse the input datetime string with a list of accepted formats. See
    /// more examples from [`Parse`], [`crate::parse()`] and [`crate::parse_with_timezone()`].
    #[inline]
    pub fn parse(&self, input: &str) -> Result<DateTime<Utc>> {
        self.rfc2822(input)
            .or_else(|| self.unix_timestamp(input))
            .or_else(|| self.slash_mdy_family(input))
            .or_else(|| self.slash_ymd_family(input))
            .or_else(|| self.ymd_family(input))
            .or_else(|| self.month_ymd(input))
            .or_else(|| self.month_mdy_family(input))
            .or_else(|| self.month_dmy_family(input))
            .unwrap_or_else(|| Err(anyhow!("{} did not match any formats.", input)))
    }

    #[inline]
    fn ymd_family(&self, input: &str) -> Option<Result<DateTime<Utc>>> {
        let re: &Regex = regex! {
            r"^\d{4}-\d{2}"

        };

        if !re.is_match(input) {
            return None;
        }
        self.rfc3339(input)
            .or_else(|| self.ymd_hms(input))
            .or_else(|| self.ymd_hms_z(input))
            .or_else(|| self.ymd(input))
            .or_else(|| self.ymd_z(input))
    }

    #[inline]
    fn month_mdy_family(&self, input: &str) -> Option<Result<DateTime<Utc>>> {
        let re: &Regex = regex! {
            r"^[a-zA-Z]{3,9}\.?\s+\d{1,2}"
        };

        if !re.is_match(input) {
            return None;
        }
        self.month_mdy_hms(input)
            .or_else(|| self.month_mdy_hms_z(input))
            .or_else(|| self.month_mdy(input))
    }

    #[inline]
    fn month_dmy_family(&self, input: &str) -> Option<Result<DateTime<Utc>>> {
        let re: &Regex = regex! {r"^\d{1,2}\s+[a-zA-Z]{3,9}"
        };

        if !re.is_match(input) {
            return None;
        }
        self.month_dmy_hms(input).or_else(|| self.month_dmy(input))
    }

    #[inline]
    fn slash_mdy_family(&self, input: &str) -> Option<Result<DateTime<Utc>>> {
        let re: &Regex = regex! {r"^\d{1,2}/\d{1,2}"
        };
        if !re.is_match(input) {
            return None;
        }
        if self.prefer_dmy {
            self.slash_dmy_hms(input)
                .or_else(|| self.slash_dmy(input))
                .or_else(|| self.slash_mdy_hms(input))
                .or_else(|| self.slash_mdy(input))
        } else {
            self.slash_mdy_hms(input)
                .or_else(|| self.slash_mdy(input))
                .or_else(|| self.slash_dmy_hms(input))
                .or_else(|| self.slash_dmy(input))
        }
    }

    #[inline]
    fn slash_ymd_family(&self, input: &str) -> Option<Result<DateTime<Utc>>> {
        let re: &Regex = regex! {r"^[0-9]{4}/[0-9]{1,2}"};
        if !re.is_match(input) {
            return None;
        }
        self.slash_ymd_hms(input).or_else(|| self.slash_ymd(input))
    }

    // unix timestamp
    // - 0
    // - -770172300
    // - 1671673426.123456789
    #[inline]
    fn unix_timestamp(&self, input: &str) -> Option<Result<DateTime<Utc>>> {
        let ts_sec_val: f64 = if let Ok(val) = fast_float2::parse(input) {
            val
        } else {
            return None;
        };

        // convert the timestamp seconds value to nanoseconds
        let ts_ns_val = ts_sec_val * 1_000_000_000_f64;

        let result = Utc.timestamp_nanos(ts_ns_val as i64).with_timezone(&Utc);
        Some(Ok(result))
    }

    // rfc3339
    // - 2021-05-01T01:17:02.604456Z
    // - 2017-11-25T22:34:50Z
    #[inline]
    fn rfc3339(&self, input: &str) -> Option<Result<DateTime<Utc>>> {
        DateTime::parse_from_rfc3339(input)
            .ok()
            .map(|parsed| parsed.with_timezone(&Utc))
            .map(Ok)
    }

    // rfc2822
    // - Wed, 02 Jun 2021 06:31:39 GMT
    #[inline]
    fn rfc2822(&self, input: &str) -> Option<Result<DateTime<Utc>>> {
        DateTime::parse_from_rfc2822(input)
            .ok()
            .map(|parsed| parsed.with_timezone(&Utc))
            .map(Ok)
    }

    // yyyy-mm-dd hh:mm:ss
    // - 2014-04-26 05:24:37 PM
    // - 2021-04-30 21:14
    // - 2021-04-30 21:14:10
    // - 2021-04-30 21:14:10.052282
    // - 2014-04-26 17:24:37.123
    // - 2014-04-26 17:24:37.3186369
    // - 2012-08-03 18:31:59.257000000
    #[inline]
    fn ymd_hms(&self, input: &str) -> Option<Result<DateTime<Utc>>> {
        let re: &Regex = regex! {
                r"^\d{4}-\d{2}-\d{2}\s+\d{2}:\d{2}(:\d{2})?(\.\d{1,9})?\s*(am|pm|AM|PM)?$"

        };
        if !re.is_match(input) {
            return None;
        }

        self.tz
            .datetime_from_str(input, "%Y-%m-%d %H:%M:%S")
            .or_else(|_| self.tz.datetime_from_str(input, "%Y-%m-%d %H:%M"))
            .or_else(|_| self.tz.datetime_from_str(input, "%Y-%m-%d %H:%M:%S%.f"))
            .or_else(|_| self.tz.datetime_from_str(input, "%Y-%m-%d %I:%M:%S %P"))
            .or_else(|_| self.tz.datetime_from_str(input, "%Y-%m-%d %I:%M %P"))
            .ok()
            .map(|parsed| parsed.with_timezone(&Utc))
            .map(Ok)
    }

    // yyyy-mm-dd hh:mm:ss z
    // - 2017-11-25 13:31:15 PST
    // - 2017-11-25 13:31 PST
    // - 2014-12-16 06:20:00 UTC
    // - 2014-12-16 06:20:00 GMT
    // - 2014-04-26 13:13:43 +0800
    // - 2014-04-26 13:13:44 +09:00
    // - 2012-08-03 18:31:59.257000000 +0000
    // - 2015-09-30 18:48:56.35272715 UTC
    #[inline]
    fn ymd_hms_z(&self, input: &str) -> Option<Result<DateTime<Utc>>> {
        // Fast pre-filter: bare dates "YYYY-MM-DD" are 10 chars; valid inputs need space + time
        if input.len() < 17 || !input.as_bytes()[10].is_ascii_whitespace() {
            return None;
        }
        let re: &Regex = regex! {
                r"^\d{4}-\d{2}-\d{2}\s+\d{2}:\d{2}(:\d{2})?(\.\d{1,9})?(?P<tz>\s*[+-:a-zA-Z0-9]{3,6})$"
        };

        if let Some(caps) = re.captures(input)
            && let Some(matched_tz) = caps.name("tz")
        {
            let parse_from_str = NaiveDateTime::parse_from_str;
            return match timezone::parse(matched_tz.as_str().trim()) {
                Ok(offset) => parse_from_str(input, "%Y-%m-%d %H:%M:%S %Z")
                    .or_else(|_| parse_from_str(input, "%Y-%m-%d %H:%M %Z"))
                    .or_else(|_| parse_from_str(input, "%Y-%m-%d %H:%M:%S%.f %Z"))
                    .ok()
                    .and_then(|parsed| offset.from_local_datetime(&parsed).single())
                    .map(|datetime| datetime.with_timezone(&Utc))
                    .map(Ok),
                Err(err) => Some(Err(err)),
            };
        }
        None
    }

    // yyyy-mm-dd
    // - 2021-02-21
    #[inline]
    fn ymd(&self, input: &str) -> Option<Result<DateTime<Utc>>> {
        let re: &Regex = regex! {r"^\d{4}-\d{2}-\d{2}$"
        };

        if !re.is_match(input) {
            return None;
        }
        let now = Utc::now()
            .date()
            .and_time(self.default_time)?
            .with_timezone(self.tz);
        NaiveDate::parse_from_str(input, "%Y-%m-%d")
            .ok()
            .map(|parsed| parsed.and_time(now.time()))
            .and_then(|datetime| self.tz.from_local_datetime(&datetime).single())
            .map(|at_tz| at_tz.with_timezone(&Utc))
            .map(Ok)
    }

    // yyyy-mm-dd z
    // - 2021-02-21 PST
    // - 2021-02-21 UTC
    // - 2020-07-20+08:00 (yyyy-mm-dd-07:00)
    #[inline]
    fn ymd_z(&self, input: &str) -> Option<Result<DateTime<Utc>>> {
        // Fast pre-filter: bare date "YYYY-MM-DD" is exactly 10 chars; timezone appended = longer
        if input.len() <= 10 {
            return None;
        }
        let re: &Regex = regex! {r"^\d{4}-\d{2}-\d{2}(?P<tz>\s*[+-:a-zA-Z0-9]{3,6})$"
        };
        if let Some(caps) = re.captures(input)
            && let Some(matched_tz) = caps.name("tz")
        {
            return match timezone::parse(matched_tz.as_str().trim()) {
                Ok(offset) => {
                    let now = Utc::now()
                        .date()
                        .and_time(self.default_time)?
                        .with_timezone(&offset);
                    NaiveDate::parse_from_str(input, "%Y-%m-%d %Z")
                        .ok()
                        .map(|parsed| parsed.and_time(now.time()))
                        .and_then(|datetime| offset.from_local_datetime(&datetime).single())
                        .map(|at_tz| at_tz.with_timezone(&Utc))
                        .map(Ok)
                }
                Err(err) => Some(Err(err)),
            };
        }
        None
    }

    // yyyy-mon-dd
    // - 2021-Feb-21
    #[inline]
    fn month_ymd(&self, input: &str) -> Option<Result<DateTime<Utc>>> {
        let re: &Regex = regex! {r"^\d{4}-\w{3,9}-\d{2}$"
        };
        if !re.is_match(input) {
            return None;
        }

        let now = Utc::now()
            .date()
            .and_time(self.default_time)?
            .with_timezone(self.tz);
        NaiveDate::parse_from_str(input, "%Y-%m-%d")
            .or_else(|_| NaiveDate::parse_from_str(input, "%Y-%b-%d"))
            .ok()
            .map(|parsed| parsed.and_time(now.time()))
            .and_then(|datetime| self.tz.from_local_datetime(&datetime).single())
            .map(|at_tz| at_tz.with_timezone(&Utc))
            .map(Ok)
    }

    // Mon dd, yyyy, hh:mm:ss
    // - May 8, 2009 5:57:51 PM
    // - September 17, 2012 10:09am
    // - September 17, 2012, 10:10:09
    #[inline]
    fn month_mdy_hms(&self, input: &str) -> Option<Result<DateTime<Utc>>> {
        let re: &Regex = regex! {
                r"^[a-zA-Z]{3,9}\.?\s+\d{1,2},\s+\d{2,4},?\s+\d{1,2}:\d{2}(:\d{2})?\s*(am|pm|AM|PM)?$"
        };
        if !re.is_match(input) {
            return None;
        }

        // The regex above enforces \s+ after any comma or period, so removing bare ',' or '.'
        // is equivalent to the previous `replace(", ", " ").replace(". ", " ")` for all
        // inputs that reach this point — marginally-malformed inputs (e.g. "May 27,2012 …")
        // still fail to parse after stripping because the digits run together.
        let dt = input.replace([',', '.'], "");
        self.tz
            .datetime_from_str(&dt, "%B %d %Y %H:%M:%S")
            .or_else(|_| self.tz.datetime_from_str(&dt, "%B %d %Y %H:%M"))
            .or_else(|_| self.tz.datetime_from_str(&dt, "%B %d %Y %I:%M:%S %P"))
            .or_else(|_| self.tz.datetime_from_str(&dt, "%B %d %Y %I:%M %P"))
            .ok()
            .map(|at_tz| at_tz.with_timezone(&Utc))
            .map(Ok)
    }

    // Mon dd, yyyy hh:mm:ss z
    // - May 02, 2021 15:51:31 UTC
    // - May 02, 2021 15:51 UTC
    // - May 26, 2021, 12:49 AM PDT
    // - September 17, 2012 at 10:09am PST
    #[inline]
    fn month_mdy_hms_z(&self, input: &str) -> Option<Result<DateTime<Utc>>> {
        // Fast pre-filter: must contain an isolated 4-digit year — eliminates "May 27 02:45:27".
        // Skip the O(n) scan entirely for inputs too short to hold a valid month+day+year+time+tz.
        if input.len() < 20 {
            return None;
        }
        let bytes = input.as_bytes();
        let has_year = (0..bytes.len().saturating_sub(3)).any(|i| {
            bytes[i..i + 4].iter().all(|b| b.is_ascii_digit())
                && (i == 0 || !bytes[i - 1].is_ascii_digit())
                && bytes.get(i + 4).is_none_or(|b| !b.is_ascii_digit())
        });
        if !has_year {
            return None;
        }
        let re: &Regex = regex! {
                r"^[a-zA-Z]{3,9}\s+\d{1,2},?\s+\d{4}\s*,?(at)?\s+\d{2}:\d{2}(:\d{2})?\s*(am|pm|AM|PM)?(?P<tz>\s+[+-:a-zA-Z0-9]{3,6})$",
        };
        if let Some(caps) = re.captures(input)
            && let Some(matched_tz) = caps.name("tz")
        {
            let parse_from_str = NaiveDateTime::parse_from_str;
            return match timezone::parse(matched_tz.as_str().trim()) {
                Ok(offset) => {
                    let mut dt = input.replace(',', "");
                    if let Some(pos) = dt.find("at") {
                        dt.replace_range(pos..pos + 2, "");
                    }
                    parse_from_str(&dt, "%B %d %Y %H:%M:%S %Z")
                        .or_else(|_| parse_from_str(&dt, "%B %d %Y %H:%M %Z"))
                        .or_else(|_| parse_from_str(&dt, "%B %d %Y %I:%M:%S %P %Z"))
                        .or_else(|_| parse_from_str(&dt, "%B %d %Y %I:%M %P %Z"))
                        .ok()
                        .and_then(|parsed| offset.from_local_datetime(&parsed).single())
                        .map(|datetime| datetime.with_timezone(&Utc))
                        .map(Ok)
                }
                Err(err) => Some(Err(err)),
            };
        }
        None
    }

    // Mon dd, yyyy
    // - May 25, 2021
    // - oct 7, 1970
    // - oct 7, 70
    // - oct. 7, 1970
    // - oct. 7, 70
    // - October 7, 1970
    #[inline]
    fn month_mdy(&self, input: &str) -> Option<Result<DateTime<Utc>>> {
        let re: &Regex = regex! {r"^[a-zA-Z]{3,9}\.?\s+\d{1,2},\s+\d{2,4}$"
        };
        if !re.is_match(input) {
            return None;
        }

        let now = Utc::now()
            .date()
            .and_time(self.default_time)?
            .with_timezone(self.tz);
        // The regex above enforces \s+ after any comma or period, so removing bare ',' or '.'
        // is equivalent to the previous `replace(", ", " ").replace(". ", " ")` for all
        // inputs that reach this point.
        let dt = input.replace([',', '.'], "");
        NaiveDate::parse_from_str(&dt, "%B %d %y")
            .or_else(|_| NaiveDate::parse_from_str(&dt, "%B %d %Y"))
            .ok()
            .map(|parsed| parsed.and_time(now.time()))
            .and_then(|datetime| self.tz.from_local_datetime(&datetime).single())
            .map(|at_tz| at_tz.with_timezone(&Utc))
            .map(Ok)
    }

    // dd Mon yyyy hh:mm:ss
    // - 12 Feb 2006, 19:17
    // - 12 Feb 2006 19:17
    // - 14 May 2019 19:11:40.164
    #[inline]
    fn month_dmy_hms(&self, input: &str) -> Option<Result<DateTime<Utc>>> {
        // Fast pre-filter: time component always contains ':', skip regex for date-only inputs.
        if !input.as_bytes().contains(&b':') {
            return None;
        }
        let re: &Regex = regex! {
                r"^\d{1,2}\s+[a-zA-Z]{3,9}\s+\d{2,4},?\s+\d{1,2}:[0-9]{2}(:[0-9]{2})?(\.[0-9]{1,9})?$"
        };
        if !re.is_match(input) {
            return None;
        }

        let dt = input.replace(',', "");
        self.tz
            .datetime_from_str(&dt, "%d %B %Y %H:%M:%S")
            .or_else(|_| self.tz.datetime_from_str(&dt, "%d %B %Y %H:%M"))
            .or_else(|_| self.tz.datetime_from_str(&dt, "%d %B %Y %H:%M:%S%.f"))
            .or_else(|_| self.tz.datetime_from_str(&dt, "%d %B %Y %I:%M:%S %P"))
            .or_else(|_| self.tz.datetime_from_str(&dt, "%d %B %Y %I:%M %P"))
            .ok()
            .map(|at_tz| at_tz.with_timezone(&Utc))
            .map(Ok)
    }

    // dd Mon yyyy
    // - 7 oct 70
    // - 7 oct 1970
    // - 03 February 2013
    // - 1 July 2013
    #[inline]
    fn month_dmy(&self, input: &str) -> Option<Result<DateTime<Utc>>> {
        let re: &Regex = regex! {r"^\d{1,2}\s+[a-zA-Z]{3,9}\s+\d{2,4}$"
        };
        if !re.is_match(input) {
            return None;
        }

        let now = Utc::now()
            .date()
            .and_time(self.default_time)?
            .with_timezone(self.tz);
        // Fast path: if the last 4 bytes are all digits and preceded by a space, it's a
        // 4-digit year — skip the always-failing %d %B %y (2-digit year) attempt.
        let bytes = input.as_bytes();
        let len = bytes.len();
        let four_digit_year = len >= 5
            && bytes[len - 4..].iter().all(|b| b.is_ascii_digit())
            && bytes[len - 5].is_ascii_whitespace();
        let parsed = if four_digit_year {
            NaiveDate::parse_from_str(input, "%d %B %Y")
        } else {
            NaiveDate::parse_from_str(input, "%d %B %y")
                .or_else(|_| NaiveDate::parse_from_str(input, "%d %B %Y"))
        };
        parsed
            .ok()
            .map(|parsed| parsed.and_time(now.time()))
            .and_then(|datetime| self.tz.from_local_datetime(&datetime).single())
            .map(|at_tz| at_tz.with_timezone(&Utc))
            .map(Ok)
    }

    // mm/dd/yyyy hh:mm:ss
    // - 4/8/2014 22:05
    // - 04/08/2014 22:05
    // - 4/8/14 22:05
    // - 04/2/2014 03:00:51
    // - 8/8/1965 12:00:00 AM
    // - 8/8/1965 01:00:01 PM
    // - 8/8/1965 01:00 PM
    // - 8/8/1965 1:00 PM
    // - 8/8/1965 12:00 AM
    // - 4/02/2014 03:00:51
    // - 03/19/2012 10:11:59
    // - 03/19/2012 10:11:59.3186369
    #[inline]
    fn slash_mdy_hms(&self, input: &str) -> Option<Result<DateTime<Utc>>> {
        let re: &Regex = regex! {
                r"^\d{1,2}/\d{1,2}/\d{2,4}\s+\d{1,2}:\d{2}(:\d{2})?(\.\d{1,9})?\s*(am|pm|AM|PM)?$"
        };
        if !re.is_match(input) {
            return None;
        }

        self.tz
            .datetime_from_str(input, "%m/%d/%y %H:%M:%S")
            .or_else(|_| self.tz.datetime_from_str(input, "%m/%d/%y %H:%M"))
            .or_else(|_| self.tz.datetime_from_str(input, "%m/%d/%y %H:%M:%S%.f"))
            .or_else(|_| self.tz.datetime_from_str(input, "%m/%d/%y %I:%M:%S %P"))
            .or_else(|_| self.tz.datetime_from_str(input, "%m/%d/%y %I:%M %P"))
            .or_else(|_| self.tz.datetime_from_str(input, "%m/%d/%Y %H:%M:%S"))
            .or_else(|_| self.tz.datetime_from_str(input, "%m/%d/%Y %H:%M"))
            .or_else(|_| self.tz.datetime_from_str(input, "%m/%d/%Y %H:%M:%S%.f"))
            .or_else(|_| self.tz.datetime_from_str(input, "%m/%d/%Y %I:%M:%S %P"))
            .or_else(|_| self.tz.datetime_from_str(input, "%m/%d/%Y %I:%M %P"))
            .ok()
            .map(|at_tz| at_tz.with_timezone(&Utc))
            .map(Ok)
    }

    // dd/mm/yyyy hh:mm:ss
    // - 8/4/2014 22:05
    // - 08/04/2014 22:05
    // - 8/4/14 22:05
    // - 2/04/2014 03:00:51
    // - 8/8/1965 12:00:00 AM
    // - 8/8/1965 01:00:01 PM
    // - 8/8/1965 01:00 PM
    // - 8/8/1965 1:00 PM
    // - 8/8/1965 12:00 AM
    // - 02/4/2014 03:00:51
    // - 19/03/2012 10:11:59
    // - 19/03/2012 10:11:59.3186369
    #[inline]
    fn slash_dmy_hms(&self, input: &str) -> Option<Result<DateTime<Utc>>> {
        let re: &Regex = regex! {
                r"^\d{1,2}/\d{1,2}/\d{2,4}\s+\d{1,2}:\d{2}(:\d{2})?(\.\d{1,9})?\s*(am|pm|AM|PM)?$"
        };
        if !re.is_match(input) {
            return None;
        }

        self.tz
            .datetime_from_str(input, "%d/%m/%y %H:%M:%S")
            .or_else(|_| self.tz.datetime_from_str(input, "%d/%m/%y %H:%M"))
            .or_else(|_| self.tz.datetime_from_str(input, "%d/%m/%y %H:%M:%S%.f"))
            .or_else(|_| self.tz.datetime_from_str(input, "%d/%m/%y %I:%M:%S %P"))
            .or_else(|_| self.tz.datetime_from_str(input, "%d/%m/%y %I:%M %P"))
            .or_else(|_| self.tz.datetime_from_str(input, "%d/%m/%Y %H:%M:%S"))
            .or_else(|_| self.tz.datetime_from_str(input, "%d/%m/%Y %H:%M"))
            .or_else(|_| self.tz.datetime_from_str(input, "%d/%m/%Y %H:%M:%S%.f"))
            .or_else(|_| self.tz.datetime_from_str(input, "%d/%m/%Y %I:%M:%S %P"))
            .or_else(|_| self.tz.datetime_from_str(input, "%d/%m/%Y %I:%M %P"))
            .ok()
            .map(|at_tz| at_tz.with_timezone(&Utc))
            .map(Ok)
    }

    // mm/dd/yyyy
    // - 3/31/2014
    // - 03/31/2014
    // - 08/21/71
    // - 8/1/71
    #[inline]
    fn slash_mdy(&self, input: &str) -> Option<Result<DateTime<Utc>>> {
        let re: &Regex = regex! {r"^\d{1,2}/\d{1,2}/\d{2,4}$"
        };
        if !re.is_match(input) {
            return None;
        }

        let now = Utc::now()
            .date()
            .and_time(self.default_time)?
            .with_timezone(self.tz);
        NaiveDate::parse_from_str(input, "%m/%d/%y")
            .or_else(|_| NaiveDate::parse_from_str(input, "%m/%d/%Y"))
            .ok()
            .map(|parsed| parsed.and_time(now.time()))
            .and_then(|datetime| self.tz.from_local_datetime(&datetime).single())
            .map(|at_tz| at_tz.with_timezone(&Utc))
            .map(Ok)
    }

    // dd/mm/yyyy
    // - 31/3/2014
    // - 31/03/2014
    // - 21/08/71
    // - 1/8/71
    #[inline]
    fn slash_dmy(&self, input: &str) -> Option<Result<DateTime<Utc>>> {
        let re: &Regex = regex! {r"^[0-9]{1,2}/[0-9]{1,2}/[0-9]{2,4}$"
        };
        if !re.is_match(input) {
            return None;
        }

        let now = Utc::now()
            .date()
            .and_time(self.default_time)?
            .with_timezone(self.tz);
        NaiveDate::parse_from_str(input, "%d/%m/%y")
            .or_else(|_| NaiveDate::parse_from_str(input, "%d/%m/%Y"))
            .ok()
            .map(|parsed| parsed.and_time(now.time()))
            .and_then(|datetime| self.tz.from_local_datetime(&datetime).single())
            .map(|at_tz| at_tz.with_timezone(&Utc))
            .map(Ok)
    }

    // yyyy/mm/dd hh:mm:ss
    // - 2014/4/8 22:05
    // - 2014/04/08 22:05
    // - 2014/04/2 03:00:51
    // - 2014/4/02 03:00:51
    // - 2012/03/19 10:11:59
    // - 2012/03/19 10:11:59.3186369
    #[inline]
    fn slash_ymd_hms(&self, input: &str) -> Option<Result<DateTime<Utc>>> {
        let re: &Regex = regex! {
                r"^[0-9]{4}/[0-9]{1,2}/[0-9]{1,2}\s+[0-9]{1,2}:[0-9]{2}(:[0-9]{2})?(\.[0-9]{1,9})?\s*(am|pm|AM|PM)?$"
        };
        if !re.is_match(input) {
            return None;
        }

        self.tz
            .datetime_from_str(input, "%Y/%m/%d %H:%M:%S")
            .or_else(|_| self.tz.datetime_from_str(input, "%Y/%m/%d %H:%M"))
            .or_else(|_| self.tz.datetime_from_str(input, "%Y/%m/%d %H:%M:%S%.f"))
            .or_else(|_| self.tz.datetime_from_str(input, "%Y/%m/%d %I:%M:%S %P"))
            .or_else(|_| self.tz.datetime_from_str(input, "%Y/%m/%d %I:%M %P"))
            .ok()
            .map(|at_tz| at_tz.with_timezone(&Utc))
            .map(Ok)
    }

    // yyyy/mm/dd
    // - 2014/3/31
    // - 2014/03/31
    #[inline]
    fn slash_ymd(&self, input: &str) -> Option<Result<DateTime<Utc>>> {
        let re: &Regex = regex! {r"^[0-9]{4}/[0-9]{1,2}/[0-9]{1,2}$"
        };
        if !re.is_match(input) {
            return None;
        }

        let now = Utc::now()
            .date()
            .and_time(self.default_time)?
            .with_timezone(self.tz);
        NaiveDate::parse_from_str(input, "%Y/%m/%d")
            .ok()
            .map(|parsed| parsed.and_time(now.time()))
            .and_then(|datetime| self.tz.from_local_datetime(&datetime).single())
            .map(|at_tz| at_tz.with_timezone(&Utc))
            .map(Ok)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unix_timestamp() {
        let parse = Parse::new(&Utc, Utc::now().time());

        let test_cases = vec![
            ("0", Utc.ymd(1970, 1, 1).and_hms(0, 0, 0)),
            ("0000000000", Utc.ymd(1970, 1, 1).and_hms(0, 0, 0)),
            ("0000000000000", Utc.ymd(1970, 1, 1).and_hms(0, 0, 0)),
            ("0000000000000000000", Utc.ymd(1970, 1, 1).and_hms(0, 0, 0)),
            ("-770172300", Utc.ymd(1945, 8, 5).and_hms(23, 15, 0)),
            (
                "1671673426.123456789",
                Utc.ymd(2022, 12, 22).and_hms_nano(1, 43, 46, 123456768),
            ),
            ("1511648546", Utc.ymd(2017, 11, 25).and_hms(22, 22, 26)),
            (
                "1620036248.420",
                Utc.ymd(2021, 5, 3).and_hms_milli(10, 4, 8, 420),
            ),
            (
                "1620036248.717915136",
                Utc.ymd(2021, 5, 3).and_hms_nano(10, 4, 8, 717915136),
            ),
        ];

        for &(input, want) in test_cases.iter() {
            assert_eq!(
                parse.unix_timestamp(input).unwrap().unwrap(),
                want,
                "unix_timestamp/{}",
                input
            )
        }
        assert!(parse.unix_timestamp("15116").is_some());
        assert!(
            parse
                .unix_timestamp("16200248727179150001620024872717915000") //DevSkim: ignore DS173237
                .is_some()
        );
        assert!(parse.unix_timestamp("not-a-ts").is_none());
    }

    #[test]
    fn rfc3339() {
        let parse = Parse::new(&Utc, Utc::now().time());

        let test_cases = [
            (
                "2021-05-01T01:17:02.604456Z",
                Utc.ymd(2021, 5, 1).and_hms_nano(1, 17, 2, 604456000),
            ),
            (
                "2017-11-25T22:34:50Z",
                Utc.ymd(2017, 11, 25).and_hms(22, 34, 50),
            ),
        ];

        for &(input, want) in test_cases.iter() {
            assert_eq!(
                parse.rfc3339(input).unwrap().unwrap(),
                want,
                "rfc3339/{}",
                input
            )
        }
        assert!(parse.rfc3339("2017-11-25 22:34:50").is_none());
        assert!(parse.rfc3339("not-date-time").is_none());
    }

    #[test]
    fn rfc2822() {
        let parse = Parse::new(&Utc, Utc::now().time());

        let test_cases = [
            (
                "Wed, 02 Jun 2021 06:31:39 GMT",
                Utc.ymd(2021, 6, 2).and_hms(6, 31, 39),
            ),
            (
                "Wed, 02 Jun 2021 06:31:39 PDT",
                Utc.ymd(2021, 6, 2).and_hms(13, 31, 39),
            ),
        ];

        for &(input, want) in test_cases.iter() {
            assert_eq!(
                parse.rfc2822(input).unwrap().unwrap(),
                want,
                "rfc2822/{}",
                input
            )
        }
        assert!(parse.rfc2822("02 Jun 2021 06:31:39").is_none());
        assert!(parse.rfc2822("not-date-time").is_none());
    }

    #[test]
    fn ymd_hms() {
        let parse = Parse::new(&Utc, Utc::now().time());

        let test_cases = vec![
            ("2021-04-30 21:14", Utc.ymd(2021, 4, 30).and_hms(21, 14, 0)),
            (
                "2021-04-30 21:14:10",
                Utc.ymd(2021, 4, 30).and_hms(21, 14, 10),
            ),
            (
                "2021-04-30 21:14:10.052282",
                Utc.ymd(2021, 4, 30).and_hms_micro(21, 14, 10, 52282),
            ),
            (
                "2014-04-26 05:24:37 PM",
                Utc.ymd(2014, 4, 26).and_hms(17, 24, 37),
            ),
            (
                "2014-04-26 17:24:37.123",
                Utc.ymd(2014, 4, 26).and_hms_milli(17, 24, 37, 123),
            ),
            (
                "2014-04-26 17:24:37.3186369",
                Utc.ymd(2014, 4, 26).and_hms_nano(17, 24, 37, 318636900),
            ),
            (
                "2012-08-03 18:31:59.257000000",
                Utc.ymd(2012, 8, 3).and_hms_nano(18, 31, 59, 257000000),
            ),
        ];

        for &(input, want) in test_cases.iter() {
            assert_eq!(
                parse.ymd_hms(input).unwrap().unwrap(),
                want,
                "ymd_hms/{}",
                input
            )
        }
        assert!(parse.ymd_hms("not-date-time").is_none());
    }

    #[test]
    fn ymd_hms_z() {
        let parse = Parse::new(&Utc, Utc::now().time());

        let test_cases = vec![
            (
                "2017-11-25 13:31:15 PST",
                Utc.ymd(2017, 11, 25).and_hms(21, 31, 15),
            ),
            (
                "2017-11-25 13:31 PST",
                Utc.ymd(2017, 11, 25).and_hms(21, 31, 0),
            ),
            (
                "2014-12-16 06:20:00 UTC",
                Utc.ymd(2014, 12, 16).and_hms(6, 20, 0),
            ),
            (
                "2014-12-16 06:20:00 GMT",
                Utc.ymd(2014, 12, 16).and_hms(6, 20, 0),
            ),
            (
                "2014-04-26 13:13:43 +0800",
                Utc.ymd(2014, 4, 26).and_hms(5, 13, 43),
            ),
            (
                "2014-04-26 13:13:44 +09:00",
                Utc.ymd(2014, 4, 26).and_hms(4, 13, 44),
            ),
            (
                "2012-08-03 18:31:59.257000000 +0000",
                Utc.ymd(2012, 8, 3).and_hms_nano(18, 31, 59, 257000000),
            ),
            (
                "2015-09-30 18:48:56.35272715 UTC",
                Utc.ymd(2015, 9, 30).and_hms_nano(18, 48, 56, 352727150),
            ),
        ];

        for &(input, want) in test_cases.iter() {
            assert_eq!(
                parse.ymd_hms_z(input).unwrap().unwrap(),
                want,
                "ymd_hms_z/{}",
                input
            )
        }
        assert!(parse.ymd_hms_z("not-date-time").is_none());
        // Pre-filter boundary: exactly 16 chars is rejected by length guard (< 17)
        assert!(parse.ymd_hms_z("2021-04-30 21:14").is_none()); // 16 chars, rejected by length guard
        // 17 chars but byte[10] is not whitespace — rejected by whitespace check
        assert!(parse.ymd_hms_z("2021-04-30X21:14Z").is_none()); // 17 chars, byte[10]='X' not space
        // 17 chars with whitespace at byte[10] proceeds to regex but regex rejects malformed input
        assert!(parse.ymd_hms_z("2021-04-30 21:1XZ").is_none()); // 17 chars, byte[10]=' ', regex rejects
    }

    #[test]
    fn ymd() {
        let parse = Parse::new(&Utc, Utc::now().time());

        let test_cases = [(
            "2021-02-21",
            Utc.ymd(2021, 2, 21).and_time(Utc::now().time()),
        )];

        for &(input, want) in test_cases.iter() {
            assert_eq!(
                parse
                    .ymd(input)
                    .unwrap()
                    .unwrap()
                    .trunc_subsecs(0)
                    .with_second(0)
                    .unwrap(),
                want.unwrap().trunc_subsecs(0).with_second(0).unwrap(),
                "ymd/{}",
                input
            )
        }
        assert!(parse.ymd("not-date-time").is_none());
    }

    #[test]
    fn ymd_z() {
        let parse = Parse::new(&Utc, Utc::now().time());
        let now_at_pst = Utc::now().with_timezone(&FixedOffset::west(8 * 3600));
        let now_at_cst = Utc::now().with_timezone(&FixedOffset::east(8 * 3600));

        let test_cases = [
            (
                "2021-02-21 PST",
                FixedOffset::west(8 * 3600)
                    .ymd(2021, 2, 21)
                    .and_time(now_at_pst.time())
                    .map(|dt| dt.with_timezone(&Utc)),
            ),
            (
                "2021-02-21 UTC",
                FixedOffset::west(0)
                    .ymd(2021, 2, 21)
                    .and_time(Utc::now().time())
                    .map(|dt| dt.with_timezone(&Utc)),
            ),
            (
                "2020-07-20+08:00",
                FixedOffset::east(8 * 3600)
                    .ymd(2020, 7, 20)
                    .and_time(now_at_cst.time())
                    .map(|dt| dt.with_timezone(&Utc)),
            ),
        ];

        for &(input, want) in test_cases.iter() {
            assert_eq!(
                parse
                    .ymd_z(input)
                    .unwrap()
                    .unwrap()
                    .trunc_subsecs(0)
                    .with_second(0)
                    .unwrap(),
                want.unwrap().trunc_subsecs(0).with_second(0).unwrap(),
                "ymd_z/{}",
                input
            )
        }
        assert!(parse.ymd_z("not-date-time").is_none());
        // Pre-filter boundary: exactly 10 chars (bare date) is rejected (<= 10 guard), 11+ proceeds
        assert!(parse.ymd_z("2021-02-21").is_none()); // exactly 10 chars, rejected
        assert!(parse.ymd_z("2021-02-21X").is_none()); // 11 chars, proceeds to regex but regex rejects
    }

    #[test]
    fn month_ymd() {
        let parse = Parse::new(&Utc, Utc::now().time());

        let test_cases = [(
            "2021-Feb-21",
            Utc.ymd(2021, 2, 21).and_time(Utc::now().time()),
        )];

        for &(input, want) in test_cases.iter() {
            assert_eq!(
                parse
                    .month_ymd(input)
                    .unwrap()
                    .unwrap()
                    .trunc_subsecs(0)
                    .with_second(0)
                    .unwrap(),
                want.unwrap().trunc_subsecs(0).with_second(0).unwrap(),
                "month_ymd/{}",
                input
            )
        }
        assert!(parse.month_ymd("not-date-time").is_none());
    }

    #[test]
    fn month_mdy_hms() {
        let parse = Parse::new(&Utc, Utc::now().time());

        let test_cases = [
            (
                "May 8, 2009 5:57:51 PM",
                Utc.ymd(2009, 5, 8).and_hms(17, 57, 51),
            ),
            (
                "September 17, 2012 10:09am",
                Utc.ymd(2012, 9, 17).and_hms(10, 9, 0),
            ),
            (
                "September 17, 2012, 10:10:09",
                Utc.ymd(2012, 9, 17).and_hms(10, 10, 9),
            ),
        ];

        for &(input, want) in test_cases.iter() {
            assert_eq!(
                parse.month_mdy_hms(input).unwrap().unwrap(),
                want,
                "month_mdy_hms/{}",
                input
            )
        }
        assert!(parse.month_mdy_hms("not-date-time").is_none());
    }

    #[test]
    fn month_mdy_hms_z() {
        let parse = Parse::new(&Utc, Utc::now().time());

        let test_cases = [
            (
                "May 02, 2021 15:51:31 UTC",
                Utc.ymd(2021, 5, 2).and_hms(15, 51, 31),
            ),
            (
                "May 02, 2021 15:51 UTC",
                Utc.ymd(2021, 5, 2).and_hms(15, 51, 0),
            ),
            (
                "May 26, 2021, 12:49 AM PDT",
                Utc.ymd(2021, 5, 26).and_hms(7, 49, 0),
            ),
            (
                "September 17, 2012 at 10:09am PST",
                Utc.ymd(2012, 9, 17).and_hms(18, 9, 0),
            ),
        ];

        for &(input, want) in test_cases.iter() {
            assert_eq!(
                parse.month_mdy_hms_z(input).unwrap().unwrap(),
                want,
                "month_mdy_hms_z/{}",
                input
            )
        }
        assert!(parse.month_mdy_hms_z("not-date-time").is_none());
        // Pre-filter: 20+ chars required; no isolated 4-digit year → has_year=false, rejected
        assert!(parse.month_mdy_hms_z("May 27, 02:45:27 XX PST").is_none()); // 23 chars, no 4-digit year
        // Pre-filter: 20+ chars with isolated 4-digit sequence → has_year=true, regex rejects format
        assert!(parse.month_mdy_hms_z("May 27 1234 something PST").is_none()); // 25 chars, has_year=true but regex rejects
    }

    #[test]
    fn month_mdy() {
        let parse = Parse::new(&Utc, Utc::now().time());

        let test_cases = [
            (
                "May 25, 2021",
                Utc.ymd(2021, 5, 25).and_time(Utc::now().time()),
            ),
            (
                "oct 7, 1970",
                Utc.ymd(1970, 10, 7).and_time(Utc::now().time()),
            ),
            (
                "oct 7, 70",
                Utc.ymd(1970, 10, 7).and_time(Utc::now().time()),
            ),
            (
                "oct. 7, 1970",
                Utc.ymd(1970, 10, 7).and_time(Utc::now().time()),
            ),
            (
                "oct. 7, 70",
                Utc.ymd(1970, 10, 7).and_time(Utc::now().time()),
            ),
            (
                "October 7, 1970",
                Utc.ymd(1970, 10, 7).and_time(Utc::now().time()),
            ),
        ];

        for &(input, want) in test_cases.iter() {
            assert_eq!(
                parse
                    .month_mdy(input)
                    .unwrap()
                    .unwrap()
                    .trunc_subsecs(0)
                    .with_second(0)
                    .unwrap(),
                want.unwrap().trunc_subsecs(0).with_second(0).unwrap(),
                "month_mdy/{}",
                input
            )
        }
        assert!(parse.month_mdy("not-date-time").is_none());
    }

    #[test]
    fn month_dmy_hms() {
        let parse = Parse::new(&Utc, Utc::now().time());

        let test_cases = [
            (
                "12 Feb 2006, 19:17",
                Utc.ymd(2006, 2, 12).and_hms(19, 17, 0),
            ),
            ("12 Feb 2006 19:17", Utc.ymd(2006, 2, 12).and_hms(19, 17, 0)),
            (
                "14 May 2019 19:11:40.164",
                Utc.ymd(2019, 5, 14).and_hms_milli(19, 11, 40, 164),
            ),
        ];

        for &(input, want) in test_cases.iter() {
            assert_eq!(
                parse.month_dmy_hms(input).unwrap().unwrap(),
                want,
                "month_dmy_hms/{}",
                input
            )
        }
        assert!(parse.month_dmy_hms("not-date-time").is_none());
    }

    #[test]
    fn month_dmy() {
        let parse = Parse::new(&Utc, Utc::now().time());

        let test_cases = [
            ("7 oct 70", Utc.ymd(1970, 10, 7).and_time(Utc::now().time())),
            (
                "7 oct 1970",
                Utc.ymd(1970, 10, 7).and_time(Utc::now().time()),
            ),
            (
                "03 February 2013",
                Utc.ymd(2013, 2, 3).and_time(Utc::now().time()),
            ),
            (
                "1 July 2013",
                Utc.ymd(2013, 7, 1).and_time(Utc::now().time()),
            ),
        ];

        for &(input, want) in test_cases.iter() {
            assert_eq!(
                parse
                    .month_dmy(input)
                    .unwrap()
                    .unwrap()
                    .trunc_subsecs(0)
                    .with_second(0)
                    .unwrap(),
                want.unwrap().trunc_subsecs(0).with_second(0).unwrap(),
                "month_dmy/{}",
                input
            )
        }
        assert!(parse.month_dmy("not-date-time").is_none());
    }

    // Explicitly tests the `four_digit_year` fast path in `month_dmy` (skips `%d %B %y`) and
    // the else-branch fallback that tries `%d %B %y` first then `%d %B %Y`.
    #[test]
    fn month_dmy_year_fast_path() {
        let parse = Parse::new(&Utc, Utc::now().time());

        // Fast path: 4-digit year — `four_digit_year` is true, goes directly to `%d %B %Y`
        let four_digit = parse.month_dmy("14 May 2019").unwrap().unwrap();
        assert_eq!(four_digit.year(), 2019);
        assert_eq!(four_digit.month(), 5);
        assert_eq!(four_digit.day(), 14);

        // Else-branch: 2-digit year — `four_digit_year` is false, tries `%d %B %y` first
        // chrono %y: 00–68 → 2000–2068, so "19" → 2019 (not 1919)
        let two_digit = parse.month_dmy("14 May 19").unwrap().unwrap();
        assert_eq!(two_digit.year(), 2019);
        assert_eq!(two_digit.month(), 5);
        assert_eq!(two_digit.day(), 14);
    }

    #[test]
    fn slash_mdy_hms() {
        let parse = Parse::new(&Utc, Utc::now().time());

        let test_cases = vec![
            ("4/8/2014 22:05", Utc.ymd(2014, 4, 8).and_hms(22, 5, 0)),
            ("04/08/2014 22:05", Utc.ymd(2014, 4, 8).and_hms(22, 5, 0)),
            ("4/8/14 22:05", Utc.ymd(2014, 4, 8).and_hms(22, 5, 0)),
            ("04/2/2014 03:00:51", Utc.ymd(2014, 4, 2).and_hms(3, 0, 51)),
            ("8/8/1965 12:00:00 AM", Utc.ymd(1965, 8, 8).and_hms(0, 0, 0)),
            (
                "8/8/1965 01:00:01 PM",
                Utc.ymd(1965, 8, 8).and_hms(13, 0, 1),
            ),
            ("8/8/1965 01:00 PM", Utc.ymd(1965, 8, 8).and_hms(13, 0, 0)),
            ("8/8/1965 1:00 PM", Utc.ymd(1965, 8, 8).and_hms(13, 0, 0)),
            ("8/8/1965 12:00 AM", Utc.ymd(1965, 8, 8).and_hms(0, 0, 0)),
            ("4/02/2014 03:00:51", Utc.ymd(2014, 4, 2).and_hms(3, 0, 51)),
            (
                "03/19/2012 10:11:59",
                Utc.ymd(2012, 3, 19).and_hms(10, 11, 59),
            ),
            (
                "03/19/2012 10:11:59.3186369",
                Utc.ymd(2012, 3, 19).and_hms_nano(10, 11, 59, 318636900),
            ),
        ];

        for &(input, want) in test_cases.iter() {
            assert_eq!(
                parse.slash_mdy_hms(input).unwrap().unwrap(),
                want,
                "slash_mdy_hms/{}",
                input
            )
        }
        assert!(parse.slash_mdy_hms("not-date-time").is_none());
    }

    #[test]
    fn slash_mdy() {
        let parse = Parse::new(&Utc, Utc::now().time());

        let test_cases = [
            (
                "3/31/2014",
                Utc.ymd(2014, 3, 31).and_time(Utc::now().time()),
            ),
            (
                "03/31/2014",
                Utc.ymd(2014, 3, 31).and_time(Utc::now().time()),
            ),
            ("08/21/71", Utc.ymd(1971, 8, 21).and_time(Utc::now().time())),
            ("8/1/71", Utc.ymd(1971, 8, 1).and_time(Utc::now().time())),
        ];

        for &(input, want) in test_cases.iter() {
            assert_eq!(
                parse
                    .slash_mdy(input)
                    .unwrap()
                    .unwrap()
                    .trunc_subsecs(0)
                    .with_second(0)
                    .unwrap(),
                want.unwrap().trunc_subsecs(0).with_second(0).unwrap(),
                "slash_mdy/{}",
                input
            )
        }
        assert!(parse.slash_mdy("not-date-time").is_none());
    }

    #[test]
    fn slash_dmy() {
        let mut parse = Parse::new(&Utc, Utc::now().time());

        let test_cases = [
            (
                "31/3/2014",
                Utc.ymd(2014, 3, 31).and_time(Utc::now().time()),
            ),
            (
                "13/11/2014",
                Utc.ymd(2014, 11, 13).and_time(Utc::now().time()),
            ),
            ("21/08/71", Utc.ymd(1971, 8, 21).and_time(Utc::now().time())),
            ("1/8/71", Utc.ymd(1971, 8, 1).and_time(Utc::now().time())),
        ];

        for &(input, want) in test_cases.iter() {
            assert_eq!(
                parse
                    .prefer_dmy(true)
                    .slash_dmy(input)
                    .unwrap()
                    .unwrap()
                    .trunc_subsecs(0)
                    .with_second(0)
                    .unwrap(),
                want.unwrap().trunc_subsecs(0).with_second(0).unwrap(),
                "slash_dmy/{}",
                input
            )
        }
        assert!(parse.slash_dmy("not-date-time").is_none());
    }

    #[test]
    fn slash_ymd_hms() {
        let parse = Parse::new(&Utc, Utc::now().time());

        let test_cases = [
            ("2014/4/8 22:05", Utc.ymd(2014, 4, 8).and_hms(22, 5, 0)),
            ("2014/04/08 22:05", Utc.ymd(2014, 4, 8).and_hms(22, 5, 0)),
            ("2014/04/2 03:00:51", Utc.ymd(2014, 4, 2).and_hms(3, 0, 51)),
            ("2014/4/02 03:00:51", Utc.ymd(2014, 4, 2).and_hms(3, 0, 51)),
            (
                "2012/03/19 10:11:59",
                Utc.ymd(2012, 3, 19).and_hms(10, 11, 59),
            ),
            (
                "2012/03/19 10:11:59.3186369",
                Utc.ymd(2012, 3, 19).and_hms_nano(10, 11, 59, 318636900),
            ),
        ];

        for &(input, want) in test_cases.iter() {
            assert_eq!(
                parse.slash_ymd_hms(input).unwrap().unwrap(),
                want,
                "slash_ymd_hms/{}",
                input
            )
        }
        assert!(parse.slash_ymd_hms("not-date-time").is_none());
    }

    #[test]
    fn slash_ymd() {
        let parse = Parse::new(&Utc, Utc::now().time());

        let test_cases = [
            (
                "2014/3/31",
                Utc.ymd(2014, 3, 31).and_time(Utc::now().time()),
            ),
            (
                "2014/03/31",
                Utc.ymd(2014, 3, 31).and_time(Utc::now().time()),
            ),
        ];

        for &(input, want) in test_cases.iter() {
            assert_eq!(
                parse
                    .slash_ymd(input)
                    .unwrap()
                    .unwrap()
                    .trunc_subsecs(0)
                    .with_second(0)
                    .unwrap(),
                want.unwrap().trunc_subsecs(0).with_second(0).unwrap(),
                "slash_ymd/{}",
                input
            )
        }
        assert!(parse.slash_ymd("not-date-time").is_none());
    }
}

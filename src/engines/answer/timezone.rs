use chrono::{DateTime, TimeZone};
use chrono_tz::{OffsetComponents, Tz};
use maud::html;

use crate::engines::EngineResponse;

use super::regex;

pub fn request(query: &str) -> EngineResponse {
    match evaluate(query) {
        None => EngineResponse::new(),
        Some(TimeResponse::Current { time, timezone }) => EngineResponse::answer_html(html! {
            p.answer-query { "Current time in " (timezone_to_string(timezone)) }
            h3 {
                b { (time.format("%-I:%M %P")) }
                span.answer-comment {
                    " (" (time.format("%B %-d")) ")"
                }
            }
        }),
        Some(TimeResponse::Conversion {
            source_timezone,
            target_timezone,
            source_time,
            target_time,
            source_offset,
            target_offset,
        }) => {
            let delta_minutes = (target_offset - source_offset).num_minutes();
            let delta = if delta_minutes % 60 == 0 {
                format!("{:+}", delta_minutes / 60)
            } else {
                format!("{:+}:{}", delta_minutes / 60, delta_minutes % 60)
            };

            EngineResponse::answer_html(html! {
                p.answer-query {
                    (source_time.format("%-I:%M %P"))
                    " "
                    (timezone_to_string(source_timezone))
                    " to "
                    (timezone_to_string(target_timezone))
                }
                h3 {
                    b { (target_time.format("%-I:%M %P")) }
                    " "
                    span.answer-comment {
                        (timezone_to_string(target_timezone)) " (" (delta) ")"
                    }
                }
            })
        }
    }
}

#[derive(Debug)]
enum TimeResponse {
    Current {
        time: DateTime<Tz>,
        timezone: Tz,
    },
    Conversion {
        source_timezone: Tz,
        target_timezone: Tz,
        source_time: DateTime<Tz>,
        target_time: DateTime<Tz>,
        source_offset: chrono::Duration,
        target_offset: chrono::Duration,
    },
}

fn evaluate(query: &str) -> Option<TimeResponse> {
    // "4pm utc to cst"
    let re = regex!(r"(\d{1,2})(?:(\d{1,2}))?\s*(am|pm|) ([\w/+\-]+) (to|as|in) ([\w/+\-]+)");
    if let Some(captures) = re.captures(query) {
        if let Some(hour) = captures.get(1).map(|m| m.as_str().parse::<u32>().unwrap()) {
            let minute = match captures.get(2) {
                Some(m) => m.as_str().parse::<u32>().ok()?,
                None => 0,
            };
            let ampm = captures.get(3).unwrap().as_str();
            let timezone1_name = captures.get(4).unwrap().as_str();
            let timezone2_name = captures.get(6).unwrap().as_str();

            let source_timezone = parse_timezone(timezone1_name)?;
            let target_timezone = parse_timezone(timezone2_name)?;

            let current_date = chrono::Utc::now().date_naive();

            let source_offset = source_timezone.offset_from_utc_date(&current_date);
            let target_offset = target_timezone.offset_from_utc_date(&current_date);

            let source_time_naive = current_date.and_hms_opt(
                if ampm == "pm" && hour != 12 {
                    hour + 12
                } else if ampm == "am" && hour == 12 {
                    0
                } else {
                    hour
                },
                minute,
                0,
            )?;
            let source_time_utc = chrono::Utc
                .from_local_datetime(&source_time_naive)
                .latest()?
                - (source_offset.base_utc_offset() + source_offset.dst_offset());

            let source_time = source_time_utc.with_timezone(&source_timezone);
            let target_time = source_time_utc.with_timezone(&target_timezone);

            return Some(TimeResponse::Conversion {
                source_timezone,
                target_timezone,
                source_time,
                target_time,
                source_offset: source_offset.base_utc_offset(),
                target_offset: target_offset.base_utc_offset(),
            });
        }
    }

    // "utc time"
    let re = regex!(r"([\w/+\-]+)(?: current)? time$");
    // "time in utc"
    let re2 = regex!(r"time (?:in|as) ([\w/+\-]+)$");
    if let Some(timezone_name) = re
        .captures(query)
        .and_then(|m| m.get(1))
        .or_else(|| re2.captures(query).and_then(|m| m.get(1)))
    {
        if let Some(timezone) = parse_timezone(timezone_name.as_str()) {
            let time = chrono::Utc::now().with_timezone(&timezone);
            return Some(TimeResponse::Current { time, timezone });
        }
    }

    None
}

fn parse_timezone(timezone_name: &str) -> Option<Tz> {
    match timezone_name.to_lowercase().as_str() {
        "cst" | "cdt" => Some(Tz::CST6CDT),
        "est" | "edt" => Some(Tz::EST5EDT),
        _ => Tz::from_str_insensitive(timezone_name)
            .ok()
            .or_else(|| Tz::from_str_insensitive(&format!("etc/{timezone_name}")).ok()),
    }
}

fn timezone_to_string(tz: Tz) -> String {
    match tz {
        Tz::CST6CDT => "CST".to_string(),
        Tz::EST5EDT => "EST".to_string(),
        _ => {
            let tz_string = tz.name();
            if let Some(tz_string) = tz_string.strip_prefix("Etc/") {
                tz_string.to_string()
            } else {
                tz_string.to_string()
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_evaluate() {
        let response = evaluate("9 pm est to CST").unwrap();
        let TimeResponse::Conversion {
            source_time,
            target_time,
            ..
        } = response
        else {
            panic!("Expected TimeResponse::Conversion, got {response:?}");
        };

        // we don't check the exact offsets since it depends on daylight savings, cst
        // will always be 1 hour behind est though

        assert_eq!(source_time.format("%-I:%M %P").to_string(), "9:00 pm");
        assert_eq!(target_time.format("%-I:%M %P").to_string(), "8:00 pm");
    }
}

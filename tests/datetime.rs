#![cfg(all(
    feature = "toml",
    feature = "json",
    feature = "yaml",
    feature = "ini",
    feature = "ron",
))]

extern crate chrono;
extern crate config;

use chrono::{DateTime, TimeZone, Utc};
use config::*;

fn make() -> Config {
    Config::builder()
        .add_source(File::from_str(
            r#"
            {
                "json_datetime": "2017-05-10T02:14:53Z"
            }
            "#,
            FileFormat::Json,
        ))
        .add_source(File::from_str(
            r#"
            yaml_datetime: 2017-06-12T10:58:30Z
            "#,
            FileFormat::Yaml,
        ))
        .add_source(File::from_str(
            r#"
            toml_datetime = 2017-05-11T14:55:15Z
            "#,
            FileFormat::Toml,
        ))
        .add_source(File::from_str(
            r#"
                ini_datetime = 2017-05-10T02:14:53Z
            "#,
            FileFormat::Ini,
        ))
        .add_source(File::from_str(
            r#"
            (
                ron_datetime: "2021-04-19T11:33:02Z"
            )
            "#,
            FileFormat::Ron,
        ))
        .build()
        .unwrap()
}

#[test]
fn test_datetime_string() {
    let s = make();

    // JSON
    let date: String = s.get("json_datetime").unwrap();

    assert_eq!(&date, "2017-05-10T02:14:53Z");

    // TOML
    let date: String = s.get("toml_datetime").unwrap();

    assert_eq!(&date, "2017-05-11T14:55:15Z");

    // YAML
    let date: String = s.get("yaml_datetime").unwrap();

    assert_eq!(&date, "2017-06-12T10:58:30Z");

    // INI
    let date: String = s.get("ini_datetime").unwrap();

    assert_eq!(&date, "2017-05-10T02:14:53Z");

    // RON
    let date: String = s.get("ron_datetime").unwrap();

    assert_eq!(&date, "2021-04-19T11:33:02Z");
}

#[test]
fn test_datetime() {
    let s = make();

    // JSON
    let date: DateTime<Utc> = s.get("json_datetime").unwrap();

    assert_eq!(date, Utc.ymd(2017, 5, 10).and_hms(2, 14, 53));

    // TOML
    let date: DateTime<Utc> = s.get("toml_datetime").unwrap();

    assert_eq!(date, Utc.ymd(2017, 5, 11).and_hms(14, 55, 15));

    // YAML
    let date: DateTime<Utc> = s.get("yaml_datetime").unwrap();

    assert_eq!(date, Utc.ymd(2017, 6, 12).and_hms(10, 58, 30));

    // INI
    let date: DateTime<Utc> = s.get("ini_datetime").unwrap();

    assert_eq!(date, Utc.ymd(2017, 5, 10).and_hms(2, 14, 53));

    // RON
    let date: DateTime<Utc> = s.get("ron_datetime").unwrap();

    assert_eq!(date, Utc.ymd(2021, 4, 19).and_hms(11, 33, 2));
}

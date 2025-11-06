use std::sync::LazyLock;
use time::format_description;

pub mod activity;
pub mod activity_class;
pub mod activity_closure;
pub mod app_config;
pub mod blocker;
pub mod day;
pub mod dirty;
pub mod identifier;
pub mod interval;
pub mod job_config;
pub mod local_time;
pub mod manager;
pub mod project;
pub mod quota;

pub static BASIC_TIME_FORMAT: LazyLock<Vec<format_description::BorrowedFormatItem<'_>>> =
    LazyLock::new(|| {
        time::format_description::parse(
            "[hour padding:zero]:[minute padding:zero]:[second padding:zero]",
        )
        .unwrap()
    });

pub static BASIC_DATE_FORMAT: LazyLock<Vec<format_description::BorrowedFormatItem<'_>>> =
    LazyLock::new(|| {
        time::format_description::parse("[year]-[month padding:zero]-[day padding:zero]").unwrap()
    });

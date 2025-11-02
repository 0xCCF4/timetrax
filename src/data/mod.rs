use std::sync::LazyLock;
use time::format_description;

mod activity;
pub mod activity_closure;
mod blocker;
pub mod day;
mod interval;
pub mod local_time;

static BASIC_TIME_FORMAT: LazyLock<Vec<format_description::BorrowedFormatItem<'_>>> =
    LazyLock::new(|| {
        time::format_description::parse(
            "[hour padding:zero]:[minute padding:zero]:[second padding:zero]",
        )
        .unwrap()
    });

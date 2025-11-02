use log::warn;
use time::OffsetDateTime;

pub fn now() -> OffsetDateTime {
    OffsetDateTime::now_local().unwrap_or_else(|e| {
        warn!("Unable to determine local time: {}", e);
        OffsetDateTime::now_utc()
    })
}
pub fn now_time() -> time::Time {
    now().time()
}
pub fn now_date() -> time::Date {
    now().date()
}

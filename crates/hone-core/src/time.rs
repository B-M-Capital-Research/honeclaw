use chrono::{DateTime, FixedOffset, Utc};

pub const BEIJING_OFFSET_SECS: i32 = 8 * 3600;

pub fn beijing_offset() -> FixedOffset {
    FixedOffset::east_opt(BEIJING_OFFSET_SECS).expect("valid Beijing offset")
}

pub fn beijing_now() -> DateTime<FixedOffset> {
    Utc::now().with_timezone(&beijing_offset())
}

pub fn beijing_now_rfc3339() -> String {
    beijing_now().to_rfc3339()
}

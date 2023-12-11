use serenity::CacheAndHttp;
use std::{sync::Arc, time::Duration};

use crate::structs::Data;

pub async fn bday_crunching(_context: Arc<CacheAndHttp>, data: Data) {
    let mut interval_timer = tokio::time::interval(Duration::from_secs(900));
    while !data.exit_flag.load(std::sync::atomic::Ordering::Relaxed) {
        interval_timer.tick().await;
    }
}

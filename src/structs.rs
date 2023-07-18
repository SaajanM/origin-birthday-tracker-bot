use chrono::{DateTime, Utc};
use chrono_tz::Tz;
use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeSet, HashMap},
    sync::Arc,
};
use tokio::sync::RwLock;

use crate::persistence::SaveManager;

pub struct Data {
    pub state: Arc<ApplicationState>,
    pub saver: Arc<SaveManager>,
} // User data, which is stored and accessible in all command invocations'

#[derive(Default, Serialize, Deserialize)]
pub struct ApplicationState {
    #[serde(with = "rw_lock_app_state")]
    pub guild_map: RwLock<HashMap<u64, RWGuildData>>,
}

mod rw_lock_app_state {
    use std::collections::HashMap;

    use serde::de::Deserializer;
    use serde::ser::Serializer;
    use serde::{Deserialize, Serialize};
    use tokio::sync::RwLock;

    use super::RWGuildData;

    pub fn serialize<S>(val: &RwLock<HashMap<u64, RWGuildData>>, s: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        tokio::task::block_in_place(|| {
            let inner = val.blocking_read();
            HashMap::<u64, RWGuildData>::serialize(&inner, s)
        })
    }

    pub fn deserialize<'de, D>(de: D) -> Result<RwLock<HashMap<u64, RWGuildData>>, D::Error>
    where
        D: Deserializer<'de>,
    {
        tokio::task::block_in_place(|| {
            let res: HashMap<u64, RWGuildData> = Deserialize::deserialize(de)?;
            Ok(RwLock::new(res))
        })
    }
}

#[derive(Serialize, Deserialize, Default)]
pub struct RWGuildData {
    #[serde(with = "rw_lock_guild_data")]
    pub rw_lock: RwLock<GuildData>,
}

mod rw_lock_guild_data {
    use serde::de::Deserializer;
    use serde::ser::Serializer;
    use serde::{Deserialize, Serialize};
    use tokio::sync::RwLock;

    use super::GuildData;

    pub fn serialize<S>(val: &RwLock<GuildData>, s: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        tokio::task::block_in_place(|| {
            let inner = val.blocking_read();
            GuildData::serialize(&inner, s)
        })
    }

    pub fn deserialize<'de, D>(de: D) -> Result<RwLock<GuildData>, D::Error>
    where
        D: Deserializer<'de>,
    {
        tokio::task::block_in_place(|| {
            let res: GuildData = Deserialize::deserialize(de)?;
            Ok(RwLock::new(res))
        })
    }
}

#[derive(Default, Clone, Serialize, Deserialize)]
pub struct GuildData {
    #[serde(default)]
    #[serde(with = "opt_tz_serde")]
    pub timezone: Option<Tz>,
    pub announcement_channel: Option<u64>,
    pub schedule: BTreeSet<Arc<BirthdayInfo>>,
    // Exists purely for fast deletion
    pub birthday_map: HashMap<u64, Arc<BirthdayInfo>>,
}

mod opt_tz_serde {
    use chrono_tz::Tz;
    use serde::de::Deserializer;
    use serde::ser::Serializer;
    use serde::{Deserialize, Serialize};
    use std::str::FromStr;

    pub fn serialize<S>(val: &Option<Tz>, s: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        Option::<String>::serialize(&val.map(|tz| tz.name().to_string()), s)
    }

    pub fn deserialize<'de, D>(de: D) -> Result<Option<Tz>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let res: Option<String> = Deserialize::deserialize(de)?;
        match res {
            Some(inner) => Ok(Some(
                Tz::from_str(&inner).expect("Invalid timezone to deserialize"),
            )),
            None => Ok(None),
        }
    }
}

#[derive(PartialEq, PartialOrd, Eq, Ord, Serialize, Deserialize)]
pub struct BirthdayInfo {
    pub datetime: DateTime<Utc>,
    pub associated_user: u64,
}

pub type Error = Box<dyn std::error::Error + Send + Sync>;
pub type Context<'a> = poise::Context<'a, Data, Error>;

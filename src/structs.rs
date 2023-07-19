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
    pub birthday_schedule: BirthdaySchedule,
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

#[derive(Serialize, Deserialize, Default, Clone)]
pub struct BirthdaySchedule {
    schedule: BTreeSet<Arc<BirthdayInfo>>,
    // Exists purely for fast deletion
    birthday_map: HashMap<u64, Arc<BirthdayInfo>>,
}

impl BirthdaySchedule {
    pub fn get(&self, user_id: u64) -> Option<&Arc<BirthdayInfo>> {
        self.birthday_map.get(&user_id)
    }

    /// Returns the old value if present
    pub fn insert(&mut self, birthday_info: Arc<BirthdayInfo>) -> Option<Arc<BirthdayInfo>> {
        let res = self
            .birthday_map
            .insert(birthday_info.associated_user, Arc::clone(&birthday_info));
        let _ = self.schedule.insert(birthday_info);
        res
    }

    pub fn remove(&mut self, user_id: &u64) -> Option<Arc<BirthdayInfo>> {
        let res = self.birthday_map.remove(user_id);
        if let Some(inner) = &res {
            self.schedule.remove(inner);
        }
        res
    }

    pub fn peek_first(&self) -> Option<&Arc<BirthdayInfo>> {
        self.schedule.first()
    }

    pub fn pop_first(&mut self) -> Option<Arc<BirthdayInfo>> {
        let res = self.schedule.pop_first();
        if let Some(inner) = &res {
            let _ = self.birthday_map.remove(&inner.associated_user);
        }
        res
    }

    pub fn peek_occured(&self) -> Vec<&Arc<BirthdayInfo>> {
        let mut res = vec![];
        let mut continue_checking = true;
        let start_time = Utc::now();

        while continue_checking {
            match self.peek_first() {
                Some(inner) => {
                    if inner.datetime < start_time {
                        res.push(inner);
                    } else {
                        continue_checking = false;
                        continue;
                    }
                }
                None => {
                    continue_checking = false;
                    continue;
                }
            }
        }
        res
    }

    pub fn pop_occured(&mut self) -> Vec<Arc<BirthdayInfo>> {
        let mut res = vec![];
        let mut continue_checking = true;
        let start_time = Utc::now();

        while continue_checking {
            match self.peek_first() {
                Some(inner) => {
                    if inner.datetime < start_time {
                        match self.pop_first() {
                            Some(inner) => {
                                res.push(inner);
                            }
                            None => {
                                continue_checking = false;
                                continue;
                            }
                        }
                    } else {
                        continue_checking = false;
                        continue;
                    }
                }
                None => {
                    continue_checking = false;
                    continue;
                }
            }
        }
        res
    }

    pub fn len(&self) -> usize {
        self.birthday_map.len()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn ordered_iter(&self) -> std::collections::btree_set::Iter<'_, Arc<BirthdayInfo>> {
        self.schedule.iter()
    }
}

#[derive(PartialEq, PartialOrd, Eq, Ord, Serialize, Deserialize)]
pub struct BirthdayInfo {
    pub datetime: DateTime<Utc>,
    pub associated_user: u64,
}

pub type Error = Box<dyn std::error::Error + Send + Sync>;
pub type Context<'a> = poise::Context<'a, Data, Error>;

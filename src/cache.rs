//! Defintion and implementations of the [Cache] trait.

use async_trait::async_trait;
use serde::{de::DeserializeOwned, Serialize};
use std::fmt::Debug;
use std::time::Duration;

pub type Result<T> = std::result::Result<T, Error>;

impl<T> From<Error> for Result<T> {
    fn from(value: Error) -> Self {
        Self::Err(value)
    }
}

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("no entry for key `{0}` has been found")]
    NotFound(String),
    #[error("unexpected error")]
    Unknown,
}

/// Represents a general purpose cache.
#[async_trait]
pub trait Cache {
    async fn find<T: DeserializeOwned>(&self, key: &str) -> Result<T>;
    async fn save<T: Serialize + Send + Sync + Debug>(
        &self,
        key: &str,
        value: T,
        expire: Option<Duration>,
    ) -> Result<()>;
    async fn delete(&self, key: &str) -> Result<()>;
}

#[cfg(feature = "redis-cache")]
pub use redis_cache::*;

#[cfg(feature = "redis-cache")]
mod redis_cache {
    use super::{Cache, Error, Result};
    use async_trait::async_trait;
    use reool::{AsyncCommands, PoolDefault, RedisPool};
    use serde::{de::DeserializeOwned, Serialize};
    use std::time::Duration;
    use std::{fmt::Debug, num::TryFromIntError};

    /// Redis implementation of [`Cache`].
    pub struct RedisCache<'a> {
        pub pool: &'a RedisPool,
    }

    #[async_trait]
    impl<'a> Cache for RedisCache<'a> {
        #[instrument(skip(self))]
        async fn find<T: DeserializeOwned>(&self, key: &str) -> Result<T> {
            let mut conn = self.pool.check_out(PoolDefault).await.map_err(|err| {
                error!(error = err.to_string(), "pulling connection for redis",);
                Error::Unknown
            })?;

            let Some(data): Option<String> = conn.get(key).await.map_err(|err| {
                error!(error = err.to_string(), "performing GET command on redis",);
                Error::Unknown
            })?
            else {
                return Error::NotFound(key.to_string()).into();
            };

            serde_json::from_str(&data).map_err(|err| {
                error!(error = err.to_string(), "deserializing data of type T",);
                Error::Unknown
            })
        }

        #[instrument(skip(self))]
        async fn save<T: Serialize + Send + Sync + Debug>(
            &self,
            key: &str,
            value: T,
            expire: Option<Duration>,
        ) -> Result<()> {
            let mut conn = self.pool.check_out(PoolDefault).await.map_err(|err| {
                error!(error = err.to_string(), "pulling connection for redis",);
                Error::Unknown
            })?;

            let data = serde_json::to_string(&value).map_err(|err| {
                error!(error = err.to_string(), "serializing data of type T",);
                Error::Unknown
            })?;

            conn.set(key, data).await.map_err(|err| {
                error!(error = err.to_string(), "performing SET command on redis",);
                Error::Unknown
            })?;

            if let Some(expire) = expire {
                let expire = expire
                    .as_secs()
                    .try_into()
                    .map_err(|err: TryFromIntError| {
                        error!(error = err.to_string(), "parsing expiration time to usize",);
                        Error::Unknown
                    })?;

                conn.expire(key, expire).await.map_err(|err| {
                    error!(
                        error = err.to_string(),
                        "performing EXPIRE command on redis",
                    );
                    Error::Unknown
                })?;
            }

            Ok(())
        }

        #[instrument(skip(self))]
        async fn delete(&self, key: &str) -> Result<()> {
            let mut conn = self.pool.check_out(PoolDefault).await.map_err(|err| {
                error!(error = err.to_string(), "pulling connection for redis",);
                Error::Unknown
            })?;

            conn.del(key).await.map_err(|err| {
                error!(
                    error = err.to_string(),
                    "performing DELETE command on redis",
                );
                Error::Unknown
            })?;

            Ok(())
        }
    }
}

#[cfg(test)]
pub mod tests {
    use super::{Cache, Error, Result};
    use async_trait::async_trait;
    use once_cell::sync::Lazy;
    use serde::{de::DeserializeOwned, Serialize};
    use std::time::Duration;
    use std::{
        collections::HashMap,
        sync::{Arc, Mutex},
    };

    pub static IN_MEMORY_CACHE: Lazy<Arc<Mutex<HashMap<String, String>>>> =
        Lazy::new(|| Arc::new(Mutex::new(HashMap::new())));

    /// In memory implementation of [`Cache`].
    #[derive(Default)]
    pub struct InMemoryCache;

    #[async_trait]
    impl Cache for InMemoryCache {
        async fn find<T: DeserializeOwned>(&self, key: &str) -> Result<T> {
            let data: String = IN_MEMORY_CACHE
                .lock()
                .unwrap()
                .get(key)
                .map(ToString::to_string)
                .ok_or(Error::NotFound(key.to_string()))?;

            serde_json::from_str(&data).map_err(|err| {
                error!(error = err.to_string(), "deserializing data of type T",);
                Error::Unknown
            })
        }

        async fn save<T: Serialize + Send + Sync>(
            &self,
            key: &str,
            value: T,
            _expire: Option<Duration>,
        ) -> Result<()> {
            let data = serde_json::to_string(&value).map_err(|err| {
                error!(error = err.to_string(), "serializing data of type T",);
                Error::Unknown
            })?;

            IN_MEMORY_CACHE
                .lock()
                .unwrap()
                .insert(key.to_string(), data);

            Ok(())
        }

        async fn delete(&self, key: &str) -> Result<()> {
            IN_MEMORY_CACHE.lock().unwrap().remove(key);
            Ok(())
        }
    }
}

use crate::configs::system::SystemConfig;
use crate::streaming::cache::memory_tracker::CacheMemoryTracker;
use crate::streaming::clients::client_manager::ClientManager;
use crate::streaming::diagnostics::metrics::Metrics;
use crate::streaming::persistence::persister::*;
use crate::streaming::session::Session;
use crate::streaming::storage::{SegmentStorage, SystemStorage};
use crate::streaming::streams::stream::Stream;
use crate::streaming::users::permissioner::Permissioner;
use iggy::error::Error;
use iggy::utils::crypto::{Aes256GcmEncryptor, Encryptor};
use sled::Db;
use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;
use tokio::fs::{create_dir, File};
use tokio::sync::RwLock;
use tokio::time::Instant;
use tracing::{info, trace};
use async_zip::tokio::write::ZipFileWriter;
use async_zip::{Compression, ZipEntryBuilder};

#[derive(Debug)]
pub struct System {
    pub permissioner: Permissioner,
    pub(crate) storage: Arc<SystemStorage>,
    pub(crate) streams: HashMap<u32, Stream>,
    pub(crate) streams_ids: HashMap<String, u32>,
    pub(crate) config: Arc<SystemConfig>,
    pub(crate) client_manager: Arc<RwLock<ClientManager>>,
    pub(crate) encryptor: Option<Box<dyn Encryptor>>,
    pub(crate) metrics: Metrics,
    pub(crate) db: Option<Arc<Db>>,
}

/// For each cache eviction, we want to remove more than the size we need.
/// This is done on purpose to avoid evicting messages on every write.
const CACHE_OVER_EVICTION_FACTOR: u64 = 5;

impl System {
    pub fn new(config: Arc<SystemConfig>, db: Option<Arc<Db>>) -> System {
        let db = match db {
            Some(db) => db,
            None => {
                let db = sled::open(config.get_database_path());
                if db.is_err() {
                    panic!("Cannot open database at: {}", config.get_database_path());
                }
                Arc::new(db.unwrap())
            }
        };
        let persister: Arc<dyn Persister> = match config.partition.enforce_fsync {
            true => Arc::new(FileWithSyncPersister {}),
            false => Arc::new(FilePersister {}),
        };
        Self::create(config, SystemStorage::new(db.clone(), persister), Some(db))
    }

    pub fn create(
        config: Arc<SystemConfig>,
        storage: SystemStorage,
        db: Option<Arc<Db>>,
    ) -> System {
        info!(
            "Server-side encryption is {}.",
            Self::map_toggle_str(config.encryption.enabled)
        );
        System {
            encryptor: match config.encryption.enabled {
                true => Some(Box::new(
                    Aes256GcmEncryptor::from_base64_key(&config.encryption.key).unwrap(),
                )),
                false => None,
            },
            config,
            streams: HashMap::new(),
            streams_ids: HashMap::new(),
            storage: Arc::new(storage),
            client_manager: Arc::new(RwLock::new(ClientManager::default())),
            permissioner: Permissioner::default(),
            metrics: Metrics::init(),
            db,
        }
    }

    pub async fn init(&mut self) -> Result<(), Error> {
        if !Path::new(&self.config.get_system_path()).exists()
            && create_dir(&self.config.get_system_path()).await.is_err()
        {
            return Err(Error::CannotCreateBaseDirectory);
        }
        if !Path::new(&self.config.get_streams_path()).exists()
            && create_dir(&self.config.get_streams_path()).await.is_err()
        {
            return Err(Error::CannotCreateStreamsDirectory);
        }

        info!(
            "Initializing system, data will be stored at: {}",
            self.config.get_system_path()
        );
        let now = Instant::now();
        self.load_version().await?;
        self.load_users().await?;
        self.load_streams().await?;
        info!("Initialized system in {} ms.", now.elapsed().as_millis());
        Ok(())
    }

    pub async fn shutdown(&mut self, storage: Arc<dyn SegmentStorage>) -> Result<(), Error> {
        self.persist_messages(storage.clone()).await?;
        Ok(())
    }

    pub async fn persist_messages(&self, storage: Arc<dyn SegmentStorage>) -> Result<(), Error> {
        trace!("Saving buffered messages on disk...");
        for stream in self.streams.values() {
            stream.persist_messages(storage.clone()).await?;
        }

        Ok(())
    }

    pub fn ensure_authenticated(&self, session: &Session) -> Result<(), Error> {
        match session.is_authenticated() {
            true => Ok(()),
            false => Err(Error::Unauthenticated),
        }
    }

    fn map_toggle_str<'a>(enabled: bool) -> &'a str {
        match enabled {
            true => "enabled",
            false => "disabled",
        }
    }

    pub async fn clean_cache(&self, size_to_clean: u64) {
        for stream in self.streams.values() {
            for topic in stream.get_topics() {
                for partition in topic.get_partitions().into_iter() {
                    tokio::task::spawn(async move {
                        let memory_tracker = CacheMemoryTracker::get_instance().unwrap();
                        let mut partition_guard = partition.write().await;
                        let cache = &mut partition_guard.cache.as_mut().unwrap();
                        let size_to_remove = (cache.current_size() as f64
                            / memory_tracker.usage_bytes() as f64
                            * size_to_clean as f64)
                            .ceil() as u64;
                        cache.evict_by_size(size_to_remove * CACHE_OVER_EVICTION_FACTOR);
                    });
                }
            }
        }
    }

    pub async fn create_snapshot_file(&self) -> Result<String, Error> {
        let file_name = format!(
            "{}/snapshot-{}.zip",
            self.config.get_system_path(),
            chrono::Utc::now().timestamp()
        );

        let mut file = File::create(file_name.clone()).await?;
        let mut writer = ZipFileWriter::with_tokio(&mut file);

        let data = b"This is an example file.";
        let builder = ZipEntryBuilder::new("bar.txt".into(), Compression::Deflate);

        // TODO: warnings
        writer.write_entry_whole(builder, data).await;
        writer.close().await;

        println!("Zip file created successfully!");
        
        Ok(file_name)
    }
}

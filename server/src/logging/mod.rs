use crate::configs::system::LoggingConfig;
use crate::server_error::ServerError;

use std::io::{self, Write};
use std::path::PathBuf;
use std::str::FromStr;
use std::sync::{Arc, Mutex};
use tracing::{info, trace};
use tracing_appender::non_blocking::WorkerGuard;
use tracing_subscriber::{
    filter::LevelFilter, fmt, fmt::MakeWriter, prelude::*, reload, reload::Handle, Layer, Registry,
};

const IGGY_LOG_FILE_PREFIX: &str = "iggy-server.log";

// Writer that does nothing
struct NullWriter;
impl Write for NullWriter {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        Ok(buf.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

// Wrapper around Arc<Mutex<Vec<String>>> to implement Write
struct VecStringWriter(Arc<Mutex<Vec<String>>>);
impl Write for VecStringWriter {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        let mut lock = self.0.lock().unwrap();
        lock.push(String::from_utf8_lossy(buf).into_owned());
        Ok(buf.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        // Just nop, we don't need to flush anything
        Ok(())
    }
}

// This struct exists solely to implement MakeWriter
struct VecStringMakeWriter(Arc<Mutex<Vec<String>>>);
impl<'a> MakeWriter<'a> for VecStringMakeWriter {
    type Writer = VecStringWriter;

    fn make_writer(&'a self) -> Self::Writer {
        VecStringWriter(self.0.clone())
    }
}

pub trait EarlyLogDumper {
    fn dump_to_file<W: Write>(&self, writer: &mut W);
    fn dump_to_stdout(&self);
}

impl EarlyLogDumper for Logging {
    fn dump_to_file<W: Write>(&self, writer: &mut W) {
        let early_logs_buffer = self.early_logs_buffer.lock().unwrap();
        for log in early_logs_buffer.iter() {
            let log = strip_ansi_escapes::strip(log);
            writer.write_all(&log).unwrap();
        }
    }

    fn dump_to_stdout(&self) {
        let stdout = io::stdout();
        let mut handle = stdout.lock();
        let early_logs_buffer = self.early_logs_buffer.lock().unwrap();
        for log in early_logs_buffer.iter() {
            handle.write_all(log.as_bytes()).unwrap();
        }
    }
}

// Make reload::Layer::new more readable
type ReloadHandle = Handle<Box<dyn Layer<Registry> + Send + Sync>, Registry>;

pub struct Logging {
    stdout_guard: Option<WorkerGuard>,
    stdout_reload_handle: Option<ReloadHandle>,

    file_guard: Option<WorkerGuard>,
    file_reload_handle: Option<ReloadHandle>,

    filtering_reload_handle: Option<ReloadHandle>,

    early_logs_buffer: Arc<Mutex<Vec<String>>>,
}

impl Logging {
    pub fn new() -> Self {
        Self {
            stdout_guard: None,
            stdout_reload_handle: None,
            file_guard: None,
            file_reload_handle: None,
            filtering_reload_handle: None,
            early_logs_buffer: Arc::new(Mutex::new(vec![])),
        }
    }

    pub fn early_init(&mut self) {
        // Initialize layers
        // First layer is filtering based on severity
        // Second layer will just consume drain log entries and has first layer as a dependency
        // Third layer will write to a safe buffer and has first layer as a dependency
        // All layers will be replaced during late_init
        let mut layers = vec![];

        let filtering_layer = Self::get_filtering_level(None);
        let (_, filtering_layer_reload_handle) = reload::Layer::new(filtering_layer.boxed());
        self.filtering_reload_handle = Some(filtering_layer_reload_handle);

        let stdout_layer = fmt::Layer::default().with_writer(|| NullWriter);
        let (stdout_layer, stdout_layer_reload_handle) = reload::Layer::new(stdout_layer.boxed());
        self.stdout_reload_handle = Some(stdout_layer_reload_handle);
        layers.push(stdout_layer.and_then(filtering_layer));

        let file_layer = fmt::Layer::default()
            .with_target(true)
            .with_writer(VecStringMakeWriter(self.early_logs_buffer.clone()))
            .with_ansi(true);
        let (file_layer, file_layer_reload_handle) = reload::Layer::new(file_layer.boxed());
        self.file_reload_handle = Some(file_layer_reload_handle);
        layers.push(file_layer.and_then(filtering_layer));

        let subscriber = tracing_subscriber::registry().with(layers);

        tracing::subscriber::set_global_default(subscriber)
            .expect("Setting global default subscriber failed");

        if option_env!("IGGY_CI_BUILD") == Some("true") {
            let version = option_env!("CARGO_PKG_VERSION").unwrap_or("unknown");
            let hash = option_env!("VERGEN_GIT_SHA").unwrap_or("unknown");
            let built_at = option_env!("VERGEN_BUILD_TIMESTAMP").unwrap_or("unknown");
            let rust_version = option_env!("VERGEN_RUSTC_SEMVER").unwrap_or("unknown");
            let target = option_env!("VERGEN_CARGO_TARGET_TRIPLE").unwrap_or("unknown");
            info!(
                "Version: {}, hash: {}, built at: {} using rust version: {} for target: {}",
                version, hash, built_at, rust_version, target
            );
        } else {
            info!("It seems that you are a developer. Environment variable IGGY_CI_BUILD is not set to 'true', skipping build info print.")
        }

        // This is moment when we can start logging something and not worry about losing it.
    }

    pub fn late_init(
        &mut self,
        base_directory: String,
        config: &LoggingConfig,
    ) -> Result<(), ServerError> {
        // Write to stdout and file at the same time.
        // Use the non_blocking appender to avoid blocking the threads.
        // Use the rolling appender to avoid having a huge log file.
        // Make sure logs are dumped to the file during graceful shutdown.

        trace!("Logging config: {}", config);

        let filtering_level = Self::get_filtering_level(Some(config));
        let _ = self
            .filtering_reload_handle
            .as_ref()
            .ok_or(ServerError::FilterReloadFailure)?
            .modify(|layer| *layer = filtering_level.boxed());

        // Initialize non-blocking stdout layer
        let (_, stdout_guard) = tracing_appender::non_blocking(std::io::stdout());
        let stdout_layer = fmt::Layer::default().with_ansi(true).boxed();
        self.stdout_guard = Some(stdout_guard);

        let _ = self
            .stdout_reload_handle
            .as_ref()
            .ok_or(ServerError::StdoutReloadFailure)?
            .modify(|layer| *layer = stdout_layer);

        self.dump_to_stdout();

        // Initialize directory and file for logs
        let base_directory = PathBuf::from(base_directory);
        let logs_subdirectory = PathBuf::from(config.path.clone());
        let logs_path = base_directory.join(logs_subdirectory.clone());
        let file_appender =
            tracing_appender::rolling::hourly(logs_path.clone(), IGGY_LOG_FILE_PREFIX);
        let (mut non_blocking_file, file_guard) = tracing_appender::non_blocking(file_appender);

        self.dump_to_file(&mut non_blocking_file);

        let file_layer = fmt::layer()
            .with_target(true)
            .with_writer(non_blocking_file)
            .with_ansi(false)
            .boxed();

        self.file_guard = Some(file_guard);
        let _ = self
            .file_reload_handle
            .as_ref()
            .ok_or(ServerError::FileReloadFailure)?
            .modify(|layer| *layer = file_layer);

        info!(
            "Logging initialized, logs will be stored at: {:?}. Logs will be rotated hourly. Log level is: {}.",
            logs_path, filtering_level
        );

        Ok(())
    }

    // RUST_LOG always takes precedence over config
    fn get_filtering_level(config: Option<&LoggingConfig>) -> LevelFilter {
        if let Ok(rust_log) = std::env::var("RUST_LOG") {
            // Parse log level from RUST_LOG env variable
            if let Ok(level) = LevelFilter::from_str(&rust_log.to_uppercase()) {
                level
            } else {
                println!("Invalid RUST_LOG value: {}, falling back to info", rust_log);
                LevelFilter::INFO
            }
        } else {
            // Parse log level from config
            if let Some(config) = config {
                if let Ok(level) = LevelFilter::from_str(&config.level.to_uppercase()) {
                    level
                } else {
                    println!(
                        "Invalid log level in config: {}, falling back to info",
                        config.level
                    );
                    LevelFilter::INFO
                }
            } else {
                // config not provided
                LevelFilter::INFO
            }
        }
    }

    fn _install_log_rotation_handler(&self) {
        todo!("Implement log rotation handler based on size and retention time");
    }
}

impl Default for Logging {
    fn default() -> Self {
        Self::new()
    }
}

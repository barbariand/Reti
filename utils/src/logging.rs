use std::path::PathBuf;

use directories::{BaseDirs, ProjectDirs};
use tracing::level_filters::LevelFilter;
use tracing_appender::{non_blocking::WorkerGuard, rolling::Rotation};
use tracing_subscriber::{
    layer::SubscriberExt, util::SubscriberInitExt, EnvFilter, Layer,
};

pub trait LogsDirectory {
    fn log_dir(&self) -> Option<PathBuf>;
}

impl LogsDirectory for ProjectDirs {
    fn log_dir(&self) -> Option<PathBuf> {
        if cfg!(target_os = "linux") {
            self.state_dir().map(|path| path.join("logs"))
        } else if cfg!(target_os = "macos") {
            Some(
                BaseDirs::new()?
                    .home_dir()
                    .join("Library")
                    .join("Logs")
                    .join(self.project_path()),
            )
        } else if cfg!(target_os = "windows") {
            Some(self.data_dir().join("logs"))
        } else {
            None
        }
    }
}

pub fn init_logger(
    project_dirs: Option<ProjectDirs>,
    level: LevelFilter,
    file_name_prefix: impl Into<String>,
) -> Option<WorkerGuard> {
    let env_filter = EnvFilter::default()
        .add_directive(level.into())
        .add_directive(
            "rustyline=off"
                .parse()
                .expect("Could not turn off rustyline tracing"),
        )
        .add_directive(
            "tokio=off"
                .parse()
                .expect("Could not turn off tokio tracing"),
        );

    let stdout = tracing_subscriber::fmt::layer()
        .with_filter(env_filter)
        .with_filter(level);
    let mut log_file_writer = None;
    let mut guard = None;
    if let Some(project_dirs) = project_dirs {
        if let Some(log_dir) = project_dirs.log_dir() {
            if let Ok(file_appender) = tracing_appender::rolling::Builder::new()
                .rotation(Rotation::DAILY)
                .filename_prefix(file_name_prefix.into())
                .filename_suffix("log")
                .build(log_dir)
            {
                let (non_blocking, inner_guard) =
                    tracing_appender::non_blocking(file_appender);
                log_file_writer = Some(
                    tracing_subscriber::fmt::layer()
                        .with_ansi(false)
                        .with_writer(non_blocking),
                );
                guard = Some(inner_guard);
            }
        }
    }

    tracing_subscriber::registry()
        .with(stdout.and_then(log_file_writer))
        .init();

    guard
}

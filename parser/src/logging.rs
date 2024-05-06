use std::path::PathBuf;

use clap::crate_name;
use directories::{BaseDirs, ProjectDirs};
use tracing::level_filters::LevelFilter;
use tracing_appender::non_blocking::WorkerGuard;
use tracing_subscriber::{
    layer::SubscriberExt, util::SubscriberInitExt, EnvFilter, Layer,
};

pub(crate) trait LogsDirectory {
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

pub(crate) fn init_logger(
    project_dirs: Option<ProjectDirs>,
    level: LevelFilter,
) -> Option<WorkerGuard> {
    let stdout = tracing_subscriber::fmt::layer().with_filter(level);

    let mut log_file_writer = None;
    let mut guard = None;
    if let Some(project_dirs) = project_dirs {
        println!("{project_dirs:?}");
        if let Some(log_dir) = project_dirs.log_dir() {
            println!("{log_dir:?}");
            let file_appender = tracing_appender::rolling::daily(
                log_dir,
                format!("{}-repl", crate_name!()),
            );
            let (non_blocking, inner_guard) =
                tracing_appender::non_blocking(file_appender);
            log_file_writer = Some(
                tracing_subscriber::fmt::layer().with_writer(non_blocking),
            );
            guard = Some(inner_guard);
        }
    }

    tracing_subscriber::registry()
        .with(stdout.and_then(log_file_writer))
        .init();

    guard
}

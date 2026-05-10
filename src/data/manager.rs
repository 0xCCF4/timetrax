use crate::data::BASIC_DATE_FORMAT;
use crate::data::app_config::AppConfig;
use crate::data::day::{Day, DayInner};
use crate::data::dirty::DirtyMarker;
use crate::data::job_config::JobConfig;
use log::{error, trace, warn};
use std::collections::BTreeMap;
use std::fs::File;
use std::path::{Path, PathBuf};
use time::Date;

/// A day's data along with its on-disk origin (if any), used to decide whether to write on save.
pub enum AnnotatedDayInformation {
    OnDisk {
        day: DirtyMarker<DayInner>,
        origin: PathBuf,
    },
    Unsaved {
        day: DirtyMarker<DayInner>,
    },
}

impl AnnotatedDayInformation {
    #[must_use]
    pub fn new(day: DayInner, origin: Option<PathBuf>) -> Self {
        match origin {
            Some(path) => AnnotatedDayInformation::OnDisk {
                day: DirtyMarker::clean(day),
                origin: path,
            },
            None => AnnotatedDayInformation::Unsaved {
                day: DirtyMarker::clean(day),
            },
        }
    }
    #[must_use]
    pub fn inner(&self) -> &DayInner {
        match self {
            AnnotatedDayInformation::OnDisk { day, .. }
            | AnnotatedDayInformation::Unsaved { day } => day,
        }
    }
    pub fn inner_mut(&mut self) -> &mut DayInner {
        match self {
            AnnotatedDayInformation::OnDisk { day, .. }
            | AnnotatedDayInformation::Unsaved { day } => day,
        }
    }
}

/// Owns all loaded day data and orchestrates load/save for the data directory.
pub struct Manager<'a> {
    pub app_config: &'a AppConfig,
    pub data_path: PathBuf,

    pub days: BTreeMap<Date, AnnotatedDayInformation>,
}

impl<'a> Manager<'a> {
    /// Load `JobConfig` from disk.
    ///
    /// # Errors
    /// Returns an error if the file cannot be opened or contains invalid JSON.
    pub fn open_job_config<P: AsRef<Path>>(
        app_config: &'a AppConfig,
        data_path: P,
    ) -> std::io::Result<JobConfig> {
        let data_path = data_path.as_ref();

        let job_config_path = data_path.join(&app_config.job_config_file_name);

        trace!("Opening job config at path: {}", data_path.display());
        let job = match File::open(&job_config_path) {
            Err(err) => {
                error!("Failed to open job config file: {err}");
                return Err(err);
            }
            Ok(file) => {
                trace!(
                    "Successfully opened job config file at {}",
                    job_config_path.display()
                );

                let job = match serde_json::from_reader(file) {
                    Err(err) => {
                        error!("Failed to parse job config file: {err}");
                        return Err(std::io::Error::new(std::io::ErrorKind::InvalidData, err));
                    }
                    Ok(job) => job,
                };

                trace!("Successfully loaded job config.");
                job
            }
        };

        Ok(job)
    }
    /// Load all day files from `data_path` into a `Manager`.
    ///
    /// # Errors
    /// Returns an error if `data_path` cannot be read or a required directory cannot be created.
    pub fn open<P: AsRef<Path>>(app_config: &'a AppConfig, data_path: P) -> std::io::Result<Self> {
        let data_path = data_path.as_ref();

        let mut days = BTreeMap::new();
        let day_folder_path = data_path.join(&app_config.job_day_folder_format);

        if !day_folder_path.exists() {
            trace!(
                "Day folder path does not exist at {}, creating it.",
                day_folder_path.display()
            );
            if let Err(err) = std::fs::create_dir_all(&day_folder_path) {
                error!(
                    "Failed to create day folder path at {}: {}",
                    day_folder_path.display(),
                    err
                );
                return Err(err);
            }
        }

        for day_file in std::fs::read_dir(&day_folder_path)? {
            let day_file = match day_file {
                Err(e) => {
                    warn!(
                        "Failed to read entry in day folder at {}: {}",
                        day_folder_path.display(),
                        e
                    );
                    continue;
                }
                Ok(entry) => entry,
            };
            let path = day_file.path();
            if path.extension().and_then(|s| s.to_str()) == Some("json") {
                trace!("Loading day file at {}", path.display());

                let file = match File::open(&path) {
                    Err(e) => {
                        warn!("Failed to open day file at {}: {}", path.display(), e);
                        continue;
                    }
                    Ok(f) => f,
                };
                let day: Day = match serde_json::from_reader(file) {
                    Err(e) => {
                        warn!("Failed to parse day file at {}: {}", path.display(), e);
                        continue;
                    }
                    Ok(d) => d,
                };

                trace!("Successfully loaded day for date {}", day.date);
                days.insert(
                    day.date,
                    AnnotatedDayInformation::new(day.inner, Some(path)),
                );
            }
        }

        Ok(Manager {
            days,
            app_config,
            data_path: data_path.to_path_buf(),
        })
    }

    /// Persist all dirty day files to disk.
    ///
    /// # Errors
    /// Returns the last serialization or write error encountered; all dirty files are attempted.
    pub fn save(&mut self) -> std::io::Result<()> {
        let mut error = None;

        for (date, day_boxed) in &mut self.days {
            if let AnnotatedDayInformation::OnDisk { day, origin } = day_boxed {
                if day.is_dirty() {
                    trace!(
                        "Saving modified day for date {} to {}",
                        date,
                        origin.display()
                    );
                    let file = match File::create(&origin) {
                        Err(e) => {
                            error!(
                                "Failed to open day file for writing at {}: {}",
                                origin.display(),
                                e
                            );
                            error = Some(e);
                            continue;
                        }
                        Ok(f) => f,
                    };

                    if let Err(e) = serde_json::to_writer_pretty(
                        file,
                        &Day {
                            date: *date,
                            inner: day.inner.clone(),
                        },
                    ) {
                        error!("Failed to write day file at {}: {}", origin.display(), e);
                        error = Some(std::io::Error::other(e));
                        continue;
                    }

                    day.mark_clean();
                }
            } else if let AnnotatedDayInformation::Unsaved { day } = day_boxed {
                let date_format = match date.format(&*BASIC_DATE_FORMAT) {
                    Err(e) => {
                        error!("Failed to format date {date} for saving: {e}");
                        error = Some(std::io::Error::other(e));
                        continue;
                    }
                    Ok(f) => f,
                };
                let day_path = self
                    .data_path
                    .join(self.app_config.job_day_folder_format.as_str())
                    .join(date_format)
                    .with_extension("json");

                trace!("Saving new day for date {} to {}", date, day_path.display());

                let file = match File::create_new(&day_path) {
                    Err(e) => {
                        error!(
                            "Failed to open day file for writing at {}: {}",
                            day_path.display(),
                            e
                        );
                        error = Some(e);
                        continue;
                    }
                    Ok(f) => f,
                };

                if let Err(e) = serde_json::to_writer_pretty(
                    file,
                    &Day {
                        date: *date,
                        inner: day.inner.clone(),
                    },
                ) {
                    error!("Failed to write day file at {}: {}", day_path.display(), e);
                    error = Some(std::io::Error::other(e));
                    continue;
                }

                *day_boxed = AnnotatedDayInformation::OnDisk {
                    day: DirtyMarker::clean(day.clone()),
                    origin: day_path,
                };
            }
        }

        if let Some(e) = error { Err(e) } else { Ok(()) }
    }

    /// Return a mutable reference to the day entry for `date`, creating a new unsaved one if absent.
    pub fn get_or_create_day(&mut self, date: Date) -> &mut AnnotatedDayInformation {
        self.days.entry(date).or_insert_with(|| {
            

            AnnotatedDayInformation::new(DayInner::default(), None)
        })
    }

    /// Return a shared reference to the inner data for `date`.
    pub fn get_or_create_day_ref(&mut self, date: Date) -> &DayInner {
        self.get_or_create_day(date).inner()
    }

    /// Return a mutable reference to the inner data for `date`.
    pub fn get_or_create_day_mut(&mut self, date: Date) -> &mut DayInner {
        self.get_or_create_day(date).inner_mut()
    }
}

impl Drop for Manager<'_> {
    fn drop(&mut self) {
        if let Err(e) = self.save() {
            error!("Failed to save data on Manager drop: {e}");
        }
    }
}

use crate::cli::ExecutableCommand;
use crate::data::activity_class::{ActivityClass, ActivityClassInner};
use crate::data::app_config::AppConfig;
use crate::data::identifier::Identifier;
use crate::data::job_config::JobConfig;
use crate::data::manager::Manager;
use clap::Parser;
use log::error;
use uuid::Uuid;

#[derive(Parser, Default)]
pub enum CommandClass {
    /// List all classes
    #[default]
    #[clap(aliases = ["ls", "show", "info", "display"])]
    List,
    /// Delete a class
    #[clap(aliases = ["delete", "del", "rm"])]
    Remove {
        /// Class identifier
        class: Identifier,
    },
    /// Add a new class
    #[clap(aliases = ["new", "create"])]
    Add {
        /// Class name
        name: String,
        /// Priority
        priority: i32,
        /// Description of the class
        description: Option<String>,
    },
}

impl ExecutableCommand for CommandClass {
    type Error = std::io::Error;
    type Output = ();
    fn execute(
        &self,
        _config: &AppConfig,
        job_config: &mut JobConfig,
        _manager: Manager,
    ) -> Result<Self::Output, Self::Error> {
        match self {
            CommandClass::List => {
                if job_config.classes.is_empty() {
                    println!("No classes found");
                    return Ok(());
                } else {
                    println!("Class:");
                    for class in &job_config.classes {
                        println!(
                            " - {}{} ({})",
                            class.inner.name,
                            class
                                .inner
                                .description
                                .as_ref()
                                .map(|description| format!(": {}", description))
                                .unwrap_or_default(),
                            class.id
                        );
                    }
                }
            }
            CommandClass::Add {
                name,
                description,
                priority,
            } => {
                if job_config.classes.iter().any(|p| p.inner.name == *name) {
                    error!("Activity class with name '{}' already exists", name);
                    return Err(std::io::Error::new(
                        std::io::ErrorKind::Other,
                        "Activity class already exists",
                    ));
                }

                let new_class = ActivityClass {
                    id: Uuid::new_v4(),
                    inner: ActivityClassInner {
                        name: name.clone(),
                        description: description.clone(),
                        priority: *priority,
                    },
                };
                job_config.classes.push(new_class);

                println!("Added new activity class: {}", name);
            }
            CommandClass::Remove { class } => {
                let len_before = job_config.classes.len();

                job_config.classes.retain(|p| match class {
                    Identifier::Uuid(id) => &p.id != id,
                    Identifier::ByName(name) => &p.inner.name != name,
                });

                let len_after = job_config.classes.len();

                if len_before == len_after {
                    error!("Activity class not found: {:?}", class);
                    return Err(std::io::Error::new(
                        std::io::ErrorKind::NotFound,
                        "Activity class not found",
                    ));
                } else {
                    println!("Removed activity class: {:?}", class);
                }

                // todo remove reference from other activities
            }
        }

        Ok(())
    }
}

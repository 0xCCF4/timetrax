use crate::cli::ExecutableCommand;
use crate::cli::json_output::print_json;
use crate::data::activity_class::{ActivityClass, ActivityClassInner};
use crate::data::app_config::AppConfig;
use crate::data::identifier::Identifier;
use crate::data::job_config::JobConfig;
use crate::data::manager::Manager;
use clap::Parser;
use log::error;
use serde_json::json;
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
        config: &AppConfig,
        job_config: &mut JobConfig,
        _manager: Manager,
    ) -> Result<Self::Output, Self::Error> {
        match self {
            CommandClass::List => {
                if config.json {
                    let list: Vec<_> = job_config.classes.iter().map(|c| json!({
                        "id": c.id.to_string(),
                        "name": c.inner.name,
                        "priority": c.inner.priority,
                        "description": c.inner.description,
                    })).collect();
                    print_json(&list);
                } else if job_config.classes.is_empty() {
                    println!("No classes found");
                } else {
                    println!("Classes:");
                    for class in &job_config.classes {
                        println!(
                            " - {}{} (priority {}, {})",
                            class.inner.name,
                            class.inner.description
                                .as_ref()
                                .map(|d| format!(": {d}"))
                                .unwrap_or_default(),
                            class.inner.priority,
                            class.id,
                        );
                    }
                }
            }
            CommandClass::Add { name, description, priority } => {
                if job_config.classes.iter().any(|p| p.inner.name == *name) {
                    error!("Activity class with name '{name}' already exists");
                    return Err(std::io::Error::other(
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
                if config.json {
                    let out = json!({ "id": new_class.id.to_string(), "name": new_class.inner.name, "priority": new_class.inner.priority, "description": new_class.inner.description });
                    job_config.classes.push(new_class);
                    print_json(&out);
                } else {
                    job_config.classes.push(new_class);
                    println!("Added new activity class: {name}");
                }
            }
            CommandClass::Remove { class } => {
                let removed: Vec<_> = job_config.classes.iter()
                    .filter(|c| match class {
                        Identifier::Uuid(id) => &c.id == id,
                        Identifier::ByName(name) => &c.inner.name == name,
                    })
                    .map(|c| json!({ "id": c.id.to_string(), "name": c.inner.name }))
                    .collect();

                if removed.is_empty() {
                    error!("Activity class not found: {class:?}");
                    return Err(std::io::Error::new(
                        std::io::ErrorKind::NotFound,
                        "Activity class not found",
                    ));
                }

                job_config.classes.retain(|c| match class {
                    Identifier::Uuid(id) => &c.id != id,
                    Identifier::ByName(name) => &c.inner.name != name,
                });

                if config.json {
                    print_json(&json!({ "removed": removed }));
                } else {
                    println!("Removed activity class: {class:?}");
                }
            }
        }

        Ok(())
    }
}

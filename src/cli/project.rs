use crate::cli::ExecutableCommand;
use crate::cli::json_output::print_json;
use crate::data::app_config::AppConfig;
use crate::data::identifier::Identifier;
use crate::data::job_config::JobConfig;
use crate::data::manager::Manager;
use crate::data::project::{Project, ProjectInner};
use clap::Parser;
use log::error;
use serde_json::json;
use uuid::Uuid;

#[derive(Parser, Default)]
pub enum CommandProject {
    /// List all projects
    #[default]
    #[clap(aliases = ["ls", "show", "info", "display"])]
    List,
    /// Delete a project
    #[clap(aliases = ["delete", "del", "rm"])]
    Remove {
        /// Project identifier
        project: Identifier,
    },
    /// Create a new project
    #[clap(aliases = ["new", "create"])]
    Add {
        /// Project name
        name: String,
        /// Description of the project
        description: Option<String>,
    },
}

impl ExecutableCommand for CommandProject {
    type Error = std::io::Error;
    type Output = ();
    fn execute(
        &self,
        config: &AppConfig,
        job_config: &mut JobConfig,
        _manager: Manager,
    ) -> Result<Self::Output, Self::Error> {
        match self {
            CommandProject::List => {
                if config.json {
                    let list: Vec<_> = job_config.projects.iter().map(|p| json!({
                        "id": p.id.to_string(),
                        "name": p.inner.name,
                        "description": p.inner.description,
                    })).collect();
                    print_json(&list);
                } else if job_config.projects.is_empty() {
                    println!("No projects found");
                } else {
                    println!("Projects:");
                    for project in &job_config.projects {
                        println!(
                            " - {}{} ({})",
                            project.inner.name,
                            project.inner.description
                                .as_ref()
                                .map(|d| format!(": {d}"))
                                .unwrap_or_default(),
                            project.id
                        );
                    }
                }
            }
            CommandProject::Add { name, description } => {
                if job_config.projects.iter().any(|p| p.inner.name == *name) {
                    error!("Project with name '{name}' already exists");
                    return Err(std::io::Error::other(
                        "Project already exists",
                    ));
                }

                let new_project = Project {
                    id: Uuid::new_v4(),
                    inner: ProjectInner {
                        name: name.clone(),
                        description: description.clone(),
                    },
                };
                if config.json {
                    let out = json!({ "id": new_project.id.to_string(), "name": new_project.inner.name, "description": new_project.inner.description });
                    job_config.projects.push(new_project);
                    print_json(&out);
                } else {
                    job_config.projects.push(new_project);
                    println!("Added new project: {name}");
                }
            }
            CommandProject::Remove { project } => {
                let removed: Vec<_> = job_config.projects.iter()
                    .filter(|p| match project {
                        Identifier::Uuid(id) => &p.id == id,
                        Identifier::ByName(name) => &p.inner.name == name,
                    })
                    .map(|p| json!({ "id": p.id.to_string(), "name": p.inner.name }))
                    .collect();

                if removed.is_empty() {
                    error!("Project not found: {project:?}");
                    return Err(std::io::Error::new(
                        std::io::ErrorKind::NotFound,
                        "Project not found",
                    ));
                }

                job_config.projects.retain(|p| match project {
                    Identifier::Uuid(id) => &p.id != id,
                    Identifier::ByName(name) => &p.inner.name != name,
                });

                if config.json {
                    print_json(&json!({ "removed": removed }));
                } else {
                    println!("Removed project: {project:?}");
                }
            }
        }

        Ok(())
    }
}

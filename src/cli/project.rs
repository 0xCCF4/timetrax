use crate::cli::ExecutableCommand;
use crate::data::app_config::AppConfig;
use crate::data::identifier::Identifier;
use crate::data::job_config::JobConfig;
use crate::data::manager::Manager;
use crate::data::project::{Project, ProjectInner};
use clap::Parser;
use log::error;
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
        _config: &AppConfig,
        job_config: &mut JobConfig,
        _manager: Manager,
    ) -> Result<Self::Output, Self::Error> {
        match self {
            CommandProject::List => {
                if job_config.projects.is_empty() {
                    println!("No projects found");
                    return Ok(());
                } else {
                    println!("Projects:");
                    for project in &job_config.projects {
                        println!(
                            " - {}{} ({})",
                            project.inner.name,
                            project
                                .inner
                                .description
                                .as_ref()
                                .map(|description| format!(": {}", description))
                                .unwrap_or_default(),
                            project.id
                        );
                    }
                }
            }
            CommandProject::Add { name, description } => {
                if job_config.projects.iter().any(|p| p.inner.name == *name) {
                    error!("Project with name '{}' already exists", name);
                    return Err(std::io::Error::new(
                        std::io::ErrorKind::Other,
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
                job_config.projects.push(new_project);

                println!("Added new project: {}", name);
            }
            CommandProject::Remove { project } => {
                let len_before = job_config.projects.len();

                job_config.projects.retain(|p| match project {
                    Identifier::Uuid(id) => &p.id != id,
                    Identifier::ByName(name) => &p.inner.name != name,
                });

                let len_after = job_config.projects.len();

                if len_before == len_after {
                    error!("Project not found: {:?}", project);
                    return Err(std::io::Error::new(
                        std::io::ErrorKind::NotFound,
                        "Project not found",
                    ));
                } else {
                    println!("Removed project: {:?}", project);
                }

                // todo remove reference from other activities
            }
        }

        Ok(())
    }
}

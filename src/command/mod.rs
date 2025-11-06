use timetrax::data::app_config::AppConfig;
use timetrax::data::job_config::JobConfig;
use timetrax::data::manager::Manager;

mod pop;
mod project;
mod push;
mod status;
mod class;

pub use pop::*;
pub use project::*;
pub use push::*;
pub use status::*;
pub use class::*;

pub trait ExecutableCommand {
    type Error;
    type Output;
    fn execute(
        &self,
        config: &AppConfig,
        job_config: &mut JobConfig,
        manager: Manager,
    ) -> Result<Self::Output, Self::Error>;
}

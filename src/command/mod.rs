mod pop;
mod push;
mod status;

pub use pop::*;
pub use push::*;
pub use status::*;

use timetrax::data::app_config::AppConfig;
use timetrax::data::job_config::JobConfig;
use timetrax::data::manager::Manager;

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

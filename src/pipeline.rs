// Get a different name for this other than pipeline
// purpose of "pipeline" is to simulate twitter browsing
// Bot should enter active stage and start "scrolling" twitter for a random amount of time around 15 minutes
// During this time bot might make several posts
// Afterwards it should sleep for for a random amount of time to simulate getting off twitter

use crate::agent::Agent;

pub struct Pipeline {
    agent: Agent,
}

impl Pipeline {
    pub fn new(agent: Agent) -> Self {
        Self { agent }
    }

    /// Should not return until agent is shut down
    pub async fn run(&mut self) {}
}

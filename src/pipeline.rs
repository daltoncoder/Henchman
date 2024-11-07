// Get a different name for this other than pipeline
// purpose of "pipeline" is to simulate twitter browsing
// Bot should enter active stage and start "scrolling" twitter for a random amount of time around 15 minutes
// During this time bot might make several posts
// Afterwards it should sleep for for a random amount of time to simulate getting off twitter

use ethsign::SecretKey;
use rand::Rng;
use std::time::Duration;
use tokio::select;

use crate::{agent::Agent, config::Config, prompts::Prompts};

pub struct Pipeline {
    /// The Ai Agent
    config: PipelineConfig,
    agent: Agent,
}

impl Pipeline {
    pub async fn new(config: &Config, prompts: Prompts) -> Self {
        let pipeline_config: PipelineConfig = config.into();
        let agent: Agent = Agent::new(config.clone(), generate_eth_private_key(), prompts)
            .await
            .expect("Failed to create Agent");

        Self {
            agent,
            config: pipeline_config,
        }
    }

    /// Should not return until agent is shut down
    pub async fn run(&mut self) {
        // Generate Ethereum Address

        // Do an initial run
        loop {
            let scroll_wait_time = self.config.get_scroll_sleep_time();

            println!("Waiting to start scrolling : {:?}", scroll_wait_time);

            tokio::time::sleep(scroll_wait_time).await;

            let scroll_duration = self.config.get_scroll_duration_time();
            println!("Starting Scrolling for: {:?}", scroll_duration);
            let scroll_duration_fut = tokio::time::sleep(scroll_duration);

            // todo: I think tokio Sleep is cancel safe we probably dont need to pin here
            tokio::pin!(scroll_duration_fut);
            loop {
                let run_sleep_time = self.config.get_run_sleep_time();
                println!("Next run in: {:?}", run_sleep_time);

                let run_sleep_fut = tokio::time::sleep(run_sleep_time);
                select! {
                    _ = &mut scroll_duration_fut => {
                        println!("End Scrolling");
                        break;
                    }

                    _ = run_sleep_fut => {
                        if let Err(e) = self.agent.run().await {
                            println!("Error while running error: {e:?}");
                        };
                    }
                }
            }
        }
    }

    /// Get time a the agent AI agent should sleep before it starts "Scrolling twitter again"
    fn get_sleep_time(&self) -> Duration {
        todo!()
    }
}

struct PipelineConfig {
    scroll_sleep_min: u64,
    scroll_sleep_max: u64,
    scroll_duration_min: u64,
    scroll_duration_max: u64,
    run_sleep_min: u64,
    run_sleep_max: u64,
}

impl PipelineConfig {
    /// Returns random time in the scroll sleep range
    pub fn get_scroll_sleep_time(&self) -> Duration {
        let mut rng = rand::thread_rng();

        let time_to_wait = rng.gen_range(self.scroll_sleep_min..=self.scroll_sleep_max);

        Duration::from_secs(time_to_wait)
    }

    /// Returns a random time do scroll for
    pub fn get_scroll_duration_time(&self) -> Duration {
        let mut rng = rand::thread_rng();

        let time_to_scroll = rng.gen_range(self.scroll_duration_min..=self.scroll_duration_max);

        Duration::from_secs(time_to_scroll)
    }

    /// Returns random time in the run sleep range
    pub fn get_run_sleep_time(&self) -> Duration {
        let mut rng = rand::thread_rng();

        let time_to_wait = rng.gen_range(self.run_sleep_min..=self.run_sleep_max);

        Duration::from_secs(time_to_wait)
    }
}

impl From<&Config> for PipelineConfig {
    fn from(value: &Config) -> Self {
        // 0-30min default
        let (scroll_sleep_min, scroll_sleep_max) = value.scroll_sleep.unwrap_or((0, 1800));
        // 15-20min default
        let (scroll_duration_min, scroll_duration_max) =
            value.scroll_duration.unwrap_or((900, 1200));
        // 30sec-3min default
        let (run_sleep_min, run_sleep_max) = value.run_sleep.unwrap_or((30, 180));

        Self {
            scroll_sleep_min,
            scroll_sleep_max,
            scroll_duration_min,
            scroll_duration_max,
            run_sleep_min,
            run_sleep_max,
        }
    }
}

impl Default for PipelineConfig {
    fn default() -> Self {
        // scroll sleep 0-30 min
        // scroll duration 15-20min
        // run sleep 30sec-3 min
        Self {
            scroll_sleep_min: 0,
            scroll_sleep_max: 1800,
            scroll_duration_min: 900,
            scroll_duration_max: 1200,
            run_sleep_min: 30,
            run_sleep_max: 180,
        }
    }
}

fn generate_eth_private_key() -> SecretKey {
    let mut rng = rand::thread_rng();
    let mut random_bytes = [0u8; 32];

    rng.fill(&mut random_bytes);

    // todo i think there is a nonzero chance this could be out of range of the eliptic curve
    SecretKey::from_raw(&random_bytes).expect("Failed to generate ethereum key")
}

#[test]
fn test_random_sleep_time() {
    let config = PipelineConfig::default();
    let scroll_min = Duration::from_secs(0);
    let scroll_max = Duration::from_secs(1800);
    let run_min = Duration::from_secs(30);
    let run_max = Duration::from_secs(180);

    for _ in 0..100 {
        let run_sleep = config.get_run_sleep_time();
        let scroll_sleep = config.get_scroll_sleep_time();

        println!("run_sleep: {:?}", run_sleep);
        println!("scroll_sleep: {:?}", scroll_sleep);

        assert!(run_sleep <= run_max && run_sleep >= run_min);
        assert!(scroll_sleep <= scroll_max && scroll_sleep >= scroll_min);
    }
}

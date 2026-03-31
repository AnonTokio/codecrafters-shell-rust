use std::sync::Mutex;

use lazy_static::lazy_static;

lazy_static! {
    pub static ref BACKGROUDN_MANAGER: Mutex<BackgroundManager> =
        Mutex::new(BackgroundManager::default());
}

#[derive(Default)]
pub struct BackgroundManager {
    jobs: Vec<Option<BackgroundJob>>,
}

impl BackgroundManager {
    pub fn add_job(&mut self, command: String, job: std::process::Child) {
        let pid = job.id();
        let id = if let Some(id) = self.jobs.iter().position(|item| item.is_none()) {
            self.jobs[id].replace(BackgroundJob {
                id,
                pid,
                command,
                job,
            });
            id
        } else {
            let id = self.jobs.len();
            self.jobs.push(Some(BackgroundJob {
                id,
                pid,
                command,
                job,
            }));
            id
        };
        println!("[{}] {}", id + 1, pid);
    }
}

pub struct BackgroundJob {
    pub id: usize,
    pub pid: u32,
    pub command: String,
    pub job: std::process::Child,
}

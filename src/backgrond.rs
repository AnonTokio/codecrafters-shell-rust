use std::sync::Mutex;

use lazy_static::lazy_static;

lazy_static! {
    pub static ref BACKGROUDN_MANAGER: Mutex<BackgroundManager> =
        Mutex::new(BackgroundManager::default());
}

#[derive(Default)]
pub struct BackgroundManager {
    pub jobs: Vec<Option<BackgroundJob>>,
    pub enque_sequence: Vec<usize>,
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
        self.enque_sequence.push(id);
        println!("[{}] {}", id + 1, pid);
    }

    pub fn delete_jobs(&mut self, ids: &[usize]) {
        for id in ids {
            if let Some(job) = self.jobs.get_mut(*id) {
                job.take();
            }
        }

        self.enque_sequence.retain(|id| !ids.contains(id));
    }

    pub fn get_most_recent_indices(&self) -> (usize, usize) {
        let most_recent_job_id = self.enque_sequence.last().copied().unwrap_or_default();
        let second_recent_job_id = self
            .enque_sequence
            .get(self.enque_sequence.len().saturating_sub(2))
            .copied()
            .unwrap_or(most_recent_job_id);
        (most_recent_job_id, second_recent_job_id)
    }
}

pub struct BackgroundJob {
    pub id: usize,
    pub pid: u32,
    pub command: String,
    pub job: std::process::Child,
}

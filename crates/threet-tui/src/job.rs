use std::future::Future;
use std::pin::Pin;

use tokio::task::JoinHandle;

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum JobStatus {
    Ready,
    Running,
    Done,
}

pub enum JobState {
    Ready(Option<Pin<Box<dyn Future<Output = ()> + Send + 'static>>>),
    Running(JoinHandle<()>),
}

pub struct Job {
    name: String,
    state: JobState,
}

impl Job {
    pub fn new(name: String, future: Pin<Box<dyn Future<Output = ()> + Send + 'static>>) -> Job {
        Job {
            name,
            state: JobState::Ready(Some(future)),
        }
    }

    pub fn start(&mut self) {
        if let JobState::Ready(ref mut future) = self.state {
            // SAFETY: if the job status is `Ready` we can be sure that the optional future will exists, the `Option` is just
            // so we can move the value of the `Ready` struct to to the `tokio::spawn`, its not really optional
            let future = future.take().unwrap();
            self.state = JobState::Running(tokio::spawn(future));
        }
    }

    pub fn status(&self) -> JobStatus {
        match &self.state {
            JobState::Ready(_) => JobStatus::Ready,
            JobState::Running(handler) => {
                if handler.is_finished() {
                    JobStatus::Done
                } else {
                    JobStatus::Running
                }
            }
        }
    }
}

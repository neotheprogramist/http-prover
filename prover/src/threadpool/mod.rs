use crate::{errors::ProverError, threadpool::prove::prove, utils::job::JobStore};

use std::sync::Arc;
use tempfile::TempDir;
use tokio::{
    spawn,
    sync::{broadcast::Sender, mpsc, Mutex},
    task::JoinHandle,
};
use tracing::trace;

pub mod prove;
pub mod run;

pub use run::CairoVersionedInput;

type ReceiverType = Arc<
    Mutex<
        mpsc::Receiver<(
            u64,
            JobStore,
            TempDir,
            CairoVersionedInput,
            Arc<Mutex<Sender<String>>>,
        )>,
    >,
>;
type SenderType = Option<
    mpsc::Sender<(
        u64,
        JobStore,
        TempDir,
        CairoVersionedInput,
        Arc<Mutex<Sender<String>>>,
    )>,
>;
pub struct ThreadPool {
    workers: Vec<Worker>,
    sender: SenderType,
}

impl ThreadPool {
    pub fn new(size: usize) -> ThreadPool {
        assert!(size > 0);

        let (sender, receiver) = mpsc::channel(100);

        let receiver = Arc::new(Mutex::new(receiver));

        let mut workers = Vec::with_capacity(size);

        for id in 0..size {
            workers.push(Worker::new(id, Arc::clone(&receiver)));
        }

        ThreadPool {
            workers,
            sender: Some(sender),
        }
    }

    pub async fn execute(
        &self,
        job_id: u64,
        job_store: JobStore,
        dir: TempDir,
        program_input: CairoVersionedInput,
        sse_tx: Arc<Mutex<Sender<String>>>,
    ) -> Result<(), ProverError> {
        self.sender
            .as_ref()
            .ok_or(ProverError::CustomError(
                "Thread pool is shutdown".to_string(),
            ))?
            .send((job_id, job_store, dir, program_input, sse_tx))
            .await?;
        Ok(())
    }

    pub async fn shutdown(&mut self) -> Result<(), ProverError> {
        if let Some(sender) = self.sender.take() {
            drop(sender); // Dropping the sender signals that no more messages will be sent
        }

        // Wait for each worker to finish its current task
        for worker in &mut self.workers {
            if let Some(handle) = worker.thread.take() {
                if let Err(e) = handle.await {
                    eprintln!("Error waiting for worker: {:?}", e);
                    return Err(ProverError::CustomError(format!("Worker error: {:?}", e)));
                }
            }
        }

        Ok(())
    }
}

struct Worker {
    _id: usize,
    thread: Option<JoinHandle<()>>,
}

impl Worker {
    fn new(id: usize, receiver: ReceiverType) -> Worker {
        let thread = spawn(async move {
            loop {
                let message = receiver.lock().await.recv().await;
                match message {
                    Some((job_id, job_store, dir, program_input, sse_tx)) => {
                        trace!("Worker {id} got a job; executing.");

                        if let Err(e) = prove(job_id, job_store, dir, program_input, sse_tx).await {
                            eprintln!("Worker {id} encountered an error: {:?}", e);
                        }

                        trace!("Worker {id} finished the job.");
                    }
                    None => {
                        trace!("Worker {id} detected shutdown signal.");
                        break;
                    }
                }
            }
        });

        Worker {
            _id: id,
            thread: Some(thread),
        }
    }
}

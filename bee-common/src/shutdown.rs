// Copyright 2020 IOTA Stiftung
//
// Licensed under the Apache License, Version 2.0 (the "License"); you may not use this file except in compliance with
// the License. You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software distributed under the License is distributed on
// an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and limitations under the License.

use crate::worker::Error as WorkerError;

use futures::channel::oneshot;
use log::error;
use thiserror::Error;

use std::future::Future;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Sending the shutdown signal to a worker failed.")]
    SendingShutdownSignalFailed,

    #[error("Waiting for worker to shut down failed.")]
    WaitingforWorkerShutdownFailed(#[from] WorkerError),
}

pub type ShutdownNotifier = oneshot::Sender<()>;
pub type ShutdownListener = oneshot::Receiver<()>;
pub type WorkerShutdown = Box<dyn Future<Output = Result<(), WorkerError>> + Unpin>;
pub type Action = Box<dyn FnOnce()>;

/// Handles the graceful shutdown of asynchronous workers.
#[derive(Default)]
pub struct Shutdown {
    notifiers: Vec<ShutdownNotifier>,
    worker_shutdowns: Vec<WorkerShutdown>,
    actions: Vec<Action>,
}

impl Shutdown {
    /// Creates a new instance.
    pub fn new() -> Self {
        Self::default()
    }

    /// Adds an asynchronous worker, and a corresponding shutdown notifier.
    pub fn add_worker_shutdown(
        &mut self,
        worker: impl Future<Output = Result<(), WorkerError>> + Unpin + 'static,
        notifier: ShutdownNotifier,
    ) {
        self.notifiers.push(notifier);
        self.worker_shutdowns.push(Box::new(worker));
    }

    /// Adds teardown logic that is executed during shutdown.
    pub fn add_action(&mut self, action: impl FnOnce() + 'static) {
        self.actions.push(Box::new(action));
    }

    /// Executes the shutdown.
    pub async fn execute(mut self) -> Result<(), Error> {
        while let Some(notifier) = self.notifiers.pop() {
            // NOTE: in case of an error the `Err` variant simply contains our shutdown signal `()` that we tried to
            // send.
            notifier.send(()).map_err(|_| Error::SendingShutdownSignalFailed)?
        }

        while let Some(worker_shutdown) = self.worker_shutdowns.pop() {
            if let Err(e) = worker_shutdown.await {
                error!("Awaiting worker failed: {:?}.", e);
            }
        }

        while let Some(action) = self.actions.pop() {
            action();
        }

        Ok(())
    }
}

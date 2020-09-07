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

use bee_common::{shutdown_stream::ShutdownStream, worker::Error as WorkerError};
use bee_crypto::ternary::Hash;

use futures::{
    channel::mpsc,
    stream::{Fuse, StreamExt},
};
use log::info;

type Receiver = ShutdownStream<Fuse<mpsc::UnboundedReceiver<BundleValidatorWorkerEvent>>>;

pub(crate) struct BundleValidatorWorkerEvent(pub(crate) Hash);

pub(crate) struct BundleValidatorWorker {
    receiver: Receiver,
}

impl BundleValidatorWorker {
    pub(crate) fn new(receiver: Receiver) -> Self {
        Self { receiver }
    }

    fn validate(&self, tail_hash: Hash) {}

    pub(crate) async fn run(mut self) -> Result<(), WorkerError> {
        info!("Running.");

        while let Some(BundleValidatorWorkerEvent(hash)) = self.receiver.next().await {
            self.validate(hash);
        }

        info!("Stopped.");

        Ok(())
    }
}
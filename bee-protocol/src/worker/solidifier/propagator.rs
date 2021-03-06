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

use crate::{
    event::{LatestSolidMilestoneChanged, TransactionSolidified},
    milestone::Milestone,
    protocol::Protocol,
    tangle::tangle,
    worker::BundleValidatorWorkerEvent,
};

use bee_common::{shutdown_stream::ShutdownStream, worker::Error as WorkerError};
use bee_common_ext::{node::Node, worker::Worker};
use bee_crypto::ternary::Hash;
use bee_transaction::Vertex;

use async_trait::async_trait;
use futures::{
    channel::mpsc,
    stream::{Fuse, StreamExt},
};
use log::{info, warn};

use std::sync::Arc;

pub(crate) struct SolidPropagatorWorkerEvent(pub(crate) Hash);

pub(crate) struct SolidPropagatorWorker {
    bundle_validator: mpsc::UnboundedSender<BundleValidatorWorkerEvent>,
}

#[async_trait]
impl<N: Node> Worker<N> for SolidPropagatorWorker {
    type Config = ();
    type Error = WorkerError;
    type Event = SolidPropagatorWorkerEvent;
    type Receiver = ShutdownStream<Fuse<mpsc::UnboundedReceiver<Self::Event>>>;

    async fn start(
        mut self,
        mut receiver: Self::Receiver,
        _node: Arc<N>,
        _config: Self::Config,
    ) -> Result<(), Self::Error> {
        info!("Running.");

        while let Some(SolidPropagatorWorkerEvent(hash)) = receiver.next().await {
            self.propagate(hash);
        }

        info!("Stopped.");

        Ok(())
    }
}

impl SolidPropagatorWorker {
    pub(crate) fn new(bundle_validator: mpsc::UnboundedSender<BundleValidatorWorkerEvent>) -> Self {
        Self { bundle_validator }
    }

    fn propagate(&mut self, root: Hash) {
        let mut children = vec![root];

        while let Some(ref hash) = children.pop() {
            if tangle().is_solid_transaction(hash) {
                continue;
            }

            if let Some(tx) = tangle().get(&hash) {
                let mut index = None;

                if tangle().is_solid_transaction(tx.trunk()) && tangle().is_solid_transaction(tx.branch()) {
                    tangle().update_metadata(&hash, |metadata| {
                        metadata.solidify();

                        // This is possibly not sufficient as there is no guarantee a milestone has been validated
                        // before being solidified, we then also need to check when a milestone gets validated if it's
                        // already solid.
                        if metadata.flags().is_milestone() {
                            index = Some(metadata.milestone_index());
                        }

                        if metadata.flags().is_tail() {
                            if let Err(e) = self.bundle_validator.unbounded_send(BundleValidatorWorkerEvent(*hash)) {
                                warn!("Failed to send hash to bundle validator: {:?}.", e);
                            }
                        }
                    });

                    for child in tangle().get_children(&hash) {
                        children.push(child);
                    }

                    Protocol::get().bus.dispatch(TransactionSolidified(*hash));
                }

                if let Some(index) = index {
                    Protocol::get()
                        .bus
                        .dispatch(LatestSolidMilestoneChanged(Milestone { hash: *hash, index }));
                }
            }
        }
    }
}

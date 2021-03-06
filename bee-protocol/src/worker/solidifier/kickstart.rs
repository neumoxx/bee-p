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

use crate::{milestone::MilestoneIndex, protocol::Protocol, tangle::tangle};

use bee_common::{shutdown_stream::ShutdownStream, worker::Error as WorkerError};
use bee_common_ext::{node::Node, worker::Worker};

use async_trait::async_trait;
use futures::{channel::oneshot, stream::Fuse, StreamExt};
use log::info;
use tokio::time::{interval, Instant, Interval};

use std::{sync::Arc, time::Duration};

pub(crate) struct KickstartWorker {
    ms_sender: oneshot::Sender<MilestoneIndex>,
    ms_sync_count: u32,
}

#[async_trait]
impl<N: Node> Worker<N> for KickstartWorker {
    type Config = ();
    type Error = WorkerError;
    type Event = Instant;
    type Receiver = ShutdownStream<Fuse<Interval>>;

    async fn start(
        mut self,
        mut receiver: Self::Receiver,
        _node: Arc<N>,
        _config: Self::Config,
    ) -> Result<(), Self::Error> {
        info!("Running.");

        while let Some(_) = receiver.next().await {
            let next_ms = *tangle().get_latest_solid_milestone_index() + 1;
            let latest_ms = *tangle().get_latest_milestone_index();

            if Protocol::get().peer_manager.handshaked_peers.len() != 0 && next_ms + self.ms_sync_count < latest_ms {
                Protocol::request_milestone(MilestoneIndex(next_ms), None);
                self.ms_sender.send(MilestoneIndex(next_ms));

                for index in next_ms..(next_ms + self.ms_sync_count) {
                    Protocol::request_milestone(MilestoneIndex(index), None);
                }
                break;
            }
        }

        info!("Stopped.");

        Ok(())
    }
}

impl KickstartWorker {
    pub(crate) fn new(ms_sender: oneshot::Sender<MilestoneIndex>, ms_sync_count: u32) -> Self {
        Self {
            ms_sender,
            ms_sync_count,
        }
    }

    pub(crate) fn interval() -> Interval {
        interval(Duration::from_secs(1))
    }
}

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

use crate::{event::TpsMetricsUpdated, protocol::Protocol};

use bee_common::{shutdown_stream::ShutdownStream, worker::Error as WorkerError};
use bee_common_ext::{node::Node, worker::Worker};

use async_trait::async_trait;
use futures::{stream::Fuse, StreamExt};
use log::info;
use tokio::time::{interval, Instant, Interval};

use std::{sync::Arc, time::Duration};

#[derive(Default)]
pub(crate) struct TpsWorker {
    incoming: u64,
    new: u64,
    known: u64,
    stale: u64,
    invalid: u64,
    outgoing: u64,
}

#[async_trait]
impl<N: Node> Worker<N> for TpsWorker {
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

        while receiver.next().await.is_some() {
            self.tps();
        }

        info!("Stopped.");

        Ok(())
    }
}

impl TpsWorker {
    pub(crate) fn new() -> Self {
        Self::default()
    }

    pub(crate) fn interval() -> Interval {
        interval(Duration::from_secs(1))
    }

    fn tps(&mut self) {
        let incoming = Protocol::get().metrics.transactions_received();
        let new = Protocol::get().metrics.new_transactions();
        let known = Protocol::get().metrics.known_transactions();
        let stale = Protocol::get().metrics.stale_transactions();
        let invalid = Protocol::get().metrics.invalid_transactions();
        let outgoing = Protocol::get().metrics.transactions_sent();

        Protocol::get().bus.dispatch(TpsMetricsUpdated {
            incoming: incoming - self.incoming,
            new: new - self.new,
            known: known - self.known,
            stale: stale - self.stale,
            invalid: invalid - self.invalid,
            outgoing: outgoing - self.outgoing,
        });

        self.incoming = incoming;
        self.new = new;
        self.known = known;
        self.stale = stale;
        self.invalid = invalid;
        self.outgoing = outgoing;
    }
}

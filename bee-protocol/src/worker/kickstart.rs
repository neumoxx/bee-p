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
    protocol::{Protocol, MILESTONE_REQUEST_RANGE},
    tangle::tangle,
};

use bee_common::worker::Error as WorkerError;

use futures::{channel::oneshot, future::Fuse, select, FutureExt};

pub(crate) struct KickstartWorker {
    shutdown: Fuse<oneshot::Receiver<()>>,
}

impl KickstartWorker {
    pub(crate) fn new(shutdown: oneshot::Receiver<()>) -> Self {
        Self {
            shutdown: shutdown.fuse(),
        }
    }

    pub(crate) async fn run(mut self) -> Result<(), WorkerError> {
        loop {
            select! {
                _ = &mut self.shutdown => break,
                default => {
                    if *tangle().get_last_solid_milestone_index() + MILESTONE_REQUEST_RANGE < *tangle().get_last_milestone_index() {
                        Protocol::request_milestone_fill();
                        break;
                    }
                },
            }
        }
        Ok(())
    }
}

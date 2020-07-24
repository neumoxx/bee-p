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


use crate::websocket::{self, DashboardWebSocket, ServerEvent};

use bee_common::worker::Error as WorkerError;

use async_std::{future::ready, prelude::*};
use serde_json::json;
use actix::prelude::*;
use chrono::Local;
use futures::channel::oneshot::Receiver;
use log::info;

use std::time::Duration;

pub struct DBSizeWorker {
    tangle: u64,
    snapshot: u64,
    spent: u64,
    timestamp_in_millis: i64,
}

impl DBSizeWorker {
    pub fn new() -> Self {
        Self {
            tangle: 0,
            snapshot: 0,
            spent: 0,
            timestamp_in_millis: 0,
        }
    }

    fn dbsize(&mut self) {
        let date = Local::now();
        let tangle = 0;
        let snapshot = 0;
        let spent = 0;
        let timestamp_in_millis = date.timestamp_millis();

        info!(
            "tangle {} snapshot {} spent {} timestamp_in_millis {}",
            tangle - self.tangle,
            snapshot - self.snapshot,
            spent - self.spent,
            timestamp_in_millis - self.timestamp_in_millis,
        );

        self.tangle = tangle;
        self.snapshot = snapshot;
        self.spent = spent;
        self.timestamp_in_millis = timestamp_in_millis;
    }

    pub async unsafe fn run(self, mut shutdown: Receiver<()>) -> Result<(), WorkerError> {
        info!("Running.");

        loop {
            match ready(Ok(()))
                .delay(Duration::from_millis(60000))
                .race(&mut shutdown)
                .await
            {
                Ok(_) => 
                        for l in websocket::get_listeners() {
                            let date = Local::now();
                            let msg = json!({
                                "type": websocket::DB_SIZE_METRIC,
                                "data": [{
                                    "tangle": 100000000,
                                    "snapshot": 200000000,
                                    "spent": 300000000,
                                    "ts": (date.timestamp_millis()/1000)
                                }]
                            });
                            l.do_send(ServerEvent{ event: String::from(msg.to_string()) });
                        },
                Err(_) => break,
            }
        }

        info!("Stopped.");

        Ok(())
    }
}

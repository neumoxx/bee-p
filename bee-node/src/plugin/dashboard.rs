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

use crate::plugin::Plugin;

use bee_common::shutdown::Shutdown;
use bee_common_ext::event::Bus;
use bee_protocol::event::{
    TpsMetricsUpdated,
    LastMilestoneChanged,
    LastSolidMilestoneChanged,
    //NewTransaction
};
use chrono::Local;
use serde_json::json;

use std::{convert::Infallible, sync::Arc};

fn tps(metrics: &TpsMetricsUpdated) {

    let date = Local::now();

    for l in unsafe{bee_dashboard::websocket::get_listeners()} {
        let msg = json!({
            "type": bee_dashboard::websocket::TPS_METRICS,
            "data": {
                "incoming": metrics.incoming,
                "new": metrics.new,
                "known": metrics.known,
                "stale": metrics.stale,
                "invalid": metrics.invalid,
                "outgoing": metrics.outgoing,
                "ts": (date.timestamp_millis()),
            }
        });
        l.do_send(bee_dashboard::websocket::ServerEvent{ event: String::from(msg.to_string()) });
    }

    status_changed();
}

fn status_changed() {

    let date = Local::now();
    let tangle = bee_protocol::tangle::tangle();
    
    for l in unsafe{bee_dashboard::websocket::get_listeners()} {

        let msg = json!({
            "type": bee_dashboard::websocket::STATUS,
            "data": {
                "lsmi": tangle.get_last_solid_milestone_index().0,
                "lmi": tangle.get_last_milestone_index().0,
                "snapshot_index": tangle.get_snapshot_milestone_index().0,
                "pruning_index": 666,
                "is_healthy": tangle.is_synced(),
                "version": "1.2",
                "latest_version": "1.3",
                "uptime": 12345,
                "autopeering_id": "what up",
                "node_alias": "chez neumo",
                "connected_peers_count": 2,
                "current_requested_ms": 3,
                "ms_request_queue_size": 4,
                "request_queue_queued": 5,
                "request_queue_pending": 6,
                "request_queue_processing": 7,
                "request_queue_avg_latency": 8,
                "ts": date.timestamp_millis(),
                "server_metrics": {
                    "NumberOfAllTransactions":        200,
                    "NumberOfNewTransactions":        190,
                    "NumberOfKnownTransactions":      180,
                    "NumberOfInvalidTransactions":    170,
                    "NumberOfInvalidRequests":        160,
                    "NumberOfStaleTransactions":      150,
                    "NumberOfReceivedTransactionReq": 140,
                    "NumberOfReceivedMilestoneReq":   130,
                    "NumberOfReceivedHeartbeats":     120,
                    "NumberOfSentTransactions":       110,
                    "NumberOfSentTransactionsReq":    100,
                    "NumberOfSentMilestoneReq":       90,
                    "NumberOfSentHeartbeats":         80,
                    "NumberOfDroppedSentPackets":     70,
                    "NumberOfSentSpamTxsCount":       60,
                    "NumberOfValidatedBundles":       50,
                    "NumberOfSeenSpentAddr":          40,
                    "ts": date.timestamp_millis(),
                },
                "mem": {
                    "Sys":          1000000,
                    "HeapSys":      1100000,
                    "HeapInuse":    1300000,
                    "HeapIdle":     1500000,
                    "HeapReleased": 500000,
                    "HeapObjects":  110,
                    "MSpanInuse":   300000,
                    "MCacheInuse":  700000,
                    "StackSys":     800000,
                    "NumGC":        10,
                    "LastPauseGC":  200,
                    "ts": date.timestamp_millis(),
                },
                "caches": {
                    "Approvers": {
                        "Size": 6,
                    },
                    "RequestQueue": {
                        "Size": 5,
                    },
                    "Bundles": {
                        "Size": 4,
                    },
                    "Milestones": {
                        "Size": 3,
                    },
                    "Transactions": {
                        "Size": 2,
                    },
                    "IncomingTransactionWorkUnits": {
                        "Size": 1,
                    },
                    "RefsInvalidBundle": {
                        "Size": 0,
                    },
                    "ts": date.timestamp_millis(),
                }
            }
        });
        l.do_send(bee_dashboard::websocket::ServerEvent{ event: String::from(msg.to_string()) });
    }

}

fn last_milestone_changed(last_milestone: &LastMilestoneChanged) {
    for l in unsafe{bee_dashboard::websocket::get_listeners()} {
        let msg = json!({
            "type": bee_dashboard::websocket::MS,
            "data": {
                "hash": last_milestone
                            .0.hash().
                            iter_trytes()
                            .map(|trit| char::from(trit))
                            .collect::<String>(),
                "index": last_milestone.0.index().0,
            }
        });
        l.do_send(bee_dashboard::websocket::ServerEvent{ event: String::from(msg.to_string()) });
    }

    status_changed();
}

fn last_solid_milestone_changed(last_solid_milestone: &LastSolidMilestoneChanged) {
    for l in unsafe{bee_dashboard::websocket::get_listeners()} {
        let msg = json!({
            "type": bee_dashboard::websocket::MS,
            "data": {
                "hash": last_solid_milestone
                            .0.hash().
                            iter_trytes()
                            .map(|trit| char::from(trit))
                            .collect::<String>(),
                "index": last_solid_milestone.0.index().0,
            }
        });
        l.do_send(bee_dashboard::websocket::ServerEvent{ event: String::from(msg.to_string()) });
    }

    status_changed();
}
/*
fn new_transaction(new_transaction: &NewTransaction) {
    for l in unsafe{bee_dashboard::websocket::get_listeners()} {
        let mut value: i64 = new_transaction.value;
        let mut tx_type: u8 = bee_dashboard::websocket::TX_VALUE;
        if value == 0 as i64{
            tx_type = bee_dashboard::websocket::TX_ZERO_VALUE;
        }
        let msg = json!({
            "type": tx_type,
            "data": {
                "hash": new_transaction.hash,
                "value": value,
            }
        });
        l.do_send(bee_dashboard::websocket::ServerEvent{ event: String::from(msg.to_string()) });
    }
}
*/

pub(crate) struct DashboardPlugin {}

impl DashboardPlugin {
    pub(crate) fn new() -> Self {
        Self {}
    }
}

impl Plugin for DashboardPlugin {
    type Error = Infallible;

    fn name(&self) -> &str {
        "dashboard"
    }

    fn init(&mut self, bus: Arc<Bus>, shutdown: &mut Shutdown) -> Result<(), Self::Error> {
        
        bus.add_listener(tps);
        bus.add_listener(last_milestone_changed);
        bus.add_listener(last_solid_milestone_changed);
        //bus.add_listener(new_transaction);
        bee_dashboard::websocket::init(shutdown);
        Ok(())
    }

    fn start(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }
}


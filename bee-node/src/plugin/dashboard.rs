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
use bee_protocol::event::{LatestMilestoneChanged, LatestSolidMilestoneChanged, TpsMetricsUpdated};
use serde_json::json;

use std::{convert::Infallible, sync::Arc, time::Instant};

fn tps(metrics: &TpsMetricsUpdated) {
    let date = Instant::now();

    for l in unsafe { bee_dashboard::websocket::get_listeners() } {
        let msg = json!({
            "type": bee_dashboard::websocket::TPS_METRICS,
            "data": {
                "incoming": metrics.incoming,
                "new": metrics.new,
                "known": metrics.known,
                "stale": metrics.stale,
                "invalid": metrics.invalid,
                "outgoing": metrics.outgoing,
                "ts": (date.elapsed().as_millis()),
            }
        });
        l.do_send(bee_dashboard::websocket::ServerEvent {
            event: String::from(msg.to_string()),
        });
    }
}

fn last_milestone_changed(last_milestone: &LatestMilestoneChanged) {
    for l in unsafe { bee_dashboard::websocket::get_listeners() } {
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
        l.do_send(bee_dashboard::websocket::ServerEvent {
            event: String::from(msg.to_string()),
        });
    }
}

fn last_solid_milestone_changed(last_solid_milestone: &LatestSolidMilestoneChanged) {
    for l in unsafe { bee_dashboard::websocket::get_listeners() } {
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
        l.do_send(bee_dashboard::websocket::ServerEvent {
            event: String::from(msg.to_string()),
        });
    }
}

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
        bee_dashboard::websocket::init(shutdown);
        Ok(())
    }

    fn start(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }
}

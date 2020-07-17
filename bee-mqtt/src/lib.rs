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

use bee_common::shutdown::Shutdown;
use bee_event::Bus;
use bee_protocol::events::{LastMilestone, LastSolidMilestone};

use std::sync::Arc;

fn handle_last_milestone(last_milestone: &LastMilestone) {
    println!("lmi: {:?}", last_milestone.0.index());
}

fn handle_last_solid_milestone(last_solid_milestone: &LastSolidMilestone) {
    println!("lsmi: {:?}", last_solid_milestone.0.index());
}

pub fn init(bus: Arc<Bus<'static>>, _shutdown: &mut Shutdown) {
    bus.add_listener(handle_last_milestone);
    bus.add_listener(handle_last_solid_milestone);
}

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

use bee_common::{shutdown::Shutdown, worker::Error as WorkerError};
use bee_event::Bus;
use bee_protocol::events::{LastMilestone, LastSolidMilestone};

use async_std::task::{block_on, spawn};
use futures::channel::{mpsc, oneshot};
use rumqtt::{MqttClient, MqttOptions, QoS};

use std::{sync::Arc, thread, time::Duration};

async fn lmi_worker(
    receiver: mpsc::UnboundedReceiver<LastMilestone>,
    shutdown: oneshot::Receiver<()>,
) -> Result<(), WorkerError> {
    Ok(())
}

async fn lsmi_worker(
    receiver: mpsc::UnboundedReceiver<LastSolidMilestone>,
    shutdown: oneshot::Receiver<()>,
) -> Result<(), WorkerError> {
    Ok(())
}

fn lmi_listener(last_milestone: &LastMilestone) {
    println!("lmi: {:?}", last_milestone.0.index());
}

fn lsmi_listener(last_solid_milestone: &LastSolidMilestone) {
    println!("lsmi: {:?}", last_solid_milestone.0.index());
}

pub fn init(bus: Arc<Bus<'static>>, shutdown: &mut Shutdown) {
    let (lmi_tx, lmi_rx) = mpsc::unbounded();
    let (lmi_shutdown_tx, lmi_shutdown_rx) = oneshot::channel();
    shutdown.add_worker_shutdown(lmi_shutdown_tx, spawn(lmi_worker(lmi_rx, lmi_shutdown_rx)));
    bus.add_listener(lmi_listener);

    let (lsmi_tx, lsmi_rx) = mpsc::unbounded();
    let (lsmi_shutdown_tx, lsmi_shutdown_rx) = oneshot::channel();
    shutdown.add_worker_shutdown(lsmi_shutdown_tx, spawn(lsmi_worker(lsmi_rx, lsmi_shutdown_rx)));
    bus.add_listener(lsmi_listener);

    // let mqtt_options = MqttOptions::new("test-pubsub1", "localhost", 1883);
    // let (mut mqtt_client, _) = MqttClient::start(mqtt_options).unwrap();
    //
    // let sleep_time = Duration::from_secs(1);
    // thread::spawn(move || {
    //     for i in 0..100 {
    //         let payload = format!("publish {}", i);
    //         thread::sleep(sleep_time);
    //         mqtt_client.publish("bee", QoS::AtLeastOnce, false, payload).unwrap();
    //     }
    // });
}

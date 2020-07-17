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
use futures::{
    channel::{mpsc, oneshot},
    future::FutureExt,
    select,
    stream::StreamExt,
};
use rumqtt::{MqttClient, MqttOptions, QoS};

use std::{ptr, sync::Arc};

static mut MQTT: *const Mqtt = ptr::null();

struct Mqtt {
    lmi: mpsc::UnboundedSender<LastMilestone>,
    lsmi: mpsc::UnboundedSender<LastSolidMilestone>,
}

fn mqtt() -> &'static Mqtt {
    if unsafe { MQTT.is_null() } {
        panic!("Uninitialized mqtt.");
    } else {
        unsafe { &*MQTT }
    }
}

async fn lmi_worker(
    receiver: mpsc::UnboundedReceiver<LastMilestone>,
    shutdown: oneshot::Receiver<()>,
) -> Result<(), WorkerError> {
    let mut receiver_fused = receiver.fuse();
    let mut shutdown_fused = shutdown.fuse();

    let mqtt_options = MqttOptions::new("bee-lmi", "localhost", 1883);
    let (mut mqtt_client, _) = MqttClient::start(mqtt_options).unwrap();

    loop {
        select! {
            event = receiver_fused.next() => {
                if let Some(LastMilestone(milestone)) = event {
                    let payload = format!("{}", *milestone.index());
                    mqtt_client.publish("lmi", QoS::AtLeastOnce, false, payload).unwrap();
                }
            },
            _ = shutdown_fused => {
                break;
            }
        }
    }

    Ok(())
}

async fn lsmi_worker(
    receiver: mpsc::UnboundedReceiver<LastSolidMilestone>,
    shutdown: oneshot::Receiver<()>,
) -> Result<(), WorkerError> {
    Ok(())
}

fn lmi_listener(last_milestone: &LastMilestone) {
    mqtt().lmi.unbounded_send(last_milestone.clone()).unwrap();
}

fn lsmi_listener(last_solid_milestone: &LastSolidMilestone) {
    mqtt().lsmi.unbounded_send(last_solid_milestone.clone()).unwrap();
}

pub fn init(bus: Arc<Bus<'static>>, shutdown: &mut Shutdown) {
    if unsafe { !MQTT.is_null() } {
        panic!("Already initialized.");
    }

    let (lmi_tx, lmi_rx) = mpsc::unbounded();
    let (lmi_shutdown_tx, lmi_shutdown_rx) = oneshot::channel();
    shutdown.add_worker_shutdown(lmi_shutdown_tx, spawn(lmi_worker(lmi_rx, lmi_shutdown_rx)));

    let (lsmi_tx, lsmi_rx) = mpsc::unbounded();
    let (lsmi_shutdown_tx, lsmi_shutdown_rx) = oneshot::channel();
    shutdown.add_worker_shutdown(lsmi_shutdown_tx, spawn(lsmi_worker(lsmi_rx, lsmi_shutdown_rx)));

    let mqtt = Mqtt {
        lmi: lmi_tx,
        lsmi: lsmi_tx,
    };

    unsafe {
        MQTT = Box::leak(mqtt.into()) as *const _;
    }

    bus.add_listener(lmi_listener);
    bus.add_listener(lsmi_listener);
}

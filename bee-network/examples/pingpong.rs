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

//! This example shows how to create and run 2 TCP nodes using `bee_network`, that will
//! automatically add eachother as peers and exchange the messages 'ping' and 'pong'
//! respectively.
//!
//! You might want to run several instances of such a node in separate
//! terminals and connect those instances by specifying commandline arguments.
//!
//! ```bash
//! cargo r --example pingpong -- --bind 127.0.0.1:1337 --peers tcp://127.0.0.1:1338 --msg ping
//! cargo r --example pingpong -- --bind 127.0.0.1:1338 --peers tcp://127.0.0.1:1337 --msg pong
//! ```

#![allow(dead_code, unused_imports)]

mod common;

use bee_common_ext::shutdown_tokio::Shutdown;
use bee_network::{Command::*, EndpointId, Event, EventReceiver, Network, NetworkConfig, Origin};

use common::*;

use futures::{
    channel::{mpsc, oneshot},
    select,
    sink::SinkExt,
    stream::{Fuse, StreamExt},
    AsyncWriteExt, FutureExt,
};
use log::*;
use structopt::StructOpt;

use std::{
    collections::{HashMap, HashSet},
    time::Duration,
};

const RECONNECT_INTERVAL: u64 = 5; // 5 seconds

#[tokio::main]
async fn main() {
    let args = Args::from_args();
    let config = args.config();

    logger::init(log::LevelFilter::Debug);

    let node = Node::builder(config).finish().await;

    let mut network = node.network.clone();
    let config = node.config.clone();

    info!("[pingpong] Adding static peers...");
    for url in &config.peers {
        network.send(AddEndpoint { url: url.clone() }).await.unwrap();
    }

    info!("[pingpong] ...finished.");

    node.run().await;
}

struct Node {
    config: Config,
    network: Network,
    events: Fuse<EventReceiver>,
    shutdown: Shutdown,
    endpoints: HashSet<EndpointId>,
    handshakes: HashMap<String, Vec<EndpointId>>,
}

impl Node {
    async fn run(self) {
        let Node {
            config,
            mut network,
            mut events,
            shutdown,
            mut endpoints,
            mut handshakes,
            ..
        } = self;

        info!("[pingpong] Node running.");

        let mut ctrl_c = ctrl_c_listener().fuse();

        loop {
            select! {
                _ = ctrl_c => {
                    break;
                },
                event = events.next() => {
                    if let Some(event) = event {
                        info!("Received {}.", event);

                        process_event(event, &config.message, &mut network, &mut endpoints, &mut handshakes).await;
                    }
                },
            }
        }

        info!("[pingpong] Stopping node...");

        shutdown.execute().await.expect("error shutting down gracefully.");

        info!("[pingpong] Shutdown complete.");
    }

    pub fn builder(config: Config) -> NodeBuilder {
        NodeBuilder { config }
    }
}

#[inline]
async fn process_event(
    event: Event,
    message: &String,
    network: &mut Network,
    endpoints: &mut HashSet<EndpointId>,
    handshakes: &mut HashMap<String, Vec<EndpointId>>,
) {
    match event {
        Event::EndpointAdded { epid } => {
            info!("[pingpong] Added endpoint {}.", epid);

            network
                .send(ConnectEndpoint { epid })
                .await
                .expect("error sending Connect command");
        }

        Event::EndpointRemoved { epid, .. } => {
            info!("[pingpong] Removed endpoint {}.", epid);
        }

        Event::EndpointConnected { epid, origin, .. } => {
            info!("[pingpong] Connected endpoint {} ({}).", epid, origin);

            let message = Utf8Message::new(message);

            network
                .send(SendMessage {
                    receiver_epid: epid,
                    message: message.as_bytes(),
                })
                .await
                .expect("error sending message to peer");
        }

        Event::MessageReceived { epid, message, .. } => {
            if !endpoints.contains(&epid) {
                // NOTE: first message is assumed to be the handshake message
                let handshake = Utf8Message::from_bytes(&message);
                info!("[pingpong] Received handshake '{}' ({})", handshake, epid);

                let epids = handshakes.entry(handshake.to_string()).or_insert(Vec::new());
                if !epids.contains(&epid) {
                    epids.push(epid);
                }

                if epids.len() > 1 {
                    info!(
                        "[pingpong] '{0}' and '{1}' are duplicate connections. Dropping '{1}'...",
                        epids[0], epids[1]
                    );

                    network
                        .send(MarkDuplicate {
                            duplicate_epid: epids[1],
                            original_epid: epids[0],
                        })
                        .await
                        .expect("error sending 'MarkDuplicate'");

                    network
                        .send(DisconnectEndpoint { epid: epids[1] })
                        .await
                        .expect("error sending 'DisconnectEndpoint' command");
                }

                endpoints.insert(epid);

                spam_endpoint(network.clone(), epid);
            } else {
                let message = Utf8Message::from_bytes(&message);
                info!("[pingpong] Received message '{}' ({})", message, epid);
            }
        }

        Event::EndpointDisconnected { epid, .. } => {
            info!("[pingpong] Disconnected endpoint {}.", epid);

            endpoints.remove(&epid);

            // TODO: remove epid from self.handshakes
            // handshakes.remove(???);
        }

        _ => warn!("Unsupported event {}.", event),
    }
}

fn ctrl_c_listener() -> oneshot::Receiver<()> {
    let (sender, receiver) = oneshot::channel();

    tokio::spawn(async move {
        tokio::signal::ctrl_c().await.unwrap();

        sender.send(()).unwrap();
    });

    receiver
}

fn spam_endpoint(mut network: Network, epid: EndpointId) {
    info!("[pingpong] Now sending spam messages to {}", epid);

    tokio::spawn(async move {
        for i in 0u64.. {
            tokio::time::delay_for(Duration::from_secs(5)).await;

            let message = Utf8Message::new(&i.to_string());

            network
                .send(SendMessage {
                    receiver_epid: epid,
                    message: message.as_bytes(),
                })
                .await
                .expect("error sending number");
        }
    });
}

struct NodeBuilder {
    config: Config,
}

impl NodeBuilder {
    pub async fn finish(self) -> Node {
        let mut shutdown = Shutdown::new();

        let network_config = NetworkConfig::builder()
            .binding_address(&self.config.binding_address.ip().to_string())
            .binding_port(self.config.binding_address.port())
            .reconnect_interval(RECONNECT_INTERVAL)
            .finish();

        info!("[pingpong] Initializing network...");
        let (network, events) = bee_network::init(network_config, &mut shutdown).await;

        info!("[pingpong] Node initialized.");
        Node {
            config: self.config,
            network,
            events: events.fuse(),
            shutdown,
            handshakes: HashMap::new(),
            endpoints: HashSet::new(),
        }
    }
}

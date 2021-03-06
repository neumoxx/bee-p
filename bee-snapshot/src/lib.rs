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

pub(crate) mod constants;
pub(crate) mod pruning;
pub(crate) mod worker;

pub mod config;
pub mod event;
pub mod global;
pub mod local;
pub mod metadata;

use bee_common::shutdown_stream::ShutdownStream;
use bee_common_ext::{bee_node::BeeNode, event::Bus, shutdown_tokio::Shutdown, worker::Worker};
use bee_crypto::ternary::Hash;
use bee_ledger::state::LedgerState;
use bee_protocol::{event::LatestSolidMilestoneChanged, tangle::tangle, MilestoneIndex};

use chrono::{offset::TimeZone, Utc};
use futures::channel::{mpsc, oneshot};
use log::{info, warn};
use tokio::spawn;

use std::{path::Path, sync::Arc};

#[derive(Debug)]
pub enum Error {
    Global(global::FileError),
    Local(local::FileError),
    Download(local::DownloadError),
}

// TODO change return type

pub async fn init(
    config: &config::SnapshotConfig,
    bee_node: Arc<BeeNode>,
    bus: Arc<Bus<'static>>,
    shutdown: &mut Shutdown,
) -> Result<(LedgerState, MilestoneIndex, u64), Error> {
    let (state, index, timestamp) = match config.load_type() {
        config::LoadType::Global => {
            info!("Loading global snapshot file {}...", config.global().path());

            let snapshot =
                global::GlobalSnapshot::from_file(config.global().path(), MilestoneIndex(*config.global().index()))
                    .map_err(Error::Global)?;

            tangle().clear_solid_entry_points();
            // The genesis transaction must be marked as SEP with snapshot index during loading a global snapshot
            // because coordinator bootstraps the network by referencing the genesis tx.
            tangle().add_solid_entry_point(Hash::zeros(), MilestoneIndex(*config.global().index()));

            info!(
                "Loaded global snapshot file from with index {} and {} balances.",
                *config.global().index(),
                snapshot.state().len()
            );

            (snapshot.into_state(), *config.global().index(), 0)
        }
        config::LoadType::Local => {
            if !Path::new(config.local().path()).exists() {
                local::download_local_snapshot(config.local())
                    .await
                    .map_err(Error::Download)?;
            }
            info!("Loading local snapshot file {}...", config.local().path());

            let snapshot = local::LocalSnapshot::from_file(config.local().path()).map_err(Error::Local)?;

            info!(
                "Loaded local snapshot file from {} with index {}, {} solid entry points, {} seen milestones and \
                {} balances.",
                Utc.timestamp(snapshot.metadata().timestamp() as i64, 0).to_rfc2822(),
                snapshot.metadata().index(),
                snapshot.metadata().solid_entry_points().len(),
                snapshot.metadata().seen_milestones().len(),
                snapshot.state.len()
            );

            tangle().update_latest_solid_milestone_index(snapshot.metadata().index().into());
            tangle().update_latest_milestone_index(snapshot.metadata().index().into());
            tangle().update_snapshot_index(snapshot.metadata().index().into());
            tangle().update_pruning_index(snapshot.metadata().index().into());
            tangle().add_solid_entry_point(Hash::zeros(), MilestoneIndex(0));
            for (hash, index) in snapshot.metadata().solid_entry_points() {
                tangle().add_solid_entry_point(*hash, MilestoneIndex(*index));
            }
            for _seen_milestone in snapshot.metadata().seen_milestones() {
                // TODO request ?
            }

            let index = snapshot.metadata().index();
            let timestamp = snapshot.metadata().timestamp();

            (snapshot.into_state(), index, timestamp)
        }
    };

    let (snapshot_worker_tx, snapshot_worker_rx) = mpsc::unbounded();
    let (snapshot_worker_shutdown_tx, snapshot_worker_shutdown_rx) = oneshot::channel();

    shutdown.add_worker_shutdown(
        snapshot_worker_shutdown_tx,
        spawn(worker::SnapshotWorker::new(config.clone()).start(
            ShutdownStream::new(snapshot_worker_shutdown_rx, snapshot_worker_rx),
            bee_node,
            (),
        )),
    );

    bus.add_listener(move |latest_solid_milestone: &LatestSolidMilestoneChanged| {
        if let Err(e) = snapshot_worker_tx.unbounded_send(worker::SnapshotWorkerEvent(latest_solid_milestone.0.clone()))
        {
            warn!(
                "Failed to send milestone {} to snapshot worker: {:?}.",
                *latest_solid_milestone.0.index(),
                e
            )
        }
    });

    Ok((state, MilestoneIndex(index), timestamp))
}

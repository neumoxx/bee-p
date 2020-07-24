use std::time::{Duration, Instant};
use actix_web::{web, App, Error, HttpRequest, HttpResponse, HttpServer};
use actix_web_actors::ws;
use actix::prelude::*;
use chrono::Local;
use serde_json::json;
use bee_common::shutdown::Shutdown;
use std::sync::mpsc;
use std::thread;
use async_std::task::spawn;
use futures::channel::oneshot;
use crate::controller::static_files;

/// How often heartbeat pings are sent
const HEARTBEAT_INTERVAL: Duration = Duration::from_secs(5);
/// How long before lack of client response causes a timeout
const CLIENT_TIMEOUT: Duration = Duration::from_secs(20);

pub const STATUS: u8 = 0;
pub const TPS_METRICS: u8 = 1;
pub const TIP_SEL_METRIC: u8 = 2;
pub const TX: u8 = 3;
pub const MS: u8 = 4;
pub const PEER_METRIC: u8 = 5;
pub const CONFIRMED_MS_METRICS: u8 = 6;
pub const VERTEX: u8 = 7;
pub const SOLID_INFO: u8 = 8;
pub const CONFIRMED_INFO: u8 = 9;
pub const MILESTONE_INFO: u8 = 10;
pub const TIP_INFO: u8 = 11;
pub const DB_SIZE_METRIC: u8 = 12;
pub const DB_CLEANUP: u8 = 13;
pub const SPAM_METRICS: u8 = 14;
pub const AVG_SPAM_METRICS: u8 = 15;

static mut LISTENERS: Vec<Addr<DashboardWebSocket>> = vec![];

pub unsafe fn get_listeners() -> &'static Vec<Addr<DashboardWebSocket>>{
    &LISTENERS
}

/// do websocket handshake and start `DashboardSocket` actor
pub async fn ws_index(r: HttpRequest, stream: web::Payload, data: web::Data<Addr<ServerMonitor>>) -> Result<HttpResponse, Error> {
    println!("ws_index {}", "hola");
    println!("{:?}", r);
    let (addr, res) = ws::start_with_addr(DashboardWebSocket::new(), &r, stream)?;
    unsafe {
        LISTENERS.push(addr);
        //self.listeners.push(msg.addr);
        println!("LISTENERS.len: {}", LISTENERS.len());
    }
    println!("{:?}", res);
    Ok(res)
}

#[derive(Message)]
#[rtype(result = "()")]
struct RegisterWSClient {
    addr: Addr<DashboardWebSocket>,
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct ServerEvent {
    pub event: String,
}

impl DashboardWebSocket {
    fn new() -> Self {
        Self { hb: Instant::now() }
    }

    /// helper method that sends ping to client every second.
    ///
    /// also this method checks heartbeats from client
    fn hb(&self, ctx: &mut <Self as Actor>::Context) {
    }
}

pub struct ServerMonitor {
    listeners: Vec<Addr<DashboardWebSocket>>,
}

impl Actor for ServerMonitor {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        ctx.run_interval(Duration::from_secs(5), |act, _| {
            for l in &act.listeners {
                l.do_send(ServerEvent{ event: String::from("Event:") });
            }
        });
    }
}

impl Handler<RegisterWSClient> for ServerMonitor {
    type Result = ();

    fn handle(&mut self, msg: RegisterWSClient, _: &mut Context<Self>) {
        println!("handle called");
        unsafe {
            LISTENERS.push(msg.addr);
            //self.listeners.push(msg.addr);
            println!("LISTENERS.len: {}", LISTENERS.len());
        }
    }
}

/// websocket connection is long running connection, it easier
/// to handle with an actor
#[derive(Debug)]
pub struct DashboardWebSocket {
    /// Client must send ping at least once per 10 seconds (CLIENT_TIMEOUT),
    /// otherwise we drop connection.
    pub hb: Instant,
}

impl Actor for DashboardWebSocket {
    type Context = ws::WebsocketContext<Self>;

    /// Method is called on actor start. We start the heartbeat process here.
    fn started(&mut self, ctx: &mut Self::Context) {
        self.hb(ctx);
    }
}

/// Handler for `ws::Message`
impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for DashboardWebSocket {
    fn handle(
        &mut self,
        msg: Result<ws::Message, ws::ProtocolError>,
        ctx: &mut Self::Context,
    ) {
        // process websocket messages
        println!("WS: {:?}", msg);
        match msg {
            Ok(ws::Message::Ping(msg)) => {
                self.hb = Instant::now();
                ctx.pong(&msg);
            }
            Ok(ws::Message::Pong(_)) => {
                self.hb = Instant::now();
            }
            Ok(ws::Message::Text(text)) => ctx.text(text),
            Ok(ws::Message::Binary(bin)) => ctx.binary(bin),
            Ok(ws::Message::Close(reason)) => {
                ctx.close(reason);
                ctx.stop();
            }
            _ => ctx.stop(),
        }
    }
}

impl Handler<ServerEvent> for DashboardWebSocket {
    type Result = ();

    fn handle(&mut self, msg: ServerEvent, ctx: &mut Self::Context) {
        ctx.text(msg.event);
    }
}

pub fn spawn_workers(shutdown: &mut Shutdown) -> (){
    let (dbsize_worker_shutdown_tx, dbsize_worker_shutdown_rx) = oneshot::channel();

    shutdown.add_worker_shutdown(
        dbsize_worker_shutdown_tx,
        spawn(unsafe{super::dbsize::DBSizeWorker::new().run(dbsize_worker_shutdown_rx)}),
    );

    let (tps_worker_shutdown_tx, tps_worker_shutdown_rx) = oneshot::channel();

    shutdown.add_worker_shutdown(
        tps_worker_shutdown_tx,
        spawn(unsafe{super::tps::TpsWorker::new().run(tps_worker_shutdown_rx)}),
    );
}

#[actix_rt::main]
pub async fn init() -> std::io::Result<()> {

    let (tx, rx) = mpsc::channel();

    println!("url: {}", "hey");

    let srvmon = ServerMonitor { listeners: vec![] }.start();

    thread::spawn(move || {
        let sys = System::new("http-server");

        let srv = HttpServer::new(move || {
            App::new()
                .data(srvmon.clone())
                // websocket route
                .service(web::resource("/ws").route(web::get().to(ws_index)))
                // static files
                .service(web::resource("/{_:.*}").route(web::get().to(static_files::ui)))
        })
        .bind("0.0.0.0:8081")?
        .shutdown_timeout(60) // <- Set shutdown timeout to 60 seconds
        .run();

        let _ = tx.send(srv);
        sys.run()
    });

    let srv = rx.recv().unwrap();

    Ok(())
}
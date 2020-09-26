use std::time::{Duration, Instant};
use actix_web::{web, App, Error, HttpRequest, HttpResponse, HttpServer};
use actix_web_actors::ws;
use actix::prelude::*;
use bee_common_ext::shutdown_tokio::Shutdown;
use std::sync::mpsc as sync_mpsc;
use std::{ptr, thread};
use async_std::task::{block_on};
use log::{warn};
use crate::controller::static_files;

pub const SYNC_STATUS: u8 = 0;
pub const STATUS: u8 = 1;
pub const TPS_METRICS: u8 = 2;
pub const TIP_SEL_METRIC: u8 = 3;
pub const TX_ZERO_VALUE: u8 = 4;
pub const TX_VALUE: u8 = 4;
pub const MS: u8 = 5;
pub const PEER_METRIC: u8 = 6;
pub const CONFIRMED_MS_METRICS: u8 = 7;
pub const VERTEX: u8 = 8;
pub const SOLID_INFO: u8 = 9;
pub const CONFIRMED_INFO: u8 = 10;
pub const MILESTONE_INFO: u8 = 11;
pub const TIP_INFO: u8 = 12;
pub const DB_SIZE_METRIC: u8 = 13;
pub const DB_CLEANUP: u8 = 14;
pub const SPAM_METRICS: u8 = 15;
pub const AVG_SPAM_METRICS: u8 = 16;

static mut LISTENERS: Vec<Addr<DashboardWebSocket>> = vec![];

pub unsafe fn get_listeners() -> &'static Vec<Addr<DashboardWebSocket>>{
    &LISTENERS
}

static mut DASHBOARD: *const Dashboard = ptr::null();

pub struct Dashboard {
}

impl Dashboard {
    pub async fn init(shutdown: &mut Shutdown) {
        if unsafe { !DASHBOARD.is_null() } {
            warn!("Already initialized.");
            return;
        }
        
        let dashboard = Dashboard {
        };

        unsafe {
            DASHBOARD = Box::leak(dashboard.into()) as *const _;
        }
    }

    pub fn get() -> &'static Dashboard {
        if unsafe { DASHBOARD.is_null() } {
            panic!("Uninitialized dashboard.");
        } else {
            unsafe { &*DASHBOARD }
        }
    }
}

/// do websocket handshake and start `DashboardSocket` actor
pub async fn ws_index(r: HttpRequest, stream: web::Payload, data: web::Data<Addr<ServerMonitor>>) -> Result<HttpResponse, Error> {
    let (addr, res) = ws::start_with_addr(DashboardWebSocket::new(), &r, stream)?;
    unsafe {
        LISTENERS.push(addr);
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
        match msg {
            Ok(ws::Message::Ping(msg)) => {
                self.hb = Instant::now();
                ctx.pong(&msg);
            }
            Ok(ws::Message::Pong(_)) => {
                self.hb = Instant::now();
            }
            Ok(ws::Message::Text(text)) => {
                println!("WS: {:?}", text);
                ctx.text(text)
            },
            Ok(ws::Message::Binary(bin)) => {
                println!("WS: {:?}", bin);
                ctx.binary(bin)
            },
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

pub fn init(shutdown: &mut Shutdown) -> (){

    block_on(Dashboard::init(shutdown));

    init_actix();

}

#[actix_rt::main]
pub async fn init_actix() -> std::io::Result<()> {

    let (tx, rx) = sync_mpsc::channel();

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
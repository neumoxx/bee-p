use actix_web::{web, App, HttpServer};
use std::io;

mod controller;
use controller::static_files;

#[actix_rt::main]
async fn main() -> io::Result<()> {

  HttpServer::new(move || {
    App::new()
      .service(web::resource("/{_:.*}").route(web::get().to(static_files::ui)))
  })
  .bind("0.0.0.0:8081")?
  .run()
  .await
}

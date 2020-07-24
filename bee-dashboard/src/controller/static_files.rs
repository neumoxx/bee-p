use actix_web::body::Body;
use actix_web::{HttpRequest, HttpResponse};
use mime_guess::from_path;
use rust_embed::RustEmbed;
use std::borrow::Cow;

#[derive(RustEmbed)]
#[folder = "frontend/build/"]
struct UIAsset;

fn handle_static_file<E: RustEmbed>(path: &str) -> HttpResponse {

  let path_parsed =  &path.replace("app/", "")[..];

  match E::get(path_parsed) {
    Some(content) => {
      let body: Body = match content {
        Cow::Borrowed(bytes) => bytes.into(),
        Cow::Owned(bytes) => bytes.into(),
      };
      HttpResponse::Ok()
        .content_type(from_path(path_parsed).first_or_octet_stream().as_ref())
        .body(body)
    }
    None => handle_static_file::<UIAsset>("index.html"),
  }
}

pub fn ui(req: HttpRequest) -> HttpResponse {
  let path = &req.path()["/".len()..];
  handle_static_file::<UIAsset>(path)
}

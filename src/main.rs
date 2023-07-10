mod provider;
mod routes;
mod util;

use std::convert::Infallible;
use std::env;
use std::sync::Arc;

use hyper::{service, Body, Request, Response, Server, header};
use tracing::{error, info};

#[tokio::main]
async fn main() {
  tracing_subscriber::fmt::init();

  let port = match env::var("PORT").map_or(Ok(80), |p| p.parse()) {
    Ok(port) => port,
    Err(err) => return error!("invalid port: {err}"),
  };

  let addr = ([0, 0, 0, 0], port).into();
  let providers = Arc::new(provider::s());
  let make_service = service::make_service_fn(|_| {
    let providers = providers.clone();
    async move {
      Ok::<_, Infallible>(service::service_fn(move |req| {
        let providers = providers.clone();
        async move { Ok::<_, Infallible>(handle_request(providers, req).await) }
      }))
    }
  });

  let server = Server::bind(&addr).serve(make_service);

  info!("listening on {addr}");

  if let Err(err) = server.await {
    error!("server died: {err}");
  }
}

async fn handle_request(providers: Arc<provider::Map>, req: Request<Body>) -> Response<Body> {
  match req.uri().path() {
    "/" => routes::root(),
    "/api/ask" => routes::ask(providers, req).await,
    "/index.min.css" => Response::builder().header(header::CONTENT_TYPE, "text/css").body(Body::from(include_str!("../static/index.min.css"))).unwrap(),
    "/index.min.js" => Response::builder().header(header::CONTENT_TYPE, "text/javascript").body(Body::from(include_str!("../static/index.min.js"))).unwrap(),
    _ => routes::default(),
  }
}

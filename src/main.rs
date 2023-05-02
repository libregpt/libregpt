use std::convert::Infallible;
use hyper::{Body, Request, Response, Server, service};
use tracing::{error, info};

#[tokio::main]
async fn main() {
  tracing_subscriber::fmt::init();

  let addr = "0.0.0.0:80".parse().unwrap();
  let make_service = service::make_service_fn(|_| async move {
    Ok::<_, Infallible>(service::service_fn(|_: Request<Body>| async move {
      Ok::<_, Infallible>(Response::new(Body::from("pp")))
    }))
  });

  let server = Server::bind(&addr).serve(make_service);

  info!("listening on {addr}");

  if let Err(err) = server.await {
    error!("server died: {err}");
  }
}

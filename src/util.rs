use std::io::{Error as IoError, ErrorKind as IoErrorKind};
use std::pin::Pin;
use std::task::{Context, Poll};

use futures_core::Stream;
use hyper::body::{self, Body, HttpBody};
use pin_project::pin_project;

#[pin_project]
pub struct BodyStream {
  #[pin]
  body: Body,
}

impl Stream for BodyStream {
  type Item = Result<body::Bytes, IoError>;

  fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
    self
      .project()
      .body
      .poll_data(cx)
      .map_err(|err| IoError::new(IoErrorKind::Other, err))
  }
}

impl From<Body> for BodyStream {
  fn from(body: Body) -> Self {
    Self { body }
  }
}

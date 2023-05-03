mod bai;

use std::borrow::Cow;
use std::collections::HashMap;

use async_trait::async_trait;
use hyper::Body;

#[async_trait]
pub trait Provider: Send + Sync {
  async fn ask<'a>(
    &self,
    prompt: &str,
    parent_msg_id: Option<Cow<'a, str>>,
  ) -> anyhow::Result<(String, Body)>;
}

pub type Map = HashMap<&'static str, Box<dyn Provider>>;

pub fn s() -> Map {
  let mut providers = HashMap::new();

  providers.insert("bai", Box::new(bai::Provider::new()) as Box<dyn Provider>);

  providers
}

mod bai;
mod deepai;
mod you;

use std::borrow::Cow;
use std::collections::HashMap;

use async_trait::async_trait;
use hyper::Body;

#[async_trait]
pub trait Provider: Send + Sync {
  async fn ask<'a>(
    &self,
    prompt: &str,
    state: Option<Cow<'a, str>>,
  ) -> anyhow::Result<(Option<String>, Body)>;
}

pub type Map = HashMap<&'static str, Box<dyn Provider>>;

pub fn s() -> Map {
  let mut providers = HashMap::new();

  providers.insert("bai", Box::new(bai::Provider::new()) as Box<dyn Provider>);
  providers.insert("deepai", Box::new(deepai::Provider::new()) as Box<dyn Provider>);
  providers.insert("you", Box::new(you::Provider::new()) as Box<dyn Provider>);

  providers
}

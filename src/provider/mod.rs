mod ava;
mod bai;
mod deepai;
mod you;

use std::collections::HashMap;

use async_trait::async_trait;
use hyper::Body;

#[async_trait]
pub trait Provider: Send + Sync {
  async fn ask<'a>(
    &self,
    prompt: &str,
    state: Option<&str>,
  ) -> anyhow::Result<(Option<String>, Body)>;
}

pub type Map = HashMap<&'static str, Box<dyn Provider>>;

pub fn s() -> Map {
  let mut providers = HashMap::new();

  providers.insert("ava", Box::new(ava::Provider::new()) as Box<dyn Provider>);
  providers.insert("bai", Box::new(bai::Provider::new()) as Box<dyn Provider>);
  providers.insert(
    "deepai",
    Box::new(deepai::Provider::new()) as Box<dyn Provider>,
  );
  providers.insert("you", Box::new(you::Provider::new()) as Box<dyn Provider>);

  providers
}

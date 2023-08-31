use std::collections::{HashMap, HashSet};
use std::rc::Rc;

use time::OffsetDateTime;
use yew::Reducible;

use crate::ui::utils::format_date_time;

#[derive(Clone, PartialEq)]
pub struct Conversation {
  pub created_at: OffsetDateTime,
  pub provider: Rc<str>,
  pub messages: Vec<String>,
  pub updating_last_msg: bool,
  pub last_msg_id: Option<String>,
}

impl Conversation {
  fn new(created_at: OffsetDateTime, provider: Rc<str>) -> Self {
    Self {
      created_at,
      provider,
      messages: Vec::new(),
      updating_last_msg: false,
      last_msg_id: None,
    }
  }
}

#[derive(PartialEq)]
pub struct Conversations {
  default_provider: Rc<str>,
  pub inner: HashMap<Rc<str>, Conversation>,
  pub curr_name: Rc<str>,
}

impl Conversations {
  pub fn new(default_provider: &str) -> Self {
    let default_provider = Rc::<str>::from(default_provider);
    let now = OffsetDateTime::now_utc();
    let first_conv_name: Rc<str> = format_date_time(now).into();

    Self {
      default_provider: default_provider.clone(),
      inner: HashMap::from([(
        first_conv_name.clone(),
        Conversation::new(now, default_provider),
      )]),
      curr_name: first_conv_name,
    }
  }

  pub fn name_set(&self) -> HashSet<Rc<str>> {
    HashSet::from_iter(self.inner.keys().cloned())
  }

  pub fn names(&self) -> Vec<Rc<str>> {
    let mut conv_names = self
      .inner
      .iter()
      .map(|(name, conv)| (name.clone(), conv.created_at))
      .collect::<Vec<_>>();
    conv_names.sort_by(|a, b| a.1.cmp(&b.1));

    conv_names.into_iter().map(|(name, _)| name).collect()
  }

  pub fn current(&self) -> &Conversation {
    self.inner.get(&self.curr_name).unwrap()
  }
}

impl Reducible for Conversations {
  type Action = ConversationsAction;

  fn reduce(self: Rc<Self>, action: Self::Action) -> Rc<Self> {
    let mut curr_name = self.curr_name.clone();
    let inner = match action {
      Self::Action::CreateConversation => {
        let mut inner = self.inner.clone();
        let now = OffsetDateTime::now_utc();
        let name: Rc<str> = format_date_time(now).into();

        inner.insert(
          name.clone(),
          Conversation::new(now, self.default_provider.clone()),
        );

        curr_name = name;

        inner
      }
      Self::Action::DeleteConversation(name, i) => {
        let mut inner = self.inner.clone();

        if inner.len() == 1 {
          let now = OffsetDateTime::now_utc();
          let name: Rc<str> = format_date_time(now).into();

          inner.insert(
            name.clone(),
            Conversation::new(now, self.default_provider.clone()),
          );
          curr_name = name;
        } else if name == self.curr_name {
          let conv_names = self.names();
          curr_name = conv_names
            .get(i + 1)
            .unwrap_or_else(|| conv_names.get(i - 1).unwrap())
            .clone();
        }

        inner.remove(&name);

        inner
      }
      Self::Action::PushMessage(name, mut msg) => {
        msg.push('\n');

        let mut inner = self.inner.clone();
        let conv = inner.get_mut(&name).unwrap();

        conv.messages.reserve_exact(2);
        conv.messages.push(msg);
        conv.messages.push("\n".to_owned());

        inner
      }
      Self::Action::SetCurrentName(name) => {
        curr_name = name;
        self.inner.clone()
      }
      Self::Action::SetLastMessageId(name, last_msg_id) => {
        let mut inner = self.inner.clone();
        let conv = inner.get_mut(&name).unwrap();
        let _ = conv.last_msg_id.insert(last_msg_id);

        inner
      }
      Self::Action::SetProvider(provider) => {
        let mut inner = self.inner.clone();
        let conv = inner.get_mut(&self.curr_name).unwrap();

        conv.provider = provider.into();

        inner
      }
      Self::Action::SetUpdatingLastMessage(name, updating_last_msg) => {
        let mut inner = self.inner.clone();

        if let Some(conv) = inner.get_mut(&name) {
          conv.updating_last_msg = updating_last_msg;
        }

        inner
      }
      Self::Action::UpdateLastMessage(name, char) => {
        let mut inner = self.inner.clone();

        if let Some(conv) = inner.get_mut(&name) {
          let last = conv.messages.last_mut().unwrap();

          last.pop();
          last.push(char);
          last.push('\n');
        }

        inner
      }
    };

    Self {
      default_provider: self.default_provider.clone(),
      inner,
      curr_name,
    }
    .into()
  }
}

pub enum ConversationsAction {
  CreateConversation,
  DeleteConversation(Rc<str>, usize),
  PushMessage(Rc<str>, String),
  SetCurrentName(Rc<str>),
  SetLastMessageId(Rc<str>, String),
  SetProvider(String),
  SetUpdatingLastMessage(Rc<str>, bool),
  UpdateLastMessage(Rc<str>, char),
}

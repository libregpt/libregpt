use std::collections::{HashMap, HashSet};
use std::rc::Rc;

use time::OffsetDateTime;
use uuid::Uuid;
use yew::Reducible;

#[derive(Clone, PartialEq)]
pub struct Conversation {
  pub created_at: OffsetDateTime,
  pub name: Rc<str>,
  pub provider: Rc<str>,
  pub messages: Vec<String>,
  pub updating_last_msg: bool,
  pub last_msg_id: Option<String>,
}

impl Conversation {
  fn new(provider: Rc<str>) -> Self {
    let now = OffsetDateTime::now_utc();
    let name = now
      .format(time::macros::format_description!(
        "[year]-[month]-[day] [hour]:[minute]:[second]"
      ))
      .unwrap();

    Self {
      created_at: now,
      name: name.into(),
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
  pub inner: HashMap<Uuid, Conversation>,
  pub current_id: Uuid,
}

impl Conversations {
  pub fn new(default_provider: &str) -> Self {
    let default_provider = Rc::<str>::from(default_provider);
    let first_id = Uuid::new_v4();

    Self {
      default_provider: default_provider.clone(),
      inner: HashMap::from([(first_id, Conversation::new(default_provider))]),
      current_id: first_id,
    }
  }

  pub fn ids(&self) -> HashSet<Uuid> {
    HashSet::from_iter(self.inner.keys().copied())
  }

  pub fn sorted_ids(&self) -> Vec<Uuid> {
    let mut ids = self
      .inner
      .iter()
      .map(|(id, conv)| (id, conv.created_at))
      .collect::<Vec<_>>();

    ids.sort_by(|a, b| a.1.cmp(&b.1));

    ids.into_iter().map(|(&id, _)| id).collect()
  }

  pub fn names(&self) -> impl Iterator<Item = (Uuid, Rc<str>)> + '_ {
    let mut names = self
      .inner
      .iter()
      .map(|(id, conv)| (id, conv.name.clone(), conv.created_at))
      .collect::<Vec<_>>();

    names.sort_by(|a, b| a.2.cmp(&b.2));

    names.into_iter().map(|(&id, name, _)| (id, name))
  }

  pub fn current(&self) -> &Conversation {
    self.inner.get(&self.current_id).unwrap()
  }

  pub fn get(&self, id: &Uuid) -> &Conversation {
    self.inner.get(id).unwrap()
  }
}

impl Reducible for Conversations {
  type Action = ConversationsAction;

  fn reduce(self: Rc<Self>, action: Self::Action) -> Rc<Self> {
    let mut current_id = self.current_id;
    let inner = match action {
      Self::Action::CreateConversation => {
        let mut inner = self.inner.clone();
        let id = Uuid::new_v4();

        inner.insert(id, Conversation::new(self.default_provider.clone()));

        current_id = id;

        inner
      }
      Self::Action::DeleteConversation(id, i) => {
        let mut inner = self.inner.clone();

        if inner.len() == 1 {
          let id = Uuid::new_v4();

          inner.insert(id, Conversation::new(self.default_provider.clone()));

          current_id = id;
        } else if id == self.current_id {
          let sorted_ids = self.sorted_ids();

          current_id = sorted_ids
            .get(i + 1)
            .unwrap_or_else(|| sorted_ids.get(i - 1).unwrap())
            .clone();
        }

        inner.remove(&id);

        inner
      }
      Self::Action::PushMessage(id, mut msg) => {
        msg.push('\n');

        let mut inner = self.inner.clone();
        let conv = inner.get_mut(&id).unwrap();

        conv.messages.reserve_exact(2);
        conv.messages.push(msg);
        conv.messages.push("\n".to_owned());

        inner
      }
      Self::Action::SetCurrentId(id) => {
        current_id = id;
        self.inner.clone()
      }
      Self::Action::SetLastMessageId(id, last_msg_id) => {
        let mut inner = self.inner.clone();
        let conv = inner.get_mut(&id).unwrap();
        let _ = conv.last_msg_id.insert(last_msg_id);

        inner
      }
      Self::Action::SetProvider(provider) => {
        let mut inner = self.inner.clone();
        let conv = inner.get_mut(&self.current_id).unwrap();

        conv.provider = provider.into();

        inner
      }
      Self::Action::SetUpdatingLastMessage(id, updating_last_msg) => {
        let mut inner = self.inner.clone();

        if let Some(conv) = inner.get_mut(&id) {
          conv.updating_last_msg = updating_last_msg;
        }

        inner
      }
      Self::Action::UpdateLastMessage(id, char) => {
        let mut inner = self.inner.clone();

        if let Some(conv) = inner.get_mut(&id) {
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
      current_id,
    }
    .into()
  }
}

pub enum ConversationsAction {
  CreateConversation,
  DeleteConversation(Uuid, usize),
  PushMessage(Uuid, String),
  SetCurrentId(Uuid),
  SetLastMessageId(Uuid, String),
  SetProvider(String),
  SetUpdatingLastMessage(Uuid, bool),
  UpdateLastMessage(Uuid, char),
}

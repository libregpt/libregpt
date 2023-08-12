use std::collections::HashMap;
use std::rc::Rc;
use time::OffsetDateTime;
use yew::Reducible;

#[derive(Clone, PartialEq)]
pub struct Conversation {
  pub created_at: OffsetDateTime,
  pub provider: String,
  pub messages: Vec<String>,
  pub updating_last_msg: bool,
  pub last_msg_id: Option<String>,
}

impl Conversation {
  fn new(created_at: OffsetDateTime, provider: String) -> Self {
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
  default_provider: String,
  pub inner: HashMap<String, Conversation>,
}

impl Conversations {
  pub fn new(first_conv_name: String, now: OffsetDateTime, default_provider: String) -> Self {
    Self {
      default_provider: default_provider.clone(),
      inner: HashMap::from([(first_conv_name, Conversation::new(now, default_provider))]),
    }
  }
}

impl Reducible for Conversations {
  type Action = ConversationsAction;

  fn reduce(self: Rc<Self>, action: Self::Action) -> Rc<Self> {
    let inner = match action {
      Self::Action::CreateConversation(name, now) => {
        let mut inner = self.inner.clone();

        inner.insert(name, Conversation::new(now, self.default_provider.clone()));

        inner
      },
      Self::Action::PushMessage(name, mut msg) => {
        msg.push('\n');

        let mut inner = self.inner.clone();
        let conv = inner.get_mut(name.as_ref()).unwrap();

        conv.messages.reserve_exact(2);
        conv.messages.push(msg);
        conv.messages.push("\n".to_owned());

        inner
      },
      Self::Action::SetLastMessageId(name, last_msg_id) => {
        let mut inner = self.inner.clone();
        let conv = inner.get_mut(name.as_ref()).unwrap();
        let _ = conv.last_msg_id.insert(last_msg_id);

        inner
      },
      Self::Action::SetProvider(name, provider) => {
        let mut inner = self.inner.clone();
        let conv = inner.get_mut(name.as_ref()).unwrap();

        conv.provider = provider;

        inner
      },
      Self::Action::SetUpdatingLastMessage(name, updating_last_msg) => {
        let mut inner = self.inner.clone();
        let conv = inner.get_mut(name.as_ref()).unwrap();

        conv.updating_last_msg = updating_last_msg;

        inner
      },
      Self::Action::UpdateLastMessage(name, char) => {
        let mut inner = self.inner.clone();
        let conv = inner.get_mut(name.as_ref()).unwrap();
        let last = conv.messages.last_mut().unwrap();

        last.pop();
        last.push(char);
        last.push('\n');

        inner
      },
    };

    Self {
      default_provider: self.default_provider.clone(),
      inner,
    }.into()
  }
}

pub enum ConversationsAction {
  CreateConversation(String, OffsetDateTime),
  PushMessage(Rc<str>, String),
  SetLastMessageId(Rc<str>, String),
  SetProvider(Rc<str>, String),
  SetUpdatingLastMessage(Rc<str>, bool),
  UpdateLastMessage(Rc<str>, char),
}

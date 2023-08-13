use std::rc::Rc;
use yew::{function_component, Html, html, Properties};

#[derive(Properties, PartialEq)]
pub struct MessageProps {
  pub index: usize,
  pub content: Rc<str>,
}

#[function_component]
pub fn Message(props: &MessageProps) -> Html {
  let mut container_class = "flex".to_owned();
  let mut bubble_class = "rounded-xl bg-[#F5F5F5] dark:bg-[#292929] break-words max-w-full flex flex-col gap-3".to_owned();

  if props.index % 2 == 0 {
    container_class.push_str(" justify-end");
    bubble_class.push_str(" bg-[#FF983F] dark:bg-[#FF7A1F]");
  }

  let mut lines = props.content.lines();
  lines.next();

  if lines.next().is_some() {
    bubble_class.push_str(" px-4 py-3.5");
  } else {
    bubble_class.push_str(" px-3 py-2.5");
  }

  html! {
    <div class={container_class}>
      <div class={bubble_class}>
        {prose::process(props.content.as_ref())}
      </div>
    </div>
  }
}

use web_sys::{HtmlElement, MouseEvent, window};
use yew::{Callback, function_component, Html, html, TargetCast};

#[function_component]
pub fn ThemeSwitcher() -> Html {
  let onclick = Callback::from(move |e: MouseEvent| {
    let target: HtmlElement = e.target_unchecked_into();
    let window = window().unwrap();
    let document_class_list = window.document().unwrap().document_element().unwrap().class_list();
    let local_storage = window.local_storage().unwrap().unwrap();

    if let Some(theme) = target.dataset().get("theme") {
      local_storage.set_item("theme", &theme).unwrap();

      if theme == "dark" {
        document_class_list.add_1("dark").unwrap();
      } else {
        document_class_list.remove_1("dark").unwrap();
      }
    } else {
      local_storage.remove_item("theme").unwrap();

      if window.match_media("(prefers-color-scheme: dark)").unwrap().is_some_and(|list| list.matches()) {
        document_class_list.add_1("dark").unwrap();
      } else {
        document_class_list.remove_1("dark").unwrap();
      }
    }

    let parent = target.parent_element().unwrap();
    let parent_children = parent.children();
    let target = target.into();
    let mut i = 0;

    while let Some(button) = parent_children.item(i) {
      button.set_attribute("aria-checked", &button.is_same_node(Some(&target)).to_string()).unwrap();
      i += 1;
    }
  });

  html! {
    <div class="px-3 py-2.5 rounded-xl bg-[#F5F5F5] dark:bg-[#292929] flex gap-2" role="radiogroup">
      <button type="button" role="radio" class="theme-switcher-button" title="Dark" data-theme="dark" onclick={onclick.clone()}>
        <svg viewBox="0 0 24 24" shape-rendering="geometricPrecision" stroke-linecap="round" stroke-linejoin="round" stroke-width="2.5px">
          <path d="M21 12.79A9 9 0 1111.21 3 7 7 0 0021 12.79z"></path>
        </svg>
      </button>
      <button type="button" role="radio" class="theme-switcher-button" title="OS" onclick={onclick.clone()}>
        <svg viewBox="0 0 24 24" shape-rendering="geometricPrecision" stroke-linecap="round" stroke-linejoin="round" stroke-width="2.5px">
          <rect x="2" y="3" width="20" height="14" rx="2" ry="2"></rect>
          <path d="M8 21h8"></path>
          <path d="M12 17v4"></path>
        </svg>
      </button>
      <button type="button" role="radio" class="theme-switcher-button" title="Light" data-theme="light" {onclick}>
        <svg viewBox="0 0 24 24" shape-rendering="geometricPrecision" stroke-linecap="round" stroke-linejoin="round" stroke-width="2.5px">
          <circle cx="12" cy="12" r="5"></circle>
          <path d="M12 1v2"></path>
          <path d="M12 21v2"></path>
          <path d="M4.22 4.22l1.42 1.42"></path>
          <path d="M18.36 18.36l1.42 1.42"></path>
          <path d="M1 12h2"></path>
          <path d="M21 12h2"></path>
          <path d="M4.22 19.78l1.42-1.42"></path>
          <path d="M18.36 5.64l1.42-1.42"></path>
        </svg>
      </button>
    </div>
  }
}

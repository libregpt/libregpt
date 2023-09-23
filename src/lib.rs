mod ui;

use std::iter;

use futures_util::StreamExt;
use gloo_timers::future::TimeoutFuture;
use serde::Serialize;
use wasm_bindgen::JsCast;
use wasm_streams::ReadableStream;
use web_sys::{
  window, Event, HtmlElement, HtmlInputElement, HtmlOptionElement, HtmlSelectElement,
  HtmlTextAreaElement, TextDecodeOptions, TextDecoder,
};
use yew::events::{KeyboardEvent, SubmitEvent};
use yew::{
  function_component, html, use_effect_with_deps, use_mut_ref, use_node_ref, use_reducer,
  use_state, Callback, Html, TargetCast,
};

use crate::ui::components::{Message, ThemeSwitcher};
use crate::ui::reducers::{Conversations, ConversationsAction};
use crate::ui::utils::{close_sidebar as close_sidebar_fn, set_scroll_top_to_scroll_height};

const PROVIDERS: &[(&str, &str, bool)] = &[
  ("ava", "Ava (gpt-3.5-turbo-0613)", true),
  ("bai", "BAI (gpt-3.5)", true),
  ("deepai", "DeepAI (gpt-3)", false),
  ("you", "You", false),
];

#[function_component]
pub fn App() -> Html {
  let submit_ref = use_node_ref();
  let onkeypress = {
    let submit_ref = submit_ref.clone();

    Callback::from(move |e: KeyboardEvent| {
      if e.key_code() == 13 && !e.shift_key() {
        e.prevent_default();
        submit_ref.cast::<HtmlElement>().unwrap().click();
      }
    })
  };

  let prompt_ref = use_node_ref();
  let messages_ref = use_node_ref();
  let oninput = {
    let prompt_ref = prompt_ref.clone();
    let messages_ref = messages_ref.clone();

    Callback::from(move |_| {
      let prompt_el: HtmlElement = prompt_ref.cast().unwrap();
      let prompt_style = prompt_el.style();

      prompt_style.set_css_text("height: auto; padding: 0;");
      prompt_style.set_css_text(&format!("height: {}px;", prompt_el.scroll_height()));

      set_scroll_top_to_scroll_height(&messages_ref);
    })
  };

  let conversations =
    use_reducer(|| Conversations::new(PROVIDERS.iter().find(|p| !p.2).unwrap().0));

  let mut_conversations = use_mut_ref(|| (conversations.ids(), conversations.current_id));

  {
    let conversations_2 = conversations.clone();
    let mut_conversations = mut_conversations.clone();

    use_effect_with_deps(
      move |_| {
        *mut_conversations.borrow_mut() = (conversations_2.ids(), conversations_2.current_id);
      },
      conversations.clone(),
    );
  }

  let onsubmit = {
    let prompt_ref = prompt_ref.clone();
    let messages_ref = messages_ref.clone();
    let conversations = conversations.clone();

    Callback::from(move |e: SubmitEvent| {
      e.prevent_default();

      let prompt_el: HtmlTextAreaElement = prompt_ref.cast().unwrap();
      let prompt_val = prompt_el.value();

      if prompt_val.is_empty() {
        return;
      }

      let task_conv_id = conversations.current_id;

      conversations.dispatch(ConversationsAction::SetUpdatingLastMessage(
        task_conv_id,
        true,
      ));
      prompt_el.set_value("");
      prompt_el
        .dispatch_event(&Event::new("input").unwrap())
        .unwrap();
      conversations.dispatch(ConversationsAction::PushMessage(
        task_conv_id,
        prompt_val.clone(),
      ));
      set_scroll_top_to_scroll_height(&messages_ref);

      let mut url = window().unwrap().location().origin().unwrap();
      url.push_str("/api/ask");

      let conv = conversations.get(&task_conv_id);
      let provider = conv.provider.clone();
      let state = match provider.as_ref() {
        "ava" | "deepai" => {
          if conv.messages.is_empty() {
            None
          } else {
            Some(
              serde_json::to_string(
                &conv
                  .messages
                  .iter()
                  .enumerate()
                  .map(|(i, msg)| DeepAiMessage {
                    role: if i % 2 == 0 { "user" } else { "assistant" },
                    content: msg,
                  })
                  .collect::<Vec<_>>(),
              )
              .unwrap(),
            )
          }
        }
        "bai" => conv.last_msg_id.clone(),
        "you" => {
          if conv.messages.is_empty() {
            None
          } else if let Some(last_msg_id) = conv.last_msg_id.as_ref() {
            let chat = serde_json::to_string(
              &conv
                .messages
                .chunks(2)
                .map(|chunk| YouMessage {
                  question: &chunk[0],
                  answer: &chunk[1],
                })
                .collect::<Vec<_>>(),
            )
            .unwrap();

            let mut state = String::with_capacity(last_msg_id.len() + chat.len());
            state.push_str(last_msg_id);
            state.push_str(&chat);

            Some(state)
          } else {
            None
          }
        }
        _ => unreachable!(),
      };

      let conversations = conversations.clone();
      let mut_conversations = mut_conversations.clone();
      let messages_ref = messages_ref.clone();

      wasm_bindgen_futures::spawn_local(async move {
        let mut params = Vec::with_capacity(3);
        params.push(("provider", provider.as_ref()));
        params.push(("prompt", prompt_val.as_str()));

        if let Some(state) = state.as_deref() {
          params.push(("state", state));
        }

        let res = gloo_net::http::Request::get(&url)
          .query(params)
          .send()
          .await
          .unwrap();

        if res.ok() {
          if let Some(msg_id) = res.headers().get("msg-id") {
            conversations.dispatch(ConversationsAction::SetLastMessageId(task_conv_id, msg_id));
          }
        }

        let decoder = TextDecoder::new().unwrap();

        let mut decode_options = TextDecodeOptions::new();
        decode_options.stream(true);

        let mut stream =
          ReadableStream::from_raw(res.body().unwrap().dyn_into().unwrap()).into_stream();

        'outer: while let Some(Ok(chunk)) = stream.next().await {
          let chunk = decoder
            .decode_with_buffer_source_and_options(&js_sys::Object::from(chunk), &decode_options)
            .unwrap();

          for char in chunk.chars() {
            if !mut_conversations.borrow().0.contains(&task_conv_id) {
              break 'outer;
            }

            conversations.dispatch(ConversationsAction::UpdateLastMessage(task_conv_id, char));

            if task_conv_id == mut_conversations.borrow().1 {
              set_scroll_top_to_scroll_height(&messages_ref);
            }

            TimeoutFuture::new(15).await;
          }
        }

        conversations.dispatch(ConversationsAction::SetUpdatingLastMessage(
          task_conv_id,
          false,
        ));
      });
    })
  };

  let sidebar_ref = use_node_ref();
  let overlay_ref = use_node_ref();
  let invisible_overlay_ref = use_node_ref();
  let open_sidebar = {
    let sidebar_ref = sidebar_ref.clone();
    let overlay_ref = overlay_ref.clone();
    let invisible_overlay_ref = invisible_overlay_ref.clone();

    Callback::from(move |_| {
      let sidebar_el: HtmlElement = sidebar_ref.cast().unwrap();
      let overlay_el: HtmlElement = overlay_ref.cast().unwrap();
      let invisible_overlay_el: HtmlElement = invisible_overlay_ref.cast().unwrap();

      sidebar_el.style().set_css_text("transform: translateX(0);");
      overlay_el
        .style()
        .set_css_text("background-color: rgba(0, 0, 0, 0.5);");
      invisible_overlay_el.style().set_css_text("display: block;");
    })
  };

  let close_sidebar = {
    let sidebar_ref = sidebar_ref.clone();
    let overlay_ref = overlay_ref.clone();
    let invisible_overlay_ref = invisible_overlay_ref.clone();

    Callback::from(move |_| close_sidebar_fn(&sidebar_ref, &overlay_ref, &invisible_overlay_ref))
  };

  let set_provider = {
    let conversations = conversations.clone();

    Callback::from(move |e: Event| {
      let provider_el: HtmlSelectElement = e.target_unchecked_into();

      conversations.dispatch(ConversationsAction::SetProvider(provider_el.value()));
    })
  };

  let conversations_ref = use_node_ref();
  let provider_ref = use_node_ref();
  let create_conv = {
    let conversations = conversations.clone();
    let conversations_ref = conversations_ref.clone();
    let provider_ref = provider_ref.clone();
    let sidebar_ref = sidebar_ref.clone();
    let overlay_ref = overlay_ref.clone();
    let invisible_overlay_ref = invisible_overlay_ref.clone();

    Callback::from(move |_| {
      conversations.dispatch(ConversationsAction::CreateConversation);

      let provider_el: HtmlSelectElement = provider_ref.cast().unwrap();

      provider_el.set_selected_index(PROVIDERS.iter().position(|p| !p.2).unwrap() as i32);

      set_scroll_top_to_scroll_height(&conversations_ref);
      close_sidebar_fn(&sidebar_ref, &overlay_ref, &invisible_overlay_ref);
    })
  };

  let curr_conv_name_ref = use_node_ref();
  let editing_name = use_state(|| false);
  let edit_curr_conv_name = {
    let curr_conv_name_ref = curr_conv_name_ref.clone();
    let editing_name = editing_name.clone();
    let conversations = conversations.clone();

    Callback::from(move |e: SubmitEvent| {
      e.prevent_default();

      let curr_conv_name_el: HtmlInputElement = curr_conv_name_ref.cast().unwrap();

      if *editing_name {
        editing_name.set(false);
        conversations.dispatch(ConversationsAction::SetCurrentConversationName(
          curr_conv_name_el.value(),
        ));
        curr_conv_name_el.set_disabled(true);
      } else {
        editing_name.set(true);
        curr_conv_name_el.set_disabled(false);
        curr_conv_name_el.focus().unwrap();
        curr_conv_name_el.select();
      }
    })
  };

  {
    let curr_conv_name_ref = curr_conv_name_ref.clone();
    let editing_name = editing_name.clone();
    let messages_ref = messages_ref.clone();
    let provider_ref = provider_ref.clone();

    use_effect_with_deps(
      {
        let conversations = conversations.clone();

        move |_| {
          if *editing_name {
            let curr_conv_name_el: HtmlInputElement = curr_conv_name_ref.cast().unwrap();

            editing_name.set(false);
            curr_conv_name_el.set_disabled(true);
          }

          let provider_el: HtmlSelectElement = provider_ref.cast().unwrap();
          let child_nodes = provider_el.child_nodes();
          let mut i = 0;
          let curr_conv = conversations.current();

          while let Some(node) = child_nodes.item(i) {
            if node.unchecked_into::<HtmlOptionElement>().value().as_str() == curr_conv.provider.as_ref() {
              provider_el.set_selected_index(i as i32);
              break;
            }
            i += 1;
          }

          set_scroll_top_to_scroll_height(&messages_ref);
        }
      },
      conversations.current_id,
    );
  }

  let curr_conv = conversations.current();

  html! {
    <div class="h-screen flex gap-4 lg:p-4 bg-[#E1E1E1] dark:bg-[#151515] text-[#333333] dark:text-[#F5F5F5]">
      <div class="absolute w-full h-full z-20 flex pointer-events-none lg:w-fit lg:relative">
        <div ref={sidebar_ref.clone()} class="p-5 pb-6 rounded-e-xl bg-[#EBEBEB] dark:bg-[#1A1A1A] pointer-events-auto flex flex-col gap-5 -translate-x-full md:w-[37%] lg:pb-5 lg:w-64 lg:translate-x-0 lg:rounded-s-xl">
          <div class="flex gap-3 items-center justify-between">
            <span class="px-2.5 py-2 rounded-xl bg-[#F5F5F5] dark:bg-[#292929] font-bold">{"LibreGPT"}</span>
            <ThemeSwitcher />
            <div class="w-10 h-10 rounded-xl bg-[#F5F5F5] dark:bg-[#292929] relative flex items-center justify-center cursor-pointer lg:hidden before:content-[''] before:absolute before:w-[0.1875rem] before:h-4 before:bg-current before:rounded before:rotate-45 after:content-[''] after:absolute after:w-[0.1875rem] after:h-4 after:bg-current after:rounded after:-rotate-45" onclick={close_sidebar.clone()}></div>
          </div>

          <div ref={conversations_ref} class="flex-1 flex flex-col gap-3 overflow-y-auto">
            {for conversations.names().enumerate().map(|(i, (id, name))| {
              let onclick = {
                let conversations = conversations.clone();
                let sidebar_ref = sidebar_ref.clone();
                let overlay_ref = overlay_ref.clone();
                let invisible_overlay_ref = invisible_overlay_ref.clone();

                Callback::from(move |_| {
                  conversations.dispatch(ConversationsAction::SetCurrentId(id));

                  close_sidebar_fn(&sidebar_ref, &overlay_ref, &invisible_overlay_ref);
                })
              };

              let mut hash_class = String::with_capacity(6 + 15);
              hash_class.push_str("mr-1.5");

              let mut trash_class = String::with_capacity(39 + 17);
              trash_class.push_str("w-4 hover:stroke-red-600 cursor-pointer");

              if id == conversations.current_id {
                hash_class.push_str(" text-[#C54A00]");
                trash_class.push_str(" stroke-[#C54A00]");
              } else {
                hash_class.push_str(" text-[#6D6D6D]");
                trash_class.push_str(" stroke-[#6D6D6D]");
              }

              let trash_onclick = {
                let conversations = conversations.clone();

                Callback::from(move |_| {
                  conversations.dispatch(ConversationsAction::DeleteConversation(id, i));
                })
              };

              html! {
                <div
                  key={id.to_string()}
                  class="rounded-xl bg-[#F5F5F5] dark:bg-[#292929] text-sm flex gap-3 justify-between items-center aria-selected:bg-[#FF983F] aria-selected:dark:bg-[#FF7A1F]"
                  aria-selected={(id == conversations.current_id).to_string()}
                >
                  <div class="w-full flex pl-2.5 py-2 cursor-pointer overflow-hidden text-ellipsis" {onclick}>
                    <span class={hash_class}>{"#"}</span>
                    <span class="whitespace-nowrap overflow-hidden text-ellipsis inline-block">{name}</span>
                  </div>
                  <div class="pr-2.5 py-2">
                    <svg viewBox="0 0 24 24" fill="none" stroke-width="2.5px" stroke-linecap="round" stroke-linejoin="round" class={trash_class} onclick={trash_onclick}>
                      <polyline points="3 6 5 6 21 6"></polyline>
                      <path d="M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6m3 0V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2"></path>
                      <line x1="12" y1="11" x2="12" y2="17"></line>
                    </svg>
                  </div>
                </div>
              }
            })}
          </div>

          <div>
            <button class="px-3 py-2.5 rounded-xl bg-[#F5F5F5] dark:bg-[#292929] text-sm flex justify-center cursor-pointer" onclick={create_conv}>{"+ New Conversation"}</button>
          </div>
        </div>
        <div ref={invisible_overlay_ref} class="flex-1 h-full pointer-events-auto hidden" onclick={close_sidebar}></div>
      </div>

      <div ref={overlay_ref} class="absolute w-full h-full z-10 pointer-events-none !transition-colors !duration-300 !ease-out lg:hidden"></div>

      <div class="flex-1 w-full p-5 pb-6 lg:pb-5 lg:rounded-xl xl:px-[13vw] bg-[#EBEBEB] dark:bg-[#1A1A1A] flex flex-col gap-5 items-center">
        <div class="w-full flex gap-3">
          <div class="px-3.5 py-3.5 rounded-xl bg-[#F5F5F5] dark:bg-[#292929] flex flex-col gap-[0.1875rem] cursor-pointer lg:hidden" onclick={open_sidebar}>
            {for iter::repeat(html! {
              <div class="w-4 h-[0.1875rem] rounded bg-current"></div>
            }).take(3)}
          </div>
          <form class="px-3 py-2 rounded-xl bg-[#F5F5F5] dark:bg-[#292929] flex gap-1.5 items-center w-full md:w-fit" onsubmit={edit_curr_conv_name}>
            <input ref={curr_conv_name_ref} type="text" required={true} disabled={true} value={curr_conv.name.clone()} class="w-full md:w-48 bg-transparent outline-none" />
            <button type="submit" class="fill-current hover:fill-[#FF7A1F]">
              <svg viewBox="0 0 20 20" class="w-5">
                if *editing_name {
                  <path fill-rule="evenodd" d="M16.707 5.293a1 1 0 010 1.414l-8 8a1 1 0 01-1.414 0l-4-4a1 1 0 011.414-1.414L8 12.586l7.293-7.293a1 1 0 011.414 0z" clip-rule="evenodd"></path>
                } else {
                  <path d="M17.414 2.586a2 2 0 00-2.828 0L7 10.172V13h2.828l7.586-7.586a2 2 0 000-2.828z"></path>
                  <path d="M2 6a2 2 0 012-2h4a1 1 0 010 2H4v10h10v-4a1 1 0 112 0v4a2 2 0 01-2 2H4a2 2 0 01-2-2V6z"></path>
                }
              </svg>
            </button>
          </form>
        </div>

        <div ref={messages_ref} class="flex-1 w-full flex flex-col gap-3 overflow-y-auto lg:gap-4">
          {for curr_conv.messages.iter().enumerate().map(|(i, msg)| html! {
            <Message key={i} index={i} content={msg.clone()} />
          })}
        </div>

        <form autocomplete="off" class="w-full flex flex-col gap-3" {onsubmit}>
          <div class="px-3.5 py-3 rounded-xl bg-[#F5F5F5] dark:bg-[#292929] flex">
            <textarea ref={prompt_ref} rows="1" placeholder="Ask anything..." autofocus=true class="flex-1 resize-none outline-none bg-transparent text-sm max-h-32 overflow-x-hidden" {onkeypress} {oninput}></textarea>
            <button ref={submit_ref} type="submit" class="ml-2.5 fill-current disabled:cursor-not-allowed disabled:opacity-50 enabled:hover:fill-[#FF7A1F]" disabled={curr_conv.updating_last_msg}>
              <svg viewBox="0 0 512 512" class="w-5">
                <path d="M440 6.5L24 246.4c-34.4 19.9-31.1 70.8 5.7 85.9L144 379.6V464c0 46.4 59.2 65.5 86.6 28.6l43.8-59.1 111.9 46.2c5.9 2.4 12.1 3.6 18.3 3.6 8.2 0 16.3-2.1 23.6-6.2 12.8-7.2 21.6-20 23.9-34.5l59.4-387.2c6.1-40.1-36.9-68.8-71.5-48.9zM192 464v-64.6l36.6 15.1L192 464zm212.6-28.7l-153.8-63.5L391 169.5c10.7-15.5-9.5-33.5-23.7-21.2L155.8 332.6 48 288 464 48l-59.4 387.3z"></path>
              </svg>
            </button>
          </div>
          <div>
            <select ref={provider_ref} class="px-2.5 py-2 rounded-xl bg-[#F5F5F5] dark:bg-[#292929] text-sm disabled:text-black/50 dark:disabled:text-white/50" disabled={!curr_conv.messages.is_empty()} onchange={set_provider}>
              {for PROVIDERS.iter().map(|&(value, name, disabled)| html! {
                <option key={value} value={value} disabled={disabled} selected={curr_conv.provider.as_ref() == value}>{name}</option>
              })}
            </select>
          </div>
        </form>
      </div>
    </div>
  }
}

#[derive(Serialize)]
struct DeepAiMessage<'m> {
  role: &'m str,
  content: &'m str,
}

#[derive(Serialize)]
struct YouMessage<'m> {
  question: &'m str,
  answer: &'m str,
}

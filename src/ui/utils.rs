use time::OffsetDateTime;
use web_sys::HtmlElement;
use yew::NodeRef;

pub fn close_sidebar(sidebar_ref: &NodeRef, overlay_ref: &NodeRef, invisible_overlay_ref: &NodeRef) {
  let sidebar_el: HtmlElement = sidebar_ref.cast().unwrap();
  let overlay_el: HtmlElement = overlay_ref.cast().unwrap();
  let invisible_overlay_el: HtmlElement = invisible_overlay_ref.cast().unwrap();

  sidebar_el.style().set_css_text("");
  overlay_el.style().set_css_text("");
  invisible_overlay_el.style().set_css_text("");
}

pub fn set_scroll_top_to_scroll_height(node_ref: &NodeRef) {
  let el: HtmlElement = node_ref.cast().unwrap();

  el.set_scroll_top(el.scroll_height());
}

pub fn format_date_time(date_time: OffsetDateTime) -> String {
  date_time.format(time::macros::format_description!("[year]-[month]-[day] [hour]:[minute]:[second]")).unwrap()
}

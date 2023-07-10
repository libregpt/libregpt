use std::fs;

use css_minify::optimizations::{Level as CssMinificationLevel, Minifier as CssMinifier};
use minify_html_onepass::{Cfg as HtmlConfig, with_friendly_error as minify_html};

fn main() {
  let mut code = fs::read("static/index.html").unwrap();
  let len = minify_html(&mut code, &HtmlConfig::new()).unwrap();

  fs::write("static/index.min.html", &code[..len]).unwrap();

  let code = fs::read_to_string("static/index.css").unwrap();
  let code = CssMinifier::default().minify(&code, CssMinificationLevel::One).unwrap();

  fs::write("static/index.min.css", code).unwrap();

  let code = fs::read("static/index.js").unwrap();
  // See https://github.com/wilsonzlin/minify-js/issues/14
  //let mut minified_code = Vec::with_capacity(code.len());
  //minify_js::minify(&minify_js::Session::new(), minify_js::TopLevelMode::Global, &code, &mut minified_code).unwrap();

  fs::write("static/index.min.js", /*minified_*/code).unwrap();
}

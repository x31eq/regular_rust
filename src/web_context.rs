use js_sys::decode_uri;
use std::collections::HashMap;
use wasm_bindgen::JsCast;
use wasm_bindgen::prelude::JsValue;
use web_sys::{Element, HtmlInputElement, HtmlTextAreaElement, console};

pub type Exceptionable = Result<(), JsValue>;

pub struct WebContext {
    pub document: web_sys::Document,
}

impl WebContext {
    pub fn new() -> Self {
        let window = web_sys::window().expect("no window");
        let document = window.document().expect("no document");
        WebContext { document }
    }

    pub fn set_body_class(&self, value: &str) {
        let body = self.document.body().expect("no body");
        body.set_attribute("class", value)
            .expect("failed to set class");
    }

    pub fn element(&self, id: &str) -> Option<Element> {
        self.document.get_element_by_id(id)
    }

    pub fn input_value(&self, id: &str) -> Option<String> {
        let element = self.element(id)?;
        if let Some(text_area) = element.dyn_ref::<HtmlTextAreaElement>() {
            return Some(text_area.value());
        } else if let Some(text_area) =
            element.dyn_ref::<HtmlTextAreaElement>()
        {
            return Some(text_area.value());
        }
        let input_element = element.dyn_ref::<HtmlInputElement>()?;
        Some(input_element.value())
    }

    /// Set an input if found: log errors and carry on
    pub fn set_input_value(&self, id: &str, value: &str) {
        if let Some(element) = self.element(id) {
            if let Some(text_area) = element.dyn_ref::<HtmlTextAreaElement>()
            {
                text_area.set_value(value);
            } else if let Some(input_element) =
                element.dyn_ref::<HtmlInputElement>()
            {
                input_element.set_value(value);
            } else {
                self.log_error("Not an input elemenet")
            }
        } else {
            self.log_error("Element not found")
        }
    }

    pub fn new_or_emptied_element(
        &self,
        parent: &Element,
        name: &str,
    ) -> Result<Element, JsValue> {
        if let Some(existing) = parent.query_selector(name)? {
            existing.set_inner_html("");
            Ok(existing)
        } else {
            let child = self.document.create_element(name)?;
            parent.append_child(&child)?;
            Ok(child)
        }
    }

    /// Get the URL-supplied parameters
    pub fn get_url_params(&self) -> HashMap<String, String> {
        let mut params = HashMap::new();
        if let Some(location) = self.document.location()
            && let Ok(query) = location.hash()
            && let Ok(query) = decode_uri(&query)
        {
            let query: String = query.into();
            for param in query.trim_start_matches('#').split('&') {
                if let Some((k, v)) = param.split_once('=') {
                    params.insert(k.to_string(), v.to_string());
                }
            }
        }
        params
    }

    /// Reset the has encoding the new params, and cause some at least of the
    /// new page events to be fired
    pub fn resubmit_with_params(&self, params: &HashMap<&str, String>) {
        let hash = self.hash_from_params(params);
        self.document
            .location()
            .expect("no location")
            .set_hash(&hash)
            .expect("unable to set URL hash");
    }

    /// Set the target (href) of a link to a local resubmit
    pub fn set_target(
        &self,
        link: &Element,
        params: &HashMap<&str, String>,
    ) -> Exceptionable {
        link.set_attribute("href", &self.hash_from_params(params))
    }

    pub fn hash_from_params(&self, params: &HashMap<&str, String>) -> String {
        let mut result = params
            .iter()
            .map(|(&k, v)| {
                let mut field = k.to_string();
                field.push('=');
                field.push_str(v);
                field.to_string()
            })
            .collect::<Vec<String>>()
            .join("&");
        result.insert(0, '#');
        result
    }

    pub fn log_error(&self, message: &str) {
        console::error_1(&message.into());
    }
}

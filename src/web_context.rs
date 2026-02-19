use js_sys::{Array, Uint8Array, decode_uri};
use std::collections::HashMap;
use wasm_bindgen::JsCast;
use wasm_bindgen::prelude::JsValue;
use web_sys::{
    Blob, Element, HtmlInputElement, HtmlTextAreaElement, Url, console,
};

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

    pub fn emptied_element(&self, id: &str) -> Option<Element> {
        let element = self.document.get_element_by_id(id)?;
        // If this has download links, clean them up to avoid memory leaks
        if revoke_blobs_in_subtree(&element).is_err() {
            self.log_error("Couldn't clean up downloadable files");
        }
        element.set_inner_html("");
        Some(element)
    }

    pub fn input_value(&self, id: &str) -> Option<String> {
        let element = self.element(id)?;
        if let Some(text_area) = element.dyn_ref::<HtmlTextAreaElement>() {
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

    /// Make a link to trigger a file download
    pub fn make_download_link(
        &self,
        label: &str,
        filename: &str,
        contents: &str,
    ) -> Result<Element, JsValue> {
        let bytes = Uint8Array::from(contents.as_bytes());
        let parts = Array::new();
        parts.push(&bytes.buffer());

        let blob = Blob::new_with_u8_array_sequence(&parts)?;
        let url = Url::create_object_url_with_blob(&blob)?;

        let link = self.document.create_element("a")?;
        link.set_attribute("href", &url)?;
        link.set_attribute("download", filename)?;
        link.set_attribute("data-blob-url", &url)?;
        link.set_text_content(Some(label));

        Ok(link)
    }

    pub fn log_error(&self, message: &str) {
        console::error_1(&message.into());
    }
}

/// Cleanup for contents of download links that use browser memory.
/// Generic function that doesn't actually use WebContext.
fn revoke_blobs_in_subtree(root: &Element) -> Exceptionable {
    let nodes = root.query_selector_all("[data-blob-url]")?;
    for i in 0..nodes.length() {
        if let Some(node) = nodes.item(i) {
            let element: Element = node.dyn_into()?;
            if let Some(url) = element.get_attribute("data-blob-url") {
                let _ = Url::revoke_object_url(&url);
            }
        }
    }
    Ok(())
}

use wasm_bindgen::prelude::{wasm_bindgen, JsValue};

use super::PrimeLimit;
use super::cangwu;

// Called when the wasm module is instantiated
#[wasm_bindgen(start)]
pub fn wasm_main() -> Result<(), JsValue> {
    // This is based on the wasm-bindgen "without-a-bundler" example
    let window = web_sys::window().expect("no window");
    let document = window.document().expect("no document");
    let body = document.body().expect("no body");

    let paragraph = document.create_element("p")?;
    let limit = PrimeLimit::new(53);
    let message = format!("{}: {:?} cents", limit.label, limit.pitches);
    paragraph.set_inner_html(&message);
    body.append_child(&paragraph)?;

    let table = document.create_element("table")?;
    let row = document.create_element("tr")?;
    for heading in limit.headings {
        let cell = document.create_element("th")?;
        cell.set_inner_html(&heading);
        row.append_child(&cell)?;
    }
    table.append_child(&row)?;
    for et in cangwu::get_equal_temperaments(&limit.pitches, 1.0, 20) {
        let row = document.create_element("tr")?;
        for element in et {
            let cell = document.create_element("td")?;
            cell.set_inner_html(&element.to_string());
            row.append_child(&cell)?;
        }
        table.append_child(&row)?;
    }
    body.append_child(&table)?;

    Ok(())
}

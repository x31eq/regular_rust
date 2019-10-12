use wasm_bindgen::prelude::{wasm_bindgen, JsValue};

use super::cangwu;
use super::PrimeLimit;

// Called when the wasm module is instantiated
#[wasm_bindgen(start)]
pub fn wasm_main() -> Result<(), JsValue> {
    let window = web_sys::window().expect("no window");
    let document = window.document().expect("no document");

    let limit = PrimeLimit::new(53);
    // This is shamelessly coupled to the HTML
    let table = document.get_element_by_id("equal-temperaments").unwrap_or({
        // If there's no matching table, let's make one!
        let table = document.create_element("table")?;
        table.set_id("equal-temperaments");
        document.body().expect("no body").append_child(&table)?;
        table
    });
    let row = document.create_element("tr")?;
    for heading in limit.headings {
        let cell = document.create_element("th")?;
        cell.set_text_content(Some(&heading));
        row.append_child(&cell)?;
    }
    table.append_child(&row)?;
    for et in cangwu::get_equal_temperaments(&limit.pitches, 1.0, 20) {
        let row = document.create_element("tr")?;
        for element in et {
            let cell = document.create_element("td")?;
            cell.set_text_content(Some(&element.to_string()));
            row.append_child(&cell)?;
        }
        table.append_child(&row)?;
    }

    Ok(())
}

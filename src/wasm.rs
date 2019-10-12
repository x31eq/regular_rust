use wasm_bindgen::prelude::{wasm_bindgen, JsValue};

use super::cangwu;
use super::{Cents, Harmonic, PrimeLimit};

#[wasm_bindgen]
pub fn consecutive_prime_limit_search(
    prime_cap: Harmonic,
    ek_adjusted: Cents,
    n_results: usize,
) -> Result<(), JsValue> {
    let window = web_sys::window().expect("no window");
    let document = window.document().expect("no document");

    let limit = PrimeLimit::new(prime_cap);
    let ek =
        ek_adjusted * 12e2 / limit.pitches.last().expect("no harmonics");
    // This is shamelessly coupled to the HTML
    let table = document.get_element_by_id("equal-temperaments").unwrap_or({
        // If there's no matching table, let's make one!
        let table = document.create_element("table")?;
        table.set_id("equal-temperaments");
        document.body().expect("no body").append_child(&table)?;
        table
    });
    table.set_inner_html("");
    let row = document.create_element("tr")?;
    for heading in limit.headings {
        let cell = document.create_element("th")?;
        cell.set_text_content(Some(&heading));
        row.append_child(&cell)?;
    }
    table.append_child(&row)?;
    for et in cangwu::get_equal_temperaments(&limit.pitches, ek, n_results) {
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

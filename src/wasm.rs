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
    let div = document.get_element_by_id("regular-temperaments").unwrap_or({
        // If there's no matching table, let's make one!
        let div = document.create_element("div")?;
        div.set_id("regular-temperaments");
        document.body().expect("no body").append_child(&div)?;
        div
    });
    div.set_inner_html("");
    let table = document.create_element("table")?;
    div.append_child(&table)?;
    table.set_inner_html("");
    let row = document.create_element("tr")?;
    for heading in limit.headings {
        let cell = document.create_element("th")?;
        cell.set_text_content(Some(&heading));
        row.append_child(&cell)?;
    }
    table.append_child(&row)?;
    let mappings = cangwu::get_equal_temperaments(
                            &limit.pitches, ek, n_results);
    for et in mappings.iter() {
        let row = document.create_element("tr")?;
        for element in et {
            let cell = document.create_element("td")?;
            cell.set_text_content(Some(&element.to_string()));
            row.append_child(&cell)?;
        }
        table.append_child(&row)?;
    }

    // Now make another table for the next lot of results
    let table = document.create_element("table")?;
    div.append_child(&table)?;
    table.set_inner_html("");
    let row = document.create_element("tr")?;
    let cell = document.create_element("th")?;
    cell.set_text_content(Some("Rank 2"));
    row.append_child(&cell)?;
    table.append_child(&row)?;
    let mut rts = Vec::with_capacity(mappings.len());
    for mapping in mappings.iter() {
        rts.push(vec![mapping.clone()]);
    }
    let new_rts = cangwu::higher_rank_search(
        &limit.pitches,
        &mappings,
        &rts,
        ek,
        // Note: no safety
        n_results);
    for rt in new_rts {
        let row = document.create_element("tr")?;
        let cell = document.create_element("td")?;
        let text = format!("{} & {}", rt[0][0], rt[1][0]);
        cell.set_text_content(Some(&text));
        row.append_child(&cell)?;
        table.append_child(&row)?;
    }
    Ok(())
}

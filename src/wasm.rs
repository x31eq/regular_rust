use wasm_bindgen::prelude::{wasm_bindgen, JsValue};

use super::cangwu;
use super::{join, Cents, ETMap, FactorElement, Harmonic, PrimeLimit};


#[wasm_bindgen]
pub fn consecutive_prime_limit_search(
    prime_cap: Harmonic,
    ek_adjusted: Cents,
    n_results: usize,
) -> Result<(), JsValue> {

    let limit = PrimeLimit::new(prime_cap);
    let dimension = limit.pitches.len();
    let ek = ek_adjusted * 12e2 / limit.pitches.last().expect("no harmonics");
    let safety = 4 * (dimension as f64).sqrt().floor() as usize;
    let mappings = cangwu::get_equal_temperaments(
        &limit.pitches,
        ek,
        n_results + safety,
    );
    let web = WebContext::new();
    show_equal_temperaments(&web, &limit, &mappings, n_results)?;

    let mut rts = Vec::with_capacity(mappings.len());
    for mapping in mappings.iter() {
        rts.push(vec![mapping.clone()]);
    }
    for rank in 2..dimension {
        // Make another table for the next lot of results
        let table = web.document.create_element("table")?;
        table.set_inner_html("");
        let row = web.document.create_element("tr")?;
        let cell = web.document.create_element("th")?;
        let text = format!("Rank {}", rank);
        cell.set_text_content(Some(&text));
        row.append_child(&cell)?;
        table.append_child(&row)?;

        let eff_n_results =
            n_results + if rank == dimension - 1 { 0 } else { safety };
        rts = cangwu::higher_rank_search(
            &limit.pitches,
            &mappings,
            &rts,
            ek,
            eff_n_results,
        );

        if rts.len() > 0 {
            for rt in rts.iter().take(n_results) {
                let row = web.document.create_element("tr")?;
                let cell = web.document.create_element("td")?;
                let octaves: Vec<FactorElement> =
                    rt.iter().map(|m| m[0]).collect();
                let text = join(" & ", &octaves);
                cell.set_text_content(Some(&text));
                row.append_child(&cell)?;
                table.append_child(&row)?;
            }
            web.div.append_child(&table)?;
        }
    }
    Ok(())
}


struct WebContext {
    document: web_sys::Document,
    div: web_sys::Element,
}


impl WebContext {
    pub fn new() -> Self {
        let window = web_sys::window().expect("no window");
        let document = window.document().expect("no document");
        let div = document
            .get_element_by_id("regular-temperaments")
            .unwrap_or({
                // If there's no matching element, let's make one!
                let div = document.create_element("div").unwrap();
                div.set_id("regular-temperaments");
                document.body().expect("no body").append_child(&div).unwrap();
                div
            });
        WebContext { document, div }
    }
}


fn show_equal_temperaments(
    web: &WebContext,
    limit: &PrimeLimit,
    mappings: &[ETMap],
    n_results: usize,
) -> Result<(), JsValue> {
    // This is shamelessly coupled to the HTML
    web.div.set_inner_html("");
    let table = web.document.create_element("table")?;
    web.div.append_child(&table)?;
    table.set_inner_html("");
    let row = web.document.create_element("tr")?;
    for heading in limit.headings.iter() {
        let cell = web.document.create_element("th")?;
        cell.set_text_content(Some(&heading));
        row.append_child(&cell)?;
    }
    table.append_child(&row)?;
    for et in mappings.iter().take(n_results) {
        let row = web.document.create_element("tr")?;
        for element in et {
            let cell = web.document.create_element("td")?;
            cell.set_text_content(Some(&element.to_string()));
            row.append_child(&cell)?;
        }
        table.append_child(&row)?;
    }
    Ok(())
}

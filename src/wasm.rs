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
    show_equal_temperaments(&web, &limit, mappings.iter().take(n_results))?;

    let mut rts = Vec::with_capacity(mappings.len());
    for mapping in mappings.iter() {
        rts.push(vec![mapping.clone()]);
    }
    for rank in 2..dimension {
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
            let visible_rts = rts.iter().take(n_results);
            show_regular_temperaments(&web, &limit, visible_rts, rank)?;
        }
    }
    Ok(())
}

struct WebContext {
    document: web_sys::Document,
    list: web_sys::Element,
}

impl WebContext {
    pub fn new() -> Self {
        let window = web_sys::window().expect("no window");
        let document = window.document().expect("no document");
        let list = document
            .get_element_by_id("temperament-list")
            .unwrap_or({
                // If there's no matching element, let's make one!
                let list = document.create_element("list").unwrap();
                list.set_id("temperament-list");
                document
                    .body()
                    .expect("no body")
                    .append_child(&list)
                    .unwrap();
                list
            });
        WebContext { document, list }
    }
}

fn show_equal_temperaments<'a>(
    web: &WebContext,
    limit: &PrimeLimit,
    mappings: impl Iterator<Item = &'a ETMap>,
) -> Result<(), JsValue> {
    // This is shamelessly coupled to the HTML
    web.list.set_inner_html("");
    let table = web.document.create_element("table")?;
    web.list.append_child(&table)?;
    table.set_inner_html("");
    let row = web.document.create_element("tr")?;
    for heading in limit.headings.iter() {
        let cell = web.document.create_element("th")?;
        cell.set_text_content(Some(&heading));
        row.append_child(&cell)?;
    }
    table.append_child(&row)?;
    for et in mappings {
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

fn show_regular_temperaments<'a>(
    web: &WebContext,
    limit: &PrimeLimit,
    rts: impl Iterator<Item = &'a Vec<ETMap>>,
    rank: usize,
) -> Result<(), JsValue> {
    // Make another table for the next lot of results
    let table = web.document.create_element("table")?;
    table.set_inner_html("");
    let row = web.document.create_element("tr")?;
    let cell = web.document.create_element("th")?;
    let text = format!("Rank {}", rank);
    cell.set_text_content(Some(&text));
    row.append_child(&cell)?;
    table.append_child(&row)?;

    for rt in rts {
        let row = web.document.create_element("tr")?;
        let cell = web.document.create_element("td")?;
        let link = web.document.create_element("a")?;
        let octaves: Vec<FactorElement> = rt.iter().map(|m| m[0]).collect();
        let rt_obj = cangwu::TemperamentClass::new(&limit.pitches, &rt);
        let url = format!(
            "/cgi-bin/rt.cgi?ets={}&limit={}&key={}",
            &join("_", &octaves),
            &limit.label,
            &join("_", &rt_obj.key()),
        );
        link.set_attribute("href", &url)?;
        let text = join(" & ", &octaves);
        link.set_text_content(Some(&text));
        cell.append_child(&link)?;
        row.append_child(&cell)?;
        table.append_child(&row)?;
        web.document
            .body()
            .expect("no body")
            .set_attribute("class", "show-list")?;
    }
    web.list.append_child(&table)?;
    Ok(())
}

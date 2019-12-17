use wasm_bindgen::prelude::{wasm_bindgen, Closure, JsValue};
use wasm_bindgen::JsCast;
use web_sys::{Element, Event, HtmlElement};

use super::cangwu;
use super::{
    join, Cents, ETMap, FactorElement, Harmonic, PrimeLimit, Tuning,
};

extern crate nalgebra as na;
use na::DMatrix;

type Exceptionable = Result<(), JsValue>;

#[wasm_bindgen]
pub fn consecutive_prime_limit_search(
    prime_cap: Harmonic,
    ek_adjusted: Cents,
    n_results: usize,
) -> Result<SearchResult, JsValue> {
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

    // Store the limit in the DOM so we can get it later
    let mut items = limit.headings.iter();
    let mut headings = "".to_string();
    if let Some(heading) = items.next() {
        headings.push_str(&heading);
    };
    for heading in items {
        headings.push_str("_");
        headings.push_str(heading);
    }
    web.list.set_attribute("data-headings", &headings)?;
    web.list
        .set_attribute("data-pitches", &join("_", &limit.pitches))?;

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

    // Callback for clicking a link
    let callback =
        Closure::wrap(Box::new(rt_click_handler)
            as Box<dyn FnMut(Event) -> Exceptionable>);
    web.list
        .dyn_ref::<HtmlElement>()
        .expect("Table isn't an HtmlElement")
        .set_onclick(Some(callback.as_ref().unchecked_ref()));

    // Return the callback so the browser keeps it alive
    Ok(SearchResult {
        _callback: callback,
    })
}

struct WebContext {
    document: web_sys::Document,
    list: web_sys::Element,
}

impl WebContext {
    pub fn new() -> Self {
        let window = web_sys::window().expect("no window");
        let document = window.document().expect("no document");
        let body = document.body().expect("no body");
        let list =
            document.get_element_by_id("temperament-list").unwrap_or({
                // If there's no matching element, let's make one!
                let list = document.create_element("list").unwrap();
                list.set_id("temperament-list");
                body.append_child(&list).unwrap();
                list
            });
        WebContext { document, list }
    }

    pub fn set_body_class(&self, value: &str) -> Exceptionable {
        let body = self.document.body().expect("no body");
        body.set_attribute("class", value)
    }
}

fn show_equal_temperaments<'a>(
    web: &WebContext,
    limit: &PrimeLimit,
    mappings: impl Iterator<Item = &'a ETMap>,
) -> Exceptionable {
    // This is shamelessly coupled to the HTML
    web.list.set_inner_html("");
    let table = web.document.create_element("table")?;
    table.set_attribute("class", "mapping")?;
    write_mapping_matrix(&web, &table, &limit, mappings)?;
    web.list.append_child(&table)?;
    Ok(())
}

fn write_mapping_matrix<'a>(
    web: &WebContext,
    table: &Element,
    limit: &PrimeLimit,
    values: impl Iterator<Item = &'a ETMap>,
) -> Exceptionable {
    write_headings(&web, &table, &limit)?;
    for vector in values {
        let row = web.document.create_element("tr")?;
        for element in vector {
            let cell = web.document.create_element("td")?;
            cell.set_text_content(Some(&element.to_string()));
            row.append_child(&cell)?;
        }
        table.append_child(&row)?;
    }
    Ok(())
}

fn write_headings(
    web: &WebContext,
    table: &Element,
    limit: &PrimeLimit,
) -> Exceptionable {
    table.set_inner_html("");
    let row = web.document.create_element("tr")?;
    for heading in limit.headings.iter() {
        let cell = web.document.create_element("th")?;
        cell.set_text_content(Some(&heading));
        row.append_child(&cell)?;
    }
    table.append_child(&row)?;
    Ok(())
}

fn write_float_row(
    web: &WebContext,
    table: &Element,
    pitches: &Tuning,
    precision: usize,
) -> Exceptionable {
    let row = web.document.create_element("tr")?;
    for element in pitches {
        let cell = web.document.create_element("td")?;
        let formatted = format!("{:.*}", precision, element);
        cell.set_text_content(Some(&formatted));
        row.append_child(&cell)?;
    }
    table.append_child(&row)?;
    Ok(())
}

fn show_regular_temperaments<'a>(
    web: &WebContext,
    limit: &PrimeLimit,
    rts: impl Iterator<Item = &'a Vec<ETMap>>,
    rank: usize,
) -> Exceptionable {
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

        // Setup the link as a link
        let octaves: Vec<FactorElement> = rt.iter().map(|m| m[0]).collect();
        let rt_obj = cangwu::TemperamentClass::new(&limit.pitches, &rt);
        let url = format!(
            "/cgi-bin/rt.cgi?ets={}&limit={}&key={}",
            &join("_", &octaves),
            &limit.label,
            &join("_", &rt_obj.key()),
        );
        link.set_attribute("href", &url)?;

        // Set data attributes so we get at the mapping later
        link.set_attribute("data-rank", &rt.len().to_string())?;
        for (i, mapping) in rt.iter().enumerate() {
            let key = format!("data-mapping{}", i);
            let value = join("_", mapping);
            link.set_attribute(&key, &value)?;
        }

        let text = join(" & ", &octaves);
        link.set_text_content(Some(&text));

        cell.append_child(&link)?;
        row.append_child(&cell)?;
        table.append_child(&row)?;
    }

    web.list.append_child(&table)?;
    web.set_body_class("show-list")?;
    Ok(())
}

#[wasm_bindgen]
/// Object to return from a search so that
/// the callbacks stay alive
pub struct SearchResult {
    _callback: Closure<dyn FnMut(Event) -> Exceptionable>,
}

/// Function to call when a temperament link is "clicked"
/// (which includes in-page activation)
fn rt_click_handler(evt: Event) -> Exceptionable {
    if let Some(target) = evt.target() {
        let target = target
            .dyn_ref::<Element>()
            .expect("Target isn't an Element");
        if target.has_attribute("href") {
            let web = WebContext::new();

            let headings = web
                .list
                .get_attribute("data-headings")
                .unwrap()
                .split('_')
                .map(|heading| heading.to_string())
                .collect();
            let pitches = web
                .list
                .get_attribute("data-pitches")
                .unwrap()
                .split('_')
                .map(|p| p.parse().unwrap())
                .collect();
            let limit = PrimeLimit {
                label: "placeholder".to_string(),
                pitches,
                headings,
            };
            let mut mapping = Vec::new();
            let rank: usize =
                target.get_attribute("data-rank").unwrap().parse().unwrap();
            for i in 0..rank {
                let vector = target
                    .get_attribute(&format!("data-mapping{}", i))
                    .unwrap()
                    .split('_')
                    .map(|m| m.parse().unwrap())
                    .collect();
                mapping.push(vector);
            }
            let rt = cangwu::TemperamentClass::new(&limit.pitches, &mapping);

            let octaves: Vec<FactorElement> =
                mapping.iter().map(|m| m[0]).collect();
            let name = join(" & ", &octaves);
            web.document
                .get_element_by_id("rt-name")
                .unwrap()
                .set_text_content(Some(&name));

            let table = web.document.get_element_by_id("rt-etmap").unwrap();
            write_mapping_matrix(&web, &table, &limit, mapping.iter())?;

            let table = web.document.get_element_by_id("rt-redmap").unwrap();
            let redmap = rt.reduced_mapping();
            write_mapping_matrix(&web, &table, &limit, redmap.iter())?;

            let table = web.document.get_element_by_id("rt-steps").unwrap();
            table.set_inner_html("");
            let tuning = rt.optimal_tuning();
            write_float_row(&web, &table, &tuning, 4)?;

            let tuning = DMatrix::from_vec(rank, 1, tuning);
            let dimension = limit.pitches.len();
            let flattened = mapping
                .iter()
                .flat_map(|mapping| mapping.iter().map(|&m| m as f64));
            let melody = DMatrix::from_iterator(dimension, rank, flattened);
            let tuning_map: DMatrix<f64> = melody * tuning;
            let tuning_map = tuning_map.iter().cloned().collect();
            let table =
                web.document.get_element_by_id("rt-tuning-map").unwrap();
            table.set_inner_html("");
            write_headings(&web, &table, &limit)?;
            write_float_row(&web, &table, &tuning_map, 3)?;

            let mistunings = tuning_map
                .iter()
                .zip(limit.pitches.iter())
                .map(|(&x, y)| x - y)
                .collect();
            let table =
                web.document.get_element_by_id("rt-mistunings").unwrap();
            table.set_inner_html("");
            write_headings(&web, &table, &limit)?;
            write_float_row(&web, &table, &mistunings, 4)?;

            web.document
                .get_element_by_id("rt-complexity")
                .unwrap()
                .set_text_content(Some(&format!("{:.6}", rt.complexity())));

            let te_error = rt.badness(0.0) / rt.complexity();
            web.document
                .get_element_by_id("rt-te-error")
                .unwrap()
                .set_text_content(Some(&format!("{:.6}", te_error)));

            let mut max_harmonic = 0.0;
            for &harmonic in limit.pitches.iter() {
                if harmonic > max_harmonic {
                    max_harmonic = harmonic;
                }
            }
            let error = te_error * max_harmonic / 12e2;
            web.document
                .get_element_by_id("error")
                .unwrap()
                .set_text_content(Some(&format!("{:.6}", error)));

            // Make another RT object to get the generator tunings
            let rt = cangwu::TemperamentClass::new(&limit.pitches, &redmap);
            let table =
                web.document.get_element_by_id("rt-generators").unwrap();
            table.set_inner_html("");
            write_float_row(&web, &table, &rt.optimal_tuning(), 4)?;

            evt.prevent_default();
            let result = web
                .document
                .get_element_by_id("regular-temperament")
                .unwrap();
            web.set_body_class("show-list show-temperament")?;
            result.scroll_into_view();
        }
    }
    Ok(())
}

use wasm_bindgen::prelude::{wasm_bindgen, Closure, JsValue};
use wasm_bindgen::JsCast;
use web_sys::{Element, Event, HtmlElement};

use super::cangwu;
use super::{join, Cents, ETMap, FactorElement, Mapping, PrimeLimit};

extern crate nalgebra as na;
use na::DMatrix;

type Exceptionable = Result<(), JsValue>;

#[wasm_bindgen]
pub fn consecutive_prime_limit_search(
    prime_cap: super::Harmonic,
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
        .expect("Result list isn't an HtmlElement")
        .set_onclick(Some(callback.as_ref().unchecked_ref()));

    // Return the callback so the browser keeps it alive
    Ok(SearchResult {
        _callback: callback,
    })
}

struct WebContext {
    document: web_sys::Document,
    list: Element,
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

    pub fn element(&self, id: &str) -> Option<Element> {
        self.document.get_element_by_id(id)
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
    pitches: &super::Tuning,
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
        let rt_obj = cangwu::CangwuTemperament::new(&limit.pitches, &rt);
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

            let limit = load_limit(&web.list);
            let mapping = load_mapping(&target);
            show_rt(&web, limit, mapping)?;
            evt.prevent_default();
        }
    }
    Ok(())
}

/// Pull the prime limit out of the DOM
fn load_limit(list: &Element) -> PrimeLimit {
    let headings = list
        .get_attribute("data-headings")
        .unwrap()
        .split('_')
        .map(|heading| heading.to_string())
        .collect();
    let pitches = list
        .get_attribute("data-pitches")
        .unwrap()
        .split('_')
        .map(|p| p.parse().unwrap())
        .collect();
    let label = "placeholder".to_string();
    PrimeLimit {
        label,
        pitches,
        headings,
    }
}

/// Get the regular temperament mapping from the DOM
fn load_mapping(link: &Element) -> Mapping {
    let mut mapping = Vec::new();
    let rank: usize =
        link.get_attribute("data-rank").unwrap().parse().unwrap();
    for i in 0..rank {
        let vector = link
            .get_attribute(&format!("data-mapping{}", i))
            .unwrap()
            .split('_')
            .map(|m| m.parse().unwrap())
            .collect();
        mapping.push(vector);
    }
    mapping
}

/// Set the fields about the regular temperament
fn show_rt(
    web: &WebContext,
    limit: PrimeLimit,
    mapping: Mapping,
) -> Exceptionable {
    let rank = mapping.len();
    let rt = cangwu::CangwuTemperament::new(&limit.pitches, &mapping);

    if let Some(name_field) = web.element("rt-name") {
        let octaves: ETMap = mapping.iter().map(|m| m[0]).collect();
        name_field.set_text_content(Some(&join(" & ", &octaves)));
    }

    if let Some(table) = web.element("rt-etmap") {
        write_mapping_matrix(&web, &table, &limit, mapping.iter())?;
    }

    let redmap = rt.reduced_mapping();
    if let Some(table) = web.element("rt-redmap") {
        write_mapping_matrix(&web, &table, &limit, redmap.iter())?;
    }

    let tuning = rt.optimal_tuning();
    if let Some(table) = web.element("rt-steps") {
        table.set_inner_html("");
        write_float_row(&web, &table, &tuning, 4)?;
    }

    let tuning = DMatrix::from_vec(rank, 1, tuning);
    let dimension = limit.pitches.len();
    let flattened = mapping
        .iter()
        .flat_map(|mapping| mapping.iter().map(|&m| m as f64));
    let melody = DMatrix::from_iterator(dimension, rank, flattened);
    let tuning_map: DMatrix<f64> = melody * tuning;
    let tuning_map = tuning_map.iter().cloned().collect();
    if let Some(table) = web.element("rt-tuning-map") {
        write_headings(&web, &table, &limit)?;
        write_float_row(&web, &table, &tuning_map, 3)?;
    }

    if let Some(table) = web.element("rt-mistunings") {
        let mistunings = tuning_map
            .iter()
            .zip(limit.pitches.iter())
            .map(|(&x, y)| x - y)
            .collect();
        write_headings(&web, &table, &limit)?;
        write_float_row(&web, &table, &mistunings, 4)?;
    }

    let complexity = rt.complexity();
    if let Some(field) = web.element("rt-complexity") {
        let text = format!("{:.6}", complexity);
        field.set_text_content(Some(&text));
    }

    let te_error = rt.badness(0.0) / complexity;
    if let Some(field) = web.element("rt-te-error") {
        field.set_text_content(Some(&format!("{:.6}", te_error)));
    }

    if let Some(field) = web.element("error") {
        let max_harmonic = limit
            .pitches
            .iter()
            .max_by(|a, b| a.partial_cmp(b).unwrap())
            .unwrap();
        let error = te_error * max_harmonic / 12e2;
        field.set_text_content(Some(&format!("{:.6}", error)));
    }

    if let Some(table) = web.element("rt-generators") {
        // Make another RT object to get the generator tunings
        let rt = cangwu::CangwuTemperament::new(&limit.pitches, &redmap);
        table.set_inner_html("");
        write_float_row(&web, &table, &rt.optimal_tuning(), 4)?;
    }

    web.set_body_class("show-list show-temperament")?;
    if let Some(result) = web.element("regular-temperament") {
        result.scroll_into_view();
    }

    Ok(())
}

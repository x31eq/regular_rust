use wasm_bindgen::prelude::{wasm_bindgen, Closure, JsValue};
use wasm_bindgen::{throw_str, JsCast};
use web_sys::{Element, Event, HtmlElement, HtmlInputElement};

use super::cangwu;
use super::te;
use super::{join, Cents, ETMap, Mapping, PrimeLimit};
use cangwu::TemperamentClass;

type Exceptionable = Result<(), JsValue>;

#[wasm_bindgen]
pub fn form_submit(evt: Event) -> SearchResult {
    evt.prevent_default();
    let web = WebContext::new();
    let limit = web.unwrap(
        web.input_value("prime-limit").parse(),
        "Unrecognized prime limit",
    );
    let eka = web.unwrap(
        web.input_value("prime-eka").parse(),
        "Unrecognized badness parameter",
    );
    let nresults = web.unwrap(
        web.input_value("n-results").parse(),
        "Unrecognized number of results",
    );
    regular_temperament_search(PrimeLimit::new(limit), eka, nresults)
}

pub fn regular_temperament_search(
    limit: PrimeLimit,
    ek_adjusted: Cents,
    n_results: usize,
) -> SearchResult {
    let dimension = limit.pitches.len();
    let ek = ek_adjusted * 12e2 / limit.pitches.last().expect("no harmonics");
    let safety = 4 * (dimension as f64).sqrt().floor() as usize;
    let mappings = cangwu::get_equal_temperaments(
        &limit.pitches,
        ek,
        n_results + safety,
    );
    let web = WebContext::new();
    web.list.set_inner_html("");
    web.set_body_class("show-list");
    web.unwrap(
        show_equal_temperaments(
            &web,
            &limit,
            mappings.iter().take(n_results),
        ),
        "Programming Error: Failed to display equal temperaments",
    );

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
    web.unwrap(
        web.list.set_attribute("data-headings", &headings),
        "Programming Error: Failed to store headings",
    );
    web.unwrap(
        web.list.set_attribute("data-label", &limit.label),
        "Programming Error: Failed to store prime limit label",
    );
    web.unwrap(
        web.list
            .set_attribute("data-pitches", &join("_", &limit.pitches)),
        "Programming Error: Failed to store pitches",
    );

    let mut rts: Vec<Mapping> = mappings
        .iter()
        .map(|mapping| vec![mapping.clone()])
        .collect();
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
        if !rts.is_empty() {
            let visible_rts = rts.iter().take(n_results);
            web.unwrap(
                show_regular_temperaments(&web, &limit, visible_rts, rank),
                "Failed to display regular temperaments",
            );
        }
    }

    // Callback for clicking a link
    let callback = Closure::wrap(
        Box::new(rt_click_handler) as Box<dyn FnMut(Event) -> ()>
    );
    web.list
        .dyn_ref::<HtmlElement>()
        .expect("Result list isn't an HtmlElement")
        .set_onclick(Some(callback.as_ref().unchecked_ref()));

    // Return the callback so the browser keeps it alive
    SearchResult {
        _callback: callback,
    }
}

struct WebContext {
    document: web_sys::Document,
    list: Element,
}

impl WebContext {
    pub fn new() -> Self {
        let window = web_sys::window().expect("no window");
        let document = window.document().expect("no document");
        if let Some(list) = document.get_element_by_id("temperament-list") {
            WebContext { document, list }
        } else {
            // This is an error in the HTML
            throw_str("Programming Error: temperament-list not found");
        }
    }

    pub fn set_body_class(&self, value: &str) {
        let body = self.document.body().expect("no body");
        body.set_attribute("class", value).unwrap();
    }

    pub fn element(&self, id: &str) -> Option<Element> {
        self.document.get_element_by_id(id)
    }

    pub fn input_value(&self, id: &str) -> String {
        let element =
            self.expect(self.element(id), "Unable to find input element");
        self.expect(
            element.dyn_ref::<HtmlInputElement>(),
            "Element isn't an input element",
        )
        .value()
    }

    /// Unwrap a value with the potential of an exception
    pub fn unwrap<T, U>(&self, result: Result<T, U>, message: &str) -> T {
        match result {
            Ok(value) => value,
            Err(_) => self.fail(message),
        }
    }

    /// Raise an exception on a none
    pub fn expect<T>(&self, result: Option<T>, message: &str) -> T {
        match result {
            Some(value) => value,
            None => self.fail(message),
        }
    }

    /// Escalate an error to an exception
    pub fn fail(&self, message: &str) -> ! {
        if let Some(error_field) = self.element("error-report") {
            error_field.set_text_content(Some(message));
        }
        self.set_body_class("show-errors");
        throw_str(message);
    }
}

fn show_equal_temperaments<'a>(
    web: &WebContext,
    limit: &PrimeLimit,
    mappings: impl Iterator<Item = &'a ETMap>,
) -> Exceptionable {
    // This is shamelessly coupled to the HTML
    let heading = web.document.create_element("h4")?;
    heading.set_text_content(Some("Equal Temperaments"));
    web.list.append_child(&heading)?;
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
    pitches: &[Cents],
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
    let heading = web.document.create_element("h4")?;
    let text = format!("Rank {}", rank);
    heading.set_text_content(Some(&text));
    web.list.append_child(&heading)?;

    // Make another table for the next lot of results
    let table = web.document.create_element("table")?;
    table.set_inner_html("");
    let row = web.document.create_element("tr")?;
    for column_heading in &["ETs", "complexity", "error"] {
        let cell = web.document.create_element("th")?;
        cell.set_text_content(Some(column_heading));
        row.append_child(&cell)?;
    }
    table.append_child(&row)?;

    for rt in rts {
        let row = rt_row(&rt, &limit, &web)?;
        table.append_child(&row)?;
    }
    web.list.append_child(&table)?;
    Ok(())
}

/// Return the table row for a regular temperament mapping
fn rt_row(
    mapping: &[ETMap],
    limit: &PrimeLimit,
    web: &WebContext,
) -> Result<(Element), JsValue> {
    let row = web.document.create_element("tr")?;
    let cell = web.document.create_element("td")?;
    let link = web.document.create_element("a")?;

    // Setup the link as a link
    let rt = te::TETemperament::new(&limit.pitches, &mapping);
    link.set_attribute("href", &rt_url(&rt, &limit.label))?;

    // Set data attributes so we get at the mapping later
    link.set_attribute("data-rank", &mapping.len().to_string())?;
    for (i, etmap) in mapping.iter().enumerate() {
        let key = format!("data-mapping{}", i);
        let value = join("_", etmap);
        link.set_attribute(&key, &value)?;
    }

    let octaves: ETMap = mapping.iter().map(|m| m[0]).collect();
    let text = join(" & ", &octaves);
    link.set_text_content(Some(&text));
    cell.append_child(&link)?;
    row.append_child(&cell)?;

    let cell = web.document.create_element("td")?;
    cell.set_text_content(Some(&format!("{:.3}", rt.complexity())));
    row.append_child(&cell)?;

    let cell = web.document.create_element("td")?;
    cell.set_text_content(Some(&format!("{:.3} cents", rt.adjusted_error())));
    row.append_child(&cell)?;

    Ok(row)
}

fn rt_url(rt: &te::TETemperament, label: &str) -> String {
    let octaves: ETMap = rt.melody.iter().map(|m| m[0]).collect();
    format!(
        "/cgi-bin/rt.cgi?ets={}&limit={}&key={}",
        &join("_", &octaves),
        &label,
        &join("_", &rt.key()),
    )
}

#[wasm_bindgen]
/// Object to return from a search so that
/// the callbacks stay alive
pub struct SearchResult {
    _callback: Closure<dyn FnMut(Event) -> ()>,
}

/// Function to call when a temperament link is "clicked"
/// (which includes in-page activation)
fn rt_click_handler(evt: Event) {
    if let Some(target) = evt.target() {
        let target = target
            .dyn_ref::<Element>()
            .expect("Target isn't an Element");
        if target.has_attribute("href") {
            let web = WebContext::new();
            let limit = web.expect(
                load_limit(&web.list),
                "Programming Error: failed to load prime limit",
            );
            let mapping = web.expect(
                load_mapping(&target),
                "Programming Error: failed to load mapping",
            );
            web.unwrap(
                show_rt(&web, limit, mapping),
                "Failed to show the regular temperament",
            );
            evt.prevent_default();
        }
    }
}

/// Pull the prime limit out of the DOM
fn load_limit(list: &Element) -> Option<PrimeLimit> {
    let label = list.get_attribute("data-label")?;
    let value = list.get_attribute("data-pitches")?;
    let pitches =
        result_to_option(value.split('_').map(|p| p.parse()).collect())?;
    let value = list.get_attribute("data-headings")?;
    let headings = value
        .split('_')
        .map(|heading| heading.to_string())
        .collect();
    Some(PrimeLimit {
        label,
        pitches,
        headings,
    })
}

/// Get the regular temperament mapping from the DOM
fn load_mapping(link: &Element) -> Option<Mapping> {
    let mut mapping = Vec::new();
    let value = link.get_attribute("data-rank")?;
    let rank: usize = result_to_option(value.parse())?;
    for i in 0..rank {
        let value = link.get_attribute(&format!("data-mapping{}", i))?;
        let vector =
            result_to_option(value.split('_').map(|m| m.parse()).collect())?;
        mapping.push(vector);
    }
    Some(mapping)
}

/// Set the fields about the regular temperament
fn show_rt(
    web: &WebContext,
    limit: PrimeLimit,
    mapping: Mapping,
) -> Exceptionable {
    let rt = te::TETemperament::new(&limit.pitches, &mapping);

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

    if let Some(table) = web.element("rt-steps") {
        table.set_inner_html("");
        write_float_row(&web, &table, &rt.tuning, 4)?;
    }

    if let Some(table) = web.element("rt-pote-steps") {
        table.set_inner_html("");
        write_float_row(&web, &table, &rt.pote_tuning(), 4)?;
    }

    if let Some(table) = web.element("rt-tuning-map") {
        write_headings(&web, &table, &limit)?;
        write_float_row(&web, &table, &rt.tuning_map(), 3)?;
    }

    if let Some(table) = web.element("rt-pote-tuning-map") {
        write_headings(&web, &table, &limit)?;
        write_float_row(&web, &table, &rt.pote_tuning_map(), 3)?;
    }

    if let Some(table) = web.element("rt-mistunings") {
        write_headings(&web, &table, &limit)?;
        write_float_row(&web, &table, &rt.mistunings(), 4)?;
    }

    if let Some(table) = web.element("rt-pote-mistunings") {
        write_headings(&web, &table, &limit)?;
        write_float_row(&web, &table, &rt.pote_mistunings(), 4)?;
    }

    if let Some(field) = web.element("rt-complexity") {
        let text = format!("{:.6}", rt.complexity());
        field.set_text_content(Some(&text));
    }

    if let Some(field) = web.element("rt-te-error") {
        field.set_text_content(Some(&format!("{:.6}", rt.error())));
    }

    if let Some(field) = web.element("error") {
        field.set_text_content(Some(&format!("{:.6}", rt.adjusted_error())));
    }

    if let Some(field) = web.element("rt-link") {
        field.set_attribute("href", &rt_url(&rt, &limit.label))?;
    }

    // Make another RT object to get the generator tunings
    let rt = te::TETemperament::new(&limit.pitches, &redmap);
    if let Some(table) = web.element("rt-generators") {
        table.set_inner_html("");
        write_float_row(&web, &table, &rt.tuning, 4)?;
    }

    if let Some(table) = web.element("rt-pote-generators") {
        table.set_inner_html("");
        write_float_row(&web, &table, &rt.pote_tuning(), 4)?;
    }

    web.set_body_class("show-list show-temperament");
    if let Some(result) = web.element("regular-temperament") {
        result.scroll_into_view();
    }

    Ok(())
}

fn result_to_option<T, E>(result: Result<T, E>) -> Option<T> {
    match result {
        Ok(value) => Some(value),
        Err(_) => None,
    }
}

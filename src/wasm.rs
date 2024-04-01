use crate::cangwu::CangwuTemperament;
use std::collections::HashMap;
use std::str::FromStr;
use wasm_bindgen::prelude::{wasm_bindgen, Closure, JsValue};
use wasm_bindgen::{throw_str, JsCast};
use web_sys::{console, Element, Event, HtmlElement, HtmlInputElement};

use super::cangwu;
use super::ratio::get_ratio_or_ket_string;
use super::te;
use super::temperament_class::TemperamentClass;
use super::uv::only_unison_vector;
use super::{
    join, map, normalize_positive, Cents, ETMap, Exponent, Mapping,
    PrimeLimit,
};

type Exceptionable = Result<(), JsValue>;

#[wasm_bindgen]
pub fn form_submit(evt: Event) -> SearchResult {
    evt.prevent_default();
    let web = WebContext::new();
    let limit = web.unwrap(
        web.input_value("prime-limit").trim().parse(),
        "Unrecognized prime limit",
    );
    let eka = web.unwrap(
        web.input_value("prime-eka").trim().parse(),
        "Unrecognized badness parameter",
    );
    let nresults = web.unwrap(
        web.input_value("n-results").trim().parse(),
        "Unrecognized number of results",
    );
    regular_temperament_search(limit, eka, nresults)
}

#[wasm_bindgen(start)]
fn main() -> Result<(), JsValue> {
    let web = WebContext::new();
    let params = web.get_url_params();
    web.log("New page load");
    web.log(&format!("URL params {:?}", params));
    if params.get("page") == Some(&"rt".to_string()) {
        web.log("Regular temperament display");
        if let Some(ets) = params.get("ets") {
            if let Some(limit) = params.get("limit") {
                // Note: the "key" format is the old way of
                // doing it, and it would be easier not to support
                // it, but only the warted ET names.
                // The trouble is, I haven't coded them either yet
                if let Some(key) = params.get("key") {
                    web.log(&ets);
                    web.log(&limit);
                    web.log(&key);
                    if let Ok(ets) = ets
                        .split('_')
                        .map(Exponent::from_str)
                        .collect::<Result<Vec<_>, _>>()
                    {
                        if let Ok(key) = key
                            .split('_')
                            .map(Exponent::from_str)
                            .collect::<Result<Vec<_>, _>>()
                        {
                            if let Ok(limit) = limit.parse::<PrimeLimit>() {
                                if let Some(rt) =
                                    CangwuTemperament::from_ets_and_key(
                                        &limit.pitches,
                                        &ets,
                                        &key,
                                    )
                                {
                                    web.log(&format!("rt: {:?}", rt.melody));
                                } else {
                                    web.log(&format!("Unable to make temperament class from {:?}, {ets:?}, {key:?}", limit.pitches));
                                }
                            } else {
                                web.log("Unable to parse limit");
                            }
                        } else {
                            web.log("Unable to parse key");
                        }
                    } else {
                        web.log("Unable to parse ETs")
                    }
                }
            }
        }
    }
    Ok(())
}

pub fn regular_temperament_search(
    limit: PrimeLimit,
    ek_adjusted: Cents,
    n_results: usize,
) -> SearchResult {
    let dimension = limit.pitches.len();
    let ek = ek_adjusted * 12e2 / limit.pitches.last().expect("no harmonics");
    let safety = if dimension < 100 {
        40
    } else {
        4 * (dimension as f64).sqrt().floor() as usize
    };
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

    let mut rts = map(|mapping| vec![mapping.clone()], &mappings);
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
        body.set_attribute("class", value)
            .expect("failed to set class");
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

    fn new_or_emptied_element(
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
        if let Some(location) = self.document.location() {
            if let Ok(url) = location.href() {
                if let Some((_, tokens)) = url.split_once('#') {
                    for param in tokens.split('&') {
                        if let Some((k, v)) = param.split_once('=') {
                            params.insert(k.to_string(), v.to_string());
                        }
                    }
                }
            }
        }
        params
    }

    pub fn log(&self, message: &str) {
        console::log_1(&message.into());
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
        console::error_1(&message.into());
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
    table.set_attribute("class", "mapping bra")?;
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
    let body = web.new_or_emptied_element(&table, "tbody")?;
    for vector in values {
        let row = web.document.create_element("tr")?;
        for element in vector {
            let cell = web.document.create_element("td")?;
            cell.set_text_content(Some(&element.to_string()));
            row.append_child(&cell)?;
        }
        body.append_child(&row)?;
    }
    Ok(())
}

fn write_headings(
    web: &WebContext,
    table: &Element,
    limit: &PrimeLimit,
) -> Exceptionable {
    let head = web.new_or_emptied_element(&table, "thead")?;
    let row = web.document.create_element("tr")?;
    for heading in limit.headings.iter() {
        let cell = web.document.create_element("th")?;
        cell.set_text_content(Some(&heading));
        row.append_child(&cell)?;
    }
    head.append_child(&row)?;
    table.append_child(&head)?;
    Ok(())
}

fn write_float_row(
    web: &WebContext,
    table: &Element,
    pitches: &[Cents],
    precision: usize,
) -> Exceptionable {
    let body = web.new_or_emptied_element(&table, "tbody")?;
    let row = web.document.create_element("tr")?;
    for element in pitches {
        let cell = web.document.create_element("td")?;
        let formatted = format!("{:.*}", precision, element);
        cell.set_text_content(Some(&formatted));
        row.append_child(&cell)?;
    }
    body.append_child(&row)?;
    table.append_child(&body)?;
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
    for column_heading in &["Name", "ETs", "complexity", "error"] {
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
) -> Result<Element, JsValue> {
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

    let octaves = map(|m| m[0], &mapping);
    let ets = join(" & ", &octaves);

    if let Some(name) = rt.name(&limit) {
        link.set_text_content(Some(&name));
    } else if let Some(uv) = only_unison_vector(&rt.melody) {
        let norm_uv = normalize_positive(&limit, uv);
        let name = get_ratio_or_ket_string(&limit, &norm_uv);
        link.set_text_content(Some(&name));
    } else {
        link.set_text_content(Some(&ets));
    }

    cell.append_child(&link)?;
    row.append_child(&cell)?;

    let cell = web.document.create_element("td")?;
    cell.set_text_content(Some(&ets));
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
    let octaves = map(|m| m[0], &rt.melody);
    format!(
        "#page=rt&ets={}&limit={}&key={}",
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
    WebContext::new().log("Click handler called");
    if let Some(target) = evt.target() {
        let target = target
            .dyn_ref::<Element>()
            .expect("Target isn't an Element");
        if target.has_attribute("href") {
            let web = WebContext::new();
            web.log(&format!("{:?}", web.get_url_params()));
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
        result_to_option(value.split('_').map(str::parse).collect())?;
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
            result_to_option(value.split('_').map(str::parse).collect())?;
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
        if let Some(name) = rt.name(&limit) {
            name_field.set_text_content(Some(&name));
        } else {
            let octaves = map(|m| m[0], &mapping);
            name_field.set_text_content(Some(&join(" & ", &octaves)));
        }
    }

    if let Some(table) = web.element("rt-etmap") {
        write_mapping_matrix(&web, &table, &limit, mapping.iter())?;
    }

    let redmap = rt.reduced_mapping();
    if let Some(table) = web.element("rt-redmap") {
        write_mapping_matrix(&web, &table, &limit, redmap.iter())?;
    }

    if let Some(table) = web.element("rt-steps") {
        write_float_row(&web, &table, &rt.tuning, 4)?;
    }

    if let Some(table) = web.element("rt-pote-steps") {
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

    if show_accordion(&web, &rt).is_err() {
        if let Some(accordion) = web.element("rt-accordion") {
            // This is an optional feature,
            // so hide it if something went wrong
            accordion.set_inner_html("<!-- accordion went wrong -->");
        }
    }

    // Make another RT object to get the generator tunings
    let rt = te::TETemperament::new(&limit.pitches, &redmap);
    if let Some(table) = web.element("rt-generators") {
        write_float_row(&web, &table, &rt.tuning, 4)?;
    }

    if let Some(table) = web.element("rt-pote-generators") {
        write_float_row(&web, &table, &rt.pote_tuning(), 4)?;
    }

    web.set_body_class("show-list show-temperament");
    if let Some(result) = web.element("regular-temperament") {
        result.scroll_into_view();
    }

    Ok(())
}

/// An accordion is an instrument with buttons
fn show_accordion(web: &WebContext, rt: &te::TETemperament) -> Exceptionable {
    let accordion = match web.element("rt-accordion") {
        Some(result) => result,
        None => return Ok(()),
    };
    accordion.set_inner_html("");
    let rank = rt.melody.len();
    if rank != 2 {
        return Ok(());
    }
    let tonic: ETMap = (0..rank).map(|_| 0).collect();
    let mut diatonic_steps = 0;
    let mut pitch_stack = vec![tonic.clone()];
    let mut grid = Vec::new();
    let octaves: ETMap = map(|m| m[0], &rt.melody);
    let diatonic_dimension = if octaves[0] < octaves[1] { 0 } else { 1 };
    let chromatic_dimension = 1 - diatonic_dimension;
    for pitch in rt.fokker_block_steps(octaves.iter().sum()) {
        if pitch[diatonic_dimension] == diatonic_steps {
            pitch_stack.push(pitch);
        } else {
            diatonic_steps = pitch[diatonic_dimension];
            grid.push(pitch_stack.clone());
            pitch_stack = vec![pitch];
        }
    }
    if diatonic_steps > 100 {
        // Don't show an overly complex accordion
        return Ok(());
    }
    grid.push(pitch_stack);

    let drift = (octaves[chromatic_dimension] as f64)
        / (octaves[diatonic_dimension] as f64);
    let margin_for_pitch = |pitch: &ETMap| {
        drift * (pitch[diatonic_dimension] as f64)
            - pitch[chromatic_dimension] as f64
    };
    let mut min_margin = 1e99;
    for pitch_stack in grid.iter() {
        if let Some(pitch) = pitch_stack.iter().last() {
            let margin = margin_for_pitch(pitch);
            if margin < min_margin {
                min_margin = margin;
            }
        }
    }

    // give up on styling and use a table
    let table = web.document.create_element("table")?;
    let row = web.document.create_element("tr")?;
    for mut pitch_stack in grid {
        // The Fokker block calculation might return duplicate pitches
        // but they should at least be in the right order
        pitch_stack.dedup();
        let column = web.document.create_element("td")?;
        // Buttons are added top-down
        for (i, pitch) in pitch_stack.iter().rev().enumerate() {
            let button = accordion_button(&web, &rt, &pitch)?;
            if i == 0 {
                let button_height = 3.0;
                let margin = margin_for_pitch(pitch) - min_margin;
                button.set_attribute(
                    "style",
                    &format!("margin-top: {:.1}em", margin * button_height),
                )?;
            }
            column.append_child(&button)?;
        }
        row.append_child(&column)?;
    }
    table.append_child(&row)?;
    accordion.append_child(&table)?;
    Ok(())
}

fn accordion_button(
    web: &WebContext,
    rt: &te::TETemperament,
    pitch: &[Exponent],
) -> Result<Element, JsValue> {
    let button = web.document.create_element("button")?;
    button.set_attribute("data-steps", &join("_", &pitch))?;
    button.set_text_content(Some(&join(", ", &pitch)));
    let pitch = rt.pitch_from_steps(&pitch);
    // Tonic is middle C for now
    let freq = 264.0 * 2.0_f64.powf(pitch / 12e2);
    button.set_attribute("data-freq", &format!("{:.6}", freq))?;
    Ok(button)
}

fn result_to_option<T, E>(result: Result<T, E>) -> Option<T> {
    match result {
        Ok(value) => Some(value),
        Err(_) => None,
    }
}

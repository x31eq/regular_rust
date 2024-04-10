use std::collections::HashMap;
use std::str::FromStr;
use wasm_bindgen::prelude::{wasm_bindgen, JsValue};
use wasm_bindgen::{throw_str, JsCast};
use web_sys::js_sys::decode_uri;
use web_sys::{console, Element, Event, HtmlInputElement};

use super::cangwu::{
    ambiguous_et, get_equal_temperaments, higher_rank_search,
    CangwuTemperament,
};
use super::ratio::get_ratio_or_ket_string;
use super::te::TETemperament;
use super::temperament_class::TemperamentClass;
use super::uv::only_unison_vector;
use super::{
    join, map, normalize_positive, warted_et_name, Cents, ETMap, Exponent,
    Mapping, PrimeLimit,
};

type Exceptionable = Result<(), JsValue>;

#[wasm_bindgen]
pub fn form_submit(evt: Event) {
    evt.prevent_default();
    let web = WebContext::new();
    let mut params = HashMap::from([("page", "pregular".to_string())]);
    // The search will fail if this is missing, but the URL should make it clear why
    if let Some(limit) = web.input_value("prime-limit") {
        params.insert("limit", limit.trim().to_string());
    }
    // Same with this
    if let Some(eka) = web.input_value("prime-eka") {
        params.insert("error", eka.trim().to_string());
    }
    if let Some(n_results) = web.input_value("n-results") {
        params.insert("nresults", n_results.trim().to_string());
    }
    let hash = web.hash_from_params(&params);
    let _ = web
        .document
        .location()
        .expect("no location")
        .set_hash(&hash);
}

fn pregular_action(web: &WebContext, params: &HashMap<String, String>) {
    if let Some(limit) = params.get("limit") {
        web.set_input_value("prime-limit", &limit);
        if let Ok(limit) = limit.parse() {
            if let Some(eka) = params.get("error") {
                web.set_input_value("prime-eka", &eka);
                if let Ok(eka) = eka.parse() {
                    let nresults = params
                        .get("nresults")
                        .cloned()
                        .unwrap_or("10".to_string());
                    web.set_input_value("n-results", &nresults.to_string());
                    if let Ok(nresults) = nresults.parse() {
                        regular_temperament_search(limit, eka, nresults);
                    } else {
                        web.log_error("Failed to parse n of results");
                    }
                } else {
                    web.log_error("Unrecognized badness parameter");
                }
            } else {
                web.log_error("No target error");
            }
        } else {
            web.log_error("Unrecognized prime limit");
        }
    } else {
        web.log_error("No prime limit");
    }
}

#[wasm_bindgen(start)]
fn main() -> Result<(), JsValue> {
    process_hash();
    Ok(())
}

#[wasm_bindgen]
pub fn hash_change(_evt: Event) {
    process_hash();
}

fn process_hash() {
    let web = WebContext::new();
    let params = web.get_url_params();
    match params.get("page").map(String::as_str) {
        Some("rt") => {
            rt_action(&web, &params);
        }
        Some("pregular") => {
            pregular_action(&web, &params);
        }
        _ => (),
    }
}

fn parse_rt_params(
    params: &HashMap<String, String>,
) -> Option<(String, String, Option<String>)> {
    let ets = params.get("ets")?;
    let limit = params.get("limit")?;
    let key = params.get("key");
    Some((ets.clone(), limit.clone(), key.map(|k| k.clone())))
}

fn rt_action(web: &WebContext, params: &HashMap<String, String>) {
    if let Some((ets, limit, key)) = parse_rt_params(&params) {
        web.set_input_value("prime-limit", &limit);
        if let Ok(limit) = limit.parse::<PrimeLimit>() {
            if let Some(rt) = match key {
                Some(key) => rt_from_ets_and_key(&limit, &ets, &key),
                None => rt_from_et_names(&limit, &ets),
            } {
                web.unwrap(
                    show_rt(&web, &limit, rt.melody),
                    "Failed to show the regular temperament",
                );
                // hide the list that got enabled by that function
            } else {
                web.log_error("Unable to make temperament class");
            }
        } else {
            web.log_error("Unable to parse limit");
        }
    }
}

fn rt_from_ets_and_key<'a>(
    limit: &'a PrimeLimit,
    ets: &str,
    key: &str,
) -> Option<CangwuTemperament<'a>> {
    let ets = ets
        .split('_')
        .map(Exponent::from_str)
        .collect::<Result<Vec<_>, _>>()
        .ok()?;
    let key = key
        .split('_')
        .map(Exponent::from_str)
        .collect::<Result<Vec<_>, _>>()
        .ok()?;
    CangwuTemperament::from_ets_and_key(&limit.pitches, &ets, &key)
}

fn rt_from_et_names<'a>(
    limit: &'a PrimeLimit,
    ets: &str,
) -> Option<CangwuTemperament<'a>> {
    let ets: Vec<String> = ets.split('_').map(|s| s.to_string()).collect();
    CangwuTemperament::from_et_names(&limit, &ets)
}

pub fn regular_temperament_search(
    limit: PrimeLimit,
    ek_adjusted: Cents,
    n_results: usize,
) {
    let dimension = limit.pitches.len();
    let ek = ek_adjusted * 12e2 / limit.pitches.last().expect("no harmonics");
    let safety = if dimension < 100 {
        40
    } else {
        4 * (dimension as f64).sqrt().floor() as usize
    };
    let mappings =
        get_equal_temperaments(&limit.pitches, ek, n_results + safety);
    let web = WebContext::new();
    let list = web.unwrap(
        web.element("temperament-list").ok_or(()),
        "Couldn't find list for results",
    );
    list.set_inner_html("");
    web.set_body_class("show-list");
    web.unwrap(
        show_equal_temperaments(
            &web,
            &list,
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
        list.set_attribute("data-headings", &headings),
        "Programming Error: Failed to store headings",
    );
    web.unwrap(
        list.set_attribute("data-label", &limit.label),
        "Programming Error: Failed to store prime limit label",
    );
    web.unwrap(
        list.set_attribute("data-pitches", &join("_", &limit.pitches)),
        "Programming Error: Failed to store pitches",
    );

    let mut rts = map(|mapping| vec![mapping.clone()], &mappings);
    for rank in 2..dimension {
        let eff_n_results =
            n_results + if rank == dimension - 1 { 0 } else { safety };
        rts = higher_rank_search(
            &limit.pitches,
            &mappings,
            &rts,
            ek,
            eff_n_results,
        );
        if !rts.is_empty() {
            let visible_rts = rts.iter().take(n_results);
            web.unwrap(
                show_regular_temperaments(
                    &web,
                    &list,
                    &limit,
                    visible_rts,
                    rank,
                ),
                "Failed to display regular temperaments",
            );
        }
    }
}

struct WebContext {
    document: web_sys::Document,
}

impl WebContext {
    pub fn new() -> Self {
        let window = web_sys::window().expect("no window");
        let document = window.document().expect("no document");
        WebContext { document }
    }

    pub fn set_body_class(&self, value: &str) {
        let body = self.document.body().expect("no body");
        body.set_attribute("class", value)
            .expect("failed to set class");
    }

    pub fn element(&self, id: &str) -> Option<Element> {
        self.document.get_element_by_id(id)
    }

    pub fn input_value(&self, id: &str) -> Option<String> {
        let element = self.element(id)?;
        let input_element = element.dyn_ref::<HtmlInputElement>()?;
        Some(input_element.value())
    }

    /// Set an input if found: log errors and carry on
    pub fn set_input_value(&self, id: &str, value: &str) {
        if let Some(element) = self.element(id) {
            if let Some(input_element) = element.dyn_ref::<HtmlInputElement>()
            {
                input_element.set_value(value);
            } else {
                self.log_error("Not an input elemenet")
            }
        } else {
            self.log_error("Element not found")
        }
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
            if let Ok(query) = location.hash() {
                if let Ok(query) = decode_uri(&query) {
                    let query: String = query.into();
                    for param in query.trim_start_matches('#').split('&') {
                        if let Some((k, v)) = param.split_once('=') {
                            params.insert(k.to_string(), v.to_string());
                        }
                    }
                }
            }
        }
        params
    }

    fn hash_from_params(&self, params: &HashMap<&str, String>) -> String {
        let mut result = params
            .iter()
            .map(|(&k, v)| {
                let mut field = k.to_string();
                field.push('=');
                field.push_str(v);
                field.to_string()
            })
            .collect::<Vec<String>>()
            .join("&");
        result.insert(0, '#');
        result
    }

    pub fn log_error(&self, message: &str) {
        console::error_1(&message.into());
    }

    /// Unwrap a value with the potential of an exception
    pub fn unwrap<T, U>(&self, result: Result<T, U>, message: &str) -> T {
        match result {
            Ok(value) => value,
            Err(_) => self.fail(message),
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
    list: &Element,
    limit: &PrimeLimit,
    mappings: impl Iterator<Item = &'a ETMap>,
) -> Exceptionable {
    // This is shamelessly coupled to the HTML
    let heading = web.document.create_element("h4")?;
    heading.set_text_content(Some("Equal Temperaments"));
    list.append_child(&heading)?;
    let table = web.document.create_element("table")?;
    table.set_attribute("class", "mapping bra")?;
    write_mapping_matrix(&web, &table, &limit, mappings)?;
    list.append_child(&table)?;
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
    list: &Element,
    limit: &PrimeLimit,
    rts: impl Iterator<Item = &'a Vec<ETMap>>,
    rank: usize,
) -> Exceptionable {
    let heading = web.document.create_element("h4")?;
    let text = format!("Rank {}", rank);
    heading.set_text_content(Some(&text));
    list.append_child(&heading)?;

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
    list.append_child(&table)?;
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
    let rt = TETemperament::new(&limit.pitches, &mapping);
    link.set_attribute("href", &rt_url(&web, &limit, &rt))?;

    let octaves = map(|et| et_name(&limit, et), &mapping);
    let ets = octaves.join(" & ");

    if let Some(name) = rt.name(&limit) {
        link.set_text_content(Some(&name));
    } else if let Some(uv) = only_unison_vector(&rt.melody) {
        let norm_uv = normalize_positive(&limit.pitches, uv);
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

fn rt_url(
    web: &WebContext,
    plimit: &PrimeLimit,
    rt: &TETemperament,
) -> String {
    let ets = map(|et| et_name(&plimit, &et), &rt.melody);
    let params = HashMap::from([
        ("page", "rt".to_string()),
        ("ets", ets.join("_")),
        ("limit", plimit.label.clone()),
    ]);
    web.hash_from_params(&params)
}

/// Set the fields about the regular temperament
fn show_rt(
    web: &WebContext,
    limit: &PrimeLimit,
    mapping: Mapping,
) -> Exceptionable {
    let rt = TETemperament::new(&limit.pitches, &mapping);

    if let Some(name_field) = web.element("rt-name") {
        name_field.set_text_content(Some(&rt_name(&limit, &rt)));
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

    if let Some(field) = web.element("rt-unison-vectors") {
        field.set_inner_html("");
        let rank = rt.rank();
        let dimension = limit.pitches.len();
        let list = web.document.create_element("ul")?;
        let n_results = if (dimension - rank) == 1 {
            1
        } else {
            (dimension - rank) * 2
        };
        for uv in rt.unison_vectors(n_results) {
            let item = web.document.create_element("li")?;
            let text = get_ratio_or_ket_string(&limit, &uv);
            item.set_text_content(Some(&text));
            list.append_child(&item)?;
        }
        field.append_child(&list)?;
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
    let rt = TETemperament::new(&limit.pitches, &redmap);
    if let Some(table) = web.element("rt-generators") {
        write_float_row(&web, &table, &rt.tuning, 4)?;
    }

    if let Some(table) = web.element("rt-pote-generators") {
        write_float_row(&web, &table, &rt.pote_tuning(), 4)?;
    }

    web.set_body_class("show-temperament");
    if let Some(result) = web.element("regular-temperament") {
        result.scroll_into_view();
    }

    Ok(())
}

fn rt_name(limit: &PrimeLimit, rt: &TETemperament) -> String {
    if let Some(name) = rt.name(&limit) {
        name.to_string()
    } else {
        let octaves = map(|et| et_name(limit, et), &rt.mapping());
        octaves.join(" & ")
    }
}

fn et_name(limit: &PrimeLimit, et: &ETMap) -> String {
    assert!(!et.is_empty());
    if ambiguous_et(&limit.pitches, et) {
        warted_et_name(&limit, et)
    } else {
        et[0].to_string()
    }
}

/// An accordion is an instrument with buttons
fn show_accordion(web: &WebContext, rt: &TETemperament) -> Exceptionable {
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
    rt: &TETemperament,
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

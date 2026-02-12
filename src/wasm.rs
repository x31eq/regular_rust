use std::collections::HashMap;
use std::str::FromStr;
use wasm_bindgen::JsCast;
use wasm_bindgen::prelude::{JsValue, wasm_bindgen};
use web_sys::{Element, Event, HtmlInputElement};

use super::cangwu::{
    CangwuTemperament, ambiguous_et, get_equal_temperaments,
    higher_rank_search,
};
use super::ratio::{
    get_ratio_or_ket_string, parse_as_vector, parse_in_simplest_limit,
};
use super::te::TETemperament;
use super::temperament_class::TemperamentClass;
use super::top::TOPTemperament;
use super::tuned_temperament::TunedTemperament;
use super::uv::{ek_for_search, get_ets_tempering_out, only_unison_vector};
use super::web_context::WebContext;
use super::{
    Cents, ETMap, Exponent, Mapping, PrimeLimit, hermite_normal_form, join,
    map, normalize_positive, warted_et_name,
};

type Exceptionable = Result<(), JsValue>;

#[wasm_bindgen]
pub fn general_form_submit(evt: Event) {
    evt.prevent_default();
    let web = WebContext::new();
    let mut params = HashMap::from([("page", "pregular".to_string())]);
    // The search will fail if this is missing, but the URL
    // should make it clear why
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
    web.resubmit_with_params(&params);
}

#[wasm_bindgen]
pub fn uv_form_submit(evt: Event) {
    evt.prevent_default();
    let web = WebContext::new();
    let mut params = HashMap::from([("page", "uv".to_string())]);
    // This is optional for the UV search
    if let Some(limit) = web.input_value("uv-limit") {
        let limit = limit.trim();
        if !limit.is_empty() {
            params.insert("limit", limit.trim().to_string());
        }
    }
    // These are quite important for a unison vector search
    if let Some(uvs) = web.input_value("uv-uvs") {
        // Make these a bit cleaner in the URL bar
        let uvs: Vec<&str> = uvs.split_whitespace().collect();
        params.insert("uvs", uvs.join("+"));
    }
    if let Some(n_results) = web.input_value("uv-n-results") {
        params.insert("nresults", n_results.trim().to_string());
    }
    web.resubmit_with_params(&params);
}

#[wasm_bindgen]
pub fn net_form_submit(evt: Event) {
    evt.prevent_default();
    let web = WebContext::new();
    let mut params = HashMap::from([("page", "rt".to_string())]);
    if let Some(limit) = web.input_value("net-limit") {
        params.insert("limit", limit.trim().to_string());
    }
    if let Some(name) = web.input_value("net-steps") {
        // Make these a bit cleaner in the URL bar
        // This variable must exist to avoid freeing temporary values
        let cleaned = name.replace(['&', '+', '_', ','], " ");
        let steps: Vec<&str> = cleaned.split_whitespace().collect();
        params.insert("ets", steps.join("_"));
    }
    web.resubmit_with_params(&params);
}

fn pregular_action(
    web: &WebContext,
    params: &HashMap<String, String>,
) -> Result<(), String> {
    let limit = params.get("limit").ok_or("No prime limit")?;
    web.set_input_value("prime-limit", limit);
    let limit = limit.parse().or(Err("Unable to parse prime limit"))?;
    let eka = params.get("error").ok_or("No target error")?;
    web.set_input_value("prime-eka", eka);
    let eka = eka.parse().or(Err("Unable to parse target error"))?;
    let nresults =
        params.get("nresults").cloned().unwrap_or("10".to_string());
    web.set_input_value("n-results", &nresults.to_string());
    let nresults =
        nresults.parse().or(Err("Failed to parse n of results"))?;
    regular_temperament_search(web, limit, eka, nresults)?;
    Ok(())
}

fn uv_action(
    web: &WebContext,
    params: &HashMap<String, String>,
) -> Result<(), String> {
    if let Some(button) = web.element("show-uv")
        && let Some(button) = button.dyn_ref::<HtmlInputElement>()
    {
        // If the URL was typed in, the right search form
        // might not be showing
        button.set_checked(true);
    }
    let uv_strings: Vec<&str> = params
        .get("uvs")
        .ok_or("Unison vectors not supplied for a unison vector search")?
        .split('+')
        .collect();
    web.set_input_value("uv-uvs", &uv_strings.join(" "));
    let (limit, uvs) = if let Some(limit) = params.get("limit") {
        let limit = limit.parse().or(Err("Unable to parse prime limit"))?;
        let uvs = uv_strings
            .iter()
            .filter_map(|uv| parse_as_vector(&limit, uv))
            .collect();
        (limit, uvs)
    } else {
        parse_in_simplest_limit(&uv_strings)
            .ok_or("Unable to determine prime limit from ratios")?
    };
    let uvs: Mapping = uvs
        .into_iter()
        // Ensure everything's positive before the size check
        .map(|uv| normalize_positive(&limit.pitches, uv))
        // Filter out anything larger than a whole tone as not a unison vector
        .filter(|uv| limit.interval_size(uv) < 200.0)
        .collect();
    if uvs.is_empty() {
        return Err("No valid unison vectors in the limit".to_string());
    }
    web.set_input_value("uv-limit", &limit.label);
    // Update the input box with the filtered unison vectors
    let uv_strings = map(|uv| get_ratio_or_ket_string(&limit, uv), &uvs);
    web.set_input_value("uv-uvs", &uv_strings.join(" "));
    let ekm = if let Some(multiplier) = params.get("errmul") {
        multiplier
            .parse()
            .or(Err("Unable to parse target error multiplier"))?
    } else {
        // Default (page 2) from the old interface
        2.0
    };
    let nresults = params.get("nresults").cloned().unwrap_or("6".to_string());
    web.set_input_value("uv-n-results", &nresults.to_string());
    let nresults =
        nresults.parse().or(Err("Failed to parse n of results"))?;
    unison_vector_search(web, uvs, limit, ekm, nresults)?;
    Ok(())
}

/// Only needed for backwards compatibility for the short time when
/// there was a different page for the net search
fn net_action(
    web: &WebContext,
    params: &HashMap<String, String>,
) -> Result<(), String> {
    let steps = params.get("steps").ok_or("No list of steps")?;
    let mut params = params.clone();
    params.insert("ets".to_string(), steps.to_string().replace('+', "_"));
    rt_action(web, &params)
}

#[wasm_bindgen(start)]
fn wasm_main() -> Result<(), JsValue> {
    clear_noscript();
    process_hash();
    Ok(())
}

#[wasm_bindgen]
pub fn hash_change(_evt: Event) {
    process_hash();
}

/// Plain <noscript> tags don't show up when scripting is disabled.
/// If this is called, it must be working, so hide the message.
fn clear_noscript() {
    let web = WebContext::new();
    if let Some(message) = web.element("noscript") {
        message.set_inner_html("");
    }
}

fn process_hash() {
    let web = WebContext::new();
    let params = web.get_url_params();
    if let Err(e) = {
        match params.get("page").map(String::as_str) {
            Some("rt") => rt_action(&web, &params),
            Some("pregular") => pregular_action(&web, &params),
            Some("uv") => uv_action(&web, &params),
            Some("net") => net_action(&web, &params),
            _ => Ok(()),
        }
    } {
        web.log_error(&e);
        if let Some(error_field) = web.element("error-report") {
            error_field.set_text_content(Some(&e));
        }
        web.set_body_class("show-errors");
    }
}

fn parse_rt_params(
    params: &HashMap<String, String>,
) -> Option<(String, String, Option<String>)> {
    let ets = params.get("ets")?;
    let limit = params.get("limit")?;
    let key = params.get("key");
    Some((ets.clone(), limit.clone(), key.cloned()))
}

fn rt_action(
    web: &WebContext,
    params: &HashMap<String, String>,
) -> Result<(), String> {
    let (ets, limit, key) =
        parse_rt_params(params).ok_or("Missing parameter")?;
    web.set_input_value("prime-limit", &limit);
    let limit = limit
        .parse::<PrimeLimit>()
        .or(Err("Unable to parse limit"))?;
    let rt = match key {
        Some(key) => rt_from_ets_and_key(&limit, &ets, &key),
        None => CangwuTemperament::from_name(&limit, &ets),
    }
    .ok_or("Couldn't generate the regular temperament!")?;
    if rt.melody.len() == 1 {
        show_et(web, &limit, rt.melody)
            .or(Err("Failed to show the regular temperament"))?;
    } else {
        show_rt(web, &limit, rt.melody)
            .or(Err("Failed to show the regular temperament"))?;
    }
    Ok(())
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

fn regular_temperament_search(
    web: &WebContext,
    limit: PrimeLimit,
    ek_adjusted: Cents,
    n_results: usize,
) -> Result<(), String> {
    let dimension = limit.pitches.len();
    let ek =
        ek_adjusted * 12e2 / limit.pitches.last().ok_or("no harmonics")?;
    let safety = if dimension < 100 {
        40
    } else {
        4 * (dimension as f64).sqrt().floor() as usize
    };
    let mappings =
        get_equal_temperaments(&limit.pitches, ek, n_results + safety);
    let list = web
        .element("temperament-list")
        .ok_or("Couldn't find list for results")?;
    list.set_inner_html("");
    web.set_body_class("show-list");
    show_equal_temperaments(
        web,
        &list,
        &limit,
        mappings.iter().take(n_results),
    )
    .or(Err("Failed to display equal temperaments"))?;

    let mut rts = map(|mapping| vec![mapping.clone()], &mappings);
    for rank in 2..dimension {
        rts = higher_rank_search(
            &limit.pitches,
            &mappings,
            &rts,
            ek,
            n_results + if rank == dimension - 1 { 0 } else { safety },
        );
        if !rts.is_empty() {
            let visible_rts = rts.iter().take(n_results);
            show_regular_temperaments(web, &list, &limit, visible_rts, rank)
                .or(Err("Failed to display regular temperaments"))?
        }
    }
    Ok(())
}

fn unison_vector_search(
    web: &WebContext,
    uvs: Mapping,
    limit: PrimeLimit,
    ek_multiplier: Cents,
    n_results: usize,
) -> Result<(), String> {
    if uvs.is_empty() {
        return Err("No unison vectors supplied".to_string());
    }
    let ek = ek_for_search(&limit.pitches, &uvs) * ek_multiplier;
    let dimension = limit.pitches.len();
    let corank = hermite_normal_form(&uvs).len();
    if corank == dimension {
        return Err(
            "Too many unison vectors: whole space matches".to_string()
        );
    }
    let highest_rank = dimension - corank;
    let mappings = get_ets_tempering_out(
        &limit.pitches,
        ek,
        &uvs,
        if highest_rank == 1 { 1 } else { n_results },
    );
    let list = web
        .element("temperament-list")
        .ok_or("Couldn't find list for results")?;
    list.set_inner_html("");
    web.set_body_class("show-list");
    show_equal_temperaments(web, &list, &limit, mappings.iter())
        .or(Err("Failed to display equal temperaments"))?;

    if highest_rank == 1 {
        return Ok(());
    }
    let mut rts = map(|mapping| vec![mapping.clone()], &mappings);
    for rank in 2..(highest_rank + 1) {
        rts = higher_rank_search(
            &limit.pitches,
            &mappings,
            &rts,
            ek,
            if rank == highest_rank { 1 } else { n_results },
        );
        if !rts.is_empty() {
            show_regular_temperaments(web, &list, &limit, rts.iter(), rank)
                .or(Err("Failed to display regular temperaments"))?
        }
    }
    Ok(())
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
    write_equal_temperaments(web, &table, limit, mappings)?;
    list.append_child(&table)?;
    Ok(())
}

/// Equal temperaments are shown as a mapping matrix
/// with links to the page for the temperament
fn write_equal_temperaments<'a>(
    web: &WebContext,
    table: &Element,
    limit: &PrimeLimit,
    values: impl Iterator<Item = &'a ETMap>,
) -> Exceptionable {
    write_headings(web, table, limit)?;
    let body = web.new_or_emptied_element(table, "tbody")?;
    for vector in values {
        let row = web.document.create_element("tr")?;
        let rt =
            TETemperament::new(&limit.pitches, std::slice::from_ref(vector));
        let url = rt_url(web, limit, &rt);
        for element in vector {
            let cell = web.document.create_element("td")?;
            let link = web.document.create_element("a")?;
            link.set_attribute("href", &url)?;
            link.set_text_content(Some(&element.to_string()));
            cell.append_child(&link)?;
            row.append_child(&cell)?;
        }
        body.append_child(&row)?;
    }
    Ok(())
}

fn write_mapping_matrix<'a>(
    web: &WebContext,
    table: &Element,
    limit: &PrimeLimit,
    values: impl Iterator<Item = &'a ETMap>,
) -> Exceptionable {
    write_headings(web, table, limit)?;
    let body = web.new_or_emptied_element(table, "tbody")?;
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
    let head = web.new_or_emptied_element(table, "thead")?;
    let row = web.document.create_element("tr")?;
    for heading in limit.headings.iter() {
        let cell = web.document.create_element("th")?;
        cell.set_text_content(Some(heading));
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
    let body = web.new_or_emptied_element(table, "tbody")?;
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
    for column_heading in &[
        "Name",
        "ETs",
        "complexity",
        "error",
        "TE error",
        "TOP error",
    ] {
        let cell = web.document.create_element("th")?;
        cell.set_text_content(Some(column_heading));
        row.append_child(&cell)?;
    }
    table.append_child(&row)?;

    for rt in rts {
        let row = rt_row(rt, limit, web)?;
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

    // Set up the link as a link
    let rt = TETemperament::new(&limit.pitches, mapping);
    link.set_attribute("href", &rt_url(web, limit, &rt))?;

    let octaves = map(|et| et_name(limit, et), mapping);
    let ets = octaves.join(" & ");

    if let Some(name) = rt.name(limit) {
        link.set_text_content(Some(name));
    } else if let Some(uv) = only_unison_vector(&rt.melody) {
        let norm_uv = normalize_positive(&limit.pitches, uv);
        let name = get_ratio_or_ket_string(limit, &norm_uv);
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

    let cell = web.document.create_element("td")?;
    cell.set_text_content(Some(&format!("{:.3} cent/oct", rt.error())));
    row.append_child(&cell)?;

    let cell = web.document.create_element("td")?;
    if let Ok(top_rt) = TOPTemperament::new(&limit.pitches, mapping) {
        cell.set_text_content(Some(&format!(
            "{:.3} cent/oct",
            top_rt.error()
        )));
    } else {
        cell.set_text_content(Some("n/a"));
    }
    row.append_child(&cell)?;

    Ok(row)
}

fn rt_url(
    web: &WebContext,
    plimit: &PrimeLimit,
    rt: &TETemperament,
) -> String {
    let ets = map(|et| et_name(plimit, et), &rt.melody);
    let params = HashMap::from([
        ("page", "rt".to_string()),
        ("ets", ets.join("_")),
        ("limit", plimit.label.clone()),
    ]);
    web.hash_from_params(&params)
}

/// Set the fields about the equal temperament
fn show_et(
    web: &WebContext,
    limit: &PrimeLimit,
    mapping: Mapping,
) -> Exceptionable {
    let rt = TETemperament::new(&limit.pitches, &mapping);

    if let Some(name_field) = web.element("et-name") {
        name_field.set_text_content(Some(&rt_name(limit, &rt)));
    }

    if let Some(table) = web.element("et-etmap") {
        write_mapping_matrix(web, &table, limit, mapping.iter())?;
    }

    if let Some(table) = web.element("et-tuning-map") {
        write_headings(web, &table, limit)?;
        write_float_row(web, &table, &rt.tuning_map(), 3)?;
    }

    if let Some(table) = web.element("et-pote-tuning-map") {
        write_headings(web, &table, limit)?;
        write_float_row(web, &table, &rt.unstretched_tuning_map(), 3)?;
    }

    if let Some(table) = web.element("et-mistunings") {
        write_headings(web, &table, limit)?;
        write_float_row(web, &table, &rt.mistunings(), 4)?;
    }

    if let Some(table) = web.element("et-pote-mistunings") {
        write_headings(web, &table, limit)?;
        write_float_row(web, &table, &rt.unstretched_mistunings(), 4)?;
    }

    if let Some(field) = web.element("et-te-error") {
        field.set_text_content(Some(&format!("{:.6}", rt.error())));
    }

    if let Some(field) = web.element("et-unison-vectors") {
        list_unison_vectors(web, limit, &rt, &field)?;
    }

    if let Some(field) = web.element("et-error") {
        field.set_text_content(Some(&format!("{:.6}", rt.adjusted_error())));
    }

    if let Some(field) = web.element("et-te-stretch") {
        let stretch = (rt.stretch() - 1.0) * 1200.0;
        field.set_text_content(Some(&format!("{:.6}", stretch)));
    }

    // Now do the TOP fields
    if let Ok(rt) = TOPTemperament::new(&limit.pitches, &mapping) {
        if let Some(table) = web.element("et-top-tuning-map") {
            write_headings(web, &table, limit)?;
            write_float_row(web, &table, &rt.tuning_map(), 3)?;
        }

        if let Some(table) = web.element("et-toppo-tuning-map") {
            write_headings(web, &table, limit)?;
            write_float_row(web, &table, &rt.unstretched_tuning_map(), 3)?;
        }

        if let Some(table) = web.element("et-top-mistunings") {
            write_headings(web, &table, limit)?;
            write_float_row(web, &table, &rt.mistunings(), 4)?;
        }

        if let Some(table) = web.element("et-toppo-mistunings") {
            write_headings(web, &table, limit)?;
            write_float_row(web, &table, &rt.unstretched_mistunings(), 4)?;
        }

        if let Some(field) = web.element("et-top-error") {
            field.set_text_content(Some(&format!("{:.6}", rt.error())));
        }

        if let Some(field) = web.element("et-top-stretch") {
            let stretch = (rt.stretch() - 1.0) * 1200.0;
            field.set_text_content(Some(&format!("{:.6}", stretch)));
        }
    }

    web.set_body_class("show-et");
    if let Some(result) = web.element("equal-temperament") {
        result.scroll_into_view();
    }

    Ok(())
}
/// Set the fields about the regular temperament
fn show_rt(
    web: &WebContext,
    limit: &PrimeLimit,
    mapping: Mapping,
) -> Exceptionable {
    let rt = TETemperament::new(&limit.pitches, &mapping);

    if let Some(name_field) = web.element("rt-name") {
        name_field.set_text_content(Some(&rt_name(limit, &rt)));
    }

    if let Some(table) = web.element("rt-etmap") {
        write_mapping_matrix(web, &table, limit, mapping.iter())?;
    }

    let redmap = rt.reduced_mapping();
    if let Some(table) = web.element("rt-redmap") {
        write_mapping_matrix(web, &table, limit, redmap.iter())?;
    }

    if let Some(table) = web.element("rt-steps") {
        write_float_row(web, &table, &rt.tuning, 4)?;
    }

    if let Some(table) = web.element("rt-pote-steps") {
        write_float_row(web, &table, &rt.unstretched_tuning(), 4)?;
    }

    if let Some(table) = web.element("rt-tuning-map") {
        write_headings(web, &table, limit)?;
        write_float_row(web, &table, &rt.tuning_map(), 3)?;
    }

    if let Some(table) = web.element("rt-pote-tuning-map") {
        write_headings(web, &table, limit)?;
        write_float_row(web, &table, &rt.unstretched_tuning_map(), 3)?;
    }

    if let Some(table) = web.element("rt-mistunings") {
        write_headings(web, &table, limit)?;
        write_float_row(web, &table, &rt.mistunings(), 4)?;
    }

    if let Some(table) = web.element("rt-pote-mistunings") {
        write_headings(web, &table, limit)?;
        write_float_row(web, &table, &rt.unstretched_mistunings(), 4)?;
    }

    if let Some(field) = web.element("rt-complexity") {
        let text = format!("{:.6}", rt.complexity());
        field.set_text_content(Some(&text));
    }

    if let Some(field) = web.element("rt-te-error") {
        field.set_text_content(Some(&format!("{:.6}", rt.error())));
    }

    if let Some(field) = web.element("rt-unison-vectors") {
        list_unison_vectors(web, limit, &rt, &field)?;
    }

    if let Some(field) = web.element("error") {
        field.set_text_content(Some(&format!("{:.6}", rt.adjusted_error())));
    }

    if show_accordion(web, &rt).is_err()
        && let Some(accordion) = web.element("rt-accordion")
    {
        // This is an optional feature,
        // so hide it if something went wrong
        accordion.set_inner_html("<!-- accordion went wrong -->");
    }

    // Make another RT object to get the generator tunings
    let rt = TETemperament::new(&limit.pitches, &redmap);
    if let Some(table) = web.element("rt-generators") {
        write_float_row(web, &table, &rt.tuning, 4)?;
    }

    if let Some(table) = web.element("rt-pote-generators") {
        write_float_row(web, &table, &rt.unstretched_tuning(), 4)?;
    }

    web.set_body_class("show-temperament");
    if let Some(result) = web.element("regular-temperament") {
        result.scroll_into_view();
    }

    // Now do it all again with TOP
    if let Ok(rt) = TOPTemperament::new(&limit.pitches, &mapping) {
        if let Some(table) = web.element("rt-top-steps") {
            write_float_row(web, &table, &rt.tuning, 4)?;
        }

        if let Some(table) = web.element("rt-toppo-steps") {
            write_float_row(web, &table, &rt.unstretched_tuning(), 4)?;
        }

        if let Some(table) = web.element("rt-top-tuning-map") {
            write_headings(web, &table, limit)?;
            write_float_row(web, &table, &rt.tuning_map(), 3)?;
        }

        if let Some(table) = web.element("rt-toppo-tuning-map") {
            write_headings(web, &table, limit)?;
            write_float_row(web, &table, &rt.unstretched_tuning_map(), 3)?;
        }

        if let Some(table) = web.element("rt-top-mistunings") {
            write_headings(web, &table, limit)?;
            write_float_row(web, &table, &rt.mistunings(), 4)?;
        }

        if let Some(table) = web.element("rt-toppo-mistunings") {
            write_headings(web, &table, limit)?;
            write_float_row(web, &table, &rt.unstretched_mistunings(), 4)?;
        }

        if let Some(field) = web.element("rt-top-error") {
            field.set_text_content(Some(&format!("{:.6}", rt.error())));
        }
    } else {
        web.log_error("Failed to calculate TOP tuning");
    }

    // Now another RT object for TOP family generator tunings
    if let Ok(rt) = TOPTemperament::new(&limit.pitches, &redmap) {
        if let Some(table) = web.element("rt-top-generators") {
            write_float_row(web, &table, &rt.tuning, 4)?;
        }

        if let Some(table) = web.element("rt-toppo-generators") {
            write_float_row(web, &table, &rt.unstretched_tuning(), 4)?;
        }
    } else {
        web.log_error("Failed to calculate TOP generator tuning");
    }
    Ok(())
}

fn list_unison_vectors(
    web: &WebContext,
    limit: &PrimeLimit,
    rt: &TETemperament,
    field: &Element,
) -> Exceptionable {
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
        let text = get_ratio_or_ket_string(limit, &uv);
        item.set_text_content(Some(&text));
        list.append_child(&item)?;
    }
    field.append_child(&list)?;
    Ok(())
}

fn rt_name(limit: &PrimeLimit, rt: &TETemperament) -> String {
    if let Some(name) = rt.name(limit) {
        name.to_string()
    } else {
        let octaves = map(|et| et_name(limit, et), rt.mapping());
        octaves.join(" & ")
    }
}

fn et_name(limit: &PrimeLimit, et: &ETMap) -> String {
    assert!(!et.is_empty());
    if ambiguous_et(&limit.pitches, et) {
        warted_et_name(limit, et)
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
            let button = accordion_button(web, rt, pitch)?;
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
    button.set_attribute("data-steps", &join("_", pitch))?;
    button.set_text_content(Some(&join(", ", pitch)));
    let pitch = rt.pitch_from_steps(pitch);
    // Tonic is middle C for now
    let freq = 264.0 * 2.0_f64.powf(pitch / 12e2);
    button.set_attribute("data-freq", &format!("{:.6}", freq))?;
    Ok(button)
}

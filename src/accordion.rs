use super::tuned_temperament::TunedTemperament;
use super::web_context::{Exceptionable, WebContext};
use super::{ETMap, Exponent, join, map};
use wasm_bindgen::prelude::JsValue;
use web_sys::Element;

/// An accordion is an instrument with buttons
pub fn show_accordion(web: &WebContext, rt: &impl TunedTemperament) -> Exceptionable {
    let Some(accordion) = web.emptied_element("rt-accordion") else {
        return Ok(());
    };
    let rank = rt.mapping().len();
    if rank != 2 {
        return Ok(());
    }
    let tonic: ETMap = (0..rank).map(|_| 0).collect();
    let mut diatonic_steps = 0;
    let mut pitch_stack = vec![tonic.clone()];
    let mut grid = Vec::new();
    let octaves: ETMap = map(|m| m[0], &rt.mapping());
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
    rt: &impl TunedTemperament,
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

mod model;
mod vocab;
mod constants;
mod interpolate;

use ndarray::{Array1, Array2, Array3};
use ndarray_npy::NpzReader;
use std::io::Cursor;

use crate::model::NeuroQuery;
use crate::constants::Constants;
use crate::vocab::Terms;

use std::error::Error;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsValue;
//use js_sys::Uint8Array;

#[macro_use]
extern crate lazy_static;
use std::sync::Mutex;
lazy_static! {
    static ref GLOBAL_CONSTANTS: Mutex<Option<Constants>> = Mutex::new(None);
}

#[wasm_bindgen]
pub fn run_query(query: &str)-> Vec<f32> {
    // Read from global variables
    let globals = GLOBAL_CONSTANTS.lock().unwrap();
    let constants = globals.as_ref().unwrap();

    // Initialize model
    let mut model = NeuroQuery {
        terms: Terms::default().terms,
        query: String::from("working memory"),
        tokens: Array2::zeros((1, 6308)),
        ntokens: 0i32,
        constants: constants.clone(), // todo: avoid cloning
        vt : Array2::zeros((0, 0)),
        vnorm: Array2::zeros((0, 0)),
        img3d : Array3::zeros((46, 55, 46))
    };

    // Set V^T and V_norm
    model.smoothing_matrices();

    // Pass query to the model
    model.new_query(query.to_string());

    // Tokenize query
    model.tokenize();

    // Transform the model and destructure the result
    let (lsurface, rsurface) = model.transform();

    // Convert the arrays to Vec<f32> and wrap in JsValue for JavaScript
    let mut lsurface_vec: Vec<f32> = lsurface.to_vec();
    let mut rsurface_vec: Vec<f32> = rsurface.to_vec();

    lsurface_vec.append(&mut rsurface_vec);

    // Return a JavaScript object
    lsurface_vec
}

#[wasm_bindgen]
pub fn run_inverse_query(vertex: JsValue) -> Vec<String> {
    // Coerce type
    let vertex = match vertex.as_f64() {
        Some(v) => v,
        None => {
            // Handle the error by returning an empty Vec or an error value
            println!("Error: Expected a float value");
            return vec![]; // Return an empty vector or handle as needed
        }
    };

    // Round the value and convert to i32
    let rounded_value = vertex.round();
    let vind = rounded_value as i32;

    // Ensure vind is non-negative and within bounds of z
    if vind < 0 {
        println!("Error: Negative index");
        return vec![];
    }

    // Read from global variables
    let globals = GLOBAL_CONSTANTS.lock().unwrap();
    let constants = globals.as_ref().unwrap();

    // Initialize model
    let mut model = NeuroQuery {
        terms: Terms::default().terms,
        query: String::from("working memory"),
        tokens: Array2::zeros((1, 6308)),
        ntokens: 0i32,
        constants: constants.clone(), // todo: avoid cloning
        vt : Array2::zeros((0, 0)),
        vnorm: Array2::zeros((0, 0)),
        img3d : Array3::zeros((46, 55, 46))
    };

    // Set V^T and V_norm
    model.smoothing_matrices();

    // Inverse transform
    model.inverse_transform(vind)
}


#[wasm_bindgen]
pub fn read_npz_file(data: &[u8]) -> Result<(), JsValue> {
    // Read npz buffer
    let cursor = Cursor::new(data);
    let mut npz = NpzReader::new(cursor).map_err(
        |e| JsValue::from_str(&format!("Failed to read .npz file: {:?}", e))
    )?;
    let constants = Constants::new(npz).unwrap();
    // Make global, e.g. so .npz is only read once
    let mut globals = GLOBAL_CONSTANTS.lock().unwrap();
    *globals = Some(constants);
    Ok(())
}

// Console logging
#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

fn console_log(message: &str) {
    log(message);
}

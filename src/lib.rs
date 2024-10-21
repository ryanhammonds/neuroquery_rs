mod model;
mod vocab;
mod constants;
mod coef;
mod mask;
mod M;
mod regfusion_left;
mod regfusion_right;
mod interpolate;

use std::collections::HashMap;
use ndarray::Array1;
use crate::model::NeuroQuery;
use wasm_bindgen::prelude::*;
//use js_sys::Float64Array;
use wasm_bindgen::JsValue;
use wasm_bindgen::prelude::*;


#[wasm_bindgen]
pub fn run_query(query: &str)-> Vec<f32> {

    let mut model = NeuroQuery {
        ..Default::default()
    };

    // Pass the query to the model
    model.new_query(query.to_string()); // Call to new_query with the query string

    // Tokenize the query (assuming this is defined in NeuroQuery)
    model.tokenize();

    // Transform the model and destructure the result
    let (lsurface, rsurface) = model.transform();

    // Convert the arrays to Vec<f32> and wrap in JsValue for JavaScript
    let mut lsurface_vec: Vec<f32> = lsurface.to_vec();
    let mut rsurface_vec: Vec<f32> = rsurface.to_vec();

    lsurface_vec.append(&mut rsurface_vec);

    // Return a JavaScript object
    //JsValue::from_vec(&surface_vec).unwrap
    lsurface_vec
}

// Couldn't get the below to work.
// Not working likely from trying to reaccess structafter it left scope
// The idea was to prevent re-inializing the strucutre, since large
// arrays are read in everytim.

//
// #[wasm_bindgen]
// pub struct Query {
//     model: NeuroQuery,  // Keep model as a private field
// }

// #[wasm_bindgen]
// impl Query {
//     // Constructor to initialize the model only once
//     #[wasm_bindgen(constructor)]
//     pub fn new() -> Query {
//         // Initialize the model only once
//         let mut model = NeuroQuery {
//             ..Default::default()
//         };
//         Query { model }
//     }

//     // Run method processes the query and returns lsurface and rsurface
//     pub fn run(&mut self, query: &str) -> Vec<f32> {
//         // Pass the query to the model
//         self.model.new_query(query.to_string()); // Call to new_query with the query string

//         // Tokenize the query (assuming this is defined in NeuroQuery)
//         self.model.tokenize();

//         // Transform the model and destructure the result
//         let (lsurface, rsurface) = self.model.transform();

//         // Convert the arrays to Vec<f32> and wrap in JsValue for JavaScript
//         let mut lsurface_vec: Vec<f32> = lsurface.to_vec();
//         let mut rsurface_vec: Vec<f32> = rsurface.to_vec();

//         lsurface_vec.append(&mut rsurface_vec);

//         // Return a JavaScript object
//         //JsValue::from_vec(&surface_vec).unwrap
//         lsurface_vec
//     }
// }


// Model
use std::collections::HashMap;
use crate::vocab::Terms;
use crate::constants::Constants;

use crate::interpolate::trilinear_interpolation;
use ndarray::{Array3, Array2, Array1, array};
use ndarray_npy::NpzReader;

use std::io::{Read, Seek};
use std::error::Error;

pub struct NeuroQuery{
    pub terms: HashMap<String, i32>,
    pub query: String,
    pub tokens: Array2<f32>,
    pub ntokens: i32,
    pub constants: Constants,
    pub vt: Array2<f32>,
    pub vnorm: Array2<f32>,
    pub img3d: Array3<f32>,
}

impl NeuroQuery {
    pub fn new<R: Read + Seek>(npz: NpzReader<R>) -> Result<Self, Box<dyn Error>> {
        // Read in constants
        let constants = Constants::new(npz)?;

        // Initialize model
        let mut model = NeuroQuery {
            terms: Terms::default().terms,
            query: String::from("working memory"),
            tokens: Array2::zeros((1, 6308)),
            ntokens: 0i32,
            constants: constants,
            vt : Array2::zeros((0, 0)),
            vnorm: Array2::zeros((0, 0)),
            img3d : Array3::zeros((46, 55, 46))
        };
        // Set V^T and V_norm
        model.smoothing_matrices();
        Ok(model)
    }
    pub fn new_query(&mut self, query : String) {
        // Allows new, subsequent queries without recomputing constant matrices
        self.query = query;
    }
    pub fn tokenize(&mut self){
        // Lazy tokenizer, needs an actual one. See sklearn or tokenizer crate
        let qvec : Vec<String> = self.query
            .split_whitespace()
            .map(|word| word.to_lowercase().chars()
                .take_while(|ch| ch.is_alphanumeric())
                .collect::<String>())
            .collect();
        // Hashmap lookup
        let mut i : usize = 0;
        let mut indices : Vec<i32> = Vec::new();
        let mut idf : Vec<f32> = Vec::new();
        let mut l2norm : f32 = 0.0;
        while i <= (qvec.len()-1){
            if i < (qvec.len()-1){
                let mut pair : String = qvec[i].clone();
                pair.push_str(" ");
                pair.push_str(&qvec[i+1].clone());
                if self.terms.contains_key(&pair){
                    // Check pairs of words
                    indices.push(self.terms[&pair]);
                    idf.push(self.constants.idf[self.terms[&pair] as usize]);
                    l2norm = l2norm + self.constants.idf[self.terms[&pair] as usize].powi(2);
                    i = i+2;
                    continue;
                }
            }
            if self.terms.contains_key(&qvec[i]){
                // Check individual words
                indices.push(self.terms[&qvec[i]]);
                idf.push(self.constants.idf[self.terms[&qvec[i]] as usize]);
                l2norm = l2norm + self.constants.idf[self.terms[&qvec[i]] as usize].powi(2);
            }
            i = i + 1;
        }

        // Compute token vector
        l2norm = l2norm.sqrt();

        let mut j : usize = 0;
        for i in 0..indices.len(){
            self.tokens[[0 as usize, indices[i] as usize]] = idf[j] / l2norm;
            j = j + 1;
        }
        self.ntokens = j as i32;
    }
    pub fn smoothing_matrices(&mut self){
        // V^T
        let mut _vt = Array1::<f32>::zeros(1387760);
        for i in 0..self.constants.i_vt_nz.len(){
            _vt[self.constants.i_vt_nz[i] as usize] = self.constants.vt_nz[i];
        }
        self.vt = _vt.into_shape_clone((220, 6308)).expect("reshape failed");
        // V normalized
        let mut _vnorm = Array1::<f32>::zeros(1387760);
        for i in 0..self.constants.i_norm_vt_nz.len(){
            _vnorm[self.constants.i_norm_vt_nz[i] as usize] = self.constants.norm_vt_nz[i];
        }
        self.vnorm = _vnorm.into_shape_clone((6308, 220)).expect("reshape failed");
    }
    pub fn transform(&mut self) -> (Array1<f32>, Array1<f32>){

        // Copmutes surface map corresponding to a query
        let mut tokens_nz = Array2::<f32>::zeros((1, self.ntokens as usize));
        let mut vnorm_nz = Array2::<f32>::zeros((self.ntokens as usize, 220));
        let mut j : usize = 0;

        for i in 0..6308{
            if self.tokens[[0, i]] != 0.0{
                tokens_nz[[0, j]] = self.tokens[[0, i]];
                for k in 0..220{
                    vnorm_nz[[j, k]] = self.vnorm[[i, k]];
                }
                j = j + 1;
            }
        }

        let smoothed_tokens =
            tokens_nz.dot(&vnorm_nz.dot(&self.vt)) * 0.1 +
            &self.tokens * 0.9
        ;

        // Feature selection
        let mut tokens_selected = Array1::<f32>::zeros(self.constants.selected_features.len());
        for i in 0..self.constants.selected_features.len(){
            tokens_selected[i] = smoothed_tokens[[0, self.constants.selected_features[i] as usize]]
        }

        // Prediction variance
        let XM = tokens_selected.dot(&self.constants.m);
        let XMMtXt = XM.mapv_into(|v| v*v).sum();

        // Transform to z-map
        let mut z = tokens_selected.dot(&self.constants.coef);
        for i in 0..z.len(){
            z[i] = z[i] / (self.constants.res_var[i] * XMMtXt).sqrt();
            if z[i] <= 0.{
                z[i] = 1e-23f32;
            }
        }

        // 3d
        let dims = self.img3d.dim();
        let mut ii : usize = 0;
        let mut jj : usize = 0;
        for i in 0..dims.0{
            for j in 0..dims.1{
                for k in 0..dims.2{
                    if self.constants.mask[jj]{
                        self.img3d[[i, j, k]] = z[ii];
                        ii = ii + 1;
                    }
                    jj = jj+ 1;
                }
            }
        }
        // Intepolation to surface
        let rzs : Array2<f32> = array![
            [0.25, 0.  , 0.  ],
            [0.  , 0.25, 0.  ],
            [0.  , 0.  , 0.25]
        ];
        let trans : Array1<f32> = array![22.5, 31.5, 18. ];

        let coords_l = self.constants.l_reg_fus.dot(&rzs) + &trans;
        let coords_r = self.constants.r_reg_fus.dot(&rzs) + &trans;

        let ix = Array1::range(0., 46., 1.0);
        let iy = Array1::range(0., 55., 1.0);
        let iz = Array1::range(0., 46., 1.0);

        let lsurface = trilinear_interpolation((&ix, &iy, &iz), self.img3d.view(), &coords_l);
        let rsurface = trilinear_interpolation((&ix, &iy, &iz), self.img3d.view(), &coords_r);

        return (lsurface, rsurface);
    }
    pub fn inverse_transform(&mut self, vertex: i32)-> Vec<String>{
        // Given an activation, predict associated words:
        //   x = (z coef^{-1} M^{-1}) \sqrt{\sigma^2}
        //   x = x / ||x||_2

        // Refactor below to a constant vector
        let mut res_sd = Array1::<f32>::zeros(self.constants.res_var.len());
        for i in 0..self.constants.res_var.len(){
            res_sd[i] = (self.constants.res_var[i]).sqrt();
        }

        // Inversion
        let mut tokens_selected = Array1::<f32>::zeros(188);
        for i in 0..188{
            tokens_selected[i] = self.constants.coef_inv[[i, vertex as usize]] * 10.0
        }

        let tvec = tokens_selected.to_vec();
        let mut top_terms: Vec<String> = Vec::new();

        // Sorted term indices
        let mut tvec_argsort: Vec<usize> = (0..tvec.len()).collect();
        tvec_argsort.sort_by(|&i, &j| tvec[j].partial_cmp(&tvec[i]).unwrap());

        // Return top ten associated terms
        for i in 0..10{
            let j = self.constants.selected_features[tvec_argsort[i] as usize];
            let key = self.terms.iter()
                .find_map(|(&ref key, &value)| if value == j { Some(key) } else { None });
            top_terms.push(key.expect("REASON").to_string());
        }
        top_terms
    }
}
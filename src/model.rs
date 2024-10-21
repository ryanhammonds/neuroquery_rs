// Model
use std::collections::HashMap;
use crate::vocab::Terms;
use crate::constants::*;
use crate::M::concat_m;
use crate::coef::concat_coefs;
use crate::mask::mask1d;
use crate::regfusion_left::*;
use crate::regfusion_right::*;
use crate::interpolate::trilinear_interpolation;
use ndarray::{Array3, Array2, Array1, array};

pub struct NeuroQuery{
    pub terms: HashMap<String, i32>,
    pub query: String,
    pub idf : Vec<f32>,
    pub tokens : Array2<f32>,
    pub ntokens : i32,
    pub Vt : Array2<f32>,
    pub Vnorm : Array2<f32>,
    pub M : Array2<f32>,
    pub residual_variance : Array1<f32>,
    pub coef : Array2<f32>,
    pub img3d : Array3<f32>,
    pub mask3d : Array3<bool>,
    pub lregfusion : Array2<f32>,
    pub rregfusion : Array2<f32>
}

impl Default for NeuroQuery{
    fn default() ->  NeuroQuery {
        let mut model = NeuroQuery {
            terms: Terms::default().terms,
            query: String::from("working memory"),
            idf: Terms::default().idf,
            tokens: Array2::zeros((1, 6308)),
            ntokens: 0i32,
            Vt: Array2::zeros((220, 6308)),
            Vnorm: Array2::zeros((220, 6308)),
            M: Array2::zeros((188, 13459)),
            residual_variance: Array1::zeros(28542),
            coef: Array2::zeros((188, 28542)),
            img3d : Array3::zeros((46, 55, 46)),
            mask3d : Array3::from_elem((46, 55, 46), false),
            lregfusion : Array2::zeros((163842, 3)),
            rregfusion : Array2::zeros((163842, 3))
        };
        // Set constant matrices
        model.smoothing_matrices();
        model.regfusion_matrices();
        model.variance_matrices();
        model
    }
}

impl NeuroQuery {
    pub fn new() -> NeuroQuery {
        // Initializes with default parameters
        Default::default()
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
                    idf.push(self.idf[self.terms[&pair] as usize]);
                    l2norm = l2norm + self.idf[self.terms[&pair] as usize].powi(2);
                    i = i+2;
                }
            }
            if self.terms.contains_key(&qvec[i]){
                // Check individual words
                indices.push(self.terms[&qvec[i]]);
                idf.push(self.idf[self.terms[&qvec[i]] as usize]);
                l2norm = l2norm + self.idf[self.terms[&qvec[i]] as usize].powi(2);
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
        let mut _Vt = Array1::<f32>::zeros(1387760);
        for i in 0..i_Vt_nz.len(){
            _Vt[i_Vt_nz[i] as usize] = Vt_nz[i];
        }
        self.Vt = _Vt.clone().into_shape((220, 6308)).expect("reshape failed");
        // V normalized
        let mut _Vnorm = Array1::<f32>::zeros(1387760);
        for i in 0..i_normalized_V_nz.len(){
            _Vnorm[i_normalized_V_nz[i] as usize] = normalized_V_nz[i];
        }
        self.Vnorm = _Vnorm.clone().into_shape((6308, 220)).expect("reshape failed");
    }
    pub fn variance_matrices(&mut self){
        self.M = concat_m();
        self.coef = concat_coefs();
    }
    pub fn regfusion_matrices(&mut self){
        for i in 0..163842{
            self.lregfusion[[i, 0]] = lregfus0[i];
            self.lregfusion[[i, 1]] = lregfus1[i];
            self.lregfusion[[i, 2]] = lregfus2[i];
            self.rregfusion[[i, 0]] = rregfus0[i];
            self.rregfusion[[i, 1]] = rregfus1[i];
            self.rregfusion[[i, 2]] = rregfus2[i];
        }
    }

    pub fn transform(&mut self) -> (Array1<f32>, Array1<f32>){

        // Copmutes surface map corresponding to a query
        let mut tokens_nz = Array2::<f32>::zeros((1, self.ntokens as usize));
        let mut Vnorm_nz = Array2::<f32>::zeros((self.ntokens as usize, 220));
        let mut j : usize = 0;

        for i in 0..6308{
            if self.tokens[[0, i]] != 0.0{
                tokens_nz[[0, j]] = self.tokens[[0, i]];
                for k in 0..220{
                    Vnorm_nz[[j, k]] = self.Vnorm[[i, k]];
                }
                j = j + 1;
            }
        }

        let smoothed_tokens =
            tokens_nz.dot(&Vnorm_nz.dot(&self.Vt)) * 0.1 +
            &self.tokens * 0.9
        ;

        // Feature selection
        let mut tokens_selected = Array1::<f32>::zeros(selected_features.len());
        for i in 0..selected_features.len(){
            tokens_selected[i] = smoothed_tokens[[0, selected_features[i] as usize]]
        }

        // Prediction variance
        let XM = tokens_selected.dot(&self.M);

        let XMMtXt = XM.mapv_into(|v| v*v).sum();

        for i in 0..self.residual_variance.len(){
            self.residual_variance[i] = (resvar[i] * XMMtXt).sqrt();
        }

        // Transform to z-map
        let mut z = tokens_selected.dot(&self.coef);
        for i in 0..z.len(){
            z[i] = z[i] / self.residual_variance[i];
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
                    if mask1d[jj]{
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

        let coords_l = self.lregfusion.dot(&rzs) + &trans;
        let coords_r = self.rregfusion.dot(&rzs) + &trans;

        let ix = Array1::range(0., 46., 1.0);
        let iy = Array1::range(0., 55., 1.0);
        let iz = Array1::range(0., 46., 1.0);

        let lsurface = trilinear_interpolation((&ix, &iy, &iz), self.img3d.view(), &coords_l);
        let rsurface = trilinear_interpolation((&ix, &iy, &iz), self.img3d.view(), &coords_r);

        return (lsurface, rsurface);
    }
}




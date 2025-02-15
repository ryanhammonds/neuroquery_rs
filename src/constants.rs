// Load constant vectors and matrices
use ndarray::{Array1, Array2};
use ndarray_npy::NpzReader;
use std::error::Error;
use std::io::{Read, Seek};

#[derive(Clone)]
pub struct Constants{
    pub m: Array2<f32>,
    pub m_inv: Array2<f32>,
    pub coef: Array2<f32>,
    pub coef_inv: Array2<f32>,
    pub idf: Array1<f32>,
    pub r_reg_fus: Array2<f32>,
    pub l_reg_fus: Array2<f32>,
    pub mask: Array1<bool>,
    pub selected_features: Array1<i32>,
    pub res_var: Array1<f32>,
    pub vt_nz: Array1<f32>,
    pub i_vt_nz: Array1<i32>,
    pub norm_vt_nz: Array1<f32>,
    pub i_norm_vt_nz: Array1<i32>,
}

impl Default for Constants {
    fn default() -> Self {
        Constants {
            m: Array2::zeros((0, 0)),
            m_inv: Array2::zeros((0, 0)),
            coef: Array2::zeros((0, 0)),
            coef_inv: Array2::zeros((0, 0)),
            idf: Array1::zeros(0),
            r_reg_fus: Array2::zeros((0, 0)),
            l_reg_fus: Array2::zeros((0, 0)),
            mask: Array1::from_elem(0, false),
            selected_features: Array1::zeros(0),
            res_var: Array1::zeros(0),
            vt_nz: Array1::zeros(0),
            i_vt_nz: Array1::zeros(0),
            norm_vt_nz: Array1::zeros(0),
            i_norm_vt_nz: Array1::zeros(0),
        }
    }
}

impl Constants {
    pub fn new<R: Read + Seek>(mut npz: NpzReader<R>) -> Result<Self, Box<dyn Error>> {

        let m: Array2<f32> = npz.by_name("M")?;
        let m_inv: Array2<f32> = npz.by_name("M_INV")?;
        let coef: Array2<f32> = npz.by_name("COEF")?;
        let coef_inv: Array2<f32> = npz.by_name("COEF_INV")?;
        let idf : Array1<f32> = npz.by_name("IDF")?;
        let r_reg_fus : Array2<f32> = npz.by_name("R_REG_FUS")?;
        let l_reg_fus : Array2<f32> = npz.by_name("L_REG_FUS")?;
        let mask : Array1<bool> = npz.by_name("MASK")?;
        let selected_features = npz.by_name("SELECTED_FEATURES")?;

        let res_var =  npz.by_name("RES_VAR")?;
        let vt_nz = npz.by_name("VT_NZ")?;
        let i_vt_nz = npz.by_name("I_VT_NZ")?;
        let norm_vt_nz = npz.by_name("NORM_VT_NZ")?;
        let i_norm_vt_nz = npz.by_name("I_NORM_VT_NZ")?;

        Ok(Constants {
            m: m,
            m_inv: m_inv,
            coef: coef,
            coef_inv:coef_inv,
            idf: idf,
            r_reg_fus: r_reg_fus,
            l_reg_fus: l_reg_fus,
            mask: mask,
            selected_features: selected_features,
            res_var: res_var,
            vt_nz: vt_nz,
            i_vt_nz: i_vt_nz,
            norm_vt_nz: norm_vt_nz,
            i_norm_vt_nz: i_norm_vt_nz
        })
    }
}
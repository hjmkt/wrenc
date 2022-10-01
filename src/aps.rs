use super::common::*;

pub struct AlfData {
    pub luma_filter_signal_flag: bool,
    pub chroma_filter_signal_flag: bool,
    pub cc_cb_filter_signal_flag: bool,
    pub cc_cr_filter_signal_flag: bool,
    pub luma_ciip_flag: bool,
    pub luma_num_filters_signalled: usize,
    pub luma_coeff_delta_idx: Vec<usize>,
    pub luma_coeff_abs: Vec<Vec<usize>>,
    pub luma_coeff_sign: Vec<Vec<bool>>,
    pub luma_ciip_idx: Vec<Vec<usize>>,
    pub chroma_ciip_flag: bool,
    pub chroma_num_alt_filters: usize,
    pub chroma_coeff_abs: Vec<Vec<usize>>,
    pub chroma_coeff_sign: Vec<Vec<bool>>,
    pub chroma_ciip_idx: Vec<Vec<usize>>,
    pub cc_cb_filters_signalled: usize,
    pub cc_cb_mapped_coeff_abs: Vec<Vec<usize>>,
    pub cc_cb_coeff_sign: Vec<Vec<bool>>,
    pub cc_cr_filters_signalled: usize,
    pub cc_cr_mapped_coeff_abs: Vec<Vec<usize>>,
    pub cc_cr_coeff_sign: Vec<Vec<bool>>,
}

impl AlfData {
    pub fn new() -> AlfData {
        AlfData {
            luma_filter_signal_flag: false,
            chroma_filter_signal_flag: false,
            cc_cb_filter_signal_flag: false,
            cc_cr_filter_signal_flag: false,
            luma_ciip_flag: false,
            luma_num_filters_signalled: 0,
            luma_coeff_delta_idx: vec![],
            luma_coeff_abs: vec![],
            luma_coeff_sign: vec![],
            luma_ciip_idx: vec![],
            chroma_ciip_flag: false,
            chroma_num_alt_filters: 0,
            chroma_coeff_abs: vec![],
            chroma_coeff_sign: vec![],
            chroma_ciip_idx: vec![],
            cc_cb_filters_signalled: 0,
            cc_cb_mapped_coeff_abs: vec![],
            cc_cb_coeff_sign: vec![],
            cc_cr_filters_signalled: 0,
            cc_cr_mapped_coeff_abs: vec![],
            cc_cr_coeff_sign: vec![],
        }
    }
}

pub struct LmcsData {
    pub min_bin_idx: usize,
    pub max_bin_idx: usize, // non-syntax
    pub delta_cw_prec: usize,
    pub delta_abs_cw: Vec<usize>,
    pub delta_sign_cw_flag: Vec<bool>,
    pub delta_abs_crs: usize,
    pub delta_sign_crs_flag: bool,
}

impl LmcsData {
    // FIXME
    pub fn new() -> LmcsData {
        LmcsData {
            min_bin_idx: 0,
            max_bin_idx: 0,
            delta_cw_prec: 1,
            delta_abs_cw: vec![],
            delta_sign_cw_flag: vec![],
            delta_abs_crs: 0,
            delta_sign_crs_flag: false,
        }
    }

    pub fn delta_max_bin_idx(&self) -> usize {
        15 - self.max_bin_idx
    }
}

pub struct ScalingListData {
    pub copy_mode_flag: [bool; 28],
    pub pred_mode_flag: [bool; 28],
    pub pred_id_delta: [usize; 28],
    pub dc_coef: [isize; 14],
    pub delta_coef: [Vec<isize>; 28],
}

impl ScalingListData {
    pub fn new() -> ScalingListData {
        ScalingListData {
            copy_mode_flag: [false; 28],
            pred_mode_flag: [false; 28],
            pred_id_delta: [0; 28],
            dc_coef: [0; 14],
            delta_coef: Default::default(),
        }
    }
}

pub struct AdaptationParameterSet {
    pub id: usize,
    pub params_type: ApsParamsType,
    pub chroma_present_flag: bool,
    pub alf_data: Option<AlfData>,
    pub lmcs_data: Option<LmcsData>,
    pub scaling_list_data: Option<ScalingListData>,
    pub extension_data: Vec<bool>,
}

impl AdaptationParameterSet {
    pub fn new_alf(id: usize) -> AdaptationParameterSet {
        AdaptationParameterSet {
            id,
            params_type: ApsParamsType::ALF_APS,
            chroma_present_flag: false,
            alf_data: Some(AlfData::new()),
            lmcs_data: None,
            scaling_list_data: None,
            extension_data: vec![],
        }
    }
    pub fn new_lmcs(id: usize) -> AdaptationParameterSet {
        AdaptationParameterSet {
            id,
            params_type: ApsParamsType::LMCS_APS,
            chroma_present_flag: false,
            alf_data: None,
            lmcs_data: Some(LmcsData::new()),
            scaling_list_data: None,
            extension_data: vec![],
        }
    }
    pub fn new_sl(id: usize) -> AdaptationParameterSet {
        AdaptationParameterSet {
            id,
            params_type: ApsParamsType::SCALING_APS,
            chroma_present_flag: false,
            alf_data: None,
            lmcs_data: None,
            scaling_list_data: Some(ScalingListData::new()),
            extension_data: vec![],
        }
    }
}

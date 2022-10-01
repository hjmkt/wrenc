pub struct GeneralTimingHrdParameters {
    pub num_units_in_tick: usize,
    pub time_scale: usize,
    pub general_nal_hrd_params_present_flag: bool,
    pub general_vcl_hrd_params_present_flag: bool,
    pub general_same_pic_timing_in_all_ols_flag: bool,
    pub general_du_hrd_params_present_flag: bool,
    pub tick_divisor: usize,
    pub bit_rate_scale: usize,
    pub cpb_size_scale: usize,
    pub cpb_size_du_scale: usize,
    pub hrd_cpb_cnt: usize,
}

pub struct OlsTimingHrdParameter {
    pub fixed_pic_rate_general_flag: bool,
    pub fixed_pic_rate_within_cvs_flag: bool,
    pub low_delay_hrd_flag: bool,
    pub elemental_duration_in_tc: usize,
    pub sublayer_hrd_parameters: SublayerHrdParameter,
}

pub struct SublayerHrdParameter {
    pub bit_rate_value: Vec<usize>,
    pub cpb_size_value: Vec<usize>,
    pub cpb_size_du_value: Vec<usize>,
    pub bit_rate_du_value: Vec<usize>,
    pub cbr_flag: Vec<bool>,
}

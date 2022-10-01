use super::bins::*;
use super::bool_coder::*;
use super::encoder_context::*;
use super::timing_hrd::*;
use std::sync::{Arc, Mutex};

pub struct HrdEncoder<'a> {
    coder: &'a mut BoolCoder,
    _encoder_context: Arc<Mutex<EncoderContext>>,
}

impl<'a> HrdEncoder<'a> {
    pub fn new(
        encoder_context: &Arc<Mutex<EncoderContext>>,
        coder: &'a mut BoolCoder,
    ) -> HrdEncoder<'a> {
        HrdEncoder {
            coder,
            _encoder_context: encoder_context.clone(),
        }
    }

    pub fn encode_general_timing_hrd_parameters(
        &mut self,
        bins: &mut Bins,
        hrd_params: &GeneralTimingHrdParameters,
    ) {
        print!("hrd.num_units_in_tick ");
        bins.push_bins_with_size(hrd_params.num_units_in_tick as u64, 32);
        print!("hrd.time_scale ");
        bins.push_bins_with_size(hrd_params.time_scale as u64, 32);
        print!("hrd.general_nal_hrd_params_present_flag ");
        bins.push_bin(hrd_params.general_nal_hrd_params_present_flag);
        print!("hrd.general_vcl_hrd_params_present_flag ");
        bins.push_bin(hrd_params.general_vcl_hrd_params_present_flag);
        if hrd_params.general_nal_hrd_params_present_flag
            || hrd_params.general_vcl_hrd_params_present_flag
        {
            print!("hrd.general_same_pic_timing_in_all_ols_flag ");
            bins.push_bin(hrd_params.general_same_pic_timing_in_all_ols_flag);
            print!("hrd.general_du_hrd_params_present_flag ");
            bins.push_bin(hrd_params.general_du_hrd_params_present_flag);
            if hrd_params.general_du_hrd_params_present_flag {
                print!("hrd.tick_divisor_minus2 ");
                bins.push_bins_with_size(hrd_params.tick_divisor as u64 - 2, 8);
            }
            print!("hrd.bit_rate_scale ");
            bins.push_bins_with_size(hrd_params.bit_rate_scale as u64, 4);
            print!("hrd.cpb_size_scale ");
            bins.push_bins_with_size(hrd_params.cpb_size_scale as u64, 4);
            if hrd_params.general_du_hrd_params_present_flag {
                print!("hrd.cpb_size_du_scale ");
                bins.push_bins_with_size(hrd_params.cpb_size_du_scale as u64, 4);
            }
            print!("hrd.hrd_cpb_cnt ");
            self.coder
                .encode_unsigned_exp_golomb(bins, hrd_params.hrd_cpb_cnt as u64);
        }
    }

    pub fn encode_ols_timing_hrd_parameters(
        &mut self,
        bins: &mut Bins,
        ols_hrd_params: &[OlsTimingHrdParameter],
        general_hrd_params: &GeneralTimingHrdParameters,
        first_sublayer: usize,
        max_sublayers_val: usize,
    ) {
        for param in ols_hrd_params
            .iter()
            .take(max_sublayers_val + 1)
            .skip(first_sublayer)
        {
            print!("ols hrd.fixed_pic_rate_general_flag ");
            bins.push_bin(param.fixed_pic_rate_general_flag);
            if !param.fixed_pic_rate_general_flag {
                print!("ols hrd.fixed_pic_rate_within_cvs_flag ");
                bins.push_bin(param.fixed_pic_rate_within_cvs_flag);
            }
            if param.fixed_pic_rate_within_cvs_flag {
                print!("ols hrd.elemental_duration_in_tc ");
                self.coder
                    .encode_unsigned_exp_golomb(bins, param.elemental_duration_in_tc as u64);
            } else if general_hrd_params.general_du_hrd_params_present_flag
                && general_hrd_params.hrd_cpb_cnt == 1
            {
                print!("ols hrd.low_delay_hrd_flag ");
                bins.push_bin(param.low_delay_hrd_flag);
            }
            if general_hrd_params.general_nal_hrd_params_present_flag {
                self.encode_sublayer_hrd_parameters(
                    bins,
                    &param.sublayer_hrd_parameters,
                    general_hrd_params,
                );
            }
            if general_hrd_params.general_vcl_hrd_params_present_flag {
                self.encode_sublayer_hrd_parameters(
                    bins,
                    &param.sublayer_hrd_parameters,
                    general_hrd_params,
                );
            }
        }
    }

    pub fn encode_sublayer_hrd_parameters(
        &mut self,
        bins: &mut Bins,
        hrd_params: &SublayerHrdParameter,
        general_hrd_params: &GeneralTimingHrdParameters,
    ) {
        for i in 0..general_hrd_params.hrd_cpb_cnt {
            print!("sublayer hrd.bit_rate_value ");
            self.coder
                .encode_unsigned_exp_golomb(bins, hrd_params.bit_rate_value[i] as u64);
            print!("sublayer hrd.cpb_size_value ");
            self.coder
                .encode_unsigned_exp_golomb(bins, hrd_params.cpb_size_value[i] as u64);
            if general_hrd_params.general_du_hrd_params_present_flag {
                print!("sublayer hrd.cpb_size_du_value ");
                self.coder
                    .encode_unsigned_exp_golomb(bins, hrd_params.cpb_size_du_value[i] as u64);
                print!("sublayer hrd.bit_rate_du_value ");
                self.coder
                    .encode_unsigned_exp_golomb(bins, hrd_params.bit_rate_du_value[i] as u64);
            }
            print!("sublayer hrd.cbr_flag ");
            bins.push_bin(hrd_params.cbr_flag[i]);
        }
    }
}

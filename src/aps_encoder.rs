#![allow(dead_code)]
use super::aps::*;
use super::bins::*;
use super::bool_coder::*;
use super::common::*;
use super::ctu::*;
use super::encoder_context::*;
use std::sync::{Arc, Mutex};

pub struct ApsEncoder<'a> {
    coder: &'a mut BoolCoder,
    encoder_context: Arc<Mutex<EncoderContext>>,
}

impl<'a> ApsEncoder<'a> {
    pub fn _new(
        encoder_context: &Arc<Mutex<EncoderContext>>,
        coder: &'a mut BoolCoder,
    ) -> ApsEncoder<'a> {
        ApsEncoder {
            coder,
            encoder_context: encoder_context.clone(),
        }
    }

    pub fn _encode(&mut self, aps: &AdaptationParameterSet) -> Vec<bool> {
        let mut bins = Bins::new();
        print!("aps.params_type ");
        bins.push_initial_bins_with_size(aps.params_type as u64, 3);
        print!("aps.id ");
        bins.push_bins_with_size(aps.id as u64, 5);
        print!("aps.chroma_present_flag ");
        bins.push_bin(aps.chroma_present_flag);
        match aps.params_type {
            ApsParamsType::ALF_APS => {
                let alf = aps.alf_data.as_ref().unwrap();
                bins.push_bin(alf.luma_filter_signal_flag);
                if aps.chroma_present_flag {
                    print!("aps.chroma_filter_signal_flag ");
                    bins.push_bin(alf.chroma_filter_signal_flag);
                    print!("aps.cc_cb_filter_signal_flag ");
                    bins.push_bin(alf.cc_cb_filter_signal_flag);
                    print!("aps.cc_cr_filter_signal_flag ");
                    bins.push_bin(alf.cc_cr_filter_signal_flag);
                }
                if alf.luma_filter_signal_flag {
                    print!("aps.luma_ciip_flag ");
                    bins.push_bin(alf.luma_ciip_flag);
                    print!("aps.luma_num_filters_signalled_minus1 ");
                    self.coder.encode_unsigned_exp_golomb(
                        &mut bins,
                        alf.luma_num_filters_signalled as u64 - 1,
                    );
                    if alf.luma_num_filters_signalled > 1 {
                        let ectx = &self.encoder_context.lock().unwrap();
                        for filt_idx in 0..ectx.num_alf_filters {
                            let n = (alf.luma_num_filters_signalled as f64).log2().ceil() as usize;
                            print!("aps.luma_coeff_delta_idx ");
                            bins.push_bins_with_size(alf.luma_coeff_delta_idx[filt_idx] as u64, n);
                        }
                    }
                    for sf_idx in 0..alf.luma_num_filters_signalled {
                        for j in 0..12 {
                            print!("aps.luma_coeff_abs ");
                            self.coder.encode_unsigned_exp_golomb(
                                &mut bins,
                                alf.luma_coeff_abs[sf_idx][j] as u64,
                            );
                            if alf.luma_coeff_abs[sf_idx][j] > 0 {
                                print!("aps.luma_coeff_sign ");
                                bins.push_bin(alf.luma_coeff_sign[sf_idx][j]);
                            }
                        }
                    }
                    if alf.luma_ciip_flag {
                        for sf_idx in 0..alf.luma_num_filters_signalled {
                            for j in 0..12 {
                                print!("aps.luma_ciip_idx ");
                                bins.push_bins_with_size(alf.luma_ciip_idx[sf_idx][j] as u64, 2);
                            }
                        }
                    }
                }
                if alf.chroma_filter_signal_flag {
                    print!("aps.chroma_ciip_flag ");
                    bins.push_bin(alf.chroma_ciip_flag);
                    print!("aps.chroma_num_alt_filters_minus1 ");
                    self.coder.encode_unsigned_exp_golomb(
                        &mut bins,
                        alf.chroma_num_alt_filters as u64 - 1,
                    );
                    for alt_idx in 0..alf.chroma_num_alt_filters {
                        for j in 0..6 {
                            print!("aps.chroma_coeff_abs ");
                            self.coder.encode_unsigned_exp_golomb(
                                &mut bins,
                                alf.chroma_coeff_abs[alt_idx][j] as u64,
                            );
                            if alf.chroma_coeff_abs[alt_idx][j] > 0 {
                                print!("aps.chroma_coeff_sign ");
                                bins.push_bin(alf.chroma_coeff_sign[alt_idx][j]);
                            }
                        }
                        if alf.chroma_ciip_flag {
                            for j in 0..6 {
                                print!("aps.chroma_ciip_idx ");
                                bins.push_bins_with_size(alf.chroma_ciip_idx[alt_idx][j] as u64, 2);
                            }
                        }
                    }
                }
                if alf.cc_cb_filter_signal_flag {
                    print!("aps.cc_cb_filters_signalled ");
                    self.coder.encode_unsigned_exp_golomb(
                        &mut bins,
                        alf.cc_cb_filters_signalled as u64 - 1,
                    );
                    for k in 0..alf.cc_cb_filters_signalled {
                        for j in 0..7 {
                            print!("aps.cc_cb_mapped_coeff_abs ");
                            bins.push_bins_with_size(alf.cc_cb_mapped_coeff_abs[k][j] as u64, 3);
                            if alf.cc_cb_mapped_coeff_abs[k][j] > 0 {
                                print!("aps.cc_cb_coeff_sign ");
                                bins.push_bin(alf.cc_cb_coeff_sign[k][j]);
                            }
                        }
                    }
                }
                if alf.cc_cr_filter_signal_flag {
                    print!("aps.cc_cr_filters_signalled ");
                    self.coder.encode_unsigned_exp_golomb(
                        &mut bins,
                        alf.cc_cr_filters_signalled as u64 - 1,
                    );
                    for k in 0..alf.cc_cr_filters_signalled {
                        for j in 0..7 {
                            print!("aps.cc_cr_mapped_coeff_abs ");
                            bins.push_bins_with_size(alf.cc_cr_mapped_coeff_abs[k][j] as u64, 3);
                            if alf.cc_cr_mapped_coeff_abs[k][j] > 0 {
                                print!("aps.cc_cr_coeff_sign ");
                                bins.push_bin(alf.cc_cr_coeff_sign[k][j]);
                            }
                        }
                    }
                }
            }
            ApsParamsType::LMCS_APS => {
                let lmcs = aps.lmcs_data.as_ref().unwrap();
                print!("aps.min_bin_idx ");
                self.coder
                    .encode_unsigned_exp_golomb(&mut bins, lmcs.min_bin_idx as u64);
                print!("aps.delta_max_bin_idx ");
                self.coder
                    .encode_unsigned_exp_golomb(&mut bins, lmcs.delta_max_bin_idx() as u64);
                print!("aps.delta_cw_prec_minus1 ");
                self.coder
                    .encode_unsigned_exp_golomb(&mut bins, lmcs.delta_cw_prec as u64 - 1);
                for i in lmcs.min_bin_idx..=lmcs.max_bin_idx {
                    print!("aps.delta_abs_cw ");
                    bins.push_bins_with_size(lmcs.delta_abs_cw[i] as u64, lmcs.delta_cw_prec);
                    if lmcs.delta_abs_cw[i] > 0 {
                        print!("aps.delta_sign_cw_flag ");
                        bins.push_bin(lmcs.delta_sign_cw_flag[i]);
                    }
                }
                if aps.chroma_present_flag {
                    print!("aps.delta_abs_crs ");
                    bins.push_bins_with_size(lmcs.delta_abs_crs as u64, 3);
                    if lmcs.delta_abs_crs > 0 {
                        print!("aps.delta_sign_crs_flag ");
                        bins.push_bin(lmcs.delta_sign_crs_flag);
                    }
                }
            }
            ApsParamsType::SCALING_APS => {
                let mut ectx = self.encoder_context.lock().unwrap();
                let sl = aps.scaling_list_data.as_ref().unwrap();
                for id in 0..28 {
                    let matrix_size = if id < 2 {
                        2
                    } else if id < 8 {
                        4
                    } else {
                        8
                    };
                    if aps.chroma_present_flag || id % 3 == 2 || id == 27 {
                        print!("aps.copy_mode_flag ");
                        bins.push_bin(sl.copy_mode_flag[id]);
                        if !sl.copy_mode_flag[id] {
                            print!("aps.pred_mode_flag ");
                            bins.push_bin(sl.pred_mode_flag[id]);
                        }
                        if (sl.copy_mode_flag[id] || sl.pred_mode_flag[id])
                            && id != 0
                            && id != 2
                            && id != 8
                        {
                            print!("aps.pred_id_delta ");
                            self.coder
                                .encode_unsigned_exp_golomb(&mut bins, sl.pred_id_delta[id] as u64);
                        }
                        if !sl.copy_mode_flag[id] {
                            let mut next_coef = 0;
                            if id > 13 {
                                print!("aps.dc_coeff ");
                                self.coder.encode_signed_exp_golomb(
                                    &mut bins,
                                    sl.dc_coef[id - 14] as i64,
                                );
                                next_coef += sl.dc_coef[id - 14];
                            }
                            for i in 0..matrix_size * matrix_size {
                                let x = DIAG_SCAN_ORDER[3][3][i].0;
                                let y = DIAG_SCAN_ORDER[3][3][i].1;
                                if !(id > 25 && x >= 4 && y >= 4) {
                                    print!("aps.delta_coef ");
                                    self.coder.encode_signed_exp_golomb(
                                        &mut bins,
                                        sl.delta_coef[id][i] as i64,
                                    );
                                    next_coef += sl.delta_coef[id][i];
                                }
                                ectx.scaling_list[id][i] = next_coef;
                            }
                        }
                    }
                }
            }
        }
        let aps_extension_flag = !aps.extension_data.is_empty();
        print!("aps.extension_flag ");
        bins.push_bin(aps_extension_flag);
        if aps_extension_flag {
            for bit in aps.extension_data.iter() {
                print!("aps.extension_data ");
                bins.push_bin(*bit);
            }
        }
        let rbsp_stop_one_bit = true;
        print!("aps.rbsp_stop_one_bit ");
        bins.push_bin(rbsp_stop_one_bit);
        bins.byte_align();
        bins.into_iter().collect()
    }
}

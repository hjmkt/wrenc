use super::bins::*;
use super::bool_coder::*;
use super::dpbp_encoder::*;
use super::encoder_context::*;
use super::hrd_encoder::*;
use super::ptl_encoder::*;
use super::vps::*;
use debug_print::*;
use std::sync::{Arc, Mutex};

pub struct VpsEncoder<'a> {
    coder: &'a mut BoolCoder,
    encoder_context: Arc<Mutex<EncoderContext>>,
}

impl<'a> VpsEncoder<'a> {
    pub fn new(
        encoder_context: &Arc<Mutex<EncoderContext>>,
        coder: &'a mut BoolCoder,
    ) -> VpsEncoder<'a> {
        VpsEncoder {
            coder,
            encoder_context: encoder_context.clone(),
        }
    }

    pub fn encode(&mut self, vps: &VideoParameterSet) -> Vec<bool> {
        let mut bins = Bins::new();
        debug_eprint!("vps.id ");
        bins.push_initial_bins_with_size(vps.id as u64, 4);
        debug_eprint!("vps.max_layers ");
        bins.push_bins_with_size(vps.max_layers as u64 - 1, 6);
        debug_eprint!("vps.max_sublayers ");
        bins.push_bins_with_size(vps.max_sublayers as u64 - 1, 3);
        if vps.max_layers > 1 && vps.max_sublayers > 1 {
            debug_eprint!("vps.default_ptl_dpb_hrd_max_tid_flag ");
            bins.push_bin(vps.default_ptl_dpb_hrd_max_tid_flag);
        }
        if vps.max_layers > 1 {
            debug_eprint!("vps.all_layers_are_independent ");
            bins.push_bin(vps.all_layers_are_independent());
        }
        for i in 0..vps.max_layers {
            //debug_eprintln!("layer id {}", vps.layers[i].id);
            debug_eprint!("vps.layer_id ");
            bins.push_bins_with_size(vps.layers[i].id as u64, 6);
            if i > 0 && !vps.all_layers_are_independent() {
                debug_eprint!("vps.layer_is_independent ");
                bins.push_bin(vps.layers[i].is_independent_layer);
                if !vps.layers[i].is_independent_layer {
                    debug_eprint!("vps.layer_max_tid_ref_present_flag ");
                    bins.push_bin(vps.layers[i].max_tid_ref_present_flag);
                    for j in 0..i {
                        debug_eprint!("vps.layer_direct_ref_layer_flag ");
                        bins.push_bin(vps.layers[i].direct_ref_layer_flag[j]);
                        if vps.layers[i].max_tid_ref_present_flag
                            && vps.layers[i].direct_ref_layer_flag[j]
                        {
                            debug_eprint!("vps.layer_max_tid_il_ref_pics ");
                            bins.push_bins_with_size(
                                vps.layers[i].max_tid_il_ref_pics[j] as u64 + 1,
                                3,
                            );
                        }
                    }
                }
            }
        }
        if vps.max_layers > 1 {
            if vps.all_layers_are_independent() {
                debug_eprint!("vps.each_layer_is_an_ols ");
                bins.push_bin(vps.each_layer_is_an_ols);
            }
            if !vps.each_layer_is_an_ols {
                //debug_eprintln!("each layer is an ols");
                if !vps.all_layers_are_independent() {
                    debug_eprint!("vps.old_mode ");
                    bins.push_bins_with_size(vps.ols_mode as u64, 2);
                }
                if let OlsMode::Explicit = vps.ols_mode {
                    //debug_eprintln!("output_layer_sets");
                    debug_eprint!("vps.num_output_layer_sets ");
                    bins.push_bins_with_size(vps.num_output_layer_sets as u64 - 2, 8);
                    for i in 1..vps.num_output_layer_sets {
                        for j in 0..vps.max_layers {
                            //debug_eprintln!("{}, {}", i, j);
                            debug_eprint!("vps.ols_output_layer_flags ");
                            bins.push_bin(vps.ols_output_layer_flags[i][j]);
                        }
                    }
                }
            }
            //debug_eprintln!("vps_num_ptls: {}", vps.num_ptls);
            debug_eprint!("vps.num_ptls ");
            bins.push_bins_with_size(vps.num_ptls as u64 - 1, 8);
        }
        for i in 0..vps.num_ptls {
            //debug_eprintln!("np");
            if i > 0 {
                debug_eprint!("vps.pt_present_flags ");
                bins.push_bin(vps.profile_tier_levels[i].pt_present_flags);
            }
            if !vps.default_ptl_dpb_hrd_max_tid_flag {
                //debug_eprintln!("max tid {}", vps.ptl_max_tids[i]);
                debug_eprint!("vps.max_tids ");
                bins.push_bins_with_size(vps.ptl_max_tids[i] as u64, 3);
            }
        }
        bins.byte_align();
        {
            let ectx = self.encoder_context.clone();
            // FIXME
            {
                let mut ectx = ectx.lock().unwrap();
                let ols_mode_idc = if !vps.each_layer_is_an_ols {
                    vps.ols_mode as usize
                } else {
                    4
                };
                ectx.total_num_olss = match ols_mode_idc {
                    4 | 0 | 1 => vps.max_layers,
                    2 => vps.num_output_layer_sets,
                    _ => panic!(),
                };
            }
            let mut ptl_encoder = PtlEncoder::new(&ectx, self.coder);
            for i in 0..vps.num_ptls {
                //debug_eprintln!("encode ptl");
                ptl_encoder.encode(
                    &mut bins,
                    &vps.profile_tier_levels[i],
                    vps.profile_tier_levels[i].pt_present_flags,
                    vps.ptl_max_tids[i],
                );
                //bits.extend(ptl_bits);
            }
            let ectx = ectx.lock().unwrap();
            for i in 0..ectx.total_num_olss {
                if vps.num_ptls > 1 && vps.num_ptls != ectx.total_num_olss {
                    //debug_eprintln!("vps_ols_ptl_idx: {} {}", i, vps.ols_ptl_idx[i]);
                    debug_eprint!("vps.ols_ptl_idx ");
                    bins.push_bins_with_size(vps.ols_ptl_idx[i] as u64, 8);
                }
            }
        }
        if !vps.each_layer_is_an_ols {
            debug_eprint!("vps.num_dpb_params ");
            self.coder
                .encode_unsigned_exp_golomb(&mut bins, vps.dpb_parameters.len() as u64 - 1);
            if vps.max_sublayers > 1 {
                debug_eprint!("vps.sublayer_dpb_params_present_flag ");
                bins.push_bin(vps.sublayer_dpb_params_present_flag);
            }
            // FIXME
            let vps_num_dpb_params = {
                let ectx = self.encoder_context.clone();
                let mut ectx = ectx.lock().unwrap();
                ectx.vps_num_dpb_params = if vps.each_layer_is_an_ols {
                    0
                } else {
                    vps.dpb_parameters.len()
                };
                ectx.vps_num_dpb_params
            };
            for i in 0..vps_num_dpb_params {
                if !vps.default_ptl_dpb_hrd_max_tid_flag {
                    debug_eprint!("dpb.max_tid ");
                    bins.push_bins_with_size(vps.dpb_parameters[i].max_tid as u64, 3);
                }
                let ectx = self.encoder_context.clone();
                let mut dpbp_encoder = DpbpEncoder::new(&ectx, self.coder);
                dpbp_encoder.encode(
                    &mut bins,
                    &vps.dpb_parameters,
                    vps.dpb_parameters[i].max_tid,
                    vps.sublayer_dpb_params_present_flag,
                );
            }
            let ectx = self.encoder_context.clone();
            let ectx = ectx.lock().unwrap();
            for i in 0..ectx.num_multi_layer_olss {
                debug_eprint!("vps.ols_dpbp_pic_width ");
                self.coder.encode_unsigned_exp_golomb(
                    &mut bins,
                    vps.ols_dpb_parameters[i].pic_width as u64,
                );
                debug_eprint!("vps.ols_dpbp_pic_height ");
                self.coder.encode_unsigned_exp_golomb(
                    &mut bins,
                    vps.ols_dpb_parameters[i].pic_height as u64,
                );
                debug_eprint!("dpb.chroma_format ");
                bins.push_bins_with_size(vps.ols_dpb_parameters[i].chroma_format as u64, 2);
                debug_eprint!("vps.ols_dpbp_bitdepth ");
                self.coder.encode_unsigned_exp_golomb(
                    &mut bins,
                    vps.ols_dpb_parameters[i].bitdepth as u64 - 8,
                );
                // FIXME num_multi_layer_olss
                if ectx.vps_num_dpb_params > 1
                    && ectx.vps_num_dpb_params != ectx.num_multi_layer_olss
                {
                    debug_eprint!("vps.ols_dpbp_params_idx ");
                    self.coder.encode_unsigned_exp_golomb(
                        &mut bins,
                        vps.ols_dpb_parameters[i].params_idx as u64,
                    );
                }
            }

            debug_eprint!("vps.general_timing_hrd_parameters ");
            bins.push_bin(vps.general_timing_hrd_parameters.is_some());
            if let Some(hrd_params) = &vps.general_timing_hrd_parameters {
                let ectx = self.encoder_context.clone();
                {
                    let mut hrd_encoder = HrdEncoder::new(&ectx, self.coder);
                    hrd_encoder.encode_general_timing_hrd_parameters(&mut bins, hrd_params);
                }
                if vps.max_sublayers > 1 {
                    debug_eprint!("vps.sublayer_cpb_params_present_flag ");
                    bins.push_bin(vps.sublayer_cpb_params_present_flag);
                }
                debug_eprint!("vps.num_ols_timing_hrd_params ");
                self.coder.encode_unsigned_exp_golomb(
                    &mut bins,
                    vps.num_ols_timing_hrd_params as u64 - 1,
                );
                for i in 0..vps.num_ols_timing_hrd_params {
                    if !vps.default_ptl_dpb_hrd_max_tid_flag {
                        debug_eprint!("vps.hrd_max_tids ");
                        bins.push_bins_with_size(vps.hrd_max_tids[i] as u64, 3);
                    }
                    let first_sublayer = if vps.sublayer_cpb_params_present_flag {
                        0
                    } else {
                        vps.hrd_max_tids[i]
                    };
                    if let Some(general_timing_hrd_parameters) = &vps.general_timing_hrd_parameters
                    {
                        let mut hrd_encoder = HrdEncoder::new(&ectx, self.coder);
                        hrd_encoder.encode_ols_timing_hrd_parameters(
                            &mut bins,
                            &vps.ols_timing_hrd_parameters,
                            general_timing_hrd_parameters,
                            first_sublayer,
                            vps.hrd_max_tids[i],
                        );
                    } else {
                        panic!();
                    }
                }
                let ectx = ectx.lock().unwrap();
                if vps.num_ols_timing_hrd_params > 1
                    && vps.num_ols_timing_hrd_params != ectx.num_multi_layer_olss
                {
                    for i in 0..ectx.num_multi_layer_olss {
                        debug_eprint!("vps.ols_timing_hrd_idxs ");
                        self.coder.encode_unsigned_exp_golomb(
                            &mut bins,
                            vps.ols_timing_hrd_idxs[i] as u64,
                        );
                    }
                }
            }
        }
        debug_eprint!("vps.extension_data_present ");
        bins.push_bin(!vps.extension_data.is_empty());
        // FIXME
        for i in 0..vps.extension_data.len() {
            debug_eprint!("vps.extension_data ");
            bins.push_bin(vps.extension_data[i]);
        }
        let rbsp_stop_one_bit = true;
        debug_eprint!("rbsp_stop_one_bit ");
        bins.push_bin(rbsp_stop_one_bit);
        bins.byte_align();
        //debug_eprintln!("{}", bits.len());
        bins.into_iter().collect()
    }
}

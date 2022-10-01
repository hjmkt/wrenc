use super::bins::*;
use super::bool_coder::*;
use super::common::*;
use super::encoder_context::*;
use super::picture_header::*;
use super::pps::*;
use super::pwt_encoder::*;
use super::rpl_encoder::*;
use super::sps::*;
use debug_print::*;
use std::sync::{Arc, Mutex};

pub struct PhEncoder<'a> {
    coder: &'a mut BoolCoder,
    encoder_context: Arc<Mutex<EncoderContext>>,
}

impl<'a> PhEncoder<'a> {
    pub fn new(
        encoder_context: &Arc<Mutex<EncoderContext>>,
        coder: &'a mut BoolCoder,
    ) -> PhEncoder<'a> {
        PhEncoder {
            coder,
            encoder_context: encoder_context.clone(),
        }
    }

    pub fn encode(
        &mut self,
        bins: &mut Bins,
        ph: &PictureHeader,
        sps: &SequenceParameterSet,
        pps: &PictureParameterSet,
    ) {
        debug_eprint!("ph.gdr_or_irap_pic_flag ");
        bins.push_bin_with_initial_check(ph.gdr_or_irap_pic_flag);
        debug_eprint!("ph.non_ref_pic_flag ");
        bins.push_bin(ph.non_ref_pic_flag);
        if ph.gdr_or_irap_pic_flag {
            debug_eprint!("ph.gdr_pic_flag ");
            bins.push_bin(ph.gdr_pic_flag);
        }
        debug_eprint!("ph.inter_slice_allowed_flag ");
        bins.push_bin(ph.inter_slice_allowed_flag);
        if ph.inter_slice_allowed_flag {
            debug_eprint!("ph.intra_slice_allowed_flag ");
            bins.push_bin(ph.intra_slice_allowed_flag);
        }
        debug_eprint!("ph.pic_parameter_set_id ");
        self.coder
            .encode_unsigned_exp_golomb(bins, ph.pic_parameter_set_id as u64);
        let n = sps.log2_max_pic_order_cnt_lsb;
        debug_eprint!("ph.pic_order_cnt_lsb ");
        bins.push_bins_with_size(ph.pic_order_cnt_lsb as u64, n);
        if ph.gdr_pic_flag {
            debug_eprint!("ph.recovery_poc_cnt ");
            self.coder
                .encode_unsigned_exp_golomb(bins, ph.recovery_poc_cnt as u64);
        }
        let ectx = self.encoder_context.clone();
        let ectx = &mut ectx.lock().unwrap();
        for i in 0..ectx.num_extra_ph_bits {
            debug_eprint!("ph.extra_bit ");
            bins.push_bin(ph.extra_bit[i]);
        }
        if sps.poc_msb_cycle_flag {
            debug_eprint!("ph.poc_msb_cycle_val ");
            bins.push_bins_with_size(ph.poc_msb_cycle_val as u64, sps.poc_msb_cycle_len);
        }
        if sps.alf_enabled_flag && pps.partition_parameters.alf_info_in_ph_flag {
            debug_eprint!("ph.alf_enabled_flag ");
            bins.push_bin(ph.alf_enabled_flag);
            if ph.alf_enabled_flag {
                debug_eprint!("ph.alf_info.num_alf_aps_ids_luma ");
                bins.push_bins_with_size(ph.alf_info.num_alf_aps_ids_luma as u64, 3);
                for i in 0..ph.alf_info.num_alf_aps_ids_luma {
                    debug_eprint!("ph.alf_info.aps_id_luma ");
                    bins.push_bins_with_size(ph.alf_info.aps_id_luma[i] as u64, 3);
                }
                if sps.chroma_format != ChromaFormat::Monochrome {
                    debug_eprint!("ph.alf_info.cb_enabled_flag ");
                    bins.push_bin(ph.alf_info.cb_enabled_flag);
                    debug_eprint!("ph.alf_info.cr_enabled_flag ");
                    bins.push_bin(ph.alf_info.cr_enabled_flag);
                }
                if ph.alf_info.cb_enabled_flag || ph.alf_info.cr_enabled_flag {
                    debug_eprint!("ph.alf_info.aps_id_chroma ");
                    bins.push_bins_with_size(ph.alf_info.aps_id_chroma as u64, 3);
                }
                if sps.ccalf_enabled_flag {
                    debug_eprint!("ph.alf_info.cc_cb_enabled_flag ");
                    bins.push_bin(ph.alf_info.cc_cb_enabled_flag);
                    if ph.alf_info.cc_cb_enabled_flag {
                        debug_eprint!("ph.alf_info.cc_cb_aps_id ");
                        bins.push_bins_with_size(ph.alf_info.cc_cb_aps_id as u64, 3);
                    }
                    debug_eprint!("ph.alf_info.cc_cr_enabled_flag ");
                    bins.push_bin(ph.alf_info.cc_cr_enabled_flag);
                    if ph.alf_info.cc_cr_enabled_flag {
                        debug_eprint!("ph.alf_info.cc_cr_aps_id ");
                        bins.push_bins_with_size(ph.alf_info.cc_cr_aps_id as u64, 3);
                    }
                }
            }
        }

        if sps.lmcs_enabled_flag {
            debug_eprint!("ph.lmcs_enabled_flag ");
            bins.push_bin(ph.lmcs_enabled_flag);
            if ph.lmcs_enabled_flag {
                debug_eprint!("ph.lmcs_aps_id ");
                bins.push_bins_with_size(ph.lmcs_aps_id as u64, 2);
                if sps.chroma_format != ChromaFormat::Monochrome {
                    debug_eprint!("ph.chroma_residual_scale_flag ");
                    bins.push_bin(ph.chroma_residual_scale_flag);
                }
            }
        }
        if sps.explicit_scaling_list_enabled_flag {
            debug_eprint!("ph.explicit_scaling_list_enabled_flag ");
            bins.push_bin(ph.explicit_scaling_list_enabled_flag);
            if ph.explicit_scaling_list_enabled_flag {
                debug_eprint!("ph.scaling_list_aps_id ");
                bins.push_bins_with_size(ph.scaling_list_aps_id as u64, 3);
            }
        }
        if sps.virtual_boundaries_enabled_flag
            && !sps
                .virtual_boundary_parameters
                .virtual_boundaries_present_flag
        {
            debug_eprint!("ph.virtual_boundaries_present_flag ");
            bins.push_bin(ph.virtual_boundary.virtual_boundaries_present_flag);
            if ph.virtual_boundary.virtual_boundaries_present_flag {
                debug_eprint!("ph.num_ver_virtual_boundaries ");
                self.coder.encode_unsigned_exp_golomb(
                    bins,
                    ph.virtual_boundary.num_ver_virtual_boundaries as u64,
                );
                for i in 0..ph.virtual_boundary.num_ver_virtual_boundaries {
                    debug_eprint!("ph.virtual_boundary_pos_xs_minus1 ");
                    self.coder.encode_unsigned_exp_golomb(
                        bins,
                        ph.virtual_boundary.virtual_boundary_pos_xs[i] as u64 - 1,
                    );
                }
                debug_eprint!("ph.num_hor_virtual_boundaries ");
                self.coder.encode_unsigned_exp_golomb(
                    bins,
                    ph.virtual_boundary.num_hor_virtual_boundaries as u64,
                );
                for i in 0..ph.virtual_boundary.num_hor_virtual_boundaries {
                    debug_eprint!("ph.virtual_boundary_pos_ys_minus1 ");
                    self.coder.encode_unsigned_exp_golomb(
                        bins,
                        ph.virtual_boundary.virtual_boundary_pos_ys[i] as u64 - 1,
                    );
                }
            }
        }

        if pps.output_flag_present_flag && !ph.non_ref_pic_flag {
            debug_eprint!("ph.pic_output_flag ");
            bins.push_bin(ph.pic_output_flag);
        }
        if pps.partition_parameters.rpl_info_in_ph_flag {
            let ectx = self.encoder_context.clone();
            let mut rpl_encoder = RefPicListStructEncoder::new(&ectx, self.coder);
            rpl_encoder.encode(bins, &ph.ref_pic_lists, sps, pps, ph);
        }
        if sps.partition_constraints_override_enabled_flag {
            debug_eprint!("ph.partition_constraints_override_flag ");
            bins.push_bin(ph.partition_constraints_override_flag);
        }
        if ph.intra_slice_allowed_flag {
            if ph.partition_constraints_override_flag {
                debug_eprint!("ph.log2_diff_min_qt_min_cb_intra_slice_luma ");
                self.coder.encode_unsigned_exp_golomb(
                    bins,
                    ph.partition_constraints
                        .as_ref()
                        .unwrap()
                        .log2_diff_min_qt_min_cb_intra_slice_luma as u64,
                );
                debug_eprint!("ph.max_mtt_hierarchy_depth_intra_slice_luma ");
                self.coder.encode_unsigned_exp_golomb(
                    bins,
                    ph.partition_constraints
                        .as_ref()
                        .unwrap()
                        .max_mtt_hierarchy_depth_intra_slice_luma as u64,
                );
                if ph
                    .partition_constraints
                    .as_ref()
                    .unwrap()
                    .max_mtt_hierarchy_depth_intra_slice_luma
                    != 0
                {
                    debug_eprint!("ph.log2_diff_max_bt_min_qt_intra_slice_luma ");
                    self.coder.encode_unsigned_exp_golomb(
                        bins,
                        ph.partition_constraints
                            .as_ref()
                            .unwrap()
                            .log2_diff_max_bt_min_qt_intra_slice_luma
                            as u64,
                    );
                    debug_eprint!("ph.log2_diff_max_tt_min_qt_intra_slice_luma ");
                    self.coder.encode_unsigned_exp_golomb(
                        bins,
                        ph.partition_constraints
                            .as_ref()
                            .unwrap()
                            .log2_diff_max_tt_min_qt_intra_slice_luma
                            as u64,
                    );
                }
                if sps.partition_constraints.qtbtt_dual_tree_intra_flag {
                    debug_eprint!("ph.log2_diff_min_qt_min_cb_intra_slice_chroma ");
                    self.coder.encode_unsigned_exp_golomb(
                        bins,
                        ph.partition_constraints
                            .as_ref()
                            .unwrap()
                            .log2_diff_min_qt_min_cb_intra_slice_chroma
                            as u64,
                    );
                    debug_eprint!("ph.max_mtt_hierarchy_depth_intra_slice_chroma ");
                    self.coder.encode_unsigned_exp_golomb(
                        bins,
                        ph.partition_constraints
                            .as_ref()
                            .unwrap()
                            .max_mtt_hierarchy_depth_intra_slice_chroma
                            as u64,
                    );
                    if ph
                        .partition_constraints
                        .as_ref()
                        .unwrap()
                        .max_mtt_hierarchy_depth_intra_slice_chroma
                        != 0
                    {
                        debug_eprint!("ph.log2_diff_max_bt_min_qt_intra_slice_chroma ");
                        self.coder.encode_unsigned_exp_golomb(
                            bins,
                            ph.partition_constraints
                                .as_ref()
                                .unwrap()
                                .log2_diff_max_bt_min_qt_intra_slice_chroma
                                as u64,
                        );
                        debug_eprint!("ph.log2_diff_max_tt_min_qt_intra_slice_chroma ");
                        self.coder.encode_unsigned_exp_golomb(
                            bins,
                            ph.partition_constraints
                                .as_ref()
                                .unwrap()
                                .log2_diff_max_tt_min_qt_intra_slice_chroma
                                as u64,
                        );
                    }
                }
            }
            if pps.cu_qp_delta_enabled_flag {
                debug_eprint!("ph.cu_qp_delta_subdiv_intra_slice ");
                self.coder
                    .encode_unsigned_exp_golomb(bins, ph.cu_qp_delta_subdiv_intra_slice as u64);
            }
            if pps
                .chroma_tool_offsets
                .cu_chroma_qp_offset_list_enabled_flag
            {
                debug_eprint!("ph.cu_chroma_qp_offset_subdiv_intra_slice ");
                self.coder.encode_unsigned_exp_golomb(
                    bins,
                    ph.cu_chroma_qp_offset_subdiv_intra_slice as u64,
                );
            }
        }
        if ph.inter_slice_allowed_flag {
            if ph.partition_constraints_override_flag {
                debug_eprint!("ph.log2_diff_min_qt_min_cb_inter_slice ");
                self.coder.encode_unsigned_exp_golomb(
                    bins,
                    ph.partition_constraints
                        .as_ref()
                        .unwrap()
                        .log2_diff_min_qt_min_cb_inter_slice as u64,
                );
                debug_eprint!("ph.max_mtt_hierarchy_depth_inter_slice ");
                self.coder.encode_unsigned_exp_golomb(
                    bins,
                    ph.partition_constraints
                        .as_ref()
                        .unwrap()
                        .max_mtt_hierarchy_depth_inter_slice as u64,
                );
                if ph
                    .partition_constraints
                    .as_ref()
                    .unwrap()
                    .max_mtt_hierarchy_depth_inter_slice
                    != 0
                {
                    debug_eprint!("ph.log2_diff_max_bt_min_qt_inter_slice ");
                    self.coder.encode_unsigned_exp_golomb(
                        bins,
                        ph.partition_constraints
                            .as_ref()
                            .unwrap()
                            .log2_diff_max_bt_min_qt_inter_slice as u64,
                    );
                    debug_eprint!("ph.log2_diff_max_tt_min_qt_inter_slice ");
                    self.coder.encode_unsigned_exp_golomb(
                        bins,
                        ph.partition_constraints
                            .as_ref()
                            .unwrap()
                            .log2_diff_max_tt_min_qt_inter_slice as u64,
                    );
                }
            }
            if pps.cu_qp_delta_enabled_flag {
                debug_eprint!("ph.cu_qp_delta_subdiv_inter_slice ");
                self.coder
                    .encode_unsigned_exp_golomb(bins, ph.cu_qp_delta_subdiv_inter_slice as u64);
            }
            if pps
                .chroma_tool_offsets
                .cu_chroma_qp_offset_list_enabled_flag
            {
                debug_eprint!("ph.cu_chroma_qp_offset_subdiv_inter_slice ");
                self.coder.encode_unsigned_exp_golomb(
                    bins,
                    ph.cu_chroma_qp_offset_subdiv_inter_slice as u64,
                );
            }
            if sps.temporal_mvp_enabled_flag {
                debug_eprint!("ph.temporal_mvp_enabled_flag ");
                bins.push_bin(ph.temporal_mvp_enabled_flag);
            }
            if ph.temporal_mvp_enabled_flag && pps.partition_parameters.rpl_info_in_ph_flag {
                if ph.ref_pic_lists[1].ref_pic_list_structs[ectx.rpls_idx[1]].num_ref_entries > 0 {
                    debug_eprint!("ph.collocated_from_l0_flag ");
                    bins.push_bin(ph.collocated_from_l0_flag);
                }
                if (ph.collocated_from_l0_flag
                    && ph.ref_pic_lists[0].ref_pic_list_structs[ectx.rpls_idx[0]].num_ref_entries
                        > 1)
                    || (!ph.collocated_from_l0_flag
                        && ph.ref_pic_lists[1].ref_pic_list_structs[ectx.rpls_idx[1]]
                            .num_ref_entries
                            > 1)
                {
                    debug_eprint!("ph.collocated_ref_idx ");
                    self.coder
                        .encode_unsigned_exp_golomb(bins, ph.collocated_ref_idx as u64);
                }
            }
            if sps.mmvd_fullpel_only_enabled_flag {
                debug_eprint!("ph.mmvd_fullpel_only_flag ");
                self.coder
                    .encode_unsigned_exp_golomb(bins, ph.mmvd_fullpel_only_flag as u64);
            }
            let presence_flag = !pps.partition_parameters.rpl_info_in_ph_flag
                || ph.ref_pic_lists[1].ref_pic_list_structs[ectx.rpls_idx[1]].num_ref_entries > 0;
            if presence_flag {
                debug_eprint!("ph.mvd_l1_zero_flag ");
                bins.push_bin(ph.mvd_l1_zero_flag);
                if sps.bdof_control_present_in_ph_flag {
                    debug_eprint!("ph.bdof_disabled_flag ");
                    bins.push_bin(ph.bdof_disabled_flag);
                }
                if sps.dmvr_control_present_in_ph_flag {
                    debug_eprint!("ph.dmvr_disabled_flag ");
                    bins.push_bin(ph.dmvr_disabled_flag);
                }
            }
            if (pps.weighted_pred_flag || pps.weighted_bipred_flag)
                && pps.partition_parameters.wp_info_in_ph_flag
            {
                let ectx = self.encoder_context.clone();
                let mut pwt_encoder = PredWeightTableEncoder::new(&ectx, self.coder);
                pwt_encoder.encode(bins, ph.pred_weight_table.as_ref().unwrap(), sps, pps, ph);
            }
        }
        if pps.partition_parameters.qp_delta_info_in_ph_flag {
            debug_eprint!("ph.qp_delta ");
            self.coder
                .encode_signed_exp_golomb(bins, ph.qp_delta as i64);
        }
        {
            ectx.slice_qp_y = pps.init_qp + ph.qp_delta;
        }
        if sps.joint_cbcr_enabled_flag {
            debug_eprint!("ph.joint_cbcr_sign_flag ");
            bins.push_bin(ph.joint_cbcr_sign_flag);
        }
        if sps.sao_enabled_flag && pps.partition_parameters.sao_info_in_ph_flag {
            debug_eprint!("ph.sao_luma_enabled_flag ");
            bins.push_bin(ph.sao_luma_enabled_flag);
            if sps.chroma_format != ChromaFormat::Monochrome {
                debug_eprint!("ph.sao_chroma_enabled_flag ");
                bins.push_bin(ph.sao_chroma_enabled_flag);
            }
        }
        if pps.deblocking_filter_control.dbf_info_in_ph_flag {
            debug_eprint!("ph.deblocking_params_present_flag ");
            bins.push_bin(ph.deblocking_params_present_flag);
            if ph.deblocking_params_present_flag {
                if !pps
                    .deblocking_filter_control
                    .deblocking_filter_disabled_flag
                {
                    debug_eprint!("ph.deblocking_filter_disabled_flag ");
                    bins.push_bin(ph.deblocking_filter_disabled_flag);
                }
                if !ph.deblocking_filter_disabled_flag {
                    debug_eprint!("ph.luma_beta_offset_div2 ");
                    self.coder
                        .encode_signed_exp_golomb(bins, ph.luma_beta_offset as i64 / 2);
                    debug_eprint!("ph.luma_tc_offset_div2 ");
                    self.coder
                        .encode_signed_exp_golomb(bins, ph.luma_tc_offset as i64 / 2);
                    if pps.chroma_tool_offsets_present_flag {
                        debug_eprint!("ph.cb_beta_offset_div2 ");
                        self.coder
                            .encode_signed_exp_golomb(bins, ph.cb_beta_offset as i64 / 2);
                        debug_eprint!("ph.cb_tc_offset_div2 ");
                        self.coder
                            .encode_signed_exp_golomb(bins, ph.cb_tc_offset as i64 / 2);
                        debug_eprint!("ph.cr_beta_offset_div2 ");
                        self.coder
                            .encode_signed_exp_golomb(bins, ph.cr_beta_offset as i64 / 2);
                        debug_eprint!("ph.cr_tc_offset_div2 ");
                        self.coder
                            .encode_signed_exp_golomb(bins, ph.cr_tc_offset as i64 / 2);
                    }
                }
            }
        }
        if pps.picture_header_extension_present_flag {
            debug_eprint!("ph.extension_data_byte_len ");
            self.coder
                .encode_unsigned_exp_golomb(bins, ph.extension_data_byte.len() as u64);
            for byte in ph.extension_data_byte.iter() {
                debug_eprint!("ph.extension_data_bytes ");
                bins.push_bins_with_size(*byte as u64, 8);
            }
        }

        let rbsp_stop_one_bit = true;
        debug_eprint!("ph.rbsp_stop_one_bit ");
        bins.push_bin(rbsp_stop_one_bit);
        bins.byte_align();
    }
}

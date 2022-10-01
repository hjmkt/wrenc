use super::bins::*;
use super::bool_coder::*;
use super::common::*;
use super::dpbp_encoder::*;
use super::encoder_context::*;
use super::hrd_encoder::*;
use super::ptl_encoder::*;
use super::rpl_encoder::*;
use super::sps::*;
use debug_print::*;
use std::sync::{Arc, Mutex};

pub struct SpsEncoder<'a> {
    coder: &'a mut BoolCoder,
    encoder_context: Arc<Mutex<EncoderContext>>,
}

impl<'a> SpsEncoder<'a> {
    pub fn new(
        encoder_context: &Arc<Mutex<EncoderContext>>,
        coder: &'a mut BoolCoder,
    ) -> SpsEncoder<'a> {
        SpsEncoder {
            coder,
            encoder_context: encoder_context.clone(),
        }
    }

    pub fn encode(&mut self, sps: &SequenceParameterSet) -> Vec<bool> {
        let mut bins = Bins::new();
        debug_eprint!("sps_id ");
        bins.push_initial_bins_with_size(sps.id as u64, 4);
        debug_eprint!("sps_video_parameter_set_id ");
        bins.push_bins_with_size(sps.video_parameter_set_id as u64, 4);
        debug_eprint!("sps_max_sublayers_minus1 ");
        bins.push_bins_with_size(sps.max_sublayers as u64 - 1, 3);
        debug_eprint!("sps_chroma_format_idc ");
        bins.push_bins_with_size(sps.chroma_format as u64, 2);
        debug_eprint!("sps_log2_ctu_size_minus5 ");
        bins.push_bins_with_size(sps.log2_ctu_size as u64 - 5, 2);
        debug_eprint!("sps_ptl_hrd_params_present_flag ");
        bins.push_bin(sps.ptl_dpb_hrd_params_present_flag);
        if sps.ptl_dpb_hrd_params_present_flag {
            if let Some(ptl) = &sps.profile_tier_level {
                let ectx = self.encoder_context.clone();
                let mut ptl_encoder = PtlEncoder::new(&ectx, self.coder);
                ptl_encoder.encode(&mut bins, ptl, true, sps.max_sublayers);
            } else {
                panic!();
            }
        }
        debug_eprint!("sps_gdr_enabled_flag ");
        bins.push_bin(sps.gdr_enabled_flag);
        debug_eprint!("sps_ref_pic_resampling_enabled_flag ");
        bins.push_bin(sps.ref_pic_resampling_enabled_flag);
        if sps.ref_pic_resampling_enabled_flag {
            debug_eprint!("sps_res_change_in_clvs_allowed_flag ");
            bins.push_bin(sps.res_change_in_clvs_allowed_flag);
        }
        debug_eprint!("sps_pic_width_max_in_luma_samples ");
        self.coder
            .encode_unsigned_exp_golomb(&mut bins, sps.pic_width_max_in_luma_samples as u64);
        debug_eprint!("sps_pic_height_max_in_luma_samples ");
        self.coder
            .encode_unsigned_exp_golomb(&mut bins, sps.pic_height_max_in_luma_samples as u64);
        debug_eprint!("sps_conformance_window_present_flag ");
        bins.push_bin(sps.conformance_window.is_some());
        if let Some(conformance_window) = &sps.conformance_window {
            debug_eprint!("sps_conformance_window_left_offset ");
            self.coder
                .encode_unsigned_exp_golomb(&mut bins, conformance_window.left_offset as u64);
            debug_eprint!("sps_conformance_window_right_offset ");
            self.coder
                .encode_unsigned_exp_golomb(&mut bins, conformance_window.right_offset as u64);
            debug_eprint!("sps_conformance_window_top_offset ");
            self.coder
                .encode_unsigned_exp_golomb(&mut bins, conformance_window.top_offset as u64);
            debug_eprint!("sps_conformance_window_bottom_offset ");
            self.coder
                .encode_unsigned_exp_golomb(&mut bins, conformance_window.bottom_offset as u64);
        }
        debug_eprint!("sps_subpic_info_present_flag ");
        bins.push_bin(sps.subpic_info.is_some());
        if let Some(subpic_info) = &sps.subpic_info {
            debug_eprint!("sps_subpic_info_num_subpics_minus1 ");
            self.coder
                .encode_unsigned_exp_golomb(&mut bins, subpic_info.num_subpics as u64 - 1);
            if subpic_info.num_subpics > 1 {
                debug_eprint!("sps_subpic_info_independent_subpics_flag ");
                bins.push_bin(subpic_info.independent_subpics_flag);
                debug_eprint!("sps_subpic_info_subpic_same_size_flag ");
                bins.push_bin(subpic_info.subpic_same_size_flag);
            }
            if subpic_info.num_subpics > 1 {
                for i in 0..subpic_info.num_subpics {
                    if subpic_info.subpic_same_size_flag || i == 0 {
                        let ectx = self.encoder_context.clone();
                        let ectx = ectx.lock().unwrap();
                        if i > 0 && sps.pic_width_max_in_luma_samples > ectx.ctb_size_y {
                            let tmp_width_val =
                                (sps.pic_width_max_in_luma_samples + ectx.ctb_size_y - 1)
                                    / ectx.ctb_size_y;
                            let n = (tmp_width_val as f64).log2().ceil() as usize;
                            debug_eprint!("sps_subpic_info_subpic_ctu_top_left_xs ");
                            bins.push_bins_with_size(
                                subpic_info.subpic_ctu_top_left_xs[i] as u64,
                                n,
                            );
                        }
                        if i > 0 && sps.pic_height_max_in_luma_samples > ectx.ctb_size_y {
                            let tmp_width_val =
                                (sps.pic_height_max_in_luma_samples + ectx.ctb_size_y - 1)
                                    / ectx.ctb_size_y;
                            let n = (tmp_width_val as f64).log2().ceil() as usize;
                            bins.push_bins_with_size(
                                subpic_info.subpic_ctu_top_left_ys[i] as u64,
                                n,
                            );
                        }
                        if i < subpic_info.num_subpics - 1
                            && sps.pic_width_max_in_luma_samples > ectx.ctb_size_y
                        {
                            let tmp_width_val =
                                (sps.pic_width_max_in_luma_samples + ectx.ctb_size_y - 1)
                                    / ectx.ctb_size_y;
                            let n = (tmp_width_val as f64).log2().ceil() as usize;
                            debug_eprint!("sps_subpic_info_subpic_widths_minus1 ");
                            bins.push_bins_with_size(subpic_info.subpic_widths[i] as u64 - 1, n);
                        }
                        if i < subpic_info.num_subpics - 1
                            && sps.pic_height_max_in_luma_samples > ectx.ctb_size_y
                        {
                            let tmp_height_val =
                                (sps.pic_height_max_in_luma_samples + ectx.ctb_size_y - 1)
                                    / ectx.ctb_size_y;
                            let n = (tmp_height_val as f64).log2().ceil() as usize;
                            debug_eprint!("sps_subpic_info_subpic_heights_minus1 ");
                            bins.push_bins_with_size(subpic_info.subpic_heights[i] as u64 - 1, n);
                        }
                    }
                    if !subpic_info.independent_subpics_flag {
                        debug_eprint!("sps_subpic_info_subpic_treated_as_pic_flag ");
                        bins.push_bin(subpic_info.subpic_treated_as_pic_flags[i]);
                        debug_eprint!("sps_subpic_info_loop_filter_across_subpic_enabled_flags ");
                        bins.push_bin(subpic_info.loop_filter_across_subpic_enabled_flags[i]);
                    }
                }
            }
            debug_eprint!("sps_subpic_info_subpic_id_len_minus1 ");
            self.coder
                .encode_unsigned_exp_golomb(&mut bins, subpic_info.subpic_id_len as u64 - 1);
            debug_eprint!("sps_subpic_info_subpic_id_mapping_explicitly_signalled_flag ");
            bins.push_bin(subpic_info.subpic_id_mapping_explicitly_signalled_flag);
            if subpic_info.subpic_id_mapping_explicitly_signalled_flag {
                debug_eprint!("sps_subpic_info_subpic_id_mapping_present_flag ");
                bins.push_bin(subpic_info.subpic_id_mapping_present_flag);
                if subpic_info.subpic_id_mapping_present_flag {
                    let n = subpic_info.subpic_id_len as usize;
                    for i in 0..subpic_info.num_subpics {
                        debug_eprint!("sps_subpic_info_subpic_id ");
                        bins.push_bins_with_size(subpic_info.subpic_id[i] as u64, n);
                    }
                }
            }
        }
        debug_eprint!("sps_bitdepth_minus8 ");
        self.coder
            .encode_unsigned_exp_golomb(&mut bins, sps.bitdepth as u64 - 8);
        debug_eprint!("sps_entropy_coding_sync_enabled_flag ");
        bins.push_bin(sps.entropy_coding_sync_enabled_flag);
        debug_eprint!("sps_entropy_point_offsets_present_flag ");
        bins.push_bin(sps.entry_point_offsets_present_flag);
        debug_eprint!("sps_log2_max_pic_order_cnt_lsb_minus4 ");
        bins.push_bins_with_size(sps.log2_max_pic_order_cnt_lsb as u64 - 4, 4);
        debug_eprint!("sps_poc_msb_cycle_flag ");
        bins.push_bin(sps.poc_msb_cycle_flag);
        if sps.poc_msb_cycle_flag {
            debug_eprint!("sps_poc_msb_cycle_len_minus1 ");
            self.coder
                .encode_unsigned_exp_golomb(&mut bins, sps.poc_msb_cycle_len as u64 - 1);
        }
        debug_eprint!("sps_num_extra_ph_bytes ");
        bins.push_bins_with_size(sps.num_extra_ph_bytes as u64, 2);
        for i in 0..sps.num_extra_ph_bytes * 8 {
            debug_eprint!("sps_extra_ph_bit_present_flags ");
            bins.push_bin(sps.extra_ph_bit_present_flags[i]);
        }
        debug_eprint!("sps_num_extra_sh_bytes ");
        bins.push_bins_with_size(sps.num_extra_sh_bytes as u64, 2);
        for i in 0..sps.num_extra_sh_bytes * 8 {
            debug_eprint!("sps_extra_sh_bit_present_flags ");
            bins.push_bin(sps.extra_sh_bit_present_flags[i]);
        }
        if sps.ptl_dpb_hrd_params_present_flag {
            if sps.max_sublayers > 1 {
                debug_eprint!("sps_sublayer_dpb_params_flag ");
                bins.push_bin(sps.sublayer_dpb_params_flag);
            }
            let ectx = self.encoder_context.clone();
            let mut dpbp_encoder = DpbpEncoder::new(&ectx, self.coder);
            dpbp_encoder.encode(
                &mut bins,
                &sps.dpb_parameters,
                sps.max_sublayers,
                sps.sublayer_dpb_params_flag,
            );
        }
        debug_eprint!("sps_log2_min_luma_coding_block_size ");
        self.coder
            .encode_unsigned_exp_golomb(&mut bins, sps.log2_min_luma_coding_block_size as u64 - 2);
        debug_eprint!("sps_partition_constraints_override_enabled_flag ");
        bins.push_bin(sps.partition_constraints_override_enabled_flag);
        debug_eprint!("sps_partition_constraints_log2_diff_min_qt_min_cb_intra_slice_luma ");
        self.coder.encode_unsigned_exp_golomb(
            &mut bins,
            sps.partition_constraints
                .log2_diff_min_qt_min_cb_intra_slice_luma as u64,
        );
        debug_eprint!("sps_partition_constraints_max_mtt_hierarchy_depth_intra_slice_luma ");
        self.coder.encode_unsigned_exp_golomb(
            &mut bins,
            sps.partition_constraints
                .max_mtt_hierarchy_depth_intra_slice_luma as u64,
        );
        if sps
            .partition_constraints
            .max_mtt_hierarchy_depth_intra_slice_luma
            != 0
        {
            debug_eprint!("sps_partition_constraints_log2_diff_max_bt_min_qt_intra_slice_luma ");
            self.coder.encode_unsigned_exp_golomb(
                &mut bins,
                sps.partition_constraints
                    .log2_diff_max_bt_min_qt_intra_slice_luma as u64,
            );
            debug_eprint!("sps_partition_constraints_log2_diff_max_tt_min_qt_intra_slice_luma ");
            self.coder.encode_unsigned_exp_golomb(
                &mut bins,
                sps.partition_constraints
                    .log2_diff_max_tt_min_qt_intra_slice_luma as u64,
            );
        }
        if sps.chroma_format != ChromaFormat::Monochrome {
            debug_eprint!("sps_partition_constraints_qtbtt_dual_tree_intra_flag ");
            bins.push_bin(sps.partition_constraints.qtbtt_dual_tree_intra_flag);
        }
        if sps.partition_constraints.qtbtt_dual_tree_intra_flag {
            debug_eprint!("sps_partition_constraints_log2_diff_min_qt_min_cb_intra_slice_chroma ");
            self.coder.encode_unsigned_exp_golomb(
                &mut bins,
                sps.partition_constraints
                    .log2_diff_min_qt_min_cb_intra_slice_chroma as u64,
            );
            debug_eprint!("sps_partition_constraints_max_mtt_hierarchy_depth_intra_slice_chroma ");
            self.coder.encode_unsigned_exp_golomb(
                &mut bins,
                sps.partition_constraints
                    .max_mtt_hierarchy_depth_intra_slice_chroma as u64,
            );
            if sps
                .partition_constraints
                .max_mtt_hierarchy_depth_intra_slice_chroma
                != 0
            {
                debug_eprint!(
                    "sps_partition_constraints_log2_diff_max_bt_min_qt_intra_slice_chroma "
                );
                self.coder.encode_unsigned_exp_golomb(
                    &mut bins,
                    sps.partition_constraints
                        .log2_diff_max_bt_min_qt_intra_slice_chroma as u64,
                );
                debug_eprint!(
                    "sps_partition_constraints_log2_diff_max_tt_min_qt_intra_slice_chroma "
                );
                self.coder.encode_unsigned_exp_golomb(
                    &mut bins,
                    sps.partition_constraints
                        .log2_diff_max_tt_min_qt_intra_slice_chroma as u64,
                );
            }
        }
        debug_eprint!("sps_partition_constraints_log2_diff_min_qt_min_cb_inter_slice ");
        self.coder.encode_unsigned_exp_golomb(
            &mut bins,
            sps.partition_constraints
                .log2_diff_min_qt_min_cb_inter_slice as u64,
        );
        debug_eprint!("sps_partition_constraints_max_mtt_hierarchy_depth_inter_slice ");
        self.coder.encode_unsigned_exp_golomb(
            &mut bins,
            sps.partition_constraints
                .max_mtt_hierarchy_depth_inter_slice as u64,
        );
        if sps
            .partition_constraints
            .max_mtt_hierarchy_depth_inter_slice
            != 0
        {
            debug_eprint!("sps_partition_constraints_log2_diff_max_bt_min_qt_inter_slice ");
            self.coder.encode_unsigned_exp_golomb(
                &mut bins,
                sps.partition_constraints
                    .log2_diff_max_bt_min_qt_inter_slice as u64,
            );
            debug_eprint!("sps_partition_constraints_log2_diff_max_tt_min_qt_inter_slice ");
            self.coder.encode_unsigned_exp_golomb(
                &mut bins,
                sps.partition_constraints
                    .log2_diff_max_tt_min_qt_inter_slice as u64,
            );
        }

        let ectx = self.encoder_context.clone();
        let ectx = ectx.lock().unwrap();
        if ectx.ctb_size_y > 32 {
            debug_eprint!("sps_max_luma_transform_size_64_flag ");
            bins.push_bin(sps.max_luma_transform_size_64_flag);
        }
        debug_eprint!("sps_transform_skip_enabled_flag ");
        bins.push_bin(sps.transform_skip_enabled_flag);
        if sps.transform_skip_enabled_flag {
            debug_eprint!("sps.log2_transform_skip_max_size ");
            self.coder
                .encode_unsigned_exp_golomb(&mut bins, sps.log2_transform_skip_max_size as u64);
            debug_eprint!("sps.bdpcm_enabled_flag ");
            bins.push_bin(sps.bdpcm_enabled_flag);
        }
        debug_eprint!("sps.mts_enabled_flag ");
        bins.push_bin(sps.mts_enabled_flag);
        if sps.mts_enabled_flag {
            debug_eprint!("sps_explicit_mts_intra_enabled_flag ");
            bins.push_bin(sps.explicit_mts_intra_enabled_flag);
            debug_eprint!("sps_explicit_mts_inter_enabled_flag ");
            bins.push_bin(sps.explicit_mts_inter_enabled_flag);
        }
        debug_eprint!("sps_lfnst_enabled_flag ");
        bins.push_bin(sps.lfnst_enabled_flag);
        if sps.chroma_format != ChromaFormat::Monochrome {
            debug_eprint!("sps_joint_cbcr_enabled_flag ");
            bins.push_bin(sps.joint_cbcr_enabled_flag);
            debug_eprint!("sps_same_qp_table_for_chroma_flag ");
            bins.push_bin(sps.same_qp_table_for_chroma_flag);
            let num_qp_tables = if sps.same_qp_table_for_chroma_flag {
                1
            } else if sps.joint_cbcr_enabled_flag {
                3
            } else {
                2
            };
            for i in 0..num_qp_tables {
                debug_eprint!("sps_qp_tables_qp_table_start_minus26 ");
                self.coder.encode_signed_exp_golomb(
                    &mut bins,
                    sps.qp_tables[i].qp_table_start as i64 - 26,
                );
                debug_eprint!("sps_qp_tables_num_points_in_qp_table_minu1 ");
                self.coder.encode_unsigned_exp_golomb(
                    &mut bins,
                    sps.qp_tables[i].num_points_in_qp_table as u64 - 1,
                );
                for j in 0..sps.qp_tables[i].num_points_in_qp_table {
                    debug_eprint!("sps_qp_tables_delta_qp_in_val_minus1 ");
                    self.coder.encode_unsigned_exp_golomb(
                        &mut bins,
                        sps.qp_tables[i].delta_qp_in_val[j] as u64 - 1,
                    );
                    debug_eprint!("sps_qp_tables_delta_qp_diff_val ");
                    self.coder.encode_unsigned_exp_golomb(
                        &mut bins,
                        sps.qp_tables[i].delta_qp_diff_val[j] as u64,
                    );
                }
            }
        }
        debug_eprint!("sps_sao_enabled_flag ");
        bins.push_bin(sps.sao_enabled_flag);
        debug_eprint!("sps_alf_enabled_flag ");
        bins.push_bin(sps.alf_enabled_flag);
        if sps.alf_enabled_flag && sps.chroma_format != ChromaFormat::Monochrome {
            debug_eprint!("sps_ccalf_enabled_flag ");
            bins.push_bin(sps.ccalf_enabled_flag);
        }
        debug_eprint!("sps_lmcs_enabled_flag ");
        bins.push_bin(sps.lmcs_enabled_flag);
        debug_eprint!("sps_weighted_pred_flag ");
        bins.push_bin(sps.weighted_pred_flag);
        debug_eprint!("sps_weighted_bipred_flag ");
        bins.push_bin(sps.weighted_bipred_flag);
        debug_eprint!("sps_long_term_ref_pics_flag ");
        bins.push_bin(sps.long_term_ref_pics_flag);
        if sps.video_parameter_set_id > 0 {
            debug_eprint!("sps_inter_layer_prediction_enabled_flag ");
            bins.push_bin(sps.inter_layer_prediction_enabled_flag);
        }
        debug_eprint!("sps_idr_rpl_present_flag ");
        bins.push_bin(sps.idr_rpl_present_flag);
        debug_eprint!("sps_rpl1_same_as_rpl0_flag ");
        bins.push_bin(sps.rpl1_same_as_rpl0_flag);
        let n = if sps.rpl1_same_as_rpl0_flag { 1 } else { 2 };
        for i in 0..n {
            debug_eprint!("sps_ref_pic_lists_num_ref_pic_list ");
            self.coder.encode_unsigned_exp_golomb(
                &mut bins,
                sps.ref_pic_lists[i].num_ref_pic_list as u64,
            );
            let ectx = self.encoder_context.clone();
            let mut ref_pic_list_struct_encoder = RefPicListStructEncoder::new(&ectx, self.coder);
            for j in 0..sps.ref_pic_lists[i].num_ref_pic_list {
                ref_pic_list_struct_encoder.encode_rpls(
                    &mut bins,
                    &sps.ref_pic_lists[i].ref_pic_list_structs[j],
                    j,
                    sps.ref_pic_lists[i].num_ref_pic_list,
                    sps,
                );
            }
        }
        debug_eprint!("sps_ref_wraparound_enabled_flag ");
        bins.push_bin(sps.ref_wraparound_enabled_flag);
        debug_eprint!("sps_temporal_mvp_enabled_flag ");
        bins.push_bin(sps.temporal_mvp_enabled_flag);
        if sps.temporal_mvp_enabled_flag {
            debug_eprint!("sps_sbtmvp_enabled_flag ");
            bins.push_bin(sps.sbtmvp_enabled_flag);
        }
        debug_eprint!("sps_amvr_enabled_flag ");
        bins.push_bin(sps.amvr_enabled_flag);
        debug_eprint!("sps_bdof_enabled_flag ");
        bins.push_bin(sps.bdof_enabled_flag);
        if sps.bdof_enabled_flag {
            debug_eprint!("sps_bdof_control_present_in_ph_flag ");
            bins.push_bin(sps.bdof_control_present_in_ph_flag);
        }
        debug_eprint!("sps_smvd_enabled_flag ");
        bins.push_bin(sps.smvd_enabled_flag);
        debug_eprint!("sps_dmvr_enabled_flag ");
        bins.push_bin(sps.dmvr_enabled_flag);
        if sps.dmvr_enabled_flag {
            debug_eprint!("sps_dmvr_control_present_flag ");
            bins.push_bin(sps.dmvr_control_present_in_ph_flag);
        }
        debug_eprint!("sps_mmvd_enabled_flag ");
        bins.push_bin(sps.mmvd_enabled_flag);
        if sps.mmvd_enabled_flag {
            debug_eprint!("sps_mmvd_fullpel_only_enabled_flag ");
            bins.push_bin(sps.mmvd_fullpel_only_enabled_flag);
        }
        debug_eprint!("sps_six_minus_max_num_merge_cand ");
        self.coder
            .encode_unsigned_exp_golomb(&mut bins, sps.six_minus_max_num_merge_cand as u64);
        debug_eprint!("sps_sbt_enabled_flag ");
        bins.push_bin(sps.sbt_enabled_flag);
        debug_eprint!("sps_affine_enabled_flag ");
        bins.push_bin(sps.affine_enabled_flag);
        if sps.affine_enabled_flag {
            debug_eprint!("sps_five_mminus_max_num_subblock_merge_cand ");
            self.coder.encode_unsigned_exp_golomb(
                &mut bins,
                sps.five_minus_max_num_subblock_merge_cand as u64,
            );
            debug_eprint!("sps_six_param_affine_enabled_flag ");
            bins.push_bin(sps.six_param_affine_enabled_flag);
            if sps.amvr_enabled_flag {
                debug_eprint!("sps_affine_amvr_enabled_flag ");
                bins.push_bin(sps.affine_amvr_enabled_flag);
            }
            debug_eprint!("sps_affine_prof_enabled_flag ");
            bins.push_bin(sps.affine_prof_enabled_flag);
            if sps.affine_prof_enabled_flag {
                debug_eprint!("sps_prof_control_present_in_ph_flag ");
                bins.push_bin(sps.prof_control_present_in_ph_flag);
            }
        }
        debug_eprint!("sps_bcw_enabled_flag ");
        bins.push_bin(sps.bcw_enabled_flag);
        debug_eprint!("sps_ciip_enabled_flag ");
        bins.push_bin(sps.ciip_enabled_flag);
        //debug_eprintln!("MaxNumMergeCand {}", ectx.max_num_merge_cand);
        if ectx.max_num_merge_cand >= 2 {
            debug_eprint!("sps_gpm_enabled_flag ");
            bins.push_bin(sps.gpm_enabled_flag);
            if sps.gpm_enabled_flag && ectx.max_num_merge_cand >= 3 {
                debug_eprint!("sps_max_num_merge_cand_minus_max_num_gpm_cand ");
                self.coder.encode_unsigned_exp_golomb(
                    &mut bins,
                    sps.max_num_merge_cand_minus_max_num_gpm_cand as u64,
                );
            }
        }
        debug_eprint!("sps_log2_parallel_merge_level ");
        self.coder
            .encode_unsigned_exp_golomb(&mut bins, sps.log2_parallel_merge_level as u64 - 2);
        debug_eprint!("sps_isp_enabled_flag ");
        bins.push_bin(sps.isp_enabled_flag);
        debug_eprint!("sps_mrl_enabled_flag ");
        bins.push_bin(sps.mrl_enabled_flag);
        debug_eprint!("sps_mip_enabled_flag ");
        bins.push_bin(sps.mip_enabled_flag);
        if sps.chroma_format != ChromaFormat::Monochrome {
            debug_eprint!("sps_cclm_enabled_flag ");
            bins.push_bin(sps.cclm_enabled_flag);
        }
        if sps.chroma_format == ChromaFormat::YCbCr420 {
            debug_eprint!("sps_chroma_horizontal_collocated_flag ");
            bins.push_bin(sps.chroma_horizontal_collocated_flag);
            debug_eprint!("sps_chroma_vertical_collocated_flag ");
            bins.push_bin(sps.chroma_vertical_collocated_flag);
        }
        debug_eprint!("sps_palette_enabled_flag ");
        bins.push_bin(sps.palette_enabled_flag);
        if sps.chroma_format == ChromaFormat::YCbCr444 && !sps.max_luma_transform_size_64_flag {
            debug_eprint!("sps_act_enabled_flag ");
            bins.push_bin(sps.act_enabled_flag);
        }
        if sps.transform_skip_enabled_flag || sps.palette_enabled_flag {
            debug_eprint!("sps_min_qp_prime_ts ");
            self.coder
                .encode_unsigned_exp_golomb(&mut bins, sps.min_qp_prime_ts as u64);
        }
        debug_eprint!("sps_ibc_enabled_flag ");
        bins.push_bin(sps.ibc_enabled_flag);
        if sps.ibc_enabled_flag {
            debug_eprint!("sps_six_minus_max_num_ibc_merge_cand ");
            self.coder
                .encode_unsigned_exp_golomb(&mut bins, sps.six_minus_max_num_ibc_merge_cand as u64);
        }
        debug_eprint!("sps_ladf_parameters_present_flag ");
        bins.push_bin(sps.ladf_parameters.is_some());
        if let Some(ladf_parameters) = &sps.ladf_parameters {
            debug_eprint!("sps_ladf_parameters_num_ladf_intervals_minus2 ");
            bins.push_bins_with_size(ladf_parameters.num_ladf_intervals as u64 - 2, 4);
            debug_eprint!("sps_ladf_parameters_lowest_interval_qp_offset ");
            self.coder.encode_signed_exp_golomb(
                &mut bins,
                ladf_parameters.lowest_interval_qp_offset as i64,
            );
            for i in 0..ladf_parameters.num_ladf_intervals - 1 {
                debug_eprint!("sps_ladf_parameters_qp_offset ");
                self.coder
                    .encode_signed_exp_golomb(&mut bins, ladf_parameters.qp_offset[i] as i64);
                debug_eprint!("sps_ladf_parameters_delta_threshold_minus1 ");
                self.coder.encode_unsigned_exp_golomb(
                    &mut bins,
                    ladf_parameters.delta_threshold[i] as u64 - 1,
                );
            }
        }
        debug_eprint!("sps_explicit_scaling_list_enabled_flag ");
        bins.push_bin(sps.explicit_scaling_list_enabled_flag);
        if sps.lfnst_enabled_flag && sps.explicit_scaling_list_enabled_flag {
            debug_eprint!("sps_scaling_matrix_for_lfnst_disabled_flag ");
            bins.push_bin(sps.scaling_matrix_for_lfnst_disabled_flag);
        }
        if sps.act_enabled_flag && sps.explicit_scaling_list_enabled_flag {
            debug_eprint!("sps_scaling_matrix_for_alternative_colour_space_disabled_flag ");
            bins.push_bin(sps.scaling_matrix_for_alternative_colour_space_disabled_flag);
        }
        if sps.scaling_matrix_for_alternative_colour_space_disabled_flag {
            debug_eprint!("sps_scaling_matrix_designated_colour_space_flag ");
            bins.push_bin(sps.scaling_matrix_designated_colour_space_flag);
        }
        debug_eprint!("sps_dep_quant_enabled_flag ");
        bins.push_bin(sps.dep_quant_enabled_flag);
        debug_eprint!("sps_sign_data_hiding_enabled_flag ");
        bins.push_bin(sps.sign_data_hiding_enabled_flag);
        debug_eprint!("sps_virtual_boundaries_enabled_flag ");
        bins.push_bin(sps.virtual_boundaries_enabled_flag);
        if sps.virtual_boundaries_enabled_flag {
            debug_eprint!("sps_virtual_boundaries_present_flag ");
            bins.push_bin(
                sps.virtual_boundary_parameters
                    .virtual_boundaries_present_flag,
            );
            if sps
                .virtual_boundary_parameters
                .virtual_boundaries_present_flag
            {
                debug_eprint!("sps_virtual_boundaries_num_ver_virtual_boundaries ");
                self.coder.encode_unsigned_exp_golomb(
                    &mut bins,
                    sps.virtual_boundary_parameters.num_ver_virtual_boundaries as u64,
                );
                for i in 0..sps.virtual_boundary_parameters.num_ver_virtual_boundaries {
                    debug_eprint!("sps_virtual_boundaries_pos_xs_minus1 ");
                    self.coder.encode_unsigned_exp_golomb(
                        &mut bins,
                        sps.virtual_boundary_parameters.virtual_boundary_pos_xs[i] as u64 - 1,
                    );
                }
                debug_eprint!("sps_virtual_boundaries_num_hor_virtual_boundaries ");
                self.coder.encode_unsigned_exp_golomb(
                    &mut bins,
                    sps.virtual_boundary_parameters.num_hor_virtual_boundaries as u64,
                );
                for i in 0..sps.virtual_boundary_parameters.num_hor_virtual_boundaries {
                    debug_eprint!("sps_virtual_boundaries_pos_ys_minus1 ");
                    self.coder.encode_unsigned_exp_golomb(
                        &mut bins,
                        sps.virtual_boundary_parameters.virtual_boundary_pos_ys[i] as u64 - 1,
                    );
                }
            }
        }
        if sps.ptl_dpb_hrd_params_present_flag {
            debug_eprint!("sps_timing_hrd_params_present_flag ");
            bins.push_bin(sps.timing_hrd_params_present_flag);
            if sps.timing_hrd_params_present_flag {
                if let Some(general_timing_hrd_parameters) = &sps.general_timing_hrd_parameters {
                    let ectx = self.encoder_context.clone();
                    {
                        let mut hrd_encoder = HrdEncoder::new(&ectx, self.coder);
                        hrd_encoder.encode_general_timing_hrd_parameters(
                            &mut bins,
                            general_timing_hrd_parameters,
                        );
                    }
                    if sps.max_sublayers > 1 {
                        debug_eprint!("sps.sublayer_cpb_params_present_flag ");
                        bins.push_bin(sps.sublayer_cpb_params_present_flag);
                    }
                    let first_sublayer = if sps.sublayer_cpb_params_present_flag {
                        0
                    } else {
                        sps.max_sublayers - 1
                    };
                    {
                        let mut hrd_encoder = HrdEncoder::new(&ectx, self.coder);
                        hrd_encoder.encode_ols_timing_hrd_parameters(
                            &mut bins,
                            &sps.ols_timing_hrd_parameters,
                            general_timing_hrd_parameters,
                            first_sublayer,
                            sps.max_sublayers,
                        );
                    }
                } else {
                    panic!();
                }
            }
        }
        debug_eprint!("sps.field_seq_flag ");
        bins.push_bin(sps.field_seq_flag);
        debug_eprint!("sps.vui_parameters_present_flag ");
        bins.push_bin(sps.vui_parameters_present_flag);
        if sps.vui_parameters_present_flag {
            debug_eprint!("sps.vui_payload_size_minus1 ");
            self.coder
                .encode_unsigned_exp_golomb(&mut bins, sps.vui_payload_size as u64 - 1);
            bins.byte_align();
            self.encode_vui_payload(&mut bins, sps.vui_payload_size, sps);
        }
        debug_eprint!("sps.extension_data_present_flag ");
        bins.push_bin(!sps.extension_data.is_empty());
        if !sps.extension_data.is_empty() {
            for i in 0..sps.extension_data.len() {
                // FIXME
                debug_eprint!("sps.extension_data ");
                bins.push_bin(!sps.extension_data[i]);
            }
        }
        // rbsp trailing bits
        let rbsp_stop_one_bit = true;
        debug_eprint!("sps.rbsp_stop_one_bit ");
        bins.push_bin(rbsp_stop_one_bit);
        bins.byte_align();
        bins.into_iter().collect()
    }

    pub fn encode_vui_payload(
        &mut self,
        _bins: &mut Bins,
        _payload_size: usize,
        _sps: &SequenceParameterSet,
    ) {
        // TODO
    }
}

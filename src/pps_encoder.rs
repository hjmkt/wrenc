use super::bins::*;
use super::bool_coder::*;
use super::encoder_context::*;
use super::pps::*;
use debug_print::*;
use std::sync::{Arc, Mutex};

pub struct PpsEncoder<'a> {
    coder: &'a mut BoolCoder,
    encoder_context: Arc<Mutex<EncoderContext>>,
}

impl<'a> PpsEncoder<'a> {
    pub fn new(
        encoder_context: &Arc<Mutex<EncoderContext>>,
        coder: &'a mut BoolCoder,
    ) -> PpsEncoder<'a> {
        PpsEncoder {
            coder,
            encoder_context: encoder_context.clone(),
        }
    }

    pub fn encode(&mut self, pps: &PictureParameterSet) -> Vec<bool> {
        let mut bins = Bins::new();
        debug_eprint!("pps.id ");
        bins.push_initial_bins_with_size(pps.id as u64, 6);
        debug_eprint!("pps.seq_parameter_set_id ");
        bins.push_bins_with_size(pps.seq_parameter_set_id as u64, 4);
        debug_eprint!("pps.mixed_nalu_types_in_pic_flag ");
        bins.push_bin(pps.mixed_nalu_types_in_pic_flag);
        debug_eprint!("pps.pic_width_in_luma_samples ");
        self.coder
            .encode_unsigned_exp_golomb(&mut bins, pps.pic_width_in_luma_samples as u64);
        debug_eprint!("pps.pic_height_in_luma_samples ");
        self.coder
            .encode_unsigned_exp_golomb(&mut bins, pps.pic_height_in_luma_samples as u64);
        debug_eprint!("pps.conformance_window_present_flag ");
        bins.push_bin(pps.conformance_window.is_some());
        if let Some(conformance_window) = &pps.conformance_window {
            debug_eprint!("pps.conformance_window.left_offset ");
            self.coder
                .encode_unsigned_exp_golomb(&mut bins, conformance_window.left_offset as u64);
            debug_eprint!("pps.conformance_window.right_offset ");
            self.coder
                .encode_unsigned_exp_golomb(&mut bins, conformance_window.right_offset as u64);
            debug_eprint!("pps.conformance_window.top_offset ");
            self.coder
                .encode_unsigned_exp_golomb(&mut bins, conformance_window.top_offset as u64);
            debug_eprint!("pps.conformance_window.bottom_offset ");
            self.coder
                .encode_unsigned_exp_golomb(&mut bins, conformance_window.bottom_offset as u64);
        }
        debug_eprint!("pps.scaling_window_explicit_signalling_flag ");
        bins.push_bin(pps.scaling_window_explicit_signalling_flag);
        if pps.scaling_window_explicit_signalling_flag {
            debug_eprint!("pps.scaling_window.left_offset ");
            self.coder
                .encode_signed_exp_golomb(&mut bins, pps.scaling_window.left_offset as i64);
            debug_eprint!("pps.scaling_window.right_offset ");
            self.coder
                .encode_signed_exp_golomb(&mut bins, pps.scaling_window.right_offset as i64);
            debug_eprint!("pps.scaling_window.top_offset ");
            self.coder
                .encode_signed_exp_golomb(&mut bins, pps.scaling_window.top_offset as i64);
            debug_eprint!("pps.scaling_window.bottom_offset ");
            self.coder
                .encode_signed_exp_golomb(&mut bins, pps.scaling_window.bottom_offset as i64);
        }
        debug_eprint!("pps.output_flag_present_flag ");
        bins.push_bin(pps.output_flag_present_flag);
        debug_eprint!("pps.no_pic_partition_flag ");
        bins.push_bin(pps.no_pic_partition_flag);
        debug_eprint!("pps.subpic_id_mapping_present_flag ");
        bins.push_bin(pps.subpic_id_mapping_present_flag);
        if pps.subpic_id_mapping_present_flag {
            if !pps.no_pic_partition_flag {
                debug_eprint!("pps.num_subpics_minus1 ");
                self.coder
                    .encode_unsigned_exp_golomb(&mut bins, pps.num_subpics as u64 - 1);
            }
            debug_eprint!("pps.subpic_id_len_minus1 ");
            self.coder
                .encode_unsigned_exp_golomb(&mut bins, pps.subpic_id_len as u64 - 1);
            for i in 0..pps.num_subpics {
                debug_eprint!("pps.subpic_id ");
                bins.push_bins_with_size(pps.subpic_id[i] as u64, pps.subpic_id_len);
            }
        }
        if !pps.no_pic_partition_flag {
            debug_eprint!("pps.log2_ctu_size_minus5 ");
            bins.push_bins_with_size(pps.log2_ctu_size as u64 - 5, 2);
            debug_eprint!("pps.num_exp_tile_columns_minus1 ");
            self.coder
                .encode_unsigned_exp_golomb(&mut bins, pps.num_exp_tile_columns as u64 - 1);
            debug_eprint!("pps.num_exp_tile_rows_minus1 ");
            self.coder
                .encode_unsigned_exp_golomb(&mut bins, pps.num_exp_tile_rows as u64 - 1);
            for i in 0..pps.num_exp_tile_columns {
                debug_eprint!("pps.tile_column_widths_minus1 ");
                self.coder
                    .encode_unsigned_exp_golomb(&mut bins, pps.tile_column_widths[i] as u64 - 1);
            }
            for i in 0..pps.num_exp_tile_rows {
                debug_eprint!("pps.tile_column_heights_minus1 ");
                self.coder
                    .encode_unsigned_exp_golomb(&mut bins, pps.tile_column_heights[i] as u64 - 1);
            }
            let ectx = self.encoder_context.clone();
            let ectx = ectx.lock().unwrap();
            if ectx.num_tiles_in_pic > 1 {
                debug_eprint!("pps.loop_filter_across_slices_enabled_flag ");
                bins.push_bin(
                    pps.partition_parameters
                        .loop_filter_across_slices_enabled_flag,
                );
                debug_eprint!("pps.rect_slice_flag ");
                bins.push_bin(pps.partition_parameters.rect_slice_flag);
            }
            if pps.partition_parameters.rect_slice_flag {
                debug_eprint!("pps.single_slice_per_subpic_flag ");
                bins.push_bin(pps.partition_parameters.single_slice_per_subpic_flag);
            }
            if pps.partition_parameters.rect_slice_flag
                && !pps.partition_parameters.single_slice_per_subpic_flag
            {
                debug_eprint!("pps.num_slices_in_pic_minus1 ");
                self.coder.encode_unsigned_exp_golomb(
                    &mut bins,
                    pps.partition_parameters.num_slices_in_pic as u64 - 1,
                );
                if pps.partition_parameters.num_slices_in_pic > 2 {
                    debug_eprint!("pps.tile_idx_delta_present_flag ");
                    bins.push_bin(pps.partition_parameters.tile_idx_delta_present_flag);
                }
                let mut i = 0;
                while i < pps.partition_parameters.num_slices_in_pic - 1 {
                    if ectx.slice_top_left_tile_idx[i] % ectx.num_tile_columns
                        != ectx.num_tile_columns - 1
                    {
                        debug_eprint!("pps.slice_width_in_tiles_minus1 ");
                        self.coder.encode_unsigned_exp_golomb(
                            &mut bins,
                            pps.partition_parameters.slices[i].slice_width_in_tiles as u64 - 1,
                        );
                    }
                    if ectx.slice_top_left_tile_idx[i] / ectx.num_tile_columns
                        != ectx.num_tile_rows - 1
                        && (pps.partition_parameters.tile_idx_delta_present_flag
                            || ectx.slice_top_left_tile_idx[i] % ectx.num_tile_columns == 0)
                    {
                        debug_eprint!("pps.slice_height_in_tiles_minus1 ");
                        self.coder.encode_unsigned_exp_golomb(
                            &mut bins,
                            pps.partition_parameters.slices[i].slice_height_in_tiles as u64 - 1,
                        );
                    }
                    if pps.partition_parameters.slices[i].slice_width_in_tiles == 1
                        && pps.partition_parameters.slices[i].slice_height_in_tiles == 1
                        && ectx.row_height_val
                            [ectx.slice_top_left_tile_idx[i] / ectx.num_tile_columns]
                            > 1
                    {
                        debug_eprint!("pps.num_exp_slices_in_tile ");
                        self.coder.encode_unsigned_exp_golomb(
                            &mut bins,
                            pps.partition_parameters.slices[i].num_exp_slices_in_tile as u64,
                        );
                        for j in 0..pps.partition_parameters.slices[i].num_exp_slices_in_tile {
                            debug_eprint!("pps.exp_slice_height_in_ctus_minus1 ");
                            self.coder.encode_unsigned_exp_golomb(
                                &mut bins,
                                pps.partition_parameters.slices[i].exp_slice_height_in_ctus[j]
                                    as u64
                                    - 1,
                            );
                        }
                        i += ectx.num_slices_in_tile[i] - 1;
                    }
                    if pps.partition_parameters.tile_idx_delta_present_flag
                        && i < pps.partition_parameters.num_slices_in_pic - 1
                    {
                        debug_eprint!("pps.tile_idx_delta_val_minus1 ");
                        self.coder.encode_signed_exp_golomb(
                            &mut bins,
                            pps.partition_parameters.slices[i].tile_idx_delta_val as i64 - 1,
                        );
                    }
                    i += 1;
                }
            }
            if !pps.partition_parameters.rect_slice_flag
                || pps.partition_parameters.single_slice_per_subpic_flag
                || pps.partition_parameters.num_slices_in_pic > 1
            {
                debug_eprint!("pps.loop_filter_across_slices_enabled_flag ");
                bins.push_bin(
                    pps.partition_parameters
                        .loop_filter_across_slices_enabled_flag,
                );
            }
        }
        debug_eprint!("pps.cabac_init_present_flag ");
        bins.push_bin(pps.cabac_init_present_flag);
        for i in 0..2 {
            debug_eprint!("pps.num_ref_idx_default_active_minus1 ");
            self.coder.encode_unsigned_exp_golomb(
                &mut bins,
                pps.num_ref_idx_default_active[i] as u64 - 1,
            );
        }
        debug_eprint!("pps.rpl1_idx_present_flag ");
        bins.push_bin(pps.rpl1_idx_present_flag);
        debug_eprint!("pps.weighted_pred_flag ");
        bins.push_bin(pps.weighted_pred_flag);
        debug_eprint!("pps.weighted_bipred_flag ");
        bins.push_bin(pps.weighted_bipred_flag);
        debug_eprint!("pps.ref_wraparound_enabled_flag ");
        bins.push_bin(pps.ref_wraparound_enabled_flag);
        if pps.ref_wraparound_enabled_flag {
            debug_eprint!("pps.pic_width_minus_wraparound_offset ");
            self.coder.encode_unsigned_exp_golomb(
                &mut bins,
                pps.pic_width_minus_wraparound_offset as u64,
            );
        }
        debug_eprint!("pps.init_qp_minus26 ");
        self.coder
            .encode_signed_exp_golomb(&mut bins, pps.init_qp as i64 - 26);
        debug_eprint!("pps.cu_qp_delta_enabled_flag ");
        bins.push_bin(pps.cu_qp_delta_enabled_flag);
        debug_eprint!("pps.chroma_tool_offsets_present_flag ");
        bins.push_bin(pps.chroma_tool_offsets_present_flag);
        if pps.chroma_tool_offsets_present_flag {
            let chroma_tool_offsets = &pps.chroma_tool_offsets;
            debug_eprint!("pps.cb_qp_offset ");
            self.coder
                .encode_signed_exp_golomb(&mut bins, chroma_tool_offsets.cb_qp_offset as i64);
            debug_eprint!("pps.cr_qp_offset ");
            self.coder
                .encode_signed_exp_golomb(&mut bins, chroma_tool_offsets.cr_qp_offset as i64);
            debug_eprint!("pps.joint_cbcr_qp_offset_present_flag ");
            bins.push_bin(chroma_tool_offsets.joint_cbcr_qp_offset_present_flag);
            if chroma_tool_offsets.joint_cbcr_qp_offset_present_flag {
                debug_eprint!("pps.joint_cbcr_qp_offset_value ");
                self.coder.encode_signed_exp_golomb(
                    &mut bins,
                    chroma_tool_offsets.joint_cbcr_qp_offset_value as i64,
                );
            }
            debug_eprint!("pps.slice_chroma_qp_offsets_present_flag ");
            bins.push_bin(chroma_tool_offsets.slice_chroma_qp_offsets_present_flag);
            debug_eprint!("pps.cu_chroma_qp_offset_list_enabled_flag ");
            bins.push_bin(chroma_tool_offsets.cu_chroma_qp_offset_list_enabled_flag);
            if chroma_tool_offsets.cu_chroma_qp_offset_list_enabled_flag {
                debug_eprint!("pps.chroma_qp_offset_list_len_minus1 ");
                self.coder.encode_unsigned_exp_golomb(
                    &mut bins,
                    chroma_tool_offsets.chroma_qp_offset_list_len as u64 - 1,
                );
                for i in 0..chroma_tool_offsets.chroma_qp_offset_list_len {
                    debug_eprint!("pps.cb_qp_offset_list ");
                    self.coder.encode_signed_exp_golomb(
                        &mut bins,
                        chroma_tool_offsets.cb_qp_offset_list[i] as i64,
                    );
                    debug_eprint!("pps.cr_qp_offset_list ");
                    self.coder.encode_signed_exp_golomb(
                        &mut bins,
                        chroma_tool_offsets.cr_qp_offset_list[i] as i64,
                    );
                    if chroma_tool_offsets.joint_cbcr_qp_offset_present_flag {
                        debug_eprint!("pps.joint_cbcr_qp_offset_list ");
                        self.coder.encode_signed_exp_golomb(
                            &mut bins,
                            chroma_tool_offsets.joint_cbcr_qp_offset_list[i] as i64,
                        );
                    }
                }
            }
        }
        debug_eprint!("pps.deblocking_filter_control_present_flag ");
        bins.push_bin(pps.deblocking_filter_control_present_flag);
        if pps.deblocking_filter_control_present_flag {
            let dfc = &pps.deblocking_filter_control;
            debug_eprint!("pps.deblocking_filter_override_enabled_flag ");
            bins.push_bin(dfc.deblocking_filter_override_enabled_flag);
            debug_eprint!("pps.deblocking_filter_disabled_flag ");
            bins.push_bin(dfc.deblocking_filter_disabled_flag);
            if !pps.no_pic_partition_flag && dfc.deblocking_filter_override_enabled_flag {
                debug_eprint!("pps.dbf_info_in_ph_flag ");
                bins.push_bin(dfc.dbf_info_in_ph_flag);
            }
            if !dfc.deblocking_filter_disabled_flag {
                debug_eprint!("pps.luma_beta_offset_div2 ");
                self.coder
                    .encode_signed_exp_golomb(&mut bins, dfc.luma_beta_offset as i64 / 2);
                debug_eprint!("pps.luma_tc_offset_div2 ");
                self.coder
                    .encode_signed_exp_golomb(&mut bins, dfc.luma_tc_offset as i64 / 2);
                if pps.chroma_tool_offsets_present_flag {
                    debug_eprint!("pps.cb_beta_offset_div2 ");
                    self.coder
                        .encode_signed_exp_golomb(&mut bins, dfc.cb_beta_offset as i64 / 2);
                    debug_eprint!("pps.cb_tc_offset_div2 ");
                    self.coder
                        .encode_signed_exp_golomb(&mut bins, dfc.cb_tc_offset as i64 / 2);
                    debug_eprint!("pps.cr_beta_offset_div2 ");
                    self.coder
                        .encode_signed_exp_golomb(&mut bins, dfc.cr_beta_offset as i64 / 2);
                    debug_eprint!("pps.cr_tc_offset_div2 ");
                    self.coder
                        .encode_signed_exp_golomb(&mut bins, dfc.cr_tc_offset as i64 / 2);
                }
            }
        }
        if !pps.no_pic_partition_flag {
            debug_eprint!("pps.rpl_info_in_ph_flag ");
            bins.push_bin(pps.partition_parameters.rpl_info_in_ph_flag);
            debug_eprint!("pps.sao_info_in_ph_flag ");
            bins.push_bin(pps.partition_parameters.sao_info_in_ph_flag);
            debug_eprint!("pps.alf_info_in_ph_flag ");
            bins.push_bin(pps.partition_parameters.alf_info_in_ph_flag);
            if (pps.weighted_pred_flag || pps.weighted_bipred_flag)
                && pps.partition_parameters.rpl_info_in_ph_flag
            {
                debug_eprint!("pps.wp_info_in_ph_flag ");
                bins.push_bin(pps.partition_parameters.wp_info_in_ph_flag);
            }
            debug_eprint!("pps.qp_delta_info_in_ph_flag ");
            bins.push_bin(pps.partition_parameters.qp_delta_info_in_ph_flag);
        }
        debug_eprint!("pps.picture_header_extension_present_flag ");
        bins.push_bin(pps.picture_header_extension_present_flag);
        debug_eprint!("pps.slice_header_extension_present_flag ");
        bins.push_bin(pps.slice_header_extension_present_flag);
        debug_eprint!("pps.extension_data_present_flag ");
        bins.push_bin(!pps.extension_data.is_empty());
        if !pps.extension_data.is_empty() {
            for i in 0..pps.extension_data.len() {
                debug_eprint!("pps.extension_data ");
                bins.push_bin(pps.extension_data[i]);
            }
        }
        let rbsp_stop_one_bit = true;
        debug_eprint!("pps.rbsp_stop_one_bit ");
        bins.push_bin(rbsp_stop_one_bit);
        bins.byte_align();
        bins.into_iter().collect()
    }
}

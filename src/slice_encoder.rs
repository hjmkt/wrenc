use super::bins::*;
use super::bool_coder::*;
//use super::cabac_contexts::*;
use super::common::*;
use super::ctu_encoder::*;
use super::encoder_context::*;
use super::nal::*;
use super::ph_encoder::*;
use super::pwt_encoder::*;
use super::rpl_encoder::*;
use super::slice::*;
use super::slice_header::*;
use debug_print::*;
use std::sync::{Arc, Mutex};

pub struct SliceEncoder<'a> {
    coder: &'a mut BoolCoder,
    encoder_context: Arc<Mutex<EncoderContext>>,
}

impl<'a> SliceEncoder<'a> {
    pub fn new(
        encoder_context: &Arc<Mutex<EncoderContext>>,
        coder: &'a mut BoolCoder,
    ) -> SliceEncoder<'a> {
        SliceEncoder {
            coder,
            encoder_context: encoder_context.clone(),
        }
    }

    pub fn encode_sh(&mut self, bins: &mut Bins, sh: &SliceHeader, slice: &Slice) {
        let sh_picture_header_in_slice_header_flag = sh.ph_in_sh.is_some();
        debug_eprint!("sh.picture_header_in_slice_header_flag ");
        bins.push_initial_bin(sh_picture_header_in_slice_header_flag);
        if let Some(ph) = &sh.ph_in_sh {
            let mut ph_encoder = PhEncoder::new(&self.encoder_context, self.coder);
            ph_encoder.encode(bins, ph, sh.sps, sh.pps);
        }
        if let Some(subpic_info) = &sh.sps.subpic_info {
            let n = subpic_info.subpic_id_len;
            debug_eprint!("sh.subpic_id ");
            bins.push_bins_with_size(sh.subpic_id as u64, n);
        }
        {
            let ectx = self.encoder_context.lock().unwrap();
            if (sh.pps.partition_parameters.rect_slice_flag
                && ectx.num_slices_in_subpic[ectx.curr_subpic_idx] > 1)
                || (!sh.pps.partition_parameters.rect_slice_flag && ectx.num_tiles_in_pic > 1)
            {
                let n = if sh.pps.partition_parameters.rect_slice_flag {
                    (ectx.num_slices_in_subpic[ectx.curr_subpic_idx] as f64)
                        .log2()
                        .ceil() as usize
                } else {
                    (ectx.num_tiles_in_pic as f64).log2().ceil() as usize
                };
                debug_eprint!("sh.slice_address ");
                bins.push_bins_with_size(sh.slice_address as u64, n);
            }
            for i in 0..ectx.num_extra_sh_bits {
                debug_eprint!("sh.extra_bit ");
                bins.push_bin(sh.extra_bit[i]);
            }
            if !sh.pps.partition_parameters.rect_slice_flag
                && ectx.num_tiles_in_pic - sh.slice_address > 1
            {
                debug_eprint!("sh.num_tiles_in_slice_minus1 ");
                self.coder
                    .encode_unsigned_exp_golomb(bins, sh.num_tiles_in_slice as u64 - 1);
            }
        }
        if sh.ph.as_ref().unwrap().inter_slice_allowed_flag {
            debug_eprint!("sh.slice_type ");
            self.coder
                .encode_unsigned_exp_golomb(bins, sh.slice_type as u64);
        }
        if let NALUnitType::IDR_W_RADL
        | NALUnitType::IDR_N_LP
        | NALUnitType::CRA_NUT
        | NALUnitType::GDR_NUT = slice.nal_unit_type
        {
            debug_eprint!("sh.no_output_of_prior_pics_flag ");
            bins.push_bin(sh.no_output_of_prior_pics_flag);
        }
        if sh.sps.alf_enabled_flag && !sh.pps.partition_parameters.alf_info_in_ph_flag {
            debug_eprint!("sh.alf_enabled_flag ");
            bins.push_bin(sh.alf_enabled_flag);
            if sh.alf_enabled_flag {
                debug_eprint!("sh.alf_info.num_alf_aps_ids_luma ");
                bins.push_bins_with_size(sh.alf_info.num_alf_aps_ids_luma as u64, 3);
                for i in 0..sh.alf_info.num_alf_aps_ids_luma {
                    debug_eprint!("sh.alf_info.aps_id_luma ");
                    bins.push_bins_with_size(sh.alf_info.aps_id_luma[i] as u64, 3);
                }
                if sh.sps.chroma_format != ChromaFormat::Monochrome {
                    debug_eprint!("sh.alf_info.cb_enabled_flag ");
                    bins.push_bin(sh.alf_info.cb_enabled_flag);
                    debug_eprint!("sh.alf_info.cr_enabled_flag ");
                    bins.push_bin(sh.alf_info.cr_enabled_flag);
                }
                if sh.alf_info.cb_enabled_flag || sh.alf_info.cr_enabled_flag {
                    debug_eprint!("sh.alf_info.aps_id_chroma ");
                    bins.push_bins_with_size(sh.alf_info.aps_id_chroma as u64, 3);
                }
                if sh.sps.ccalf_enabled_flag {
                    debug_eprint!("sh.alf_info.cc_cb_enabled_flag ");
                    bins.push_bin(sh.alf_info.cc_cb_enabled_flag);
                    if sh.alf_info.cc_cb_enabled_flag {
                        debug_eprint!("sh.alf_info.cc_cb_aps_id ");
                        bins.push_bins_with_size(sh.alf_info.cc_cb_aps_id as u64, 3);
                    }
                    debug_eprint!("sh.alf_info.cc_cr_enabled_flag ");
                    bins.push_bin(sh.alf_info.cc_cr_enabled_flag);
                    if sh.alf_info.cc_cr_enabled_flag {
                        debug_eprint!("sh.alf_info.cc_cr_aps_id ");
                        bins.push_bins_with_size(sh.alf_info.cc_cr_aps_id as u64, 3);
                    }
                }
            }
        }
        if sh.ph.as_ref().unwrap().lmcs_enabled_flag && sh.ph_in_sh.is_none() {
            debug_eprint!("sh.alf_info.lmcs_used_flag ");
            bins.push_bin(sh.lmcs_used_flag);
        }
        if sh.ph.as_ref().unwrap().explicit_scaling_list_enabled_flag && sh.ph_in_sh.is_none() {
            debug_eprint!("sh.alf_info.explicit_scaling_list_used_flag ");
            bins.push_bin(sh.explicit_scaling_list_used_flag);
        }
        if !sh.pps.partition_parameters.rpl_info_in_ph_flag
            && ((slice.nal_unit_type != NALUnitType::IDR_W_RADL
                && slice.nal_unit_type != NALUnitType::IDR_N_LP)
                || sh.sps.idr_rpl_present_flag)
        {
            let ectx = self.encoder_context.clone();
            let mut rpl_encoder = RefPicListStructEncoder::new(&ectx, self.coder);
            rpl_encoder.encode(
                bins,
                &sh.ref_pic_lists,
                sh.sps,
                sh.pps,
                sh.ph.as_ref().unwrap(),
            );
        }
        let ectx = &mut self.encoder_context.lock().unwrap();
        if (sh.slice_type != SliceType::I
            && sh.ref_pic_lists[0].ref_pic_list_structs[ectx.rpls_idx[0]].num_ref_entries > 1)
            || (sh.slice_type == SliceType::B
                && sh.ref_pic_lists[1].ref_pic_list_structs[ectx.rpls_idx[1]].num_ref_entries > 1)
        {
            debug_eprint!("sh.num_ref_idx_active_override_flag ");
            bins.push_bin(sh.num_ref_idx_active_override_flag);
            if sh.num_ref_idx_active_override_flag {
                for i in 0..if sh.slice_type == SliceType::B { 2 } else { 1 } {
                    if sh.ref_pic_lists[i].ref_pic_list_structs[ectx.rpls_idx[i]].num_ref_entries
                        > 1
                    {
                        debug_eprint!("sh.num_ref_idx_active_minus1 ");
                        self.coder
                            .encode_unsigned_exp_golomb(bins, sh.num_ref_idx_active[i] as u64 - 1);
                    }
                }
            }
        }
        if sh.slice_type != SliceType::I {
            if sh.pps.cabac_init_present_flag {
                debug_eprint!("sh.cabac_init_flag ");
                bins.push_bin(sh.cabac_init_flag);
            }
            if sh.ph.as_ref().unwrap().temporal_mvp_enabled_flag
                && !sh.pps.partition_parameters.rpl_info_in_ph_flag
            {
                if sh.slice_type == SliceType::B {
                    debug_eprint!("sh.collocated_from_l0_flag ");
                    bins.push_bin(sh.collocated_from_l0_flag);
                }
                if (sh.collocated_from_l0_flag && ectx.num_ref_idx_active[0] > 1)
                    || (!sh.collocated_from_l0_flag && ectx.num_ref_idx_active[1] > 1)
                {
                    debug_eprint!("sh.collocated_ref_idx ");
                    self.coder
                        .encode_unsigned_exp_golomb(bins, sh.collocated_ref_idx as u64);
                }
            }
            if !sh.pps.partition_parameters.wp_info_in_ph_flag
                && ((sh.pps.weighted_pred_flag && sh.slice_type == SliceType::P)
                    || (sh.pps.weighted_bipred_flag && sh.slice_type == SliceType::B))
            {
                let ectx = self.encoder_context.clone();
                let mut pwt_encoder = PredWeightTableEncoder::new(&ectx, self.coder);
                pwt_encoder.encode(
                    bins,
                    sh.ph.as_ref().unwrap().pred_weight_table.as_ref().unwrap(),
                    sh.sps,
                    sh.pps,
                    sh.ph.as_ref().unwrap(),
                );
            }
        }
        if !sh.pps.partition_parameters.qp_delta_info_in_ph_flag {
            debug_eprint!("sh.qp_delta {}", sh.qp_delta);
            self.coder
                .encode_signed_exp_golomb(bins, sh.qp_delta as i64);
        }
        if sh
            .pps
            .chroma_tool_offsets
            .slice_chroma_qp_offsets_present_flag
        {
            debug_eprint!("sh.cb_qp_offset ");
            self.coder
                .encode_signed_exp_golomb(bins, sh.cb_qp_offset as i64);
            debug_eprint!("sh.cr_qp_offset ");
            self.coder
                .encode_signed_exp_golomb(bins, sh.cr_qp_offset as i64);
            if sh.sps.joint_cbcr_enabled_flag {
                debug_eprint!("sh.joint_cbcr_qp_offset ");
                self.coder
                    .encode_signed_exp_golomb(bins, sh.joint_cbcr_qp_offset as i64);
            }
        }
        if sh
            .pps
            .chroma_tool_offsets
            .cu_chroma_qp_offset_list_enabled_flag
        {
            debug_eprint!("sh.cu_chroma_qp_offset_enabled_flag ");
            bins.push_bin(sh.cu_chroma_qp_offset_enabled_flag);
        }
        if sh.sps.sao_enabled_flag && !sh.pps.partition_parameters.sao_info_in_ph_flag {
            debug_eprint!("sh.sao_luma_used_flag ");
            bins.push_bin(sh.sao_luma_used_flag);
            if sh.sps.chroma_format != ChromaFormat::Monochrome {
                debug_eprint!("sh.sao_chroma_used_flag ");
                bins.push_bin(sh.sao_chroma_used_flag);
            }
        }
        if sh
            .pps
            .deblocking_filter_control
            .deblocking_filter_override_enabled_flag
            && !sh.pps.deblocking_filter_control.dbf_info_in_ph_flag
        {
            debug_eprint!("sh.deblocking_params_present_flag ");
            bins.push_bin(sh.deblocking_params_present_flag);
        }
        if sh.deblocking_params_present_flag {
            if !sh
                .pps
                .deblocking_filter_control
                .deblocking_filter_disabled_flag
            {
                debug_eprint!("sh.deblocking_filter_disabled_flag ");
                bins.push_bin(sh.deblocking_filter_disabled_flag);
            }
            if !sh.deblocking_filter_disabled_flag {
                debug_eprint!("sh.luma_beta_offset_div2 ");
                self.coder
                    .encode_signed_exp_golomb(bins, sh.luma_beta_offset as i64 / 2);
                debug_eprint!("sh.luma_tc_offset_div2 ");
                self.coder
                    .encode_signed_exp_golomb(bins, sh.luma_tc_offset as i64 / 2);
                if sh.pps.chroma_tool_offsets_present_flag {
                    debug_eprint!("sh.cb_beta_offset_div2 ");
                    self.coder
                        .encode_signed_exp_golomb(bins, sh.cb_beta_offset as i64 / 2);
                    debug_eprint!("sh.cb_tc_offset_div2 ");
                    self.coder
                        .encode_signed_exp_golomb(bins, sh.cb_tc_offset as i64 / 2);
                    debug_eprint!("sh.cr_beta_offset_div2 ");
                    self.coder
                        .encode_signed_exp_golomb(bins, sh.cr_beta_offset as i64 / 2);
                    debug_eprint!("sh.cr_tc_offset_div2 ");
                    self.coder
                        .encode_signed_exp_golomb(bins, sh.cr_tc_offset as i64 / 2);
                }
            }
        }
        if sh.sps.dep_quant_enabled_flag {
            debug_eprint!("sh.dep_quant_used_flag ");
            bins.push_bin(sh.dep_quant_used_flag);
        }
        if sh.sps.sign_data_hiding_enabled_flag && !sh.dep_quant_used_flag {
            debug_eprint!("sh.sign_data_hiding_used_flag ");
            bins.push_bin(sh.sign_data_hiding_used_flag);
        }
        if sh.sps.transform_skip_enabled_flag
            && !sh.dep_quant_used_flag
            && !sh.sign_data_hiding_used_flag
        {
            debug_eprint!("sh.ts_residual_coding_disabled_flag ");
            bins.push_bin(sh.ts_residual_coding_disabled_flag);
        }
        if sh.pps.slice_header_extension_present_flag {
            debug_eprint!("sh.slice_header_extension_length ");
            self.coder
                .encode_unsigned_exp_golomb(bins, sh.slice_header_extension_length as u64);
            for i in 0..sh.slice_header_extension_length {
                bins.push_bins_with_size(sh.slice_header_extension_data_byte[i] as u64, 8);
            }
        }
        {
            // FIXME shouldn't be here
            ectx.num_entry_points = 0;
            if sh.sps.entry_point_offsets_present_flag {
                for i in 1..ectx.num_ctus_in_curr_slice {
                    let ctb_addr_x = ectx.ctb_addr_in_curr_slice[i] % ectx.pic_width_in_ctbs_y;
                    let ctb_addr_y = ectx.ctb_addr_in_curr_slice[i] / ectx.pic_width_in_ctbs_y;
                    let prev_ctb_addr_x =
                        ectx.ctb_addr_in_curr_slice[i - 1] % ectx.pic_width_in_ctbs_y;
                    let prev_ctb_addr_y =
                        ectx.ctb_addr_in_curr_slice[i - 1] / ectx.pic_width_in_ctbs_y;
                    if ectx.ctb_to_tile_col_bd[ctb_addr_y]
                        != ectx.ctb_to_tile_col_bd[prev_ctb_addr_y]
                        || ectx.ctb_to_tile_col_bd[ctb_addr_x]
                            != ectx.ctb_to_tile_col_bd[prev_ctb_addr_x]
                        || (ctb_addr_y != prev_ctb_addr_y
                            && sh.sps.entropy_coding_sync_enabled_flag)
                    {
                        ectx.num_entry_points += 1;
                    }
                }
            }
            if ectx.num_entry_points > 0 {
                debug_eprint!("sh.entry_offset_len_minus1 ");
                self.coder
                    .encode_unsigned_exp_golomb(bins, sh.entry_offset_len as u64 - 1);
                for i in 0..ectx.num_entry_points {
                    let n = sh.entry_offset_len;
                    bins.push_bins_with_size(sh.entry_point_offset[i] as u64, n);
                }
            }
        }
        //let byte_alignment_bit = true;
        //self.coder.push_bit(&mut bits, byte_alignment_bit);
        //self.coder.byte_align(&mut bits);
        // byte_alignment
        let byte_alignment_bit_equal_to_one = true;
        bins.push_bin(byte_alignment_bit_equal_to_one);
        bins.byte_align();
    }

    pub fn encode(&mut self, slice: &Slice, sh: &SliceHeader) -> Bins {
        let mut bins = Bins::new();
        self.encode_sh(&mut bins, sh, slice);

        let ectx = self.encoder_context.clone();
        {
            let ectx = &mut ectx.lock().unwrap();
            ectx.first_ctb_row_in_slice = true;
        }
        let tiles = slice.tiles.lock().unwrap();
        for (tile_idx, tile) in tiles.iter().enumerate() {
            let (ctus, num_ctu_rows, num_ctu_cols) = {
                let tile = tile.lock().unwrap();
                (tile.ctus.clone(), tile.num_ctu_rows, tile.num_ctu_cols)
            };
            let ctus = {
                let tmp = ctus.lock().unwrap();
                tmp.clone()
            };
            for (ctu_row, row_ctus) in ctus.iter().enumerate().take(num_ctu_rows) {
                for (ctu_col, ctu) in row_ctus.iter().enumerate().take(num_ctu_cols) {
                    debug_eprintln!("ctu {} {}", ctu_row, ctu_col);
                    {
                        let mut ectx = ectx.lock().unwrap();
                        ectx.ctb_addr_x = ctu_col + ctu_col;
                        ectx.ctb_addr_y = ctu_row + ctu_row;
                        ectx.ctb_addr_in_rs =
                            ectx.ctb_addr_y * ectx.pic_width_in_ctbs_y + ectx.ctb_addr_y;
                        if ctu_col == 0 {
                            ectx.num_hmvp_cand = 0;
                            ectx.num_hmvp_ibc_cand = 0;
                            ectx.reset_ibc_buf = true;
                        }
                    }
                    let ctu = ctu.clone();
                    let mut ctu_encoder = CtuEncoder::new(&ectx, self.coder);
                    ctu_encoder.encode(&mut bins, ctu.clone(), sh);
                    if tile_idx == tiles.len() - 1
                        && ctu_row == num_ctu_rows - 1
                        && ctu_col == num_ctu_cols - 1
                    {
                        debug_eprintln!("slice end_of_slice_one_bit ");
                        self.coder.encode_cabac_end_one_bit(
                            &mut bins,
                            //CabacContext::EndOfSliceOneBit
                        );
                    } else if ctu_col == num_ctu_cols - 1 {
                        if ctu_row == num_ctu_rows - 1 {
                            debug_eprintln!("slice end_of_tile_one_bit ");
                            self.coder.encode_cabac_end_one_bit(
                                &mut bins,
                                //CabacContext::EndOfSliceOneBit,
                            );
                            let byte_alignment_bit_equal_to_one = true;
                            bins.push_bin(byte_alignment_bit_equal_to_one);
                            bins.byte_align();
                        } else if sh.sps.entropy_coding_sync_enabled_flag {
                            debug_eprintln!("slice end_of_subset_one_bit ");
                            self.coder.encode_cabac_end_one_bit(
                                &mut bins,
                                //CabacContext::EndOfSliceOneBit,
                            );
                            let byte_alignment_bit_equal_to_one = true;
                            bins.push_bin(byte_alignment_bit_equal_to_one);
                            bins.byte_align();
                        }
                        let mut ectx = ectx.lock().unwrap();
                        ectx.first_ctb_row_in_slice = false;
                    }
                }
            }
        }
        // TODO See 9.3.4.3.5
        // rbsp_slice_trailing_bits
        //let rbsp_stop_one_bit = true;
        //self.coder.push_bit(&mut bits, rbsp_stop_one_bit);
        bins.byte_align();
        //println!("all bits {:?}", bits);
        //let rbsp_cabac_zero_word = 0;
        //self.coder
        //.push_bits_with_size(&mut bits, rbsp_cabac_zero_word, 16);
        //println!("actual bits {:?}", bits);
        //bins.into_iter().collect()
        bins
    }
}

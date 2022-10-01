//use super::aps::*;
use super::bins::*;
use super::cabac_contexts::*;
use super::common::*;
use super::ctu::*;
use super::encoder_context::*;
use super::pps::*;
use super::slice_header::*;
use super::sps::*;
use debug_print::*;
use std::sync::{Arc, Mutex};

// TODO enable parallel coding

#[derive(Clone)]
pub struct BoolCoder {
    pub cabac_p_state_idx: Vec<[Vec<[u16; 2]>; 3]>,
    pub cabac_table_state_sync: Vec<[Vec<[u16; 2]>; 3]>,
    pub cabac_ivl_curr_range: u16,
    pub cabac_ivl_offset: u16,
    pub cabac_first_bit_flag: bool,
    pub cabac_bits_outstanding: usize,
}

impl BoolCoder {
    pub fn new() -> BoolCoder {
        let mut cabac_p_state_idx = vec![[vec![], vec![], vec![]]; ctx_table.len()];
        let mut cabac_table_state_sync = vec![[vec![], vec![], vec![]]; ctx_table.len()];
        for ctx in 0..ctx_table.len() {
            if ctx_table[ctx].is_empty() {
                continue;
            }
            for init_type in 0..3 {
                cabac_p_state_idx[ctx][init_type] =
                    vec![[0, 0]; ctx_table[ctx][0][init_type].len()];
                cabac_table_state_sync[ctx][init_type] =
                    vec![[0, 0]; ctx_table[ctx][0][init_type].len()];
            }
        }
        BoolCoder {
            cabac_p_state_idx,
            cabac_table_state_sync,
            cabac_ivl_curr_range: 0,
            cabac_ivl_offset: 0,
            cabac_first_bit_flag: true,
            cabac_bits_outstanding: 0,
        }
    }

    //#[inline(always)]
    //pub fn push_bin(&self, out_bins: &mut Bins, bin: bool) {
    //out_bins.push_bin(bin);
    //}

    //#[inline(always)]
    //pub fn push_bins_with_size(&self, out_bins: &mut Bins, bins: u64, size: usize) {
    //out_bins.push_bins_with_size(bins, size);
    //}

    //#[inline(always)]
    //pub fn byte_align(&self, out_bins: &mut Bins) {
    //out_bins.byte_align();
    //}

    #[inline(always)]
    fn msb_u64(v: u64) -> u64 {
        let mut msb = 0;
        let mut v = v;
        while v > 0 {
            msb += 1;
            v >>= 1;
        }
        msb
    }

    #[inline(always)]
    pub fn encode_unsigned_exp_golomb(&mut self, out_bins: &mut Bins, v: u64) {
        if v == 0 {
            out_bins.push_bin(true);
        } else {
            let n = Self::msb_u64(v + 1) - 1;
            let r = (1 << n) - 1;
            let golomb = v - r;
            out_bins.push_bins_with_size(0, n as usize);
            out_bins.push_bin(true);
            out_bins.push_bins_with_size(golomb, n as usize);
        }
    }

    #[inline(always)]
    pub fn encode_signed_exp_golomb(&mut self, out_bins: &mut Bins, v: i64) {
        if v == 0 {
            out_bins.push_bin(true);
        } else {
            let sign = (v < 0) as u64;
            let code = (v.unsigned_abs() - 1) * 2 + 1 + sign;
            self.encode_unsigned_exp_golomb(out_bins, code);
        }
    }

    pub fn init_cabac(
        &mut self,
        is_first_ctu_in_slice_or_tile: bool,
        ctu: &CodingTreeUnit,
        sps: &SequenceParameterSet,
        pps: &PictureParameterSet,
        ectx: Arc<Mutex<EncoderContext>>,
    ) {
        let ctb_addr_x = ctu.x / ctu.width;
        let ctb_to_tile_col_bd = {
            let tile = ctu.tile.as_ref().unwrap();
            let tile = tile.lock().unwrap();
            tile.ctu_col
        };
        if is_first_ctu_in_slice_or_tile {
            let mut ectx = ectx.lock().unwrap();
            self.init_ctx_table(&ectx);
            for ch_type in 0..=1 {
                ectx.predictor_palette_size[ch_type] = 0;
            }
        } else if sps.entropy_coding_sync_enabled_flag && ctb_addr_x == ctb_to_tile_col_bd {
            let mut ectx = ectx.lock().unwrap();
            let x_nb_t = ctu.x as isize;
            let y_nb_t = ctu.y as isize - ectx.ctb_size_y as isize;
            let available_flag_t = ectx.derive_neighbouring_block_availability(
                ctu.x, ctu.y, x_nb_t, y_nb_t, ctu.width, ctu.height, false, false, false, sps, pps,
            );
            if available_flag_t {
                self.sync_ctx_table();
                if sps.palette_enabled_flag {
                    self.sync_palette_predictor(&mut ectx);
                }
            } else {
                self.init_ctx_table(&ectx);

                for ch_type in 0..=1 {
                    ectx.predictor_palette_size[ch_type] = 0;
                }
            }
        } else {
            let mut ectx = ectx.lock().unwrap();
            self.init_ctx_table(&ectx);
            for ch_type in 0..=1 {
                ectx.predictor_palette_size[ch_type] = 0;
            }
        }
        self.init_arithmetic_engine();
    }

    #[inline(always)]
    pub fn transition_cabac_state(
        &mut self,
        bin_val: bool,
        ctx: CabacContext,
        init_type: usize,
        ctx_idx: usize,
    ) {
        let shift_idx = ctx_table[ctx as usize][1][init_type][ctx_idx];
        let shift0 = (shift_idx >> 2) + 2;
        let shift1 = (shift_idx & 3) + 3 + shift0;
        self.cabac_p_state_idx[ctx as usize][init_type][ctx_idx][0] = self.cabac_p_state_idx
            [ctx as usize][init_type][ctx_idx][0]
            - (self.cabac_p_state_idx[ctx as usize][init_type][ctx_idx][0] >> shift0)
            + ((1023 * bin_val as u16) >> shift0);
        self.cabac_p_state_idx[ctx as usize][init_type][ctx_idx][1] = self.cabac_p_state_idx
            [ctx as usize][init_type][ctx_idx][1]
            - (self.cabac_p_state_idx[ctx as usize][init_type][ctx_idx][1] >> shift1)
            + ((16383 * bin_val as u16) >> shift1);
    }

    #[inline(always)]
    pub fn renorm_cabac_encode_engine(&mut self, out_bins: &mut Bins) {
        while self.cabac_ivl_curr_range < 256 {
            if self.cabac_ivl_offset < 256 {
                self.flush_cabac_bin(out_bins, false);
            } else if self.cabac_ivl_offset >= 512 {
                self.cabac_ivl_offset -= 512;
                self.flush_cabac_bin(out_bins, true);
            } else {
                self.cabac_ivl_offset -= 256;
                self.cabac_bits_outstanding += 1;
            }
            self.cabac_ivl_curr_range <<= 1;
            self.cabac_ivl_offset <<= 1;
        }
    }

    #[inline(always)]
    pub fn flush_cabac_bin(&mut self, out_bins: &mut Bins, bin: bool) {
        // TODO reduce cost to check it everytime after initial bin
        if !self.cabac_first_bit_flag {
            out_bins.push_bin(bin);
        }
        self.cabac_first_bit_flag = false;
        // TODO optimize without loop
        //if self.cabac_bits_outstanding < 7 {
        while self.cabac_bits_outstanding > 0 {
            out_bins.push_bin(!bin);
            self.cabac_bits_outstanding -= 1;
        }
        //} else {
        //out_bins.push_same_bins(!bin, self.cabac_bits_outstanding);
        //self.cabac_bits_outstanding = 0;
        //}
    }

    #[inline(always)]
    pub fn flush_cabac_trailing_bin(&mut self, out_bins: &mut Bins, bin: bool) {
        out_bins.push_bin(bin);
        while self.cabac_bits_outstanding > 0 {
            out_bins.push_bin(!bin);
            self.cabac_bits_outstanding -= 1;
        }
    }

    #[inline(always)]
    pub fn encode_cabac_bypass(&mut self, bins: &mut Bins, bin: bool) {
        self.cabac_ivl_offset <<= 1;
        if bin {
            self.cabac_ivl_offset += self.cabac_ivl_curr_range;
        }
        if self.cabac_ivl_offset >= 1024 {
            self.flush_cabac_bin(bins, true);
            self.cabac_ivl_offset -= 1024;
        } else if self.cabac_ivl_offset < 512 {
            self.flush_cabac_bin(bins, false)
        } else {
            self.cabac_ivl_offset -= 512;
            self.cabac_bits_outstanding += 1;
        }
    }

    pub fn encode_arithmetic_stop_one_bit(&mut self, out_bins: &mut Bins, bin: bool) {
        //self.encode_cabac_terminate(bit)
        self.cabac_ivl_curr_range -= 2;
        if bin {
            self.cabac_ivl_offset += self.cabac_ivl_curr_range;
            self.cabac_ivl_curr_range = 2;
            self.renorm_cabac_encode_engine(out_bins);
            let ex_bin = (self.cabac_ivl_offset >> 9) & 1 > 0;
            self.flush_cabac_bin(out_bins, ex_bin);
            let ex_2bins = ((self.cabac_ivl_offset >> 7) & 3) | 1;
            self.flush_cabac_trailing_bin(out_bins, (ex_2bins >> 1) & 1 > 0);
            self.flush_cabac_trailing_bin(out_bins, ex_2bins & 1 > 0);
        } else {
            self.renorm_cabac_encode_engine(out_bins);
        };
        self.cabac_first_bit_flag = true;
        self.cabac_bits_outstanding = 0;
    }

    #[inline(always)]
    pub fn encode_arithmetic(
        &mut self,
        out_bins: &mut Bins,
        bin: bool,
        ctx: CabacContext,
        ctx_idx: usize,
        bypass_flag: bool,
        init_type: usize,
    ) {
        if bypass_flag {
            self.encode_cabac_bypass(out_bins, bin);
        } else {
            self.encode_cabac_binary_decision(out_bins, bin, ctx, init_type, ctx_idx);
        }
    }

    pub fn encode_cabac_binary_decision(
        &mut self,
        out_bins: &mut Bins,
        bin_val: bool,
        ctx: CabacContext,
        init_type: usize,
        ctx_idx: usize,
    ) {
        let q_range_idx = self.cabac_ivl_curr_range >> 5;
        let p_state = self.cabac_p_state_idx[ctx as usize][init_type][ctx_idx][1]
            + 16 * self.cabac_p_state_idx[ctx as usize][init_type][ctx_idx][0];
        let val_mps = p_state >> 14;
        let bin_val_is_mps = val_mps == (bin_val as u16);
        let lps_range = ((q_range_idx
            * ((if val_mps == 0 {
                p_state
            } else {
                32767 - p_state
            }) >> 9))
            >> 1)
            + 4;
        debug_eprintln!(
            "{} p_state={}, q_range_idx={}, bin={} mps={}, curr_offset={}, curr_range={}, lps_range={}, new_offset={}",
            ctx as usize,
            p_state,
            q_range_idx,
            bin_val,
            val_mps,
            self.cabac_ivl_offset,
            self.cabac_ivl_curr_range,
            lps_range,
            if bin_val_is_mps {self.cabac_ivl_offset} else {self.cabac_ivl_offset+self.cabac_ivl_curr_range-lps_range}
        );
        if bin_val_is_mps {
            self.cabac_ivl_curr_range -= lps_range;
        } else {
            self.cabac_ivl_offset += self.cabac_ivl_curr_range - lps_range;
            self.cabac_ivl_curr_range = lps_range;
        }

        self.renorm_cabac_encode_engine(out_bins);
        self.transition_cabac_state(bin_val, ctx, init_type, ctx_idx);
    }

    #[inline(always)]
    pub fn get_init_type(sh: &SliceHeader) -> usize {
        match sh.slice_type {
            SliceType::I => 0,
            SliceType::P => {
                if sh.cabac_init_flag {
                    2
                } else {
                    1
                }
            }
            SliceType::B => {
                if sh.cabac_init_flag {
                    1
                } else {
                    2
                }
            }
        }
    }

    #[inline(always)]
    pub fn encode_cabac_for_sb_coded_flag(
        &mut self,
        out_bins: &mut Bins,
        sb_coded_flag: bool,
        tu: &TransformUnit,
        c_idx: usize,
        x_s: usize,
        y_s: usize,
        sh: &SliceHeader,
    ) {
        let init_type = Self::get_init_type(sh);
        let (ctx_idx, bypass_flag) =
            self.derive_context_and_bypass_flag_for_sb_coded_flag(tu, c_idx, x_s, y_s, sh);
        self.encode_arithmetic(
            out_bins,
            sb_coded_flag,
            CabacContext::SbCodedFlag,
            ctx_idx,
            bypass_flag,
            init_type,
        );
    }

    #[inline(always)]
    pub fn encode_cabac_for_par_level_flag_and_abs_level_gtx_flag(
        &mut self,
        out_bins: &mut Bins,
        level_flag: bool,
        ctx: CabacContext,
        abs_level_gtx_flag_j: usize,
        tu: &TransformUnit,
        c_idx: usize,
        x_c: usize,
        y_c: usize,
        last_sig_coeff_pos: (usize, usize),
        sh: &SliceHeader,
        ectx: &EncoderContext,
    ) {
        debug_eprintln!("x_c={}, y_c={}", x_c, y_c);
        let init_type = Self::get_init_type(sh);
        let (ctx_idx, bypass_flag) = self
            .derive_context_and_bypass_flag_for_par_level_flag_and_abs_level_gtx_flag(
                ctx,
                tu,
                c_idx,
                x_c,
                y_c,
                abs_level_gtx_flag_j,
                last_sig_coeff_pos,
                sh,
                ectx,
            );
        self.encode_arithmetic(out_bins, level_flag, ctx, ctx_idx, bypass_flag, init_type);
    }

    #[inline(always)]
    pub fn encode_cabac_for_coeff_sign_flag(
        &mut self,
        out_bins: &mut Bins,
        coeff_sign_flag: bool,
        last_scan_pos_pass1: isize,
        coeff_sign_flag_n: usize,
        tu: &TransformUnit,
        c_idx: usize,
        x_c: usize,
        y_c: usize,
        sh: &SliceHeader,
        ectx: &EncoderContext,
    ) {
        let init_type = Self::get_init_type(sh);
        let (ctx_idx, bypass_flag) = self.derive_context_and_bypass_flag_for_coeff_sign_flag(
            tu,
            c_idx,
            x_c,
            y_c,
            last_scan_pos_pass1,
            coeff_sign_flag_n,
            sh,
            ectx,
        );
        self.encode_arithmetic(
            out_bins,
            coeff_sign_flag,
            CabacContext::CoeffSignFlag,
            ctx_idx,
            bypass_flag,
            init_type,
        );
    }

    #[inline(always)]
    pub fn _encode_cabac_for_run_copy_flag(
        &mut self,
        out_bins: &mut Bins,
        run_copy_flag: bool,
        previous_run_type: usize,
        previous_run_position: usize,
        cur_pos: usize,
        sh: &SliceHeader,
    ) {
        let init_type = Self::get_init_type(sh);
        let (ctx_idx, bypass_flag) = self._derive_context_and_bypass_flag_for_run_copy_flag(
            previous_run_type,
            previous_run_position,
            cur_pos,
            sh,
        );
        self.encode_arithmetic(
            out_bins,
            run_copy_flag,
            CabacContext::RunCopyFlag,
            ctx_idx,
            bypass_flag,
            init_type,
        );
    }

    #[inline(always)]
    pub fn binarize_cabac_for_abs_remainder(
        &mut self,
        out_bins: &mut Bins,
        val: usize,
        x_c: usize,
        y_c: usize,
        //first_abs_remainder_in_subblock: bool,
        abs_n: usize,
        tu: &TransformUnit,
        c_idx: usize,
        sh: &SliceHeader,
        ectx: &mut EncoderContext,
    ) {
        self.encode_abs_remainder(
            out_bins,
            val,
            tu,
            c_idx,
            x_c,
            y_c,
            //first_abs_remainder_in_subblock,
            abs_n,
            tu.transform_skip_flag[c_idx],
            sh,
            ectx,
        )
    }

    #[inline(always)]
    pub fn binarize_cabac_for_dec_abs_level(
        &mut self,
        out_bins: &mut Bins,
        val: usize,
        x_c: usize,
        y_c: usize,
        abs_n: usize,
        tu: &TransformUnit,
        c_idx: usize,
        ectx: &mut EncoderContext,
    ) {
        self.encode_dec_abs_level(
            out_bins,
            val,
            abs_n,
            x_c,
            y_c,
            tu.get_log2_tb_size(c_idx).0,
            tu.get_log2_tb_size(c_idx).1,
            ectx,
        )
    }

    #[inline(always)]
    pub fn _binarize_cabac_for_palette_idx_idc(
        &mut self,
        out_bins: &mut Bins,
        val: usize,
        first_palette_idx_idc_in_block: bool,
        ectx: &mut EncoderContext,
    ) {
        self._encode_palette_idx_idc(out_bins, val, first_palette_idx_idc_in_block, ectx)
    }

    #[inline(always)]
    pub fn binarize_cabac_for_last_sig_coeff_x_suffix(
        &mut self,
        out_bins: &mut Bins,
        val: usize,
        last_sig_coeff_x_prefix: usize,
    ) {
        let c_max = (1 << ((last_sig_coeff_x_prefix >> 1) - 1)) - 1;
        self.encode_fixed_length(out_bins, val, c_max)
    }

    #[inline(always)]
    pub fn binarize_cabac_for_last_sig_coeff_y_suffix(
        &mut self,
        out_bins: &mut Bins,
        val: usize,
        last_sig_coeff_y_prefix: usize,
    ) {
        let c_max = (1 << ((last_sig_coeff_y_prefix >> 1) - 1)) - 1;
        self.encode_fixed_length(out_bins, val, c_max)
    }

    pub fn binarize_cabac_ctu(
        &mut self,
        out_bins: &mut Bins,
        val: usize,
        ctx: CabacContext,
        sh: &SliceHeader,
        ectx: &mut EncoderContext,
    ) {
        match ctx {
            CabacContext::AlfCtbCcCbIdc => {
                let c_max = sh.aps[0].alf_data.as_ref().unwrap().cc_cb_filters_signalled;
                let c_rice_param = 0;
                self.encode_trancated_rice(out_bins, val, c_max, c_rice_param)
            }
            CabacContext::AlfCtbCcCrIdc => {
                let c_max = sh.aps[0].alf_data.as_ref().unwrap().cc_cr_filters_signalled;
                let c_rice_param = 0;
                self.encode_trancated_rice(out_bins, val, c_max, c_rice_param)
            }
            CabacContext::AlfLumaPrevFilterIdx => {
                let c_max = sh.alf_info.num_alf_aps_ids_luma - 1;
                self.encode_trancated_binary(out_bins, val, c_max)
            }
            CabacContext::AlfCtbFilterAltIdx => {
                let c_max = sh.aps[0].alf_data.as_ref().unwrap().chroma_num_alt_filters - 1;
                let c_rice_param = 0;
                self.encode_trancated_rice(out_bins, val, c_max, c_rice_param)
            }
            CabacContext::SaoOffsetAbs => {
                let c_max = (1 << (ectx.bit_depth.min(10) - 5)) - 1;
                let c_rice_param = 0;
                self.encode_trancated_rice(out_bins, val, c_max, c_rice_param)
            }
            _ => {
                let bin_process = &ctx_to_bin_process[ctx as usize];
                match bin_process {
                    BinProcess::FL(c_max) => self.encode_fixed_length(out_bins, val, *c_max),
                    BinProcess::TB(c_max) => self.encode_trancated_binary(out_bins, val, *c_max),
                    BinProcess::TR(c_max, c_rice_param) => {
                        self.encode_trancated_rice(out_bins, val, *c_max, *c_rice_param)
                    }
                    BinProcess::EG(k) => self.encode_kth_order_exp_golomb(out_bins, val, *k),
                    _ => panic!(),
                }
            }
        }
    }

    pub fn binarize_cabac_ct(&mut self, out_bins: &mut Bins, val: usize, ctx: CabacContext) {
        let bin_process = &ctx_to_bin_process[ctx as usize];
        match bin_process {
            BinProcess::FL(c_max) => self.encode_fixed_length(out_bins, val, *c_max),
            BinProcess::TB(c_max) => self.encode_trancated_binary(out_bins, val, *c_max),
            BinProcess::TR(c_max, c_rice_param) => {
                self.encode_trancated_rice(out_bins, val, *c_max, *c_rice_param)
            }
            BinProcess::EG(k) => self.encode_kth_order_exp_golomb(out_bins, val, *k),
            _ => panic!(),
        }
    }

    pub fn binarize_cabac_cu(
        &mut self,
        out_bins: &mut Bins,
        val: usize,
        ctx: CabacContext,
        cu: &CodingUnit,
        sh: &SliceHeader,
        ectx: &mut EncoderContext,
    ) {
        match ctx {
            CabacContext::IntraMipMode => {
                let c_max = if cu.width == 4 && cu.height == 4 {
                    15
                } else if cu.width == 4 || cu.height == 4 || (cu.width == 8 && cu.height == 8) {
                    7
                } else {
                    5
                };
                self.encode_trancated_binary(out_bins, val, c_max)
            }
            CabacContext::IntraChromaPredMode => self.encode_intra_chroma_pred_mode(out_bins, val),
            CabacContext::NewPaletteEntries => {
                let c_max = (1 << ectx.bit_depth) - 1;
                self.encode_fixed_length(out_bins, val, c_max)
            }
            CabacContext::InterPredIdc => {
                self.encode_inter_pred_idc(out_bins, val, cu.width, cu.height)
            }
            CabacContext::RefIdxL0 => {
                let c_max = ectx.num_ref_idx_active[0] - 1;
                let c_rice_param = 0;
                self.encode_trancated_rice(out_bins, val, c_max, c_rice_param)
            }
            CabacContext::RefIdxL1 => {
                let c_max = ectx.num_ref_idx_active[1] - 1;
                let c_rice_param = 0;
                self.encode_trancated_rice(out_bins, val, c_max, c_rice_param)
            }
            CabacContext::AmvrPrecisionIdx => {
                let c_max = if !cu.inter_affine_flag
                    && ectx.cu_pred_mode[0][cu.x][cu.y] != ModeType::MODE_IBC
                {
                    2
                } else {
                    1
                };
                let c_rice_param = 0;
                self.encode_trancated_rice(out_bins, val, c_max, c_rice_param)
            }
            CabacContext::BcwIdx => {
                let c_max = if ectx.no_backward_pred_flag { 4 } else { 2 };
                let c_rice_param = 0;
                self.encode_trancated_rice(out_bins, val, c_max, c_rice_param)
            }
            CabacContext::MergeSubblockIdx => {
                let c_max = ectx.max_num_subblock_merge_cand - 1;
                let c_rice_param = 0;
                self.encode_trancated_rice(out_bins, val, c_max, c_rice_param)
            }
            CabacContext::MergeIdx => {
                let c_max = (if ectx.cu_pred_mode[0][cu.x][cu.y] != ModeType::MODE_IBC {
                    ectx.max_num_merge_cand
                } else {
                    ectx.max_num_ibc_merge_cand
                }) - 1;
                let c_rice_param = 0;
                self.encode_trancated_rice(out_bins, val, c_max, c_rice_param)
            }
            CabacContext::MergeGpmIdx0 => {
                let c_max = ectx.max_num_gpm_merge_cand - 1;
                let c_rice_param = 0;
                self.encode_trancated_rice(out_bins, val, c_max, c_rice_param)
            }
            CabacContext::MergeGpmIdx1 => {
                let c_max = ectx.max_num_gpm_merge_cand - 2;
                let c_rice_param = 0;
                self.encode_trancated_rice(out_bins, val, c_max, c_rice_param)
            }
            CabacContext::AbsMvd => self.encode_abs_mvd_minus2(out_bins, val),
            CabacContext::CuQpDeltaAbs => self.encode_cu_qp_delta_abs(out_bins, val),
            CabacContext::CuChromaQpOffsetIdx => {
                let c_max = sh.pps.chroma_tool_offsets.chroma_qp_offset_list_len - 1;
                let c_rice_param = 0;
                self.encode_trancated_rice(out_bins, val, c_max, c_rice_param)
            }
            _ => {
                let bin_process = &ctx_to_bin_process[ctx as usize];
                match bin_process {
                    BinProcess::FL(c_max) => self.encode_fixed_length(out_bins, val, *c_max),
                    BinProcess::TB(c_max) => self.encode_trancated_binary(out_bins, val, *c_max),
                    BinProcess::TR(c_max, c_rice_param) => {
                        self.encode_trancated_rice(out_bins, val, *c_max, *c_rice_param)
                    }
                    BinProcess::EG(k) => self.encode_kth_order_exp_golomb(out_bins, val, *k),
                    _ => panic!(),
                }
            }
        }
    }

    pub fn binarize_cabac_tu(
        &mut self,
        out_bins: &mut Bins,
        val: usize,
        ctx: CabacContext,
        sh: &SliceHeader,
    ) {
        match ctx {
            CabacContext::CuQpDeltaAbs => self.encode_cu_qp_delta_abs(out_bins, val),
            CabacContext::CuChromaQpOffsetIdx => {
                let c_max = sh.pps.chroma_tool_offsets.chroma_qp_offset_list_len - 1;
                let c_rice_param = 0;
                self.encode_trancated_rice(out_bins, val, c_max, c_rice_param)
            }
            _ => {
                let bin_process = &ctx_to_bin_process[ctx as usize];
                match bin_process {
                    BinProcess::FL(c_max) => self.encode_fixed_length(out_bins, val, *c_max),
                    BinProcess::TB(c_max) => self.encode_trancated_binary(out_bins, val, *c_max),
                    BinProcess::TR(c_max, c_rice_param) => {
                        self.encode_trancated_rice(out_bins, val, *c_max, *c_rice_param)
                    }
                    BinProcess::EG(k) => self.encode_kth_order_exp_golomb(out_bins, val, *k),
                    _ => panic!(),
                }
            }
        }
    }

    pub fn binarize_cabac_last_sig_coeff_x_prefix(
        &mut self,
        out_bins: &mut Bins,
        val: usize,
        tu: &TransformUnit,
        c_idx: usize,
        sh: &SliceHeader,
    ) {
        let log2_zo_tb_width = tu.get_log2_zo_tb_size(sh.sps, c_idx).0;
        let c_max = (log2_zo_tb_width << 1) - 1;
        let c_rice_param = 0;
        self.encode_trancated_rice(out_bins, val, c_max, c_rice_param)
    }

    pub fn binarize_cabac_last_sig_coeff_y_prefix(
        &mut self,
        out_bins: &mut Bins,
        val: usize,
        tu: &TransformUnit,
        c_idx: usize,
        sh: &SliceHeader,
    ) {
        let log2_zo_tb_height = tu.get_log2_zo_tb_size(sh.sps, c_idx).1;
        let c_max = (log2_zo_tb_height << 1) - 1;
        let c_rice_param = 0;
        self.encode_trancated_rice(out_bins, val, c_max, c_rice_param)
    }

    //pub fn binarize_cabac_residual(&mut self, out_bins: &mut Bins, val: usize, ctx: CabacContext) {
    //let bin_process = &ctx_to_bin_process[ctx as usize];
    //match bin_process {
    //BinProcess::FL(c_max) => self.encode_fixed_length(out_bins, val, *c_max),
    //BinProcess::TB(c_max) => self.encode_trancated_binary(out_bins, val, *c_max),
    //BinProcess::TR(c_max, c_rice_param) => {
    //self.encode_trancated_rice(out_bins, val, *c_max, *c_rice_param)
    //}
    //BinProcess::EG(k) => self.encode_kth_order_exp_golomb(out_bins, val, *k),
    //_ => panic!(),
    //}
    //}

    #[inline(always)]
    pub fn binarize_cabac_end_one_bit(&self, out_bins: &mut Bins) {
        self.encode_fixed_length(out_bins, 1, 1);
    }

    #[inline(always)]
    pub fn binarize_cabac_sig_coeff_flag(&mut self, out_bins: &mut Bins, val: usize) {
        self.encode_fixed_length(out_bins, val, 1);
    }

    pub fn _binarize_cabac(&mut self, out_bins: &mut Bins, val: usize, ctx: CabacContext) {
        let bin_process = &ctx_to_bin_process[ctx as usize];
        match bin_process {
            BinProcess::FL(c_max) => self.encode_fixed_length(out_bins, val, *c_max),
            BinProcess::TB(c_max) => self.encode_trancated_binary(out_bins, val, *c_max),
            BinProcess::TR(c_max, c_rice_param) => {
                self.encode_trancated_rice(out_bins, val, *c_max, *c_rice_param)
            }
            BinProcess::EG(k) => self.encode_kth_order_exp_golomb(out_bins, val, *k),
            _ => panic!(),
        }
    }

    #[inline(always)]
    pub fn encode_cabac_ctu(
        &mut self,
        out_bins: &mut Bins,
        val: usize,
        ctx: CabacContext,
        sh: &SliceHeader,
        ectx: &mut EncoderContext,
    ) {
        debug_eprintln!("ec {}", ctx as usize);
        let init_type = Self::get_init_type(sh);
        let mut bins = Bins::new();
        self.binarize_cabac_ctu(&mut bins, val, ctx, sh, ectx);
        for (bin_idx, bin) in bins.into_iter().enumerate() {
            debug_eprintln!("ecbin {:?}", bin);
            let (ctx_idx, bypass_flag) = self.derive_context_and_bypass_flag_ctu(bin_idx, ctx, sh);
            self.encode_arithmetic(out_bins, bin, ctx, ctx_idx, bypass_flag, init_type);
        }
    }

    #[inline(always)]
    pub fn encode_cabac_ct(
        &mut self,
        out_bins: &mut Bins,
        val: usize,
        ctx: CabacContext,
        ct: &CodingTree,
        sh: &SliceHeader,
        ectx: &mut EncoderContext,
    ) {
        let init_type = Self::get_init_type(sh);
        let mut bins = Bins::new();
        self.binarize_cabac_ct(&mut bins, val, ctx);
        for (bin_idx, bin) in bins.into_iter().enumerate() {
            debug_eprintln!("ecbin {:?}", bin);
            let (ctx_idx, bypass_flag) =
                self.derive_context_and_bypass_flag_ct(bin_idx, ctx, ct, sh, ectx);
            self.encode_arithmetic(out_bins, bin, ctx, ctx_idx, bypass_flag, init_type);
        }
    }

    #[inline(always)]
    pub fn encode_cabac_cu(
        &mut self,
        out_bins: &mut Bins,
        val: usize,
        ctx: CabacContext,
        cu: &CodingUnit,
        sh: &SliceHeader,
        ectx: &mut EncoderContext,
    ) {
        let init_type = Self::get_init_type(sh);
        let mut bins = Bins::new();
        self.binarize_cabac_cu(&mut bins, val, ctx, cu, sh, ectx);
        let ct = &cu.parent.lock().unwrap();
        for (bin_idx, bin) in bins.into_iter().enumerate() {
            debug_eprintln!("ecbin {:?}", bin);
            let (ctx_idx, bypass_flag) =
                self.derive_context_and_bypass_flag_cu(bin_idx, ctx, ct, cu, sh, ectx);
            self.encode_arithmetic(out_bins, bin, ctx, ctx_idx, bypass_flag, init_type);
        }
    }

    #[inline(always)]
    pub fn encode_cabac_tu(
        &mut self,
        out_bins: &mut Bins,
        val: usize,
        c_idx: usize,
        ctx: CabacContext,
        tu: &TransformUnit,
        sh: &SliceHeader,
        ectx: &mut EncoderContext,
    ) {
        let init_type = Self::get_init_type(sh);
        let mut bins = Bins::new();
        self.binarize_cabac_tu(&mut bins, val, ctx, sh);
        for (bin_idx, bin) in bins.into_iter().enumerate() {
            debug_eprintln!("ecbin {:?}", bin);
            let (ctx_idx, bypass_flag) =
                self.derive_context_and_bypass_flag_tu(bin_idx, c_idx, ctx, tu, sh, ectx);
            self.encode_arithmetic(out_bins, bin, ctx, ctx_idx, bypass_flag, init_type);
        }
    }

    //#[inline(always)]
    //pub fn encode_cabac_residual(
    //&mut self,
    //out_bins: &mut Bins,
    //val: usize,
    //ctx: CabacContext,
    //tu: &TransformUnit,
    //c_idx: usize,
    //sh: &SliceHeader,
    //) {
    //let init_type = Self::get_init_type(sh);
    //let mut bins = Bins::new();
    //self.binarize_cabac_residual(&mut bins, val, ctx);
    //let mut bin_idx = 0;
    //for bin in bins.into_iter() {
    //debug_eprintln!("ecbin {:?}", bin);
    //let (ctx_idx, bypass_flag) =
    //self.derive_context_and_bypass_flag_residual(bin_idx, c_idx, ctx, tu, sh);
    //self.encode_arithmetic(out_bins, bin, ctx, ctx_idx, bypass_flag, init_type);
    //bin_idx += 1;
    //}
    //}

    #[inline(always)]
    pub fn encode_cabac_last_sig_coeff_x_prefix(
        &mut self,
        out_bins: &mut Bins,
        val: usize,
        tu: &TransformUnit,
        c_idx: usize,
        sh: &SliceHeader,
    ) {
        let init_type = Self::get_init_type(sh);
        let mut bins = Bins::new();
        self.binarize_cabac_last_sig_coeff_x_prefix(&mut bins, val, tu, c_idx, sh);
        for (bin_idx, bin) in bins.into_iter().enumerate() {
            debug_eprintln!("ecbin {:?}", bin);
            let (ctx_idx, bypass_flag) =
                self.derive_context_and_bypass_flag_last_sig_coeff_x_prefix(bin_idx, c_idx, tu, sh);
            self.encode_arithmetic(
                out_bins,
                bin,
                CabacContext::LastSigCoeffXPrefix,
                ctx_idx,
                bypass_flag,
                init_type,
            );
        }
    }

    #[inline(always)]
    pub fn encode_cabac_last_sig_coeff_y_prefix(
        &mut self,
        out_bins: &mut Bins,
        val: usize,
        tu: &TransformUnit,
        c_idx: usize,
        sh: &SliceHeader,
    ) {
        let init_type = Self::get_init_type(sh);
        let mut bins = Bins::new();
        self.binarize_cabac_last_sig_coeff_y_prefix(&mut bins, val, tu, c_idx, sh);
        for (bin_idx, bin) in bins.into_iter().enumerate() {
            debug_eprintln!("ecbin {:?}", bin);
            let (ctx_idx, bypass_flag) =
                self.derive_context_and_bypass_flag_last_sig_coeff_y_prefix(bin_idx, c_idx, tu, sh);
            self.encode_arithmetic(
                out_bins,
                bin,
                CabacContext::LastSigCoeffYPrefix,
                ctx_idx,
                bypass_flag,
                init_type,
            );
        }
    }

    #[inline(always)]
    pub fn encode_cabac_for_sig_coeff_flag(
        &mut self,
        out_bins: &mut Bins,
        val: usize,
        x_c: usize,
        y_c: usize,
        ctx: CabacContext,
        tu: &TransformUnit,
        c_idx: usize,
        sh: &SliceHeader,
        ectx: &mut EncoderContext,
    ) {
        debug_eprintln!("ec {}", ctx as usize);
        let init_type = Self::get_init_type(sh);
        let mut bins = Bins::new();
        self.binarize_cabac_sig_coeff_flag(&mut bins, val);
        //let mut bin_idx = 0;
        for bin in bins.into_iter() {
            debug_eprintln!("ecbin {:?}", bin);
            let (ctx_idx, bypass_flag) = self.derive_context_and_bypass_flag_for_sig_coeff_flag(
                x_c, y_c, c_idx, ctx, tu, sh, ectx,
            );
            self.encode_arithmetic(out_bins, bin, ctx, ctx_idx, bypass_flag, init_type);
            //bin_idx += 1;
        }
    }

    pub fn encode_cabac_for_abs_remainder(
        &mut self,
        out_bins: &mut Bins,
        val: usize,
        x_c: usize,
        y_c: usize,
        //first_abs_remainder_in_subblock: bool,
        abs_n: usize,
        ctx: CabacContext,
        tu: &TransformUnit,
        c_idx: usize,
        sh: &SliceHeader,
        ectx: &mut EncoderContext,
    ) {
        debug_eprintln!("ec {}", ctx as usize);
        let init_type = Self::get_init_type(sh);
        let mut bins = Bins::new();
        self.binarize_cabac_for_abs_remainder(
            &mut bins, val, x_c, y_c, //first_abs_remainder_in_subblock,
            abs_n, tu, c_idx, sh, ectx,
        );
        for (bin_idx, bin) in bins.into_iter().enumerate() {
            debug_eprintln!("ecbin {:?}", bin);
            let (ctx_idx, bypass_flag) = self.derive_context_and_bypass_flag(bin_idx, ctx, sh);
            self.encode_arithmetic(out_bins, bin, ctx, ctx_idx, bypass_flag, init_type);
        }
    }

    pub fn encode_cabac_for_dec_abs_level(
        &mut self,
        out_bins: &mut Bins,
        val: usize,
        x_c: usize,
        y_c: usize,
        abs_n: usize,
        ctx: CabacContext,
        tu: &TransformUnit,
        c_idx: usize,
        sh: &SliceHeader,
        ectx: &mut EncoderContext,
    ) {
        debug_eprintln!("ec {}", ctx as usize);
        let init_type = Self::get_init_type(sh);
        let mut bins = Bins::new();
        self.binarize_cabac_for_dec_abs_level(&mut bins, val, x_c, y_c, abs_n, tu, c_idx, ectx);
        for (bin_idx, bin) in bins.into_iter().enumerate() {
            debug_eprintln!("ecbin {:?}", bin);
            let (ctx_idx, bypass_flag) = self.derive_context_and_bypass_flag(bin_idx, ctx, sh);
            self.encode_arithmetic(out_bins, bin, ctx, ctx_idx, bypass_flag, init_type);
        }
    }

    //#[inline(always)]
    //pub fn binarize_cabac_for_last_sig_coeff_x_suffix(
    //&mut self,
    //out_bins: &mut Bins,
    //val: usize,
    //last_sig_coeff_x_prefix: usize,
    //ectx: &mut EncoderContext,
    //) {
    //let c_max = (1 << ((last_sig_coeff_x_prefix >> 1) - 1)) - 1;
    //self.encode_fixed_length(out_bins, val, c_max)
    //}

    //#[inline(always)]
    //pub fn binarize_cabac_for_last_sig_coeff_y_suffix(

    pub fn _encode_cabac_for_palette_idx_idc(
        &mut self,
        out_bins: &mut Bins,
        val: usize,
        first_palette_idx_idc_in_block: bool,
        ctx: CabacContext,
        sh: &SliceHeader,
        ectx: &mut EncoderContext,
    ) {
        debug_eprintln!("ec {}", ctx as usize);
        let init_type = Self::get_init_type(sh);
        let mut bins = Bins::new();
        self._binarize_cabac_for_palette_idx_idc(
            &mut bins,
            val,
            first_palette_idx_idc_in_block,
            ectx,
        );
        for (bin_idx, bin) in bins.into_iter().enumerate() {
            debug_eprintln!("ecbin {:?}", bin);
            let (ctx_idx, bypass_flag) = self.derive_context_and_bypass_flag(bin_idx, ctx, sh);
            self.encode_arithmetic(out_bins, bin, ctx, ctx_idx, bypass_flag, init_type);
        }
    }

    pub fn encode_cabac_for_last_sig_coeff_x_suffix(
        &mut self,
        out_bins: &mut Bins,
        val: usize,
        last_sig_coeff_x_prefix: usize,
        ctx: CabacContext,
        sh: &SliceHeader,
    ) {
        debug_eprintln!("ec {}", ctx as usize);
        let init_type = Self::get_init_type(sh);
        let mut bins = Bins::new();
        self.binarize_cabac_for_last_sig_coeff_x_suffix(&mut bins, val, last_sig_coeff_x_prefix);
        for (bin_idx, bin) in bins.into_iter().enumerate() {
            debug_eprintln!("ecbin {:?}", bin);
            let (ctx_idx, bypass_flag) = self.derive_context_and_bypass_flag(bin_idx, ctx, sh);
            self.encode_arithmetic(out_bins, bin, ctx, ctx_idx, bypass_flag, init_type);
        }
    }

    pub fn encode_cabac_for_last_sig_coeff_y_suffix(
        &mut self,
        out_bins: &mut Bins,
        val: usize,
        last_sig_coeff_y_prefix: usize,
        ctx: CabacContext,
        sh: &SliceHeader,
    ) {
        debug_eprintln!("ec {}", ctx as usize);
        let init_type = Self::get_init_type(sh);
        let mut bins = Bins::new();
        self.binarize_cabac_for_last_sig_coeff_y_suffix(&mut bins, val, last_sig_coeff_y_prefix);
        for (bin_idx, bin) in bins.into_iter().enumerate() {
            debug_eprintln!("ecbin {:?}", bin);
            let (ctx_idx, bypass_flag) = self.derive_context_and_bypass_flag(bin_idx, ctx, sh);
            self.encode_arithmetic(out_bins, bin, ctx, ctx_idx, bypass_flag, init_type);
        }
    }

    pub fn encode_cabac_end_one_bit(&mut self, out_bins: &mut Bins) {
        //debug_eprintln!("ec {}", ctx as usize);
        let mut bins = Bins::new();
        self.binarize_cabac_end_one_bit(&mut bins);
        for bin in bins.into_iter() {
            debug_eprintln!("ecbin {:?}", bin);
            self.encode_arithmetic_stop_one_bit(out_bins, bin);
        }
    }

    pub fn _encode_cabac(
        &mut self,
        out_bins: &mut Bins,
        val: usize,
        ctx: CabacContext,
        sh: &SliceHeader,
    ) {
        debug_eprintln!("ec {}", ctx as usize);
        let init_type = Self::get_init_type(sh);
        let mut bins = Bins::new();
        self._binarize_cabac(&mut bins, val, ctx);
        for (bin_idx, bin) in bins.into_iter().enumerate() {
            debug_eprintln!("ecbin {:?}", bin);
            let (ctx_idx, bypass_flag) = self.derive_context_and_bypass_flag(bin_idx, ctx, sh);
            self.encode_arithmetic(out_bins, bin, ctx, ctx_idx, bypass_flag, init_type);
        }
    }

    pub fn init_ctx_table(&mut self, ectx: &EncoderContext) {
        for ctx in 0..ctx_table.len() {
            if ctx_table[ctx].is_empty() {
                continue;
            }
            for init_type in 0..3 {
                for ctx_idx in 0..ctx_table[ctx][0][init_type].len() {
                    let init_value = ctx_table[ctx][0][init_type][ctx_idx];
                    let slope_idx = init_value >> 3;
                    let offset_idx = init_value & 7;
                    let m = slope_idx as isize - 4;
                    let n = offset_idx * 18 + 1;
                    let pre_ctx_state = (((m * (ectx.slice_qp_y.clamp(0, 63) - 16)) >> 1)
                        + n as isize)
                        .clamp(1, 127) as u16;
                    self.cabac_p_state_idx[ctx][init_type][ctx_idx][0] = pre_ctx_state << 3;
                    self.cabac_p_state_idx[ctx][init_type][ctx_idx][1] = pre_ctx_state << 7;
                }
            }
        }
    }

    #[inline(always)]
    pub fn storage_ctx_table(&mut self) {
        self.cabac_table_state_sync = self.cabac_p_state_idx.clone();
    }

    #[inline(always)]
    pub fn sync_ctx_table(&mut self) {
        self.cabac_p_state_idx = self.cabac_table_state_sync.clone();
    }

    #[inline(always)]
    pub fn init_arithmetic_engine(&mut self) {
        self.cabac_ivl_curr_range = 510;
        self.cabac_ivl_offset = 0;
        assert_ne!(self.cabac_ivl_offset, 510);
        assert_ne!(self.cabac_ivl_offset, 511);
    }

    pub fn _storage_palette_predictor(&mut self, ectx: &mut EncoderContext) {
        for c_idx in 0..3 {
            let ch_type = (c_idx != 0) as usize;
            ectx.table_palette_size_wpp[ch_type] = ectx.predictor_palette_size[ch_type];
            for i in 0..ectx.predictor_palette_size[ch_type] {
                ectx.table_palette_entries_wpp[c_idx][i] = ectx.predictor_palette_entries[c_idx][i];
            }
        }
    }

    pub fn sync_palette_predictor(&mut self, ectx: &mut EncoderContext) {
        for c_idx in 0..3 {
            let ch_type = (c_idx != 0) as usize;
            ectx.predictor_palette_size[ch_type] = ectx.table_palette_size_wpp[ch_type];
            for i in 0..ectx.predictor_palette_size[ch_type] {
                ectx.predictor_palette_entries[c_idx][i] = ectx.table_palette_entries_wpp[c_idx][i];
            }
        }
    }

    pub fn derive_rice_parameter(
        &mut self,
        base_level: usize,
        x_c: usize,
        y_c: usize,
        log2_tb_width: usize,
        log2_tb_height: usize,
        n: usize,
        ectx: &mut EncoderContext,
    ) -> usize {
        let mut loc_sum_abs = 0;
        let abs_level = &ectx.abs_level.data[y_c << ectx.abs_level.log2_stride..];
        if x_c < (1 << log2_tb_width) - 1 {
            //loc_sum_abs += tu.residuals[c_idx][y_c][x_c + 1].abs();
            loc_sum_abs += abs_level[x_c + 1];
            if x_c < (1 << log2_tb_width) - 2 {
                //loc_sum_abs += tu.residuals[c_idx][y_c][x_c + 2].abs();
                loc_sum_abs += abs_level[x_c + 2];
            }
            if y_c < (1 << log2_tb_height) - 1 {
                let abs_level = &abs_level[1 << ectx.abs_level.log2_stride..];
                //loc_sum_abs += tu.residuals[c_idx][y_c + 1][x_c + 1].abs();
                loc_sum_abs += abs_level[x_c + 1];
            }
        }
        if y_c < (1 << log2_tb_height) - 1 {
            let abs_level = &abs_level[1 << ectx.abs_level.log2_stride..];
            //loc_sum_abs += tu.residuals[c_idx][y_c + 1][x_c].abs();
            loc_sum_abs += abs_level[x_c];
            if y_c < (1 << log2_tb_height) - 2 {
                let abs_level = &abs_level[1 << ectx.abs_level.log2_stride..];
                //loc_sum_abs += tu.residuals[c_idx][y_c + 2][x_c].abs();
                loc_sum_abs += abs_level[x_c];
            }
        }
        loc_sum_abs = (loc_sum_abs as isize - base_level as isize * 5).clamp(0, 31) as usize;
        let c_rice_param = c_rice_params[loc_sum_abs as usize];
        if base_level == 0 {
            ectx.zero_pos[n] = (if ectx.q_state < 2 { 1 } else { 2 }) << c_rice_param;
        }
        c_rice_param
    }

    pub fn encode_trancated_rice(
        &self,
        out_bins: &mut Bins,
        symbol_val: usize,
        c_max: usize,
        c_rice_param: usize,
    ) {
        let prefix_val = symbol_val >> c_rice_param;
        if prefix_val < c_max >> c_rice_param {
            //for _ in 0..prefix_val {
            //out_bins.push_bin(true);
            //}
            out_bins.push_bins_with_size(((1 << prefix_val) - 1) << 1, prefix_val + 1);
            //out_bins.push_bin(false);
        } else {
            let n = c_max >> c_rice_param;
            out_bins.push_bins_with_size((1 << n) - 1, n);
            //for _ in 0..(c_max >> c_rice_param) {
            //out_bins.push_bin(true);
            //}
        }
        if c_max > symbol_val && c_rice_param > 0 {
            let suffix_val = symbol_val - (prefix_val << c_rice_param);
            self.encode_fixed_length(out_bins, suffix_val, (1 << c_rice_param) - 1);
        }
    }

    pub fn encode_trancated_rice_to_vec(
        &self,
        symbol_val: usize,
        c_max: usize,
        c_rice_param: usize,
    ) -> Vec<bool> {
        let prefix_val = symbol_val >> c_rice_param;
        let mut out_bins = if prefix_val < c_max >> c_rice_param {
            let mut out_bins = vec![true; prefix_val];
            out_bins.push(false);
            out_bins
        } else {
            vec![true; c_max >> c_rice_param]
        };
        if c_max > symbol_val && c_rice_param > 0 {
            let suffix_val = symbol_val - (prefix_val << c_rice_param);
            self.encode_fixed_length_to_vec(&mut out_bins, suffix_val, (1 << c_rice_param) - 1);
        }
        out_bins
    }

    #[inline(always)]
    pub fn encode_fixed_length(&self, out_bins: &mut Bins, symbol_val: usize, c_max: usize) {
        //let fixed_length = ((c_max + 1) as f64).ilog2().ceil() as usize;
        let fixed_length = c_max.ilog2() + 1;
        out_bins.push_bins_with_size(symbol_val as u64, fixed_length as usize);
    }

    #[inline(always)]
    pub fn encode_fixed_length_to_vec(
        &self,
        out_bins: &mut Vec<bool>,
        symbol_val: usize,
        c_max: usize,
    ) {
        //let fixed_length = ((c_max + 1) as f64).ilog2().ceil() as usize;
        let fixed_length = c_max.ilog2() + 1;
        for i in (0..fixed_length).rev() {
            out_bins.push((symbol_val >> i) & 1 == 1);
        }
    }

    #[inline(always)]
    pub fn encode_trancated_binary(&self, out_bins: &mut Bins, syn_val: usize, c_max: usize) {
        let n = c_max + 1;
        let k = n.ilog2();
        let u = (1 << (k + 1)) - n;
        if syn_val < u {
            self.encode_fixed_length(out_bins, syn_val, (1 << k) - 1);
        } else {
            self.encode_fixed_length(out_bins, syn_val + u, (1 << (k + 1)) - 1);
        }
    }

    pub fn encode_kth_order_exp_golomb(&self, out_bins: &mut Bins, symbol_val: usize, k: usize) {
        let mut abs_v = symbol_val;
        let mut stop_loop = false;
        let mut k = k;
        while {
            if abs_v >= 1 << k {
                out_bins.push_bin(true);
                abs_v -= 1 << k;
                k += 1;
            } else {
                out_bins.push_bin(false);
                while k > 0 {
                    k -= 1;
                    out_bins.push_bin((abs_v >> k) & 1 == 1);
                }
                stop_loop = true;
            }
            !stop_loop
        } {}
    }

    pub fn encode_limited_kth_order_exp_golomb(
        &self,
        out_bins: &mut Bins,
        symbol_val: usize,
        k: usize,
        max_pre_ext_len: usize,
        trunc_suffix_len: usize,
    ) {
        let code_value = symbol_val >> k;
        let mut pre_ext_len = 0;
        while pre_ext_len < max_pre_ext_len && code_value > (2 << pre_ext_len) - 2 {
            pre_ext_len += 1;
            out_bins.push_bin(true);
        }
        let mut escape_length = if pre_ext_len == max_pre_ext_len {
            trunc_suffix_len
        } else {
            out_bins.push_bin(false);
            pre_ext_len + k
        };
        let symbol_val = symbol_val - (((1 << pre_ext_len) - 1) << k);
        while escape_length > 0 {
            escape_length -= 1;
            out_bins.push_bin((symbol_val >> escape_length) & 1 == 1);
        }
    }

    pub fn encode_limited_kth_order_exp_golomb_to_vec(
        &self,
        symbol_val: usize,
        k: usize,
        max_pre_ext_len: usize,
        trunc_suffix_len: usize,
    ) -> Vec<bool> {
        let mut out_bins = vec![];
        let code_value = symbol_val >> k;
        let mut pre_ext_len = 0;
        while pre_ext_len < max_pre_ext_len && code_value > (2 << pre_ext_len) - 2 {
            pre_ext_len += 1;
            out_bins.push(true);
        }
        let mut escape_length = if pre_ext_len == max_pre_ext_len {
            trunc_suffix_len
        } else {
            out_bins.push(false);
            pre_ext_len + k
        };
        let symbol_val = symbol_val - (((1 << pre_ext_len) - 1) << k);
        while escape_length > 0 {
            escape_length -= 1;
            out_bins.push((symbol_val >> escape_length) & 1 == 1);
        }
        out_bins
    }

    pub fn encode_intra_chroma_pred_mode(
        &self,
        out_bins: &mut Bins,
        intra_chroma_pred_mode: usize,
    ) {
        match intra_chroma_pred_mode {
            0 => out_bins.push_bins_with_size(0b100, 3),
            1 => out_bins.push_bins_with_size(0b101, 3),
            2 => out_bins.push_bins_with_size(0b110, 3),
            3 => out_bins.push_bins_with_size(0b111, 3),
            4 => out_bins.push_bin(false),
            _ => panic!(),
        }
    }

    pub fn encode_inter_pred_idc(
        &self,
        out_bins: &mut Bins,
        inter_pred_idc: usize,
        cb_width: usize,
        cb_height: usize,
    ) {
        match inter_pred_idc {
            0 => {
                if cb_width + cb_height > 12 {
                    out_bins.push_bins_with_size(0b00, 2)
                } else {
                    out_bins.push_bin(false)
                }
            }
            1 => {
                if cb_width + cb_height > 12 {
                    out_bins.push_bins_with_size(0b01, 2)
                } else {
                    out_bins.push_bin(true)
                }
            }
            2 => out_bins.push_bin(true),
            _ => panic!(),
        }
    }

    pub fn encode_cu_qp_delta_abs(&self, out_bins: &mut Bins, cu_qp_delta_abs: usize) {
        let prefix_val = cu_qp_delta_abs.min(5);
        self.encode_trancated_rice(out_bins, prefix_val, 5, 0);
        if prefix_val > 4 {
            let suffiv_val = cu_qp_delta_abs - 5;
            self.encode_kth_order_exp_golomb(out_bins, suffiv_val, 0);
        }
    }

    pub fn encode_abs_remainder(
        &mut self,
        out_bins: &mut Bins,
        abs_remainder: usize,
        tu: &TransformUnit,
        c_idx: usize,
        x_c: usize,
        y_c: usize,
        //first: bool,
        n: usize,
        transform_skip_flag: bool,
        sh: &SliceHeader,
        ectx: &mut EncoderContext,
    ) {
        // FIXME ? derived, but not used in spec
        //let [_last_abs_remainder, _last_rice_param] = if first {
        //[0, 0]
        //} else {
        //[abs_remainder, abs_remainder]
        //};
        let c_rice_param = if transform_skip_flag && !sh.ts_residual_coding_disabled_flag {
            1
        } else {
            self.derive_rice_parameter(
                4,
                x_c,
                y_c,
                tu.get_log2_tb_size(c_idx).0,
                tu.get_log2_tb_size(c_idx).1,
                n,
                ectx,
            )
        };
        let c_max = 6 << c_rice_param;
        let prefix_val = c_max.min(abs_remainder);
        let mut prefix_bins = self.encode_trancated_rice_to_vec(prefix_val, c_max, c_rice_param);
        if prefix_bins.len() == 6 && prefix_bins.iter().all(|x| *x) {
            let suffix_val = abs_remainder - c_max;
            let suffix_bins = self.encode_limited_kth_order_exp_golomb_to_vec(
                suffix_val,
                c_rice_param + 1,
                11,
                15,
            );
            prefix_bins.extend(suffix_bins);
        }
        debug_eprintln!("abs rem bins = {:?}", prefix_bins);
        for bin in prefix_bins.into_iter() {
            out_bins.push_bin(bin);
        }
    }

    pub fn encode_dec_abs_level(
        &mut self,
        out_bins: &mut Bins,
        dec_abs_level: usize,
        n: usize,
        x_c: usize,
        y_c: usize,
        log2_tb_width: usize,
        log2_tb_height: usize,
        ectx: &mut EncoderContext,
    ) {
        let c_rice_param =
            self.derive_rice_parameter(0, x_c, y_c, log2_tb_width, log2_tb_height, n, ectx);
        let c_max = 6 << c_rice_param;
        let prefix_val = c_max.min(dec_abs_level);
        let mut prefix_bins = self.encode_trancated_rice_to_vec(prefix_val, c_max, c_rice_param);
        if prefix_bins.len() == 6 && prefix_bins.iter().all(|x| *x) {
            let suffix_val = dec_abs_level - c_max;
            let suffix_bins = self.encode_limited_kth_order_exp_golomb_to_vec(
                suffix_val,
                c_rice_param + 1,
                11,
                15,
            );
            prefix_bins.extend(suffix_bins);
        }
        for bin in prefix_bins.iter() {
            out_bins.push_bin(*bin);
        }
    }

    pub fn _encode_palette_idx_idc(
        &self,
        out_bins: &mut Bins,
        palette_idx_idc: usize,
        first: bool,
        ectx: &EncoderContext,
    ) {
        let c_max = if first {
            ectx.max_palette_index
        } else {
            ectx.max_palette_index - 1
        };
        self.encode_trancated_binary(out_bins, palette_idx_idc, c_max);
    }

    pub fn encode_abs_mvd_minus2(&mut self, out_bins: &mut Bins, abs_mvd: usize) {
        self.encode_limited_kth_order_exp_golomb(out_bins, abs_mvd - 2, 1, 15, 17);
    }

    pub fn derive_context_and_bypass_flag_for_sb_coded_flag(
        &self,
        tu: &TransformUnit,
        c_idx: usize,
        x_s: usize,
        y_s: usize,
        sh: &SliceHeader,
    ) -> (usize, bool) {
        let (log2_tb_width, log2_tb_height) = tu.get_log2_tb_size(c_idx);
        let ctx_inc = self.derive_ctx_inc_for_sb_coded_flag(
            x_s,
            y_s,
            log2_tb_width,
            log2_tb_height,
            c_idx,
            tu,
            sh,
        );
        let (ctx_idx, bypass_flag) = match ctx_inc {
            CtxInc::Number(ctx_inc) => {
                //let init_type = Self::get_init_type(sh);
                //let ctx_idx_offset =
                //ctx_table[CabacContext::SbCodedFlag as usize][0][0].len() * init_type;
                //let ctx_idx = ctx_idx_offset + ctx_inc;
                let ctx_idx = ctx_inc;
                (ctx_idx, false)
            }
            CtxInc::Bypass => (0, true),
            CtxInc::Terminate => (0, false),
            _ => panic!(),
        };
        (ctx_idx, bypass_flag)
    }

    pub fn derive_context_and_bypass_flag_for_par_level_flag_and_abs_level_gtx_flag(
        &self,
        ctx: CabacContext,
        tu: &TransformUnit,
        c_idx: usize,
        x_c: usize,
        y_c: usize,
        j: usize,
        last_sig_coeff_pos: (usize, usize),
        sh: &SliceHeader,
        ectx: &EncoderContext,
    ) -> (usize, bool) {
        let (log2_tb_width, log2_tb_height) = tu.get_log2_zo_tb_size(sh.sps, c_idx);
        let ctx_inc = self.derive_ctx_inc_for_par_level_flag_and_abs_level_gtx_flag(
            ctx,
            x_c,
            y_c,
            log2_tb_width,
            log2_tb_height,
            tu,
            c_idx,
            j,
            last_sig_coeff_pos,
            sh,
            ectx,
        );
        let (ctx_idx, bypass_flag) = match ctx_inc {
            CtxInc::Number(ctx_inc) => {
                let init_type = Self::get_init_type(sh);
                let ctx_idx_offset = ctx_table[ctx as usize][0][0].len() * init_type;
                let ctx_idx = ctx_idx_offset + ctx_inc;
                (ctx_idx, false)
            }
            CtxInc::Bypass => (0, true),
            CtxInc::Terminate => (0, false),
            _ => panic!(),
        };
        (ctx_idx, bypass_flag)
    }

    pub fn _derive_context_and_bypass_flag_for_run_copy_flag(
        &self,
        previous_run_type: usize,
        previous_run_position: usize,
        cur_pos: usize,
        sh: &SliceHeader,
    ) -> (usize, bool) {
        let ctx_inc = self._derive_ctx_inc_for_run_copy_flag(
            previous_run_type,
            previous_run_position,
            cur_pos,
        );
        let (ctx_idx, bypass_flag) = match ctx_inc {
            CtxInc::Number(ctx_inc) => {
                let init_type = Self::get_init_type(sh);
                let ctx_idx_offset =
                    ctx_table[CabacContext::RunCopyFlag as usize][0][0].len() * init_type;
                let ctx_idx = ctx_idx_offset + ctx_inc;
                (ctx_idx, false)
            }
            CtxInc::Bypass => (0, true),
            CtxInc::Terminate => (0, false),
            _ => panic!(),
        };
        (ctx_idx, bypass_flag)
    }

    pub fn derive_context_and_bypass_flag_for_coeff_sign_flag(
        &self,
        tu: &TransformUnit,
        c_idx: usize,
        x_c: usize,
        y_c: usize,
        last_scan_pos_pass1: isize,
        coeff_sign_flag_n: usize,
        sh: &SliceHeader,
        ectx: &EncoderContext,
    ) -> (usize, bool) {
        let (x_c, y_c, transform_skip_flag) = (x_c, y_c, tu.transform_skip_flag[c_idx]);
        let ctx_inc = self.derive_ctx_inc_for_coeff_sign_flag(
            x_c,
            y_c,
            c_idx,
            transform_skip_flag,
            coeff_sign_flag_n,
            last_scan_pos_pass1,
            tu,
            sh,
            ectx,
        );
        let (ctx_idx, bypass_flag) = match ctx_inc {
            CtxInc::Number(ctx_inc) => {
                let init_type = Self::get_init_type(sh);
                let ctx_idx_offset =
                    ctx_table[CabacContext::CoeffSignFlag as usize][0][0].len() * init_type;
                let ctx_idx = ctx_idx_offset + ctx_inc;
                (ctx_idx, false)
            }
            CtxInc::Bypass => (0, true),
            CtxInc::Terminate => (0, false),
            _ => panic!(),
        };
        (ctx_idx, bypass_flag)
    }

    #[inline(always)]
    pub fn derive_context_and_bypass_flag_for_sig_coeff_flag(
        &self,
        x_c: usize,
        y_c: usize,
        c_idx: usize,
        ctx: CabacContext,
        tu: &TransformUnit,
        sh: &SliceHeader,
        ectx: &EncoderContext,
    ) -> (usize, bool) {
        let ctx_inc = {
            let (log2_tb_width, log2_tb_height) = { tu.get_log2_tb_size(c_idx) };
            self.derive_ctx_inc_for_sig_coeff_flag(
                x_c,
                y_c,
                log2_tb_width,
                log2_tb_height,
                tu,
                c_idx,
                sh,
                ectx,
            )
        };
        let (ctx_idx, bypass_flag) = match ctx_inc {
            CtxInc::Number(ctx_inc) => {
                let init_type = Self::get_init_type(sh);
                let ctx_idx_offset = ctx_table[ctx as usize][0][0].len() * init_type;
                let ctx_idx = ctx_idx_offset + ctx_inc;
                (ctx_idx, false)
            }
            CtxInc::Bypass => (0, true),
            CtxInc::Terminate => (0, false),
            _ => panic!(),
        };
        (ctx_idx, bypass_flag)
    }

    pub fn _derive_context_and_bypass_flag_for_alf_ctb_filter_alt_idx(
        &self,
        ref_l: usize,
        ctx: CabacContext,
        cu: Arc<Mutex<CodingUnit>>,
        sh: &SliceHeader,
    ) -> (usize, bool) {
        let ctx_inc = {
            let ref_idx = cu.lock().unwrap().ref_idx[ref_l];
            self._derive_ctx_inc_for_alf_ctb_filter_alt_idx(ref_idx)
        };
        let (ctx_idx, bypass_flag) = match ctx_inc {
            CtxInc::Number(ctx_inc) => {
                let init_type = Self::get_init_type(sh);
                let ctx_idx_offset = ctx_table[ctx as usize][0][0].len() * init_type;
                let ctx_idx = ctx_idx_offset + ctx_inc;
                (ctx_idx, false)
            }
            CtxInc::Bypass => (0, true),
            CtxInc::Terminate => (0, false),
            _ => panic!(),
        };
        (ctx_idx, bypass_flag)
    }

    pub fn derive_context_and_bypass_flag_ctu(
        &self,
        bin_idx: usize,
        ctx: CabacContext,
        sh: &SliceHeader,
    ) -> (usize, bool) {
        let ctx_inc = CTX_INC_TABLE[ctx as usize][bin_idx.min(5)]; // FIXME no hard-coded magic number
        let (ctx_idx, bypass_flag) = match ctx_inc {
            CtxInc::Number(ctx_inc) => {
                let init_type = Self::get_init_type(sh);
                let ctx_idx_offset = ctx_table[ctx as usize][0][0].len() * init_type;
                let ctx_idx = ctx_idx_offset + ctx_inc;
                (ctx_idx, false)
            }
            CtxInc::Bypass => (0, true),
            CtxInc::Terminate => (0, false),
            _ => panic!(),
        };
        (ctx_idx, bypass_flag)
    }

    pub fn derive_context_and_bypass_flag_ct(
        &self,
        bin_idx: usize,
        ctx: CabacContext,
        ct: &CodingTree,
        sh: &SliceHeader,
        ectx: &EncoderContext,
    ) -> (usize, bool) {
        let ctx_inc = match ctx {
            CabacContext::MttSplitCuBinaryFlag => self.derive_ctx_inc_for_mtt_split_cu_binary_flag(
                ct.mtt_split_cu_vertical_flag(),
                ct.depth,
            ),
            CabacContext::MttSplitCuVerticalFlag => {
                let (
                    x,
                    y,
                    ch_type,
                    width,
                    height,
                    allow_split_bt_ver,
                    allow_split_bt_hor,
                    allow_split_tt_ver,
                    allow_split_tt_hor,
                ) = (
                    ct.x,
                    ct.y,
                    ct.ch_type(),
                    ct.width,
                    ct.height,
                    ct.allow_split_bt(MttSplitMode::SPLIT_BT_VER, sh.pps, ectx),
                    ct.allow_split_bt(MttSplitMode::SPLIT_BT_HOR, sh.pps, ectx),
                    ct.allow_split_tt(MttSplitMode::SPLIT_TT_VER, sh.pps, ectx),
                    ct.allow_split_tt(MttSplitMode::SPLIT_TT_HOR, sh.pps, ectx),
                );
                self.derive_ctx_inc_for_mtt_split_cu_vertical_flag(
                    x,
                    y,
                    ch_type,
                    width,
                    height,
                    allow_split_bt_ver,
                    allow_split_bt_hor,
                    allow_split_tt_ver,
                    allow_split_tt_hor,
                    sh.sps,
                    sh.pps,
                    ectx,
                )
            }
            CabacContext::SplitCuFlag | CabacContext::SplitQtFlag | CabacContext::NonInterFlag => {
                let (
                    x,
                    y,
                    cqt_depth,
                    width,
                    height,
                    allow_split_bt_ver,
                    allow_split_bt_hor,
                    allow_split_tt_ver,
                    allow_split_tt_hor,
                    allow_split_qt,
                ) = (
                    ct.x,
                    ct.y,
                    ct.cqt_depth,
                    ct.width,
                    ct.height,
                    ct.allow_split_bt(MttSplitMode::SPLIT_BT_VER, sh.pps, ectx),
                    ct.allow_split_bt(MttSplitMode::SPLIT_BT_HOR, sh.pps, ectx),
                    ct.allow_split_tt(MttSplitMode::SPLIT_TT_VER, sh.pps, ectx),
                    ct.allow_split_tt(MttSplitMode::SPLIT_TT_HOR, sh.pps, ectx),
                    ct.allow_split_qt(ectx),
                );
                if ctx == CabacContext::IntraMipFlag
                    && (width.ilog2() as isize - height.ilog2() as isize).unsigned_abs() > 1
                {
                    CtxInc::Number(3)
                } else {
                    self.derive_ctx_inc_using_left_and_above_ct(
                        ctx,
                        ct,
                        x,
                        y,
                        cqt_depth,
                        width,
                        height,
                        allow_split_bt_ver,
                        allow_split_bt_hor,
                        allow_split_tt_ver,
                        allow_split_tt_hor,
                        allow_split_qt,
                        sh.sps,
                        sh.pps,
                        ectx,
                    )
                }
            }
            _ => CTX_INC_TABLE[ctx as usize][bin_idx.min(5)], // FIXME no hard-coded magic number
        };
        let (ctx_idx, bypass_flag) = match ctx_inc {
            CtxInc::Number(ctx_inc) => {
                let init_type = Self::get_init_type(sh);
                let ctx_idx_offset = ctx_table[ctx as usize][0][0].len() * init_type;
                let ctx_idx = ctx_idx_offset + ctx_inc;
                (ctx_idx, false)
            }
            CtxInc::Bypass => (0, true),
            CtxInc::Terminate => (0, false),
            _ => panic!(),
        };
        (ctx_idx, bypass_flag)
    }

    pub fn derive_context_and_bypass_flag_cu(
        &self,
        bin_idx: usize,
        ctx: CabacContext,
        ct: &CodingTree,
        cu: &CodingUnit,
        sh: &SliceHeader,
        ectx: &EncoderContext,
    ) -> (usize, bool) {
        let ctx_inc = match ctx {
            CabacContext::IntraLumaNotPlanarFlag => {
                self.derive_ctx_inc_for_intra_luma_not_planar_flag(cu.intra_subpartitions_mode_flag)
            }
            CabacContext::RegularMergeFlag => {
                self.derive_ctx_inc_for_regular_merge_flag(cu.skip_flag)
            }
            CabacContext::CuSbtFlag => {
                self.derive_ctx_inc_for_cu_sbt_flag(bin_idx, cu.width, cu.height)
            }
            CabacContext::CuSbtHorizontalFlag => {
                self.derive_ctx_inc_for_cu_sbt_horizontal_flag(bin_idx, cu.width, cu.height)
            }
            CabacContext::LfnstIdx => {
                let ct = cu.parent.lock().unwrap();
                self.derive_ctx_inc_for_lfnst_idx(bin_idx, ct.tree_type)
            }
            CabacContext::BcwIdx => self.derive_ctx_inc_for_bcw_idx(bin_idx, ectx),
            CabacContext::AmvrPrecisionIdx => self.derive_ctx_inc_for_amvr_precision_idx(
                cu.x,
                cu.y,
                bin_idx,
                cu.inter_affine_flag,
                ectx,
            ),
            CabacContext::InterPredIdc => {
                self.derive_ctx_inc_for_inter_pred_idc(bin_idx, cu.width, cu.height)
            }
            CabacContext::CuSkipFlag
            | CabacContext::PredModeFlag
            | CabacContext::PredModeIbcFlag
            | CabacContext::MergeSubblockFlag
            | CabacContext::InterAffineFlag
            | CabacContext::IntraMipFlag => {
                let (x, y, width, height) = { (cu.x, cu.y, cu.width, cu.height) };
                if ctx == CabacContext::IntraMipFlag
                    && (width.ilog2() as isize - height.ilog2() as isize).unsigned_abs() > 1
                {
                    CtxInc::Number(3)
                } else {
                    self.derive_ctx_inc_using_left_and_above_cu(
                        ctx, ct, x, y, width, height, sh.sps, sh.pps, ectx,
                    )
                }
            }
            _ => CTX_INC_TABLE[ctx as usize][bin_idx.min(5)], // FIXME no hard-coded magic number
        };
        let (ctx_idx, bypass_flag) = match ctx_inc {
            CtxInc::Number(ctx_inc) => {
                let init_type = Self::get_init_type(sh);
                let ctx_idx_offset = ctx_table[ctx as usize][0][0].len() * init_type;
                let ctx_idx = ctx_idx_offset + ctx_inc;
                (ctx_idx, false)
            }
            CtxInc::Bypass => (0, true),
            CtxInc::Terminate => (0, false),
            _ => panic!(),
        };
        (ctx_idx, bypass_flag)
    }

    pub fn derive_context_and_bypass_flag_tu(
        &self,
        bin_idx: usize,
        c_idx: usize,
        ctx: CabacContext,
        tu: &TransformUnit,
        sh: &SliceHeader,
        ectx: &EncoderContext,
    ) -> (usize, bool) {
        let ctx_inc = match ctx {
            CabacContext::TuYCodedFlag => {
                let prev_tu_cbf_y = match tu.prev_tu_in_cu() {
                    Some(prev_tu) => {
                        let prev_tu = prev_tu.borrow();
                        prev_tu.get_y_coded_flag() as usize
                    }
                    None => 0,
                };
                let bdpcm_flag = tu.cu_bdpcm_flag[0];
                self.derive_ctx_inc_for_tu_y_coded_flag(
                    tu.first_in_cu(),
                    prev_tu_cbf_y,
                    bdpcm_flag,
                    ectx,
                )
            }
            CabacContext::TuCbCodedFlag => {
                self.derive_ctx_inc_for_tu_cb_coded_flag(bin_idx, tu.cu_bdpcm_flag[1])
            }
            CabacContext::TuCrCodedFlag => self.derive_ctx_inc_for_tu_cr_coded_flag(
                bin_idx,
                tu.get_cb_coded_flag(),
                tu.cu_bdpcm_flag[2],
            ),
            CabacContext::TransformSkipFlag => {
                self.derive_ctx_inc_for_transform_skip_flag(bin_idx, c_idx)
            }
            CabacContext::TuJointCbcrResidualFlag => self
                .derive_ctx_inc_for_tu_joint_cbcr_residual_flag(
                    bin_idx,
                    tu.get_cb_coded_flag(),
                    tu.get_cr_coded_flag(),
                ),
            _ => CTX_INC_TABLE[ctx as usize][bin_idx.min(5)], // FIXME no hard-coded magic number
        };
        let (ctx_idx, bypass_flag) = match ctx_inc {
            CtxInc::Number(ctx_inc) => {
                let init_type = Self::get_init_type(sh);
                let ctx_idx_offset = ctx_table[ctx as usize][0][0].len() * init_type;
                let ctx_idx = ctx_idx_offset + ctx_inc;
                (ctx_idx, false)
            }
            CtxInc::Bypass => (0, true),
            CtxInc::Terminate => (0, false),
            _ => panic!(),
        };
        (ctx_idx, bypass_flag)
    }

    pub fn derive_context_and_bypass_flag_last_sig_coeff_x_prefix(
        &self,
        bin_idx: usize,
        c_idx: usize,
        tu: &TransformUnit,
        sh: &SliceHeader,
    ) -> (usize, bool) {
        let ctx_inc = self.derive_ctx_inc_for_last_sig_coeff_x_prefix(
            bin_idx,
            c_idx,
            tu.get_log2_tb_size(c_idx).0,
        );
        let (ctx_idx, bypass_flag) = match ctx_inc {
            CtxInc::Number(ctx_inc) => {
                let init_type = Self::get_init_type(sh);
                let ctx_idx_offset =
                    ctx_table[CabacContext::LastSigCoeffXPrefix as usize][0][0].len() * init_type;
                let ctx_idx = ctx_idx_offset + ctx_inc;
                (ctx_idx, false)
            }
            CtxInc::Bypass => (0, true),
            CtxInc::Terminate => (0, false),
            _ => panic!(),
        };
        (ctx_idx, bypass_flag)
    }

    pub fn derive_context_and_bypass_flag_last_sig_coeff_y_prefix(
        &self,
        bin_idx: usize,
        c_idx: usize,
        tu: &TransformUnit,
        sh: &SliceHeader,
    ) -> (usize, bool) {
        let ctx_inc = self.derive_ctx_inc_for_last_sig_coeff_y_prefix(
            bin_idx,
            c_idx,
            tu.get_log2_tb_size(c_idx).1,
        );
        let (ctx_idx, bypass_flag) = match ctx_inc {
            CtxInc::Number(ctx_inc) => {
                let init_type = Self::get_init_type(sh);
                let ctx_idx_offset =
                    ctx_table[CabacContext::LastSigCoeffYPrefix as usize][0][0].len() * init_type;
                let ctx_idx = ctx_idx_offset + ctx_inc;
                (ctx_idx, false)
            }
            CtxInc::Bypass => (0, true),
            CtxInc::Terminate => (0, false),
            _ => panic!(),
        };
        (ctx_idx, bypass_flag)
    }

    pub fn _derive_context_and_bypass_flag_residual(
        &self,
        bin_idx: usize,
        ctx: CabacContext,
        sh: &SliceHeader,
    ) -> (usize, bool) {
        let ctx_inc = match ctx {
            CabacContext::EndOfSliceOneBit
            | CabacContext::EndOfTileOneBit
            | CabacContext::EndOfSubsetOneBit => CtxInc::Terminate,
            _ => CTX_INC_TABLE[ctx as usize][bin_idx.min(5)], // FIXME no hard-coded magic number
        };
        let (ctx_idx, bypass_flag) = match ctx_inc {
            CtxInc::Number(ctx_inc) => {
                let init_type = Self::get_init_type(sh);
                let ctx_idx_offset = ctx_table[ctx as usize][0][0].len() * init_type;
                let ctx_idx = ctx_idx_offset + ctx_inc;
                (ctx_idx, false)
            }
            CtxInc::Bypass => (0, true),
            CtxInc::Terminate => (0, false),
            _ => panic!(),
        };
        (ctx_idx, bypass_flag)
    }

    pub fn derive_context_and_bypass_flag(
        &self,
        bin_idx: usize,
        ctx: CabacContext,
        sh: &SliceHeader,
    ) -> (usize, bool) {
        let ctx_inc = CTX_INC_TABLE[ctx as usize][bin_idx.min(5)]; // FIXME no hard-coded magic number
        let (ctx_idx, bypass_flag) = match ctx_inc {
            CtxInc::Number(ctx_inc) => {
                let init_type = Self::get_init_type(sh);
                let ctx_idx_offset = ctx_table[ctx as usize][0][0].len() * init_type;
                let ctx_idx = ctx_idx_offset + ctx_inc;
                (ctx_idx, false)
            }
            CtxInc::Bypass => (0, true),
            //CtxInc::Terminate => (0, false),
            _ => panic!(),
        };
        (ctx_idx, bypass_flag)
    }

    pub fn derive_ctx_inc_for_last_sig_coeff_x_prefix(
        &self,
        bin_idx: usize,
        c_idx: usize,
        log2_tb_width: usize,
    ) -> CtxInc {
        const OFFSET_Y: [usize; 6] = [0, 0, 3, 6, 10, 15];
        let log2_tb_size = log2_tb_width;
        let (ctx_offset, ctx_shift) = if c_idx == 0 {
            (OFFSET_Y[log2_tb_size - 1], (log2_tb_size + 1) >> 2)
        } else {
            (20, ((1 << log2_tb_size) >> 3).clamp(0, 2))
        };
        CtxInc::Number((bin_idx >> ctx_shift) + ctx_offset)
    }

    pub fn derive_ctx_inc_for_last_sig_coeff_y_prefix(
        &self,
        bin_idx: usize,
        c_idx: usize,
        log2_tb_height: usize,
    ) -> CtxInc {
        const OFFSET_Y: [usize; 6] = [0, 0, 3, 6, 10, 15];
        let log2_tb_size = log2_tb_height;
        let (ctx_offset, ctx_shift) = if c_idx == 0 {
            (OFFSET_Y[log2_tb_size - 1], (log2_tb_size + 1) >> 2)
        } else {
            (20, ((1 << log2_tb_size) >> 3).clamp(0, 2))
        };
        CtxInc::Number((bin_idx >> ctx_shift) + ctx_offset)
    }

    pub fn derive_ctx_inc_for_tu_y_coded_flag(
        &self,
        first_in_cu: bool,
        prev_tu_cbf_y: usize,
        bdpcm_flag: bool,
        ectx: &EncoderContext,
    ) -> CtxInc {
        if bdpcm_flag {
            CtxInc::Number(1)
        } else if ectx.intra_subpartitions_split_type == IntraSubpartitionsSplitType::ISP_NO_SPLIT {
            CtxInc::Number(0)
        } else {
            let prev_tu_cbf_y = if first_in_cu { 0 } else { prev_tu_cbf_y };
            CtxInc::Number(2 + prev_tu_cbf_y)
        }
    }

    pub fn derive_ctx_inc_for_sb_coded_flag(
        &self,
        x_s: usize,
        y_s: usize,
        log2_tb_width: usize,
        log2_tb_height: usize,
        c_idx: usize,
        tu: &TransformUnit,
        sh: &SliceHeader,
    ) -> CtxInc {
        let transform_skip_flag = tu.transform_skip_flag[c_idx];
        let mut log2_sb_width = if log2_tb_width.min(log2_tb_height) < 2 {
            1
        } else {
            2
        };
        let mut log2_sb_height = log2_sb_width;
        if log2_tb_width < 2 && c_idx == 0 {
            log2_sb_width = log2_tb_width;
            log2_sb_height = 4 - log2_sb_width;
        } else if log2_tb_height < 2 && c_idx == 0 {
            log2_sb_height = log2_tb_height;
            log2_sb_width = 4 - log2_sb_height;
        }
        let mut csbf_ctx = 0;
        if transform_skip_flag && !sh.ts_residual_coding_disabled_flag {
            if x_s > 0 {
                csbf_ctx += tu.get_sb_coded_flag(c_idx, x_s - 1, y_s) as usize;
            }
            if y_s > 0 {
                csbf_ctx += tu.get_sb_coded_flag(c_idx, x_s, y_s - 1) as usize;
            }
        } else {
            if x_s < (1 << (log2_tb_width - log2_sb_width)) - 1 {
                csbf_ctx += tu.get_sb_coded_flag(c_idx, x_s + 1, y_s) as usize;
            }
            if y_s < (1 << (log2_tb_height - log2_sb_height)) - 1 {
                csbf_ctx += tu.get_sb_coded_flag(c_idx, x_s, y_s + 1) as usize;
            }
        }
        if transform_skip_flag && !sh.ts_residual_coding_disabled_flag {
            CtxInc::Number(4 + csbf_ctx)
        } else if c_idx == 0 {
            CtxInc::Number(csbf_ctx.min(1))
        } else {
            CtxInc::Number(2 + csbf_ctx.min(1))
        }
    }

    #[inline(always)]
    pub fn derive_loc_num_sig_and_loc_sum_abs_pass1(
        &self,
        tu: &TransformUnit,
        c_idx: usize,
        x_c: usize,
        y_c: usize,
        log2_tb_width: usize,
        log2_tb_height: usize,
        sh: &SliceHeader,
        ectx: &EncoderContext,
    ) -> (usize, usize) {
        let mut loc_num_sig = 0;
        let mut loc_sum_abs_pass1 = 0;
        let abs_level_pass1 = &ectx.abs_level_pass1.data[y_c << ectx.abs_level_pass1.log2_stride..];
        if tu.transform_skip_flag[c_idx] && !sh.ts_residual_coding_disabled_flag {
            if x_c > 0 {
                loc_num_sig += (tu.get_sig_coeff_flag(c_idx, x_c - 1, y_c) as usize)
                    .min(abs_level_pass1[x_c - 1]);
                loc_sum_abs_pass1 += abs_level_pass1[x_c - 1];
            }
            if y_c > 0 {
                loc_num_sig += (tu.get_sig_coeff_flag(c_idx, x_c, y_c - 1) as usize)
                    .min(ectx.abs_level_pass1[y_c - 1][x_c]);
                loc_sum_abs_pass1 += ectx.abs_level_pass1[y_c - 1][x_c];
            }
        } else {
            //if x_c < (1 << log2_tb_width) - 1 {
            //loc_num_sig += (tu.get_sig_coeff_flag(c_idx, x_c + 1, y_c) as usize)
            //.min(abs_level_pass1[x_c + 1]);
            //loc_sum_abs_pass1 += abs_level_pass1[x_c + 1];
            //if x_c < (1 << log2_tb_width) - 2 {
            //loc_num_sig += (tu.get_sig_coeff_flag(c_idx, x_c + 2, y_c) as usize)
            //.min(abs_level_pass1[x_c + 2]);
            //loc_sum_abs_pass1 += abs_level_pass1[x_c + 2];
            //}
            //if y_c < (1 << log2_tb_height) - 1 {
            //let abs_level_pass1 = &abs_level_pass1[1 << ectx.abs_level_pass1.log2_stride..];
            //loc_num_sig += (tu.get_sig_coeff_flag(c_idx, x_c + 1, y_c + 1) as usize)
            //.min(abs_level_pass1[x_c + 1]);
            //loc_sum_abs_pass1 += abs_level_pass1[x_c + 1];
            //}
            //}
            //if y_c < (1 << log2_tb_height) - 1 {
            //let abs_level_pass1 = &abs_level_pass1[1 << ectx.abs_level_pass1.log2_stride..];
            //loc_num_sig +=
            //(tu.get_sig_coeff_flag(c_idx, x_c, y_c + 1) as usize).min(abs_level_pass1[x_c]);
            //loc_sum_abs_pass1 += abs_level_pass1[x_c];
            //if y_c < (1 << log2_tb_height) - 2 {
            //let abs_level_pass1 = &abs_level_pass1[1 << ectx.abs_level_pass1.log2_stride..];
            //loc_num_sig += (tu.get_sig_coeff_flag(c_idx, x_c, y_c + 2) as usize)
            //.min(abs_level_pass1[x_c]);
            //loc_sum_abs_pass1 += abs_level_pass1[x_c];
            //}
            //}
            let width = 1 << log2_tb_width;
            let height = 1 << log2_tb_height;
            let pass1_stride = 1 << ectx.abs_level_pass1.log2_stride;
            let qtc = &tu.quantized_transformed_coeffs[c_idx];
            let stride = 1 << qtc.log2_stride;
            let qtc = &qtc.data[(y_c << qtc.log2_stride) + x_c..];
            let abs_level_pass1 = &abs_level_pass1[x_c..];
            if x_c < width - 1 {
                let abs_level_pass1 = &abs_level_pass1[1..];
                let level_r = abs_level_pass1[0];
                loc_num_sig += ((qtc[1] != 0) as usize).min(level_r);
                loc_sum_abs_pass1 += level_r;
                if x_c < width - 2 {
                    let level_r2 = abs_level_pass1[1];
                    loc_num_sig += ((qtc[2] != 0) as usize).min(level_r2);
                    loc_sum_abs_pass1 += level_r2;
                }
                if y_c < height - 1 {
                    let abs_level_pass1 = &abs_level_pass1[pass1_stride..];
                    let level_r = abs_level_pass1[0];
                    loc_num_sig += ((qtc[stride + 1] != 0) as usize).min(level_r);
                    loc_sum_abs_pass1 += level_r;
                }
            }
            if y_c < height - 1 {
                let abs_level_pass1 = &abs_level_pass1[pass1_stride..];
                let level = abs_level_pass1[0];
                loc_num_sig += ((qtc[stride] != 0) as usize).min(level);
                loc_sum_abs_pass1 += level;
                if y_c < height - 2 {
                    let abs_level_pass1 = &abs_level_pass1[pass1_stride..];
                    let level = abs_level_pass1[0];
                    loc_num_sig += ((qtc[stride * 2] != 0) as usize).min(level);
                    loc_sum_abs_pass1 += level;
                }
            }
        }
        (loc_num_sig, loc_sum_abs_pass1)
    }

    pub fn derive_ctx_inc_for_sig_coeff_flag(
        &self,
        x_c: usize,
        y_c: usize,
        log2_tb_width: usize,
        log2_tb_height: usize,
        tu: &TransformUnit,
        c_idx: usize,
        sh: &SliceHeader,
        ectx: &EncoderContext,
    ) -> CtxInc {
        let (loc_num_sig, loc_sum_abs_pass1) = self.derive_loc_num_sig_and_loc_sum_abs_pass1(
            tu,
            c_idx,
            x_c,
            y_c,
            log2_tb_width,
            log2_tb_height,
            sh,
            ectx,
        );
        let d = x_c + y_c;
        let transform_skip_flag = tu.transform_skip_flag[c_idx];
        if transform_skip_flag && !sh.ts_residual_coding_disabled_flag {
            CtxInc::Number(60 + loc_num_sig)
        } else if c_idx == 0 {
            CtxInc::Number(
                12 * (ectx.q_state as isize - 1).max(0) as usize
                    + ((loc_sum_abs_pass1 + 1) >> 1).min(3)
                    + if d < 2 {
                        8
                    } else if d < 5 {
                        4
                    } else {
                        0
                    },
            )
        } else {
            CtxInc::Number(
                36 + 8 * (ectx.q_state as isize - 1).max(0) as usize
                    + ((loc_sum_abs_pass1 + 1) >> 1).min(3)
                    + if d < 2 { 4 } else { 0 },
            )
        }
    }

    pub fn derive_ctx_inc_for_par_level_flag_and_abs_level_gtx_flag(
        &self,
        ctx: CabacContext,
        x_c: usize,
        y_c: usize,
        log2_tb_width: usize,
        log2_tb_height: usize,
        tu: &TransformUnit,
        c_idx: usize,
        j: usize,
        last_sig_coeff_pos: (usize, usize),
        sh: &SliceHeader,
        ectx: &EncoderContext,
    ) -> CtxInc {
        let transform_skip_flag = tu.transform_skip_flag[c_idx];
        if transform_skip_flag && !sh.ts_residual_coding_disabled_flag {
            if ctx == CabacContext::ParLevelFlag {
                CtxInc::Number(32)
            } else if j == 0 {
                let cu = tu.get_cu();
                let cu = cu.lock().unwrap();
                let bdpcm_flag = cu.get_bdpcm_flag(c_idx);
                if bdpcm_flag {
                    CtxInc::Number(67)
                } else if x_c > 0 && y_c > 0 {
                    CtxInc::Number(
                        64 + tu.get_sig_coeff_flag(c_idx, x_c - 1, y_c) as usize
                            + tu.get_sig_coeff_flag(c_idx, x_c, y_c - 1) as usize,
                    )
                } else if x_c > 0 {
                    CtxInc::Number(64 + tu.get_sig_coeff_flag(c_idx, x_c - 1, y_c) as usize)
                } else if y_c > 0 {
                    CtxInc::Number(64 + tu.get_sig_coeff_flag(c_idx, x_c, y_c - 1) as usize)
                } else {
                    CtxInc::Number(64)
                }
            } else {
                CtxInc::Number(67 + j)
            }
        } else {
            let (loc_num_sig, loc_sum_abs_pass1) = self.derive_loc_num_sig_and_loc_sum_abs_pass1(
                tu,
                c_idx,
                x_c,
                y_c,
                log2_tb_width,
                log2_tb_height,
                sh,
                ectx,
            );
            let ctx_offset = (loc_sum_abs_pass1 - loc_num_sig).min(4);
            let d = x_c + y_c;
            let (last_significant_coeff_x, last_significant_coeff_y) = last_sig_coeff_pos;
            let mut ctx_inc = if x_c == last_significant_coeff_x && y_c == last_significant_coeff_y
            {
                if c_idx == 0 {
                    0
                } else {
                    21
                }
            } else if c_idx == 0 {
                1 + ctx_offset
                    + if d == 0 {
                        15
                    } else if d < 3 {
                        10
                    } else if d < 10 {
                        5
                    } else {
                        0
                    }
            } else {
                22 + ctx_offset + if d == 0 { 5 } else { 0 }
            };
            if ctx == CabacContext::AbsLevelGtxFlag && j == 1 {
                ctx_inc += 32;
            }
            CtxInc::Number(ctx_inc)
        }
    }

    pub fn derive_ctx_inc_for_coeff_sign_flag_for_transform_skip_mode(
        &self,
        x_c: usize,
        y_c: usize,
        c_idx: usize,
        tu: &TransformUnit,
        ectx: &EncoderContext,
    ) -> CtxInc {
        let left_sign = if x_c == 0 {
            0
        } else {
            ectx.coeff_sign_level[x_c - 1][y_c]
        };
        let above_sign = if y_c == 0 {
            0
        } else {
            ectx.coeff_sign_level[x_c][y_c - 1]
        };
        let bdpcm_flag = tu.cu_bdpcm_flag[c_idx];
        if (left_sign == 0 && above_sign == 0) || left_sign == -above_sign {
            CtxInc::Number(if !bdpcm_flag { 0 } else { 3 })
        } else if left_sign >= 0 && above_sign >= 0 {
            CtxInc::Number(if !bdpcm_flag { 1 } else { 4 })
        } else {
            CtxInc::Number(if !bdpcm_flag { 2 } else { 5 })
        }
    }

    pub fn _derive_ctx_inc_for_run_copy_flag(
        &self,
        previous_run_type: usize,
        previous_run_position: usize,
        cur_pos: usize,
    ) -> CtxInc {
        const CTX_INC_TABLE: [[usize; 5]; 2] = [[0, 1, 2, 3, 4], [5, 6, 6, 7, 7]];
        let bin_dist = cur_pos - previous_run_position - 1;
        let ctx_inc = CTX_INC_TABLE[previous_run_type][bin_dist.min(4)];
        CtxInc::Number(ctx_inc)
    }

    pub fn _derive_ctx_inc_for_alf_ctb_filter_alt_idx(&self, ref_idx: usize) -> CtxInc {
        CtxInc::Number(ref_idx)
    }

    pub fn derive_ctx_inc_for_mtt_split_cu_binary_flag(
        &self,
        mtt_split_cu_vertical_flag: bool,
        mtt_depth: usize,
    ) -> CtxInc {
        CtxInc::Number(2 * mtt_split_cu_vertical_flag as usize + (mtt_depth <= 1) as usize)
    }

    pub fn derive_ctx_inc_for_intra_luma_not_planar_flag(
        &self,
        intra_subpartitions_mode_flag: bool,
    ) -> CtxInc {
        CtxInc::Number((!intra_subpartitions_mode_flag) as usize)
    }

    pub fn derive_ctx_inc_for_regular_merge_flag(&self, cu_skip_flag: bool) -> CtxInc {
        CtxInc::Number((!cu_skip_flag) as usize)
    }

    pub fn derive_ctx_inc_for_cu_sbt_flag(
        &self,
        bin_idx: usize,
        cb_width: usize,
        cb_height: usize,
    ) -> CtxInc {
        if bin_idx == 0 {
            CtxInc::Number((cb_width * cb_height <= 256) as usize)
        } else {
            CTX_INC_TABLE[CabacContext::CuSbtFlag as usize][bin_idx]
        }
    }

    pub fn derive_ctx_inc_for_cu_sbt_horizontal_flag(
        &self,
        bin_idx: usize,
        cb_width: usize,
        cb_height: usize,
    ) -> CtxInc {
        if bin_idx == 0 {
            CtxInc::Number(if cb_width == cb_height {
                0
            } else if cb_width < cb_height {
                1
            } else {
                2
            })
        } else {
            CTX_INC_TABLE[CabacContext::CuSbtFlag as usize][bin_idx]
        }
    }

    pub fn derive_ctx_inc_for_lfnst_idx(&self, bin_idx: usize, tree_type: TreeType) -> CtxInc {
        if bin_idx == 0 {
            CtxInc::Number((tree_type != TreeType::SINGLE_TREE) as usize)
        } else {
            CTX_INC_TABLE[CabacContext::LfnstIdx as usize][bin_idx]
        }
    }

    pub fn derive_ctx_inc_for_tu_cb_coded_flag(
        &self,
        bin_idx: usize,
        intra_bdpcm_chroma_flag: bool,
    ) -> CtxInc {
        if bin_idx == 0 {
            CtxInc::Number(intra_bdpcm_chroma_flag as usize)
        } else {
            panic!();
        }
    }

    pub fn derive_ctx_inc_for_tu_cr_coded_flag(
        &self,
        bin_idx: usize,
        tu_cb_coded_flag: bool,
        intra_bdpcm_chroma_flag: bool,
    ) -> CtxInc {
        if bin_idx == 0 {
            CtxInc::Number(if intra_bdpcm_chroma_flag {
                2
            } else {
                tu_cb_coded_flag as usize
            })
        } else {
            panic!();
        }
    }

    pub fn derive_ctx_inc_for_transform_skip_flag(&self, bin_idx: usize, c_idx: usize) -> CtxInc {
        if bin_idx == 0 {
            CtxInc::Number((c_idx != 0) as usize)
        } else {
            CTX_INC_TABLE[CabacContext::TransformSkipFlag as usize][bin_idx]
        }
    }

    pub fn derive_ctx_inc_for_tu_joint_cbcr_residual_flag(
        &self,
        bin_idx: usize,
        tu_cb_coded_flag: bool,
        tu_cr_coded_flag: bool,
    ) -> CtxInc {
        if bin_idx == 0 {
            CtxInc::Number(2 * (tu_cb_coded_flag as usize) + tu_cr_coded_flag as usize - 1)
        } else {
            CTX_INC_TABLE[CabacContext::TransformSkipFlag as usize][bin_idx]
        }
    }

    pub fn derive_ctx_inc_for_coeff_sign_flag(
        &self,
        x_c: usize,
        y_c: usize,
        c_idx: usize,
        transform_skip_flag: bool,
        n: usize,
        last_scan_pos_pass1: isize,
        tu: &TransformUnit,
        sh: &SliceHeader,
        ectx: &EncoderContext,
    ) -> CtxInc {
        if !transform_skip_flag
            || n as isize > last_scan_pos_pass1
            || sh.ts_residual_coding_disabled_flag
        {
            CtxInc::Bypass
        } else {
            self.derive_ctx_inc_for_coeff_sign_flag_for_transform_skip_mode(
                x_c, y_c, c_idx, tu, ectx,
            )
        }
    }

    pub fn derive_ctx_inc_for_bcw_idx(&self, bin_idx: usize, ectx: &EncoderContext) -> CtxInc {
        if ectx.no_backward_pred_flag {
            if bin_idx == 2 || bin_idx == 3 {
                CtxInc::Bypass
            } else {
                CTX_INC_TABLE[CabacContext::BcwIdx as usize][bin_idx]
            }
        } else {
            CTX_INC_TABLE[CabacContext::BcwIdx as usize][bin_idx]
        }
    }

    pub fn derive_ctx_inc_for_amvr_precision_idx(
        &self,
        x0: usize,
        y0: usize,
        bin_idx: usize,
        inter_affine_flag: bool,
        ectx: &EncoderContext,
    ) -> CtxInc {
        if bin_idx == 0 {
            CtxInc::Number(if ectx.cu_pred_mode[0][x0][y0] == ModeType::MODE_IBC {
                1
            } else if inter_affine_flag {
                0
            } else {
                2
            })
        } else {
            CTX_INC_TABLE[CabacContext::AmvrPrecisionIdx as usize][bin_idx]
        }
    }

    pub fn derive_ctx_inc_for_inter_pred_idc(
        &self,
        bin_idx: usize,
        cb_width: usize,
        cb_height: usize,
    ) -> CtxInc {
        if bin_idx == 0 {
            CtxInc::Number(if cb_width + cb_height > 12 {
                7 - ((1
                    + ((cb_width - 1).ilog2() + 1) as usize
                    + ((cb_height - 1).ilog2() + 1) as usize)
                    >> 1)
            } else {
                5
            })
        } else {
            CTX_INC_TABLE[CabacContext::InterPredIdc as usize][bin_idx]
        }
    }

    pub fn derive_ctx_inc_for_mtt_split_cu_vertical_flag(
        &self,
        x0: usize,
        y0: usize,
        ch_type: usize,
        cb_width: usize,
        cb_height: usize,
        allow_split_bt_ver: bool,
        allow_split_bt_hor: bool,
        allow_split_tt_ver: bool,
        allow_split_tt_hor: bool,
        sps: &SequenceParameterSet,
        pps: &PictureParameterSet,
        ectx: &EncoderContext,
    ) -> CtxInc {
        let x_nb_l = x0 as isize - 1;
        let y_nb_l = y0 as isize;
        let available_l = ectx.derive_neighbouring_block_availability(
            x0, y0, x_nb_l, y_nb_l, cb_width, cb_height, false, false, false, sps, pps,
        );
        let x_nb_a = x0 as isize;
        let y_nb_a = y0 as isize - 1;
        let available_a = ectx.derive_neighbouring_block_availability(
            x0, y0, x_nb_a, y_nb_a, cb_width, cb_height, false, false, false, sps, pps,
        );
        if allow_split_bt_ver as usize + allow_split_tt_ver as usize
            > allow_split_bt_hor as usize + allow_split_tt_hor as usize
        {
            CtxInc::Number(4)
        } else if (allow_split_bt_ver as usize + allow_split_tt_ver as usize)
            < (allow_split_bt_hor as usize + allow_split_tt_hor as usize)
        {
            CtxInc::Number(3)
        } else {
            let d_a = cb_width
                / if available_a {
                    ectx.cb_width[ch_type][x_nb_a as usize][y_nb_a as usize]
                } else {
                    1
                };
            let d_l = cb_height
                / if available_l {
                    ectx.cb_height[ch_type][x_nb_l as usize][y_nb_l as usize]
                } else {
                    1
                };
            if d_a == d_l || !available_a || !available_l {
                CtxInc::Number(0)
            } else if d_a < d_l {
                CtxInc::Number(1)
            } else {
                CtxInc::Number(2)
            }
        }
    }

    pub fn derive_ctx_inc_using_left_and_above_ct(
        &self,
        cabac_context: CabacContext,
        ct: &CodingTree,
        x0: usize,
        y0: usize,
        cqt_depth: usize,
        cb_width: usize,
        cb_height: usize,
        allow_split_bt_ver: bool,
        allow_split_bt_hor: bool,
        allow_split_tt_ver: bool,
        allow_split_tt_hor: bool,
        allow_split_qt: bool,
        sps: &SequenceParameterSet,
        pps: &PictureParameterSet,
        ectx: &EncoderContext,
    ) -> CtxInc {
        let x_nb_l = x0 as isize - 1;
        let y_nb_l = y0 as isize;
        let available_l = ectx.derive_neighbouring_block_availability(
            x0, y0, x_nb_l, y_nb_l, cb_width, cb_height, false, false, false, sps, pps,
        );
        let x_nb_a = x0 as isize;
        let y_nb_a = y0 as isize - 1;
        let available_a = ectx.derive_neighbouring_block_availability(
            x0, y0, x_nb_a, y_nb_a, cb_width, cb_height, false, false, false, sps, pps,
        );

        let ctx_inc = match cabac_context {
            ctx @ (CabacContext::SplitQtFlag | CabacContext::SplitCuFlag) => {
                let (cond_l, cond_a, ctx_set_idx) = match ctx {
                    CabacContext::SplitQtFlag => (
                        if available_l {
                            let left_ct = ct.left_ct();
                            let left_ct = left_ct.as_ref().unwrap();
                            let left_ct = left_ct.lock().unwrap();
                            left_ct.cqt_depth > 0
                        } else {
                            false
                        },
                        if available_a {
                            let above_ct = ct.above_ct();
                            let above_ct = above_ct.as_ref().unwrap();
                            let above_ct = above_ct.lock().unwrap();
                            above_ct.cqt_depth > 0
                        } else {
                            false
                        },
                        (cqt_depth >= 2) as usize,
                    ),
                    CabacContext::SplitCuFlag => (
                        if available_l {
                            let left_ct = ct.left_ct();
                            let left_ct = left_ct.as_ref().unwrap();
                            let left_ct = left_ct.lock().unwrap();
                            left_ct.height < cb_height
                        } else {
                            false
                        },
                        if available_a {
                            let above_ct = ct.above_ct();
                            let above_ct = above_ct.as_ref().unwrap();
                            let above_ct = above_ct.lock().unwrap();
                            above_ct.width < cb_width
                        } else {
                            false
                        },
                        (allow_split_bt_ver as usize
                            + allow_split_bt_hor as usize
                            + allow_split_tt_ver as usize
                            + allow_split_tt_hor as usize
                            + 2 * allow_split_qt as usize
                            - 1)
                            / 2,
                    ),
                    _ => {
                        panic!()
                    }
                };
                CtxInc::Number(
                    (cond_l && available_l) as usize
                        + (cond_a && available_a) as usize
                        + ctx_set_idx * 3,
                )
            }
            ctx @ CabacContext::NonInterFlag => {
                let (cond_l, cond_a) = match ctx {
                    CabacContext::PredModeFlag => {
                        let left_ct = ct.left_ct();
                        let left_ct = left_ct.as_ref().unwrap();
                        let left_ct = left_ct.lock().unwrap();
                        let left_cu = left_ct.cus[0].clone();
                        let left_cu = left_cu.lock().unwrap();
                        (
                            // FIXME ch_type
                            left_cu.mode_type == ModeType::MODE_INTRA,
                            left_cu.mode_type == ModeType::MODE_INTRA,
                        )
                    }
                    CabacContext::NonInterFlag => {
                        let above_ct = ct.above_ct();
                        let above_ct = above_ct.as_ref().unwrap();
                        let above_ct = above_ct.lock().unwrap();
                        let above_cu = above_ct.cus[0].clone();
                        let above_cu = above_cu.lock().unwrap();
                        (
                            // FIXME ch_type
                            above_cu.mode_type == ModeType::MODE_INTRA,
                            above_cu.mode_type == ModeType::MODE_INTRA,
                        )
                    }
                    _ => {
                        panic!()
                    }
                };
                CtxInc::Number(((cond_l && available_l) || (cond_a && available_a)) as usize)
            }
            _ => {
                panic!()
            }
        };
        ctx_inc
    }

    pub fn derive_ctx_inc_using_left_and_above_cu(
        &self,
        cabac_context: CabacContext,
        ct: &CodingTree,
        x0: usize,
        y0: usize,
        cb_width: usize,
        cb_height: usize,
        sps: &SequenceParameterSet,
        pps: &PictureParameterSet,
        ectx: &EncoderContext,
    ) -> CtxInc {
        let x_nb_l = x0 as isize - 1;
        let y_nb_l = y0 as isize;
        let available_l = ectx.derive_neighbouring_block_availability(
            x0, y0, x_nb_l, y_nb_l, cb_width, cb_height, false, false, false, sps, pps,
        );
        let x_nb_a = x0 as isize;
        let y_nb_a = y0 as isize - 1;
        let available_a = ectx.derive_neighbouring_block_availability(
            x0, y0, x_nb_a, y_nb_a, cb_width, cb_height, false, false, false, sps, pps,
        );

        let ctx_inc = match cabac_context {
            ctx @ (CabacContext::CuSkipFlag
            | CabacContext::PredModeIbcFlag
            | CabacContext::IntraMipFlag
            | CabacContext::MergeSubblockFlag
            | CabacContext::InterAffineFlag) => {
                let (cond_l, cond_a, ctx_set_idx) = match ctx {
                    CabacContext::CuSkipFlag => (
                        if available_l {
                            let left_ct = ct.left_ct();
                            let left_ct = left_ct.as_ref().unwrap();
                            let left_ct = left_ct.lock().unwrap();
                            let left_cu = left_ct.cus[0].clone();
                            let left_cu = left_cu.lock().unwrap();
                            left_cu.skip_flag
                        } else {
                            false
                        },
                        if available_a {
                            let above_ct = ct.above_ct();
                            let above_ct = above_ct.as_ref().unwrap();
                            let above_ct = above_ct.lock().unwrap();
                            let above_cu = above_ct.cus[0].clone();
                            let above_cu = above_cu.lock().unwrap();
                            above_cu.skip_flag
                        } else {
                            false
                        },
                        0,
                    ),
                    CabacContext::PredModeIbcFlag => (
                        if available_l {
                            let left_ct = ct.left_ct();
                            let left_ct = left_ct.as_ref().unwrap();
                            let left_ct = left_ct.lock().unwrap();
                            let left_cu = left_ct.cus[0].clone();
                            let left_cu = left_cu.lock().unwrap();
                            // FIXME ch_type
                            left_cu.mode_type == ModeType::MODE_IBC
                        } else {
                            false
                        },
                        if available_a {
                            let above_ct = ct.above_ct();
                            let above_ct = above_ct.as_ref().unwrap();
                            let above_ct = above_ct.lock().unwrap();
                            let above_cu = above_ct.cus[0].clone();
                            let above_cu = above_cu.lock().unwrap();
                            // FIXME ch_type
                            above_cu.mode_type == ModeType::MODE_IBC
                        } else {
                            false
                        },
                        0,
                    ),
                    CabacContext::IntraMipFlag => (
                        if available_l {
                            let left_ct = ct.left_ct();
                            let left_ct = left_ct.as_ref().unwrap();
                            let left_ct = left_ct.lock().unwrap();
                            let left_cu = left_ct.cus[0].clone();
                            let left_cu = left_cu.lock().unwrap();
                            left_cu.intra_mip_flag
                        } else {
                            false
                        },
                        if available_a {
                            let above_ct = ct.above_ct();
                            let above_ct = above_ct.as_ref().unwrap();
                            let above_ct = above_ct.lock().unwrap();
                            let above_cu = above_ct.cus[0].clone();
                            let above_cu = above_cu.lock().unwrap();
                            above_cu.intra_mip_flag
                        } else {
                            false
                        },
                        0,
                    ),
                    CabacContext::MergeSubblockFlag => (
                        if available_l {
                            let left_ct = ct.left_ct();
                            let left_ct = left_ct.as_ref().unwrap();
                            let left_ct = left_ct.lock().unwrap();
                            let left_cu = left_ct.cus[0].clone();
                            let left_cu = left_cu.lock().unwrap();
                            let merge_data = left_cu.merge_data.as_ref().unwrap();
                            merge_data.merge_subblock_flag || left_cu.inter_affine_flag
                        } else {
                            false
                        },
                        if available_a {
                            let above_ct = ct.above_ct();
                            let above_ct = above_ct.as_ref().unwrap();
                            let above_ct = above_ct.lock().unwrap();
                            let above_cu = above_ct.cus[0].clone();
                            let above_cu = above_cu.lock().unwrap();
                            let merge_data = above_cu.merge_data.as_ref().unwrap();
                            merge_data.merge_subblock_flag || above_cu.inter_affine_flag
                        } else {
                            false
                        },
                        0,
                    ),
                    CabacContext::InterAffineFlag => (
                        if available_l {
                            let left_ct = ct.left_ct();
                            let left_ct = left_ct.as_ref().unwrap();
                            let left_ct = left_ct.lock().unwrap();
                            let left_cu = left_ct.cus[0].clone();
                            let left_cu = left_cu.lock().unwrap();
                            let merge_data = left_cu.merge_data.as_ref().unwrap();
                            merge_data.merge_subblock_flag || left_cu.inter_affine_flag
                        } else {
                            false
                        },
                        if available_a {
                            let above_ct = ct.above_ct();
                            let above_ct = above_ct.as_ref().unwrap();
                            let above_ct = above_ct.lock().unwrap();
                            let above_cu = above_ct.cus[0].clone();
                            let above_cu = above_cu.lock().unwrap();
                            let merge_data = above_cu.merge_data.as_ref().unwrap();
                            merge_data.merge_subblock_flag || above_cu.inter_affine_flag
                        } else {
                            false
                        },
                        0,
                    ),
                    _ => {
                        panic!()
                    }
                };
                CtxInc::Number(
                    (cond_l && available_l) as usize
                        + (cond_a && available_a) as usize
                        + ctx_set_idx * 3,
                )
            }
            CabacContext::PredModeFlag => {
                let (cond_l, cond_a) = {
                    let left_ct = ct.left_ct();
                    let left_ct = left_ct.as_ref().unwrap();
                    let left_ct = left_ct.lock().unwrap();
                    let left_cu = left_ct.cus[0].clone();
                    let left_cu = left_cu.lock().unwrap();
                    (
                        // FIXME ch_type
                        left_cu.mode_type == ModeType::MODE_INTRA,
                        left_cu.mode_type == ModeType::MODE_INTRA,
                    )
                };
                CtxInc::Number(((cond_l && available_l) || (cond_a && available_a)) as usize)
            }
            _ => {
                panic!()
            }
        };
        ctx_inc
    }
}

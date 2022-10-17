use super::common::*;
use super::picture_header::*;
use super::pps::*;
use super::reference_picture::*;
use super::slice_header::*;
use super::sps::*;
use super::vps::*;
use debug_print::*;
use std::collections::HashMap;

pub struct EncoderContext {
    pub vps_num_dpb_params: usize,
    pub input_picture_width: usize,
    pub input_picture_height: usize,
    pub output_picture_width: usize,
    pub output_picture_height: usize,
    pub total_num_olss: usize,
    pub num_direct_ref_layers: Vec<usize>,
    pub direct_ref_layers_idx: Vec<Vec<usize>>,
    pub num_ref_layers: Vec<usize>,
    pub referenced_layer_idx: Vec<Vec<usize>>,
    pub layer_used_as_ref_layer_flag: Vec<bool>,
    pub num_output_layers_in_ols: Vec<usize>,
    pub output_layer_id_in_ols: Vec<Vec<usize>>,
    pub output_layer_idx: Vec<Vec<usize>>,
    pub num_sub_layers_in_layer_in_ols: Vec<Vec<usize>>,
    pub num_layers_in_ols: Vec<usize>,
    pub layer_id_in_ols: Vec<Vec<usize>>,
    pub num_multi_layer_olss: usize,
    pub multi_layer_ols_idx: Vec<usize>,
    pub num_extra_ph_bits: usize,
    pub num_extra_sh_bits: usize,
    pub min_cb_log2_size_y: usize,
    pub min_cb_size_y: usize,
    pub ibc_buf_width_y: usize,
    pub ibc_buf_width_c: usize,
    pub v_size: usize,
    pub ctb_width_c: usize,
    pub ctb_height_c: usize,
    pub ctb_size_y: usize,
    pub sub_width_c: usize,
    pub sub_height_c: usize,
    pub min_qt_log2_size_y: usize,
    pub min_qt_log2_size_c: usize,
    pub min_qt_log2_size_intra_y: usize,
    pub min_qt_log2_size_inter_y: usize,
    pub min_tb_log2_size_y: usize,
    pub max_tb_log2_size_y: usize,
    pub min_tb_size_y: usize,
    pub max_tb_size_y: usize,
    pub chroma_qp_table: Vec<Vec<isize>>,
    pub qp_bd_offset: isize,
    pub sps_ladf_interval_lower_bound: Vec<usize>,
    pub subpic_id_val: Vec<usize>,
    pub pps_ref_wraparound_offset: usize,
    pub max_pic_order_cnt_lsb: usize,
    pub qp_prime_ts_min: usize,
    pub max_num_merge_cand: usize,
    pub pic_width_in_ctbs_y: usize,
    pub pic_height_in_ctbs_y: usize,
    pub pic_size_in_ctbs_y: usize,
    pub pic_width_in_min_cbs_y: usize,
    pub pic_height_in_min_cbs_y: usize,
    pub pic_size_in_min_cbs_y: usize,
    pub pic_size_in_samples_y: usize,
    pub pic_width_in_samples_c: usize,
    pub pic_height_in_samples_c: usize,
    pub num_tile_columns: usize,
    pub num_tile_rows: usize,
    pub num_tiles_in_pic: usize,
    pub curr_pic_scal_win_width_l: usize,
    pub curr_pic_scal_win_height_l: usize,
    pub num_ctus_in_slice: Vec<usize>,
    pub num_ctus_in_curr_slice: usize,
    pub slice_top_left_tile_idx: Vec<usize>,
    pub num_slices_in_tile: Vec<usize>,
    pub ctb_addr_in_slice: Vec<Vec<usize>>,
    pub row_height_val: Vec<usize>,
    pub num_ltrp_entries: Vec<Vec<usize>>,
    pub rpls_idx: Vec<usize>,
    pub first_ctb_row_in_slice: bool,
    pub ctb_addr_in_rs: usize,
    pub ctb_addr_in_curr_slice: Vec<usize>,
    pub ctb_addr_x: usize,
    pub ctb_addr_y: usize,
    pub ctb_to_tile_col_bd: Vec<usize>,
    pub num_hmvp_cand: usize,
    pub num_hmvp_ibc_cand: usize,
    pub reset_ibc_buf: bool,
    pub ctb_log2_size_y: usize,
    pub predictor_palette_size: [usize; 2],
    pub slice_qp_y: isize,
    pub table_palette_size_wpp: Vec<usize>,
    pub table_palette_entries_wpp: Vec<Vec<usize>>,
    pub predictor_palette_entries: Vec<Vec<usize>>,
    pub zero_pos: Vec<usize>,
    pub abs_level: Vec2d<usize>,
    pub q_state: usize,
    pub max_palette_index: usize,
    pub cu_pred_mode: Vec<Vec<Vec<ModeType>>>,
    pub is_available: Vec<Vec<Vec<bool>>>,
    pub cqt_depth: Vec<Vec<Vec<usize>>>,
    pub cb_width: Vec<Vec<Vec<usize>>>,
    pub cb_height: Vec<Vec<Vec<usize>>>,
    pub cu_skip_flag: Vec<Vec<bool>>,
    pub intra_mip_flag: Vec<Vec<bool>>,
    pub merge_subblock_flag: Vec<Vec<bool>>,
    pub inter_affine_flag: Vec<Vec<bool>>,
    pub bdpcm_flag: Vec<Vec<Vec<bool>>>,
    pub intra_subpartitions_split_type: IntraSubpartitionsSplitType,
    pub abs_level_pass1: Vec2d<usize>,
    pub abs_level_pass2: Vec2d<usize>,
    pub last_significant_coeff_x: usize,
    pub last_significant_coeff_y: usize,
    pub coeff_sign_level: Vec<Vec<isize>>,
    pub no_backward_pred_flag: bool,
    pub min_qt_size_y: usize,
    pub min_qt_size_c: usize,
    pub min_bt_size_y: usize,
    pub min_tt_size_y: usize,
    pub mtt_split_mode: Vec<Vec<Vec<MttSplitMode>>>,
    pub max_mtt_depth_y: usize,
    pub max_mtt_depth_c: usize,
    pub max_qt_size_y: usize,
    pub max_bt_size_y: usize,
    pub max_tt_size_y: usize,
    pub max_qt_size_c: usize,
    pub max_bt_size_c: usize,
    pub max_tt_size_c: usize,
    pub bit_depth: usize,
    pub num_ref_idx_active: [usize; 2],
    pub max_num_subblock_merge_cand: usize,
    pub max_num_ibc_merge_cand: usize,
    pub max_num_gpm_merge_cand: usize,
    pub sao_type_idx: Vec<Vec<Vec<usize>>>,
    pub is_cu_qp_delta_coded: bool,
    pub cu_qp_delta_val: isize,
    pub cu_qg_top_left_x: usize,
    pub cu_qg_top_left_y: usize,
    pub is_cu_chroma_qp_offset_coded: bool,
    pub cu_qp_offset_cb: usize,
    pub cu_qp_offset_cr: usize,
    pub cu_qp_offset_cbcr: usize,
    pub cu_qp_delta_sub_div: usize,
    pub cu_chroma_qp_offset_subdiv: usize,
    pub mode_type_condition: usize,
    pub max_ts_size: usize,
    pub cclm_enabled: bool,
    pub mvd_l0: (isize, isize),
    pub mvd_l1: (isize, isize),
    pub ref_idx_sym_l0: isize,
    pub ref_idx_sym_l1: isize,
    pub motion_model_idc: usize,
    pub mvd_cp_l0: [(isize, isize); 3],
    pub mvd_cp_l1: [(isize, isize); 3],
    pub lfnst_dc_only: bool,
    pub lfnst_zero_out_sig_coeff_flag: bool,
    pub mts_dc_only: bool,
    pub mts_zero_out_sig_coeff_flag: bool,
    pub num_intra_subpartitions: usize,
    pub infer_tu_cbf_luma: bool,
    pub cb_pos_x: Vec<Vec<Vec<usize>>>,
    pub cb_pos_y: Vec<Vec<Vec<usize>>>,
    pub q_state_trans_table: [[usize; 2]; 4],
    pub trans_coeff_level: Vec<Vec<Vec<Vec<Vec<isize>>>>>,
    pub rem_ccbs: usize,
    pub num_slices_in_subpic: Vec<usize>,
    pub curr_subpic_idx: usize,
    pub num_weights_l0: usize,
    pub num_weights_l1: usize,
    pub num_alf_filters: usize,
    pub scaling_list: Vec<Vec<isize>>,
    pub num_entry_points: usize,
    pub scaling_matrix_rec: Vec<Vec<Vec<i32>>>,
    pub scaling_matrix_dc_rec: Vec<i32>,
    pub apply_lfnst_flag: [bool; 3],
    pub qp_y: usize,
    pub fixed_qp: Option<usize>,
    pub max_split_depth: usize,
    pub extra_params: HashMap<String, String>,
    pub enable_print: bool,
}

impl EncoderContext {
    pub fn new() -> EncoderContext {
        EncoderContext {
            vps_num_dpb_params: 0,
            input_picture_width: 0,
            input_picture_height: 0,
            output_picture_width: 0,
            output_picture_height: 0,
            total_num_olss: 1,
            num_direct_ref_layers: vec![],
            direct_ref_layers_idx: vec![],
            num_ref_layers: vec![],
            referenced_layer_idx: vec![],
            layer_used_as_ref_layer_flag: vec![],
            num_output_layers_in_ols: vec![],
            output_layer_id_in_ols: vec![],
            output_layer_idx: vec![],
            num_sub_layers_in_layer_in_ols: vec![],
            num_layers_in_ols: vec![],
            layer_id_in_ols: vec![],
            num_multi_layer_olss: 0,
            multi_layer_ols_idx: vec![],
            num_extra_ph_bits: 0,
            num_extra_sh_bits: 0,
            min_cb_log2_size_y: 0,
            min_cb_size_y: 0,
            ibc_buf_width_y: 0,
            ibc_buf_width_c: 0,
            v_size: 0,
            ctb_width_c: 0,
            ctb_height_c: 0,
            ctb_size_y: 0,
            sub_width_c: 2,
            sub_height_c: 2,
            min_qt_log2_size_intra_y: 0,
            min_qt_log2_size_inter_y: 0,
            min_tb_log2_size_y: 0,
            max_tb_log2_size_y: 0,
            min_tb_size_y: 0,
            max_tb_size_y: 0,
            chroma_qp_table: vec![],
            qp_bd_offset: 0,
            sps_ladf_interval_lower_bound: vec![],
            subpic_id_val: vec![],
            pps_ref_wraparound_offset: 0,
            max_pic_order_cnt_lsb: 0,
            qp_prime_ts_min: 0,
            max_num_merge_cand: 0,
            pic_width_in_ctbs_y: 0,
            pic_height_in_ctbs_y: 0,
            pic_size_in_ctbs_y: 0,
            pic_width_in_min_cbs_y: 0,
            pic_height_in_min_cbs_y: 0,
            pic_size_in_min_cbs_y: 0,
            pic_size_in_samples_y: 0,
            pic_width_in_samples_c: 0,
            pic_height_in_samples_c: 0,
            num_tile_columns: 0,
            num_tile_rows: 0,
            num_tiles_in_pic: 0,
            curr_pic_scal_win_width_l: 0,
            curr_pic_scal_win_height_l: 0,
            num_ctus_in_slice: vec![],
            num_ctus_in_curr_slice: 0,
            slice_top_left_tile_idx: vec![],
            num_slices_in_tile: vec![],
            ctb_addr_in_slice: vec![],
            row_height_val: vec![],
            num_ltrp_entries: vec![],
            rpls_idx: vec![],
            first_ctb_row_in_slice: false,
            ctb_addr_in_rs: 0,
            ctb_addr_in_curr_slice: vec![],
            ctb_addr_x: 0,
            ctb_addr_y: 0,
            ctb_to_tile_col_bd: vec![],
            num_hmvp_cand: 0,
            num_hmvp_ibc_cand: 0,
            reset_ibc_buf: false,
            ctb_log2_size_y: 0,
            predictor_palette_size: [0; 2],
            slice_qp_y: 26,
            table_palette_size_wpp: vec![],
            table_palette_entries_wpp: vec![],
            predictor_palette_entries: vec![],
            zero_pos: vec![0; 64 * 64],
            abs_level: vec2d![0; 64; 64],
            q_state: 0,
            max_palette_index: 0,
            cu_pred_mode: vec![],
            is_available: vec![],
            cqt_depth: vec![],
            cb_width: vec![],
            cb_height: vec![],
            cu_skip_flag: vec![],
            intra_mip_flag: vec![],
            merge_subblock_flag: vec![],
            inter_affine_flag: vec![],
            bdpcm_flag: vec![],
            intra_subpartitions_split_type: IntraSubpartitionsSplitType::ISP_NO_SPLIT,
            abs_level_pass1: vec2d![0; 64; 64],
            abs_level_pass2: vec2d![0; 64; 64],
            last_significant_coeff_x: 0,
            last_significant_coeff_y: 0,
            coeff_sign_level: vec![vec![0; 64]; 64],
            no_backward_pred_flag: false,
            min_qt_log2_size_y: 0,
            min_qt_log2_size_c: 0,
            min_qt_size_y: 0,
            min_qt_size_c: 0,
            min_bt_size_y: 0,
            min_tt_size_y: 0,
            mtt_split_mode: vec![],
            max_mtt_depth_y: 0,
            max_mtt_depth_c: 0,
            max_qt_size_y: 0,
            max_bt_size_y: 0,
            max_tt_size_y: 0,
            max_qt_size_c: 0,
            max_bt_size_c: 0,
            max_tt_size_c: 0,
            bit_depth: 8,
            num_ref_idx_active: [0; 2],
            max_num_subblock_merge_cand: 0,
            max_num_ibc_merge_cand: 0,
            max_num_gpm_merge_cand: 0,
            sao_type_idx: vec![],
            is_cu_qp_delta_coded: false,
            cu_qp_delta_val: 0,
            cu_qg_top_left_x: 0,
            cu_qg_top_left_y: 0,
            is_cu_chroma_qp_offset_coded: false,
            cu_qp_offset_cb: 0,
            cu_qp_offset_cr: 0,
            cu_qp_offset_cbcr: 0,
            cu_qp_delta_sub_div: 0,
            cu_chroma_qp_offset_subdiv: 0,
            mode_type_condition: 0,
            max_ts_size: 0,
            cclm_enabled: true,
            mvd_l0: (0, 0),
            mvd_l1: (0, 0),
            ref_idx_sym_l0: 0,
            ref_idx_sym_l1: 0,
            motion_model_idc: 0,
            mvd_cp_l0: [(0, 0); 3],
            mvd_cp_l1: [(0, 0); 3],
            lfnst_dc_only: false,
            lfnst_zero_out_sig_coeff_flag: false,
            mts_dc_only: false,
            mts_zero_out_sig_coeff_flag: false,
            num_intra_subpartitions: 0,
            infer_tu_cbf_luma: false,
            cb_pos_x: vec![],
            cb_pos_y: vec![],
            q_state_trans_table: [[0, 2], [2, 0], [1, 3], [3, 1]],
            trans_coeff_level: vec![],
            rem_ccbs: 0,
            num_slices_in_subpic: vec![],
            curr_subpic_idx: 0,
            num_weights_l0: 0,
            num_weights_l1: 0,
            num_alf_filters: 0,
            scaling_list: vec![],
            num_entry_points: 0,
            scaling_matrix_rec: vec![],
            scaling_matrix_dc_rec: vec![],
            apply_lfnst_flag: [false; 3],
            qp_y: 26,
            fixed_qp: None,
            max_split_depth: 0,
            extra_params: hashmap![],
            enable_print: false,
        }
    }

    pub fn update_from_vps(&mut self, vps: &VideoParameterSet) {
        self.vps_num_dpb_params = if vps.each_layer_is_an_ols {
            0
        } else {
            vps.dpb_parameters.len()
        };

        let ols_mode_idc = if !vps.each_layer_is_an_ols {
            vps.get_ols_mode_idc()
        } else {
            4
        };
        self.total_num_olss = match ols_mode_idc {
            4 | 0 | 1 => vps.max_layers,
            2 => vps.num_output_layer_sets,
            _ => panic!(),
        };
        //println!("total num olss {}", self.total_num_olss);
        self.num_output_layers_in_ols = vec![0; self.total_num_olss];
        self.output_layer_id_in_ols = vec![vec![0]; self.total_num_olss];

        self.num_output_layers_in_ols[0] = 1;
        self.output_layer_id_in_ols[0][0] = vps.layers[0].id;
        self.num_sub_layers_in_layer_in_ols = vec![vec![]; self.total_num_olss];
        self.num_sub_layers_in_layer_in_ols[0] = vec![0];
        self.num_sub_layers_in_layer_in_ols[0][0] = vps.ptl_max_tids[0] + 1;
        self.layer_used_as_ref_layer_flag = vec![false; vps.max_layers];
        self.layer_used_as_ref_layer_flag[0] = true;
        for i in 1..vps.max_layers {
            self.layer_used_as_ref_layer_flag[i] = match ols_mode_idc {
                4 | 0 | 1 => true,
                2 => false,
                _ => self.layer_used_as_ref_layer_flag[i],
            }
        }
        let mut layer_included_in_ols_flag = vec![vec![false; vps.max_layers]; vps.max_layers];
        self.output_layer_idx = vec![vec![]; self.total_num_olss];
        self.num_ref_layers = vec![1, 2];
        for (i, liiof) in layer_included_in_ols_flag
            .iter_mut()
            .enumerate()
            .take(self.total_num_olss)
            .skip(1)
        {
            match ols_mode_idc {
                4 | 0 => {
                    self.num_output_layers_in_ols[i] = 1;
                    self.output_layer_id_in_ols[i][0] = vps.layers[i].id;
                    if vps.each_layer_is_an_ols {
                        self.num_sub_layers_in_layer_in_ols[i] = vec![0];
                        self.num_sub_layers_in_layer_in_ols[i][0] = vps.ptl_max_tids[i] + 1;
                    } else {
                        self.num_sub_layers_in_layer_in_ols[i] = vec![0; i + 1];
                        self.num_sub_layers_in_layer_in_ols[i][i] = vps.ptl_max_tids[i] + 1;
                        for k in (0..i).rev() {
                            self.num_sub_layers_in_layer_in_ols[i][k] = 0;
                            for m in k + 1..=i {
                                let max_sublayer_needed = std::cmp::min(
                                    self.num_sub_layers_in_layer_in_ols[i][m],
                                    vps.layers[m].max_tid_il_ref_pics[k],
                                );
                                if vps.layers[m].direct_ref_layer_flag[k]
                                    && self.num_sub_layers_in_layer_in_ols[i][k]
                                        < max_sublayer_needed
                                {
                                    self.num_sub_layers_in_layer_in_ols[i][k] = max_sublayer_needed;
                                }
                            }
                        }
                    }
                }
                1 => {
                    self.num_output_layers_in_ols[i] = i + 1;
                    self.num_sub_layers_in_layer_in_ols[i] = vec![0; i + 1];
                    self.output_layer_id_in_ols[i] = vec![0; self.num_output_layers_in_ols[i]];
                    for j in 0..self.num_output_layers_in_ols[i] {
                        self.output_layer_id_in_ols[i][j] = vps.layers[j].id;
                        self.num_sub_layers_in_layer_in_ols[i][j] =
                            vps.ptl_max_tids[vps.ols_ptl_idx[i]] + 1;
                    }
                }
                2 => {
                    self.num_sub_layers_in_layer_in_ols[i] = vec![0; vps.max_layers];
                    for j in 0..vps.max_layers {
                        self.num_sub_layers_in_layer_in_ols[i][j] = 0;
                    }
                    let mut highest_included_layer = 0;
                    self.output_layer_idx[i] = vec![0; vps.max_layers];
                    self.output_layer_id_in_ols[i] = vec![0; vps.max_layers];
                    let mut j = 0;
                    for (k, flag) in liiof.iter_mut().enumerate().take(vps.max_layers) {
                        if vps.ols_output_layer_flags[i][k] {
                            *flag = true;
                            highest_included_layer = k;
                            self.layer_used_as_ref_layer_flag[k] = true;
                            self.output_layer_idx[i][j] = k;
                            self.output_layer_id_in_ols[i][j] = vps.layers[i].id;
                            self.num_sub_layers_in_layer_in_ols[i][k] =
                                vps.ptl_max_tids[vps.ols_ptl_idx[i]] + 1;
                            j += 1;
                        }
                    }
                    self.num_output_layers_in_ols[i] = j;
                    self.output_layer_idx[i] = vec![0; self.num_output_layers_in_ols[i]];
                    self.referenced_layer_idx = vec![vec![0, 1], vec![0, 1]];
                    for j in 0..self.num_output_layers_in_ols[i] {
                        let idx = self.output_layer_idx[i][j];
                        for k in 0..self.num_ref_layers[idx] {
                            if !liiof[self.referenced_layer_idx[idx][k]] {
                                liiof[self.referenced_layer_idx[idx][k]] = true;
                            }
                        }
                    }
                    for k in (0..highest_included_layer).rev() {
                        if liiof[k] && !vps.ols_output_layer_flags[i][k] {
                            for (m, flag) in liiof
                                .iter()
                                .enumerate()
                                .take(highest_included_layer + 1)
                                .skip(k + 1)
                            {
                                let max_sublayer_needed = std::cmp::min(
                                    self.num_sub_layers_in_layer_in_ols[i][m],
                                    vps.layers[m].max_tid_il_ref_pics[k],
                                );
                                if vps.layers[m].direct_ref_layer_flag[k]
                                    && *flag
                                    && self.num_sub_layers_in_layer_in_ols[i][k]
                                        < max_sublayer_needed
                                {
                                    self.num_sub_layers_in_layer_in_ols[i][k] = max_sublayer_needed;
                                }
                            }
                        }
                    }
                }
                _ => panic!(),
            }
        }

        self.num_layers_in_ols = vec![0; self.total_num_olss];
        self.num_layers_in_ols[0] = 1;
        self.layer_id_in_ols = vec![vec![]; self.total_num_olss];
        self.layer_id_in_ols[0] = vec![0];
        self.layer_id_in_ols[0][0] = vps.layers[0].id;
        self.num_multi_layer_olss = 0;
        self.multi_layer_ols_idx = vec![0; self.total_num_olss];
        for (i, liiof) in layer_included_in_ols_flag
            .iter()
            .enumerate()
            .take(self.total_num_olss)
            .skip(1)
        {
            if vps.each_layer_is_an_ols {
                self.num_layers_in_ols[i] = 1;
                self.layer_id_in_ols[i] = vec![0];
                self.layer_id_in_ols[i][0] = vps.layers[i].id;
            } else {
                match ols_mode_idc {
                    0 | 1 => {
                        self.num_layers_in_ols[i] = i + 1;
                        self.layer_id_in_ols[i] = vec![0; self.num_layers_in_ols[i]];
                        for j in 0..self.num_layers_in_ols[i] {
                            self.layer_id_in_ols[i][j] = vps.layers[j].id;
                        }
                    }
                    2 => {
                        let mut j = 0;
                        self.layer_id_in_ols[i] = vec![0; vps.max_layers];
                        for (k, flag) in liiof.iter().enumerate().take(vps.max_layers) {
                            if *flag {
                                self.layer_id_in_ols[i][j] = vps.layers[k].id;
                                j += 1;
                            }
                            self.num_layers_in_ols[i] = j;
                        }
                    }
                    _ => panic!(),
                }
            }
            if self.num_layers_in_ols[i] > 1 {
                self.multi_layer_ols_idx[i] = self.num_multi_layer_olss;
                self.num_multi_layer_olss += 1;
            }
        }
        debug_eprintln!("num multi layer olss {}", self.num_multi_layer_olss);
    }

    pub fn update_from_sps(&mut self, sps: &SequenceParameterSet) {
        self.max_pic_order_cnt_lsb = 1 << sps.log2_max_pic_order_cnt_lsb;

        self.qp_prime_ts_min = 4 + 6 * sps.min_qp_prime_ts;
        self.bit_depth = sps.bitdepth;

        self.max_ts_size = 1 << sps.log2_transform_skip_max_size;

        self.num_extra_ph_bits = 0;
        for i in 0..sps.num_extra_ph_bytes * 8 {
            if sps.extra_ph_bit_present_flags[i] {
                self.num_extra_ph_bits += 1;
            }
        }

        self.num_extra_sh_bits = 0;
        for i in 0..sps.num_extra_sh_bytes * 8 {
            if sps.extra_sh_bit_present_flags[i] {
                self.num_extra_sh_bits += 1;
            }
        }

        self.ctb_log2_size_y = sps.log2_ctu_size;
        self.ctb_size_y = 1 << self.ctb_log2_size_y;

        self.min_cb_log2_size_y = sps.log2_min_luma_coding_block_size;
        self.min_cb_size_y = 1 << self.min_cb_log2_size_y;
        self.ibc_buf_width_y = 256 * 128 / self.ctb_size_y;
        self.ibc_buf_width_c = self.ibc_buf_width_y / self.sub_width_c;
        self.v_size = std::cmp::min(64, self.ctb_size_y);

        match sps.chroma_format {
            ChromaFormat::Monochrome => {
                self.ctb_width_c = 0;
                self.ctb_height_c = 0;
            }
            _ => {
                self.ctb_width_c = self.ctb_size_y / self.sub_width_c;
                self.ctb_height_c = self.ctb_size_y / self.sub_height_c;
            }
        }

        self.min_qt_log2_size_intra_y = sps
            .partition_constraints
            .log2_diff_min_qt_min_cb_intra_slice_luma
            + self.min_cb_log2_size_y;
        self.min_qt_log2_size_inter_y = sps
            .partition_constraints
            .log2_diff_min_qt_min_cb_inter_slice
            + self.min_cb_log2_size_y;

        self.min_tb_log2_size_y = 2;
        self.max_tb_log2_size_y = if sps.max_luma_transform_size_64_flag {
            6
        } else {
            5
        };
        self.min_tb_size_y = 1 << self.min_tb_log2_size_y;
        self.max_tb_size_y = 1 << self.max_tb_log2_size_y;

        // FIXME possibly iterated by negative indices when qp_bd_offset>0
        self.chroma_qp_table = vec![vec![0; 64]; sps.num_qp_tables];

        for i in 0..sps.num_qp_tables {
            // FIXME
            let mut qp_in_val = vec![0isize; sps.qp_tables[i].num_points_in_qp_table + 1];
            qp_in_val[0] = sps.qp_tables[i].qp_table_start;
            let mut qp_out_val = qp_in_val.clone();
            for j in 0..sps.qp_tables[i].num_points_in_qp_table {
                qp_in_val[j + 1] = qp_in_val[j] + sps.qp_tables[i].delta_qp_in_val[j];
                qp_out_val[j + 1] = qp_out_val[j]
                    + ((sps.qp_tables[i].delta_qp_in_val[j] - 1)
                        ^ sps.qp_tables[i].delta_qp_diff_val[j]);
            }
            debug_eprintln!("in val {:?}", qp_in_val);
            debug_eprintln!("out val {:?}", qp_out_val);
            self.chroma_qp_table[i][qp_in_val[0] as usize] = qp_out_val[0];
            for k in (-self.qp_bd_offset..qp_in_val[0]).rev() {
                self.chroma_qp_table[i][k as usize] = num::clamp(
                    self.chroma_qp_table[i][k as usize + 1] - 1,
                    -self.qp_bd_offset,
                    63,
                );
            }
            for j in 0..sps.qp_tables[i].num_points_in_qp_table {
                let sh = sps.qp_tables[i].delta_qp_in_val[j] >> 1;
                let mut m = 1;
                for k in qp_in_val[j] + 1..=qp_in_val[j + 1] {
                    self.chroma_qp_table[i][k as usize] = self.chroma_qp_table[i]
                        [qp_in_val[j] as usize]
                        + ((qp_out_val[j + 1] - qp_out_val[j]) * m + sh)
                            / (sps.qp_tables[i].delta_qp_in_val[j]);
                    m += 1;
                }
                for k in qp_in_val[sps.qp_tables[i].num_points_in_qp_table] + 1..=63 {
                    self.chroma_qp_table[i][k as usize] = num::clamp(
                        self.chroma_qp_table[i][k as usize - 1] + 1,
                        -self.qp_bd_offset,
                        63,
                    );
                }
            }
        }
        debug_eprintln!("chroma qp table");
        debug_eprintln!("{:?}", self.chroma_qp_table);
        //panic!();

        if let Some(ladf_parameters) = &sps.ladf_parameters {
            for i in 0..ladf_parameters.num_ladf_intervals - 1 {
                self.sps_ladf_interval_lower_bound[i + 1] =
                    self.sps_ladf_interval_lower_bound[i] + ladf_parameters.delta_threshold[i];
            }
        }
        //else {
        //panic!();
        //}

        self.max_num_merge_cand = 6 - sps.six_minus_max_num_merge_cand;
    }

    pub fn update_from_sps_and_pps(
        &mut self,
        sps: &SequenceParameterSet,
        pps: &PictureParameterSet,
    ) {
        if let Some(sps_subpic_info) = &sps.subpic_info {
            for i in 0..sps_subpic_info.num_subpics {
                if sps_subpic_info.subpic_id_mapping_present_flag {
                    self.subpic_id_val[i] = if pps.subpic_id_mapping_present_flag {
                        pps.subpic_id[i]
                    } else {
                        sps_subpic_info.subpic_id[i]
                    };
                }
            }
        }
        //else {
        //panic!();
        //}

        if pps.ref_wraparound_enabled_flag {
            self.pps_ref_wraparound_offset = pps.pic_width_in_luma_samples / self.min_cb_size_y
                - pps.pic_width_minus_wraparound_offset;
        }

        self.slice_qp_y = pps.init_qp;
        self.qp_y = self.slice_qp_y as usize;
        assert!(pps.init_qp >= -self.qp_bd_offset && pps.init_qp <= 53);

        self.pic_width_in_ctbs_y =
            (pps.pic_width_in_luma_samples + self.ctb_size_y - 1) / self.ctb_size_y;
        self.pic_height_in_ctbs_y =
            (pps.pic_height_in_luma_samples + self.ctb_size_y - 1) / self.ctb_size_y;
        self.pic_size_in_ctbs_y = self.pic_width_in_ctbs_y * self.pic_height_in_ctbs_y;
        self.pic_width_in_min_cbs_y = pps.pic_width_in_luma_samples / self.min_cb_size_y;
        self.pic_height_in_min_cbs_y = pps.pic_height_in_luma_samples / self.min_cb_size_y;
        self.pic_size_in_min_cbs_y = self.pic_width_in_min_cbs_y * self.pic_height_in_min_cbs_y;
        self.pic_size_in_samples_y = pps.pic_width_in_luma_samples * pps.pic_height_in_luma_samples;
        self.pic_width_in_samples_c = pps.pic_width_in_luma_samples / self.sub_width_c;
        self.pic_height_in_samples_c = pps.pic_height_in_luma_samples / self.sub_height_c;

        self.curr_pic_scal_win_width_l = (pps.pic_width_in_luma_samples as isize
            - self.sub_width_c as isize
                * (pps.scaling_window.left_offset + pps.scaling_window.right_offset))
            as usize;
        self.curr_pic_scal_win_height_l = (pps.pic_height_in_luma_samples as isize
            - self.sub_height_c as isize
                * (pps.scaling_window.bottom_offset + pps.scaling_window.top_offset))
            as usize;

        self.num_tiles_in_pic = self.num_tile_columns * self.num_tile_rows;
    }

    pub fn update_from_ph(&mut self, ph: &PictureHeader, pps: &PictureParameterSet) {
        if pps.partition_parameters.qp_delta_info_in_ph_flag {
            self.slice_qp_y = pps.init_qp + ph.qp_delta;
            self.qp_y = self.slice_qp_y as usize;
        }
    }

    pub fn update_from_sh(&mut self, sh: &SliceHeader, pps: &PictureParameterSet) {
        if !pps.partition_parameters.qp_delta_info_in_ph_flag {
            self.slice_qp_y = pps.init_qp + sh.qp_delta;
            self.qp_y = self.slice_qp_y as usize;
        }
        if sh.slice_type == SliceType::I {
            self.min_qt_log2_size_y = self.min_cb_log2_size_y
                + match &sh.ph.as_ref().unwrap().partition_constraints {
                    Some(pc) => pc.log2_diff_min_qt_min_cb_intra_slice_luma,
                    None => {
                        sh.sps
                            .partition_constraints
                            .log2_diff_min_qt_min_cb_intra_slice_luma
                    }
                };
            self.min_qt_log2_size_c = self.min_cb_log2_size_y
                + match &sh.ph.as_ref().unwrap().partition_constraints {
                    Some(pc) => pc.log2_diff_min_qt_min_cb_intra_slice_chroma,
                    None => {
                        sh.sps
                            .partition_constraints
                            .log2_diff_min_qt_min_cb_intra_slice_chroma
                    }
                };
            self.max_bt_size_y = 1
                << (self.min_qt_log2_size_y
                    + match &sh.ph.as_ref().unwrap().partition_constraints {
                        Some(pc) => pc.log2_diff_max_bt_min_qt_intra_slice_luma,
                        None => {
                            sh.sps
                                .partition_constraints
                                .log2_diff_max_bt_min_qt_intra_slice_luma
                        }
                    });
            self.max_bt_size_c = 1
                << (self.min_qt_log2_size_c
                    + match &sh.ph.as_ref().unwrap().partition_constraints {
                        Some(pc) => pc.log2_diff_max_bt_min_qt_intra_slice_chroma,
                        None => {
                            sh.sps
                                .partition_constraints
                                .log2_diff_max_bt_min_qt_intra_slice_chroma
                        }
                    });
            self.max_tt_size_y = 1
                << (self.min_qt_log2_size_y
                    + match &sh.ph.as_ref().unwrap().partition_constraints {
                        Some(pc) => pc.log2_diff_max_tt_min_qt_intra_slice_luma,
                        None => {
                            sh.sps
                                .partition_constraints
                                .log2_diff_max_tt_min_qt_intra_slice_luma
                        }
                    });
            self.max_tt_size_c = 1
                << (self.min_qt_log2_size_c
                    + match &sh.ph.as_ref().unwrap().partition_constraints {
                        Some(pc) => pc.log2_diff_max_tt_min_qt_intra_slice_chroma,
                        None => {
                            sh.sps
                                .partition_constraints
                                .log2_diff_max_tt_min_qt_intra_slice_chroma
                        }
                    });
            self.max_mtt_depth_y = match &sh.ph.as_ref().unwrap().partition_constraints {
                Some(pc) => pc.max_mtt_hierarchy_depth_intra_slice_luma,
                None => {
                    sh.sps
                        .partition_constraints
                        .max_mtt_hierarchy_depth_intra_slice_luma
                }
            };
            self.max_mtt_depth_c = match &sh.ph.as_ref().unwrap().partition_constraints {
                Some(pc) => pc.max_mtt_hierarchy_depth_intra_slice_chroma,
                None => {
                    sh.sps
                        .partition_constraints
                        .max_mtt_hierarchy_depth_intra_slice_chroma
                }
            };
            self.cu_qp_delta_sub_div = match &sh.ph.as_ref().unwrap().partition_constraints {
                Some(pc) => pc.cu_qp_delta_subdiv_intra_slice,
                None => sh.sps.partition_constraints.cu_qp_delta_subdiv_intra_slice,
            };
            self.cu_chroma_qp_offset_subdiv = match &sh.ph.as_ref().unwrap().partition_constraints {
                Some(pc) => pc.cu_chroma_qp_offset_subdiv_intra_slice,
                None => {
                    sh.sps
                        .partition_constraints
                        .cu_chroma_qp_offset_subdiv_intra_slice
                }
            };
        } else {
            self.min_qt_log2_size_y = self.min_cb_log2_size_y
                + match &sh.ph.as_ref().unwrap().partition_constraints {
                    Some(pc) => pc.log2_diff_min_qt_min_cb_inter_slice,
                    None => {
                        sh.sps
                            .partition_constraints
                            .log2_diff_min_qt_min_cb_inter_slice
                    }
                };
            self.min_qt_log2_size_c = self.min_cb_log2_size_y
                + match &sh.ph.as_ref().unwrap().partition_constraints {
                    Some(pc) => pc.log2_diff_min_qt_min_cb_inter_slice,
                    None => {
                        sh.sps
                            .partition_constraints
                            .log2_diff_min_qt_min_cb_inter_slice
                    }
                };
            self.max_bt_size_y = 1
                << (self.min_qt_log2_size_y
                    + match &sh.ph.as_ref().unwrap().partition_constraints {
                        Some(pc) => pc.log2_diff_max_bt_min_qt_inter_slice,
                        None => {
                            sh.sps
                                .partition_constraints
                                .log2_diff_max_bt_min_qt_inter_slice
                        }
                    });
            self.max_bt_size_c = 1
                << (self.min_qt_log2_size_c
                    + match &sh.ph.as_ref().unwrap().partition_constraints {
                        Some(pc) => pc.log2_diff_max_bt_min_qt_inter_slice,
                        None => {
                            sh.sps
                                .partition_constraints
                                .log2_diff_max_bt_min_qt_inter_slice
                        }
                    });
            self.max_tt_size_y = 1
                << (self.min_qt_log2_size_y
                    + match &sh.ph.as_ref().unwrap().partition_constraints {
                        Some(pc) => pc.log2_diff_max_tt_min_qt_inter_slice,
                        None => {
                            sh.sps
                                .partition_constraints
                                .log2_diff_max_tt_min_qt_inter_slice
                        }
                    });
            self.max_tt_size_c = 1
                << (self.min_qt_log2_size_c
                    + match &sh.ph.as_ref().unwrap().partition_constraints {
                        Some(pc) => pc.log2_diff_max_tt_min_qt_inter_slice,
                        None => {
                            sh.sps
                                .partition_constraints
                                .log2_diff_max_tt_min_qt_inter_slice
                        }
                    });
            self.max_mtt_depth_y = match &sh.ph.as_ref().unwrap().partition_constraints {
                Some(pc) => pc.max_mtt_hierarchy_depth_inter_slice,
                None => {
                    sh.sps
                        .partition_constraints
                        .max_mtt_hierarchy_depth_inter_slice
                }
            };
            self.max_mtt_depth_c = match &sh.ph.as_ref().unwrap().partition_constraints {
                Some(pc) => pc.max_mtt_hierarchy_depth_inter_slice,
                None => {
                    sh.sps
                        .partition_constraints
                        .max_mtt_hierarchy_depth_inter_slice
                }
            };
            self.cu_qp_delta_sub_div = match &sh.ph.as_ref().unwrap().partition_constraints {
                Some(pc) => pc.cu_qp_delta_subdiv_inter_slice,
                None => sh.sps.partition_constraints.cu_qp_delta_subdiv_inter_slice,
            };
            self.cu_chroma_qp_offset_subdiv = match &sh.ph.as_ref().unwrap().partition_constraints {
                Some(pc) => pc.cu_chroma_qp_offset_subdiv_inter_slice,
                None => {
                    sh.sps
                        .partition_constraints
                        .cu_chroma_qp_offset_subdiv_inter_slice
                }
            };
        }
        self.min_qt_size_y = 1 << self.min_qt_log2_size_y;
        self.min_qt_size_c = 1 << self.min_qt_log2_size_c;
        self.min_bt_size_y = 1 << self.min_cb_log2_size_y;
        self.min_tt_size_y = 1 << self.min_cb_log2_size_y;
        assert!(self.slice_qp_y >= -self.qp_bd_offset && self.slice_qp_y <= 63);
    }

    pub fn _update_from_ref_pic_lists(&mut self, _ref_pic_lists: Vec<RefPicList>) {}

    #[inline(always)]
    pub fn derive_neighbouring_block_availability(
        &self,
        x_curr: usize,
        y_curr: usize,
        x_nb_y: isize,
        y_nb_y: isize,
        width: usize,
        height: usize,
        is_above_right_available: bool,
        is_below_left_available: bool,
        check_pred_mode_y: bool,
        sps: &SequenceParameterSet,
        pps: &PictureParameterSet,
    ) -> bool {
        let is_in_same_slice = true; // FIXME
        let is_in_same_tile = true; // FIXME
        let mut available_n = x_nb_y >= 0
            && y_nb_y >= 0
            && x_nb_y < pps.pic_width_in_luma_samples as isize
            && y_nb_y < pps.pic_height_in_luma_samples as isize
            && ((x_nb_y >> self.ctb_log2_size_y) <= (x_curr >> self.ctb_log2_size_y) as isize
                || ((y_nb_y >> self.ctb_log2_size_y) < (y_curr >> self.ctb_log2_size_y) as isize))
            && (y_nb_y >> self.ctb_log2_size_y) < (y_curr >> self.ctb_log2_size_y) as isize + 1
            //&& self.is_available[c_idx][x_nb_y as usize][y_nb_y as usize]
            && (x_nb_y<x_curr as isize + width as isize || is_above_right_available)
            && (y_nb_y<y_curr as isize + height as isize || is_below_left_available)
            && is_in_same_slice
            && is_in_same_tile
            && (!sps.entropy_coding_sync_enabled_flag
                || (x_nb_y >> self.ctb_log2_size_y)
                    < (x_curr >> self.ctb_log2_size_y) as isize + 1);
        if check_pred_mode_y
            && self.cu_pred_mode[0][x_nb_y as usize][y_nb_y as usize]
                != self.cu_pred_mode[0][x_curr][y_curr]
        {
            available_n = false;
        }
        available_n
    }

    pub fn derive_allow_split_qt(
        &self,
        cb_size: usize,
        mtt_depth: usize,
        tree_type: TreeType,
        mode_type: ModeType,
    ) -> bool {
        !(((tree_type == TreeType::SINGLE_TREE || tree_type == TreeType::DUAL_TREE_LUMA)
            && cb_size <= self.min_qt_size_y)
            || (tree_type == TreeType::DUAL_TREE_CHROMA && cb_size <= self.min_qt_size_c)
            || mtt_depth > 0
            || (tree_type == TreeType::DUAL_TREE_CHROMA && (cb_size / self.sub_width_c) <= 4)
            || (tree_type == TreeType::DUAL_TREE_CHROMA && mode_type == ModeType::MODE_TYPE_INTRA))
    }

    pub fn derive_allow_split_bt(
        &self,
        bt_split: MttSplitMode,
        cb_width: usize,
        cb_height: usize,
        x0: usize,
        y0: usize,
        mtt_depth: usize,
        max_mtt_depth: usize,
        max_tt_size: usize,
        min_qt_size: usize,
        part_idx: usize,
        tree_type: TreeType,
        mode_type: ModeType,
        pps: &PictureParameterSet,
    ) -> bool {
        let parallel_tt_split = if bt_split == MttSplitMode::SPLIT_BT_VER {
            MttSplitMode::SPLIT_TT_VER
        } else {
            MttSplitMode::SPLIT_TT_HOR
        };
        let cb_size = if bt_split == MttSplitMode::SPLIT_BT_VER {
            cb_width
        } else {
            cb_height
        };
        !((cb_size <= self.min_bt_size_y
            || cb_width > max_tt_size
            || cb_height > max_tt_size
            || mtt_depth >= max_mtt_depth
            || (tree_type == TreeType::DUAL_TREE_CHROMA
                && (cb_width / self.sub_width_c) * (cb_height / self.sub_height_c) <= 16)
            || (tree_type == TreeType::DUAL_TREE_CHROMA
                && (cb_width / self.sub_width_c) == 4
                && bt_split == MttSplitMode::SPLIT_BT_VER)
            || (tree_type == TreeType::DUAL_TREE_CHROMA && mode_type == ModeType::MODE_TYPE_INTRA)
            || (cb_width * cb_height == 32 && mode_type == ModeType::MODE_TYPE_INTER))
            || (bt_split == MttSplitMode::SPLIT_BT_VER
                && y0 + cb_height > pps.pic_height_in_luma_samples)
            || (bt_split == MttSplitMode::SPLIT_BT_VER
                && cb_height > 64
                && x0 + cb_width > pps.pic_width_in_luma_samples)
            || (bt_split == MttSplitMode::SPLIT_BT_HOR
                && cb_width > 64
                && y0 + cb_height > pps.pic_height_in_luma_samples)
            || (x0 + cb_width > pps.pic_width_in_luma_samples
                && y0 + cb_height > pps.pic_height_in_luma_samples
                && cb_width > min_qt_size)
            || (bt_split == MttSplitMode::SPLIT_BT_HOR
                && x0 + cb_width > pps.pic_width_in_luma_samples
                && y0 + cb_height <= pps.pic_height_in_luma_samples)
            || (mtt_depth > 0
                && part_idx == 1
                && self.mtt_split_mode[x0][y0][mtt_depth - 1] == parallel_tt_split)
            || (bt_split == MttSplitMode::SPLIT_BT_VER && cb_width <= 64 && cb_height <= 64)
            || (bt_split == MttSplitMode::SPLIT_BT_HOR && cb_width > 64 && cb_height <= 64))
    }

    pub fn _derive_allow_split_tt(
        &self,
        tt_split: MttSplitMode,
        cb_width: usize,
        cb_height: usize,
        x0: usize,
        y0: usize,
        max_tt_size: usize,
        tree_type: TreeType,
        mode_type: ModeType,
        pps: &PictureParameterSet,
    ) -> bool {
        let cb_size = if tt_split == MttSplitMode::SPLIT_TT_VER {
            cb_width
        } else {
            cb_height
        };
        cb_size > 2 * self.min_tt_size_y
            && cb_width <= max_tt_size.min(64)
            && cb_height <= max_tt_size.min(64)
            && x0 + cb_width <= pps.pic_width_in_luma_samples
            && y0 + cb_height <= pps.pic_height_in_luma_samples
            && (tree_type != TreeType::DUAL_TREE_CHROMA
                || (cb_width / self.sub_width_c) * (cb_height / self.sub_height_c) > 32)
            && (tree_type != TreeType::DUAL_TREE_CHROMA
                || cb_width / self.sub_width_c != 8
                || tt_split != MttSplitMode::SPLIT_TT_VER)
            && (tree_type != TreeType::DUAL_TREE_CHROMA || mode_type != ModeType::MODE_TYPE_INTRA)
            && (cb_width * cb_height != 64 || mode_type != ModeType::MODE_TYPE_INTER)
    }
}

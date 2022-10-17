use super::bins::*;
use super::block_splitter::*;
use super::bool_coder::*;
use super::cabac_contexts::*;
use super::common::*;
use super::ctu::*;
use super::encoder_context::*;
use super::intra_predictor::*;
use super::quantizer::*;
use super::slice_header::*;
use super::transformer::*;
use debug_print::*;
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::{Arc, Mutex};

pub struct CtuEncoder<'a> {
    coder: &'a mut BoolCoder,
    encoder_context: Arc<Mutex<EncoderContext>>,
}

impl<'a> CtuEncoder<'a> {
    pub fn new(
        encoder_context: &Arc<Mutex<EncoderContext>>,
        coder: &'a mut BoolCoder,
    ) -> CtuEncoder<'a> {
        CtuEncoder {
            coder,
            encoder_context: encoder_context.clone(),
        }
    }

    pub fn encode(&mut self, bins: &mut Bins, ctu: Arc<Mutex<CodingTreeUnit>>, sh: &SliceHeader) {
        debug_eprintln!("start ctu");
        {
            let mut ct = {
                let ctu = ctu.lock().unwrap();
                let first = ctu.x == 0 && ctu.y == 0;
                let ectx = &self.encoder_context;
                //// FIXME
                //let ectx = &mut ectx.lock().unwrap();
                //ectx.cu_qg_top_left_x = x;
                //ectx.cu_qg_top_left_y = y;
                if first || (sh.sps.entropy_coding_sync_enabled_flag && ctu.x == ctu.x_tile) {
                    self.coder
                        .init_cabac(first, &ctu, sh.sps, sh.pps, ectx.clone());
                }
                ctu.ct[0].clone()
            };
            {
                let ectx = &self.encoder_context;
                let mut ectx = ectx.lock().unwrap();
                let mut block_splitter = BlockSplitter::new(&ectx);
                block_splitter.split_ct(&mut ct, ectx.max_split_depth, sh, &mut ectx);
            }
        }
        let ectx = self.encoder_context.clone();
        {
            let ctu = &ctu.lock().unwrap();
            let mut ectx = ectx.lock().unwrap();
            if sh.sao_luma_used_flag || sh.sao_chroma_used_flag {
                self.encode_sao(bins, ctu.sao.clone(), sh, ectx.ctb_addr_x, ectx.ctb_addr_y);
            }
            if sh.alf_enabled_flag {
                debug_eprintln!("ctu alf_ctb_flag ");
                self.coder.encode_cabac_ctu(
                    bins,
                    ctu.alf.ctb_flag[0] as usize,
                    CabacContext::AlfCtbFlag,
                    sh,
                    &mut ectx,
                );
                if ctu.alf.ctb_flag[0] {
                    if sh.alf_info.num_alf_aps_ids_luma > 0 {
                        debug_eprintln!("ctu alf_use_aps_flag ");
                        self.coder.encode_cabac_ctu(
                            bins,
                            ctu.alf.use_aps_flag as usize,
                            CabacContext::AlfUseApsFlag,
                            sh,
                            &mut ectx,
                        );
                    }
                    if ctu.alf.use_aps_flag {
                        if sh.alf_info.num_alf_aps_ids_luma > 1 {
                            debug_eprintln!("ctu alf_luma_prev_filter_idx ");
                            self.coder.encode_cabac_ctu(
                                bins,
                                ctu.alf.luma_prev_filter_idx,
                                CabacContext::AlfLumaPrevFilterIdx,
                                sh,
                                &mut ectx,
                            );
                        } else {
                            debug_eprintln!("ctu alf_luma_fixed_filter_idx ");
                            self.coder.encode_cabac_ctu(
                                bins,
                                ctu.alf.luma_fixed_filter_idx,
                                CabacContext::AlfLumaFixedFilterIdx,
                                sh,
                                &mut ectx,
                            );
                        }
                    }
                    if sh.alf_info.cb_enabled_flag {
                        debug_eprintln!("ctu alf_ctb_flag ");
                        self.coder.encode_cabac_ctu(
                            bins,
                            ctu.alf.ctb_flag[1] as usize,
                            CabacContext::AlfCtbFlag,
                            sh,
                            &mut ectx,
                        );
                        if ctu.alf.ctb_flag[1]
                            && sh.aps[0].alf_data.as_ref().unwrap().chroma_num_alt_filters > 1
                        {
                            debug_eprintln!("ctu alf_ctb_filter_alt_idx ");
                            self.coder.encode_cabac_ctu(
                                bins,
                                ctu.alf.ctb_filter_alt_idx[0],
                                CabacContext::AlfCtbFilterAltIdx,
                                sh,
                                &mut ectx,
                            );
                        }
                    }
                    if sh.alf_info.cr_enabled_flag {
                        debug_eprintln!("ctu alf_ctb_flag ");
                        self.coder.encode_cabac_ctu(
                            bins,
                            ctu.alf.ctb_flag[2] as usize,
                            CabacContext::AlfCtbFlag,
                            sh,
                            &mut ectx,
                        );
                        if ctu.alf.ctb_flag[2]
                            && sh.aps[0].alf_data.as_ref().unwrap().chroma_num_alt_filters > 1
                        {
                            debug_eprintln!("ctu alf_ctb_filter_alt_idx ");
                            self.coder.encode_cabac_ctu(
                                bins,
                                ctu.alf.ctb_filter_alt_idx[1],
                                CabacContext::AlfCtbFilterAltIdx,
                                sh,
                                &mut ectx,
                            );
                        }
                    }
                }
            }
            if sh.alf_info.cc_cb_enabled_flag {
                debug_eprintln!("ctu alf_ctb_cc_cb_idc_bits ");
                self.coder.encode_cabac_ctu(
                    bins,
                    ctu.alf.ctb_cc_cb_idc,
                    CabacContext::AlfCtbCcCbIdc,
                    sh,
                    &mut ectx,
                );
            }
            if sh.alf_info.cc_cr_enabled_flag {
                debug_eprintln!("ctu alf_ctb_cc_cr_idc_bits ");
                self.coder.encode_cabac_ctu(
                    bins,
                    ctu.alf.ctb_cc_cr_idc,
                    CabacContext::AlfCtbCcCrIdc,
                    sh,
                    &mut ectx,
                );
            }
        }
        if sh.slice_type == SliceType::I && sh.sps.partition_constraints.qtbtt_dual_tree_intra_flag
        {
            let dtiqs = {
                let ctu = ctu.lock().unwrap();
                ctu.dtiqs[0].clone()
            };
            self.encode_dual_tree_implicit_qt_split(bins, ctu.clone(), dtiqs, sh);
        } else {
            // luma
            let ct = {
                let ctu = ctu.lock().unwrap();
                ctu.ct[0].clone()
            };
            self.encode_coding_tree(bins, ctu.clone(), ct, sh);
        }
        {
            let ctb_addr_x = {
                let ctu = ctu.lock().unwrap();
                ctu.x / ctu.width
            };
            let ctb_to_tile_col_bd = {
                let ctu = ctu.lock().unwrap();
                let tile = ctu.tile.as_ref().unwrap();
                let tile = tile.lock().unwrap();
                tile.ctu_col
            };
            if sh.sps.entropy_coding_sync_enabled_flag && ctb_addr_x == ctb_to_tile_col_bd {
                self.coder.storage_ctx_table();
            }
        }
    }

    pub fn encode_dual_tree_implicit_qt_split(
        &mut self,
        bins: &mut Bins,
        ctu: Arc<Mutex<CodingTreeUnit>>,
        dtiqs: Arc<Mutex<DTIQS>>,
        sh: &SliceHeader,
    ) {
        let dtiqss = {
            let dtqis = dtiqs.lock().unwrap();
            dtqis.dtiqss.clone()
        };
        for dtiqs in dtiqss.iter() {
            self.encode_dual_tree_implicit_qt_split(bins, ctu.clone(), dtiqs.clone(), sh);
        }
        let cts = {
            let dtqis = dtiqs.lock().unwrap();
            dtqis.cts.clone()
        };
        for ct in cts.iter() {
            self.encode_coding_tree(bins, ctu.clone(), ct.clone(), sh);
        }
    }

    pub fn encode_coding_tree(
        &mut self,
        bins: &mut Bins,
        ctu: Arc<Mutex<CodingTreeUnit>>,
        ct: Arc<Mutex<CodingTree>>,
        sh: &SliceHeader,
    ) {
        debug_eprintln!("start coding tree");
        let ectx = &self.encoder_context;
        {
            let mut ectx = ectx.lock().unwrap();
            let ct = &ct.lock().unwrap();

            let (
                x,
                y,
                width,
                height,
                allow_split_bt_ver,
                allow_split_bt_hor,
                allow_split_tt_ver,
                allow_split_tt_hor,
                allow_split_qt,
                split_cu_flag,
                split_qt_flag,
                qg_on_y,
                qg_on_c,
                cb_subdiv,
                mtt_split_cu_vertical_flag,
                mtt_split_cu_binary_flag,
                mode_type,
                non_inter_flag,
                split_mode,
            ) = {
                (
                    ct.x,
                    ct.y,
                    ct.width,
                    ct.height,
                    ct.allow_split_bt(MttSplitMode::SPLIT_BT_VER, sh.pps, &ectx),
                    ct.allow_split_bt(MttSplitMode::SPLIT_BT_HOR, sh.pps, &ectx),
                    ct.allow_split_tt(MttSplitMode::SPLIT_TT_VER, sh.pps, &ectx),
                    ct.allow_split_tt(MttSplitMode::SPLIT_TT_HOR, sh.pps, &ectx),
                    ct.allow_split_qt(&ectx),
                    ct.get_split_cu_flag(),
                    ct.get_split_qt_flag(),
                    ct.qg_on_y,
                    ct.qg_on_c,
                    ct.get_cb_subdiv(),
                    ct.mtt_split_cu_vertical_flag(),
                    ct.mtt_split_cu_binary_flag(),
                    ct.mode_type,
                    ct.non_inter_flag,
                    ct.split_mode,
                )
            };

            if (allow_split_bt_ver || allow_split_bt_hor || allow_split_qt)
                && y + height <= sh.pps.pic_height_in_luma_samples
            {
                debug_eprintln!("ct split_cu_flag ");
                self.coder.encode_cabac_ct(
                    bins,
                    split_cu_flag as usize,
                    CabacContext::SplitCuFlag,
                    ct,
                    sh,
                    &mut ectx,
                );
            }
            debug_eprintln!("update is_cu_qp_delta_coded");
            debug_eprintln!(
                "cu_qp_delta_enabled_flag={}, qg_on_y={}, cb_subdiv={}, cu_qp_delta_sub_div={}",
                sh.pps.cu_qp_delta_enabled_flag,
                qg_on_y,
                cb_subdiv,
                ectx.cu_qp_delta_sub_div
            );
            if sh.pps.cu_qp_delta_enabled_flag && qg_on_y && cb_subdiv <= ectx.cu_qp_delta_sub_div {
                ectx.is_cu_qp_delta_coded = false;
                ectx.cu_qp_delta_val = 0;
                ectx.cu_qg_top_left_x = x;
                ectx.cu_qg_top_left_y = y;
            }
            if sh.cu_chroma_qp_offset_enabled_flag
                && qg_on_c
                && cb_subdiv <= ectx.cu_chroma_qp_offset_subdiv
            {
                ectx.is_cu_chroma_qp_offset_coded = false;
                ectx.cu_qp_offset_cb = 0;
                ectx.cu_qp_offset_cr = 0;
                ectx.cu_qp_offset_cbcr = 0;
            }
            if split_cu_flag {
                if (allow_split_bt_ver
                    || allow_split_bt_hor
                    || allow_split_tt_ver
                    || allow_split_tt_hor)
                    && allow_split_qt
                {
                    debug_eprintln!("ct split_qt_flag ");
                    self.coder.encode_cabac_ct(
                        bins,
                        split_qt_flag as usize,
                        CabacContext::SplitQtFlag,
                        ct,
                        sh,
                        &mut ectx,
                    );
                }
                if !split_qt_flag {
                    if allow_split_bt_ver
                        || allow_split_bt_hor
                        || allow_split_tt_ver
                        || allow_split_tt_hor
                    {
                        debug_eprintln!("ct mtt_split_cu_vertical_flag ");
                        self.coder.encode_cabac_ct(
                            bins,
                            mtt_split_cu_vertical_flag as usize,
                            CabacContext::MttSplitCuVerticalFlag,
                            ct,
                            sh,
                            &mut ectx,
                        );
                    }
                    if (allow_split_bt_hor || allow_split_tt_hor)
                        && (allow_split_bt_ver || allow_split_tt_ver)
                    {
                        debug_eprintln!("ct mtt_split_cu_binary_flag ");
                        self.coder.encode_cabac_ct(
                            bins,
                            mtt_split_cu_binary_flag as usize,
                            CabacContext::MttSplitCuBinaryFlag,
                            ct,
                            sh,
                            &mut ectx,
                        );
                    }
                }
                ectx.mode_type_condition = if (sh.slice_type == SliceType::I
                    && sh.sps.partition_constraints.qtbtt_dual_tree_intra_flag)
                    || mode_type != ModeType::MODE_TYPE_ALL
                    || sh.sps.chroma_format == ChromaFormat::Monochrome
                    || sh.sps.chroma_format == ChromaFormat::YCbCr444
                {
                    0
                } else if (width * height == 64 && (split_qt_flag)
                    || (!split_qt_flag && split_mode.is_tt()))
                    || (width * height == 32 && split_mode.is_bt())
                {
                    1
                } else if (width * height == 64
                    && (split_mode == MttSplitMode::SPLIT_BT_HOR
                        || split_mode == MttSplitMode::SPLIT_BT_VER)
                    && sh.sps.chroma_format == ChromaFormat::YCbCr420)
                    || (width * height == 128
                        && split_mode.is_tt()
                        && sh.sps.chroma_format == ChromaFormat::YCbCr420)
                    || (width == 8
                        && split_mode == MttSplitMode::SPLIT_BT_VER
                        && sh.sps.chroma_format == ChromaFormat::YCbCr420)
                    || (width == 16 && !split_qt_flag && split_mode == MttSplitMode::SPLIT_TT_VER)
                {
                    1 + (sh.slice_type != SliceType::I) as usize
                } else {
                    0
                };
                if ectx.mode_type_condition == 1 {
                    //assert_eq!(mode_type, ModeType::MODE_TYPE_INTRA);
                } else if ectx.mode_type_condition == 2 {
                    debug_eprintln!("ct non_inter_flag ");
                    self.coder.encode_cabac_ct(
                        bins,
                        non_inter_flag as usize,
                        CabacContext::NonInterFlag,
                        ct,
                        sh,
                        &mut ectx,
                    );
                    assert_eq!(
                        mode_type,
                        if non_inter_flag {
                            ModeType::MODE_TYPE_INTRA
                        } else {
                            ModeType::MODE_TYPE_INTER
                        }
                    );
                } else {
                    //assert_eq!(mode_type, prev_mode_type);
                };
            }
        }
        // TODO?
        let cts = {
            let ct = ct.lock().unwrap();
            ct.cts.clone()
        };
        if !cts.is_empty() {
            for ct in cts.iter() {
                self.encode_coding_tree(bins, ctu.clone(), ct.clone(), sh);
            }
        } else {
            let cus = {
                let ct = ct.lock().unwrap();
                ct.cus.clone()
            };
            for cu in cus.iter() {
                self.encode_coding_unit(bins, cu.clone(), sh);
            }
        }
    }

    pub fn encode_coding_unit(
        &mut self,
        bins: &mut Bins,
        cu: Arc<Mutex<CodingUnit>>,
        sh: &SliceHeader,
    ) {
        debug_eprintln!("encode cu");

        let (
            x,
            y,
            width,
            height,
            mode_type,
            tree_type,
            skip_flag,
            pred_mode,
            pred_mode_flag,
            pred_mode_ibc_flag,
            pred_mode_plt_flag,
            cu_act_enabled_flag,
            intra_bdpcm_luma_flag,
            intra_bdpcm_luma_dir_flag,
            intra_bdpcm_chroma_flag,
            intra_bdpcm_chroma_dir_flag,
            intra_mip_flag,
            intra_mip_transposed_flag,
            intra_mip_mode,
            intra_luma_ref_idx,
            intra_subpartitions_mode_flag,
            intra_subpartitions_split_flag,
            intra_luma_not_planar_flag,
            intra_chroma_pred_mode,
            general_merge_flag,
            mvd_coding,
            mvp_l0_flag,
            mvp_l1_flag,
            amvr_precision_idx,
            inter_pred_idc,
            inter_affine_flag,
            affine_type_flag,
            sym_mvd_flag,
            ref_idx,
            amvr_flag,
            coded_flag,
            merge_data,
            sbt_flag,
            sbt_quad_flag,
            sbt_horizontal_flag,
            sbt_pos_flag,
            transform_tree,
            lfnst_idx,
            mts_idx,
            bcw_idx,
        ) = {
            let cu = cu.lock().unwrap();
            (
                cu.x,
                cu.y,
                cu.width,
                cu.height,
                cu.mode_type,
                cu.tree_type,
                cu.skip_flag,
                cu.pred_mode,
                cu.pred_mode_flag,
                cu.pred_mode_ibc_flag,
                cu.pred_mode_plt_flag,
                cu.act_enabled_flag,
                cu.intra_bdpcm_luma_flag,
                cu.intra_bdpcm_luma_dir_flag,
                cu.intra_bdpcm_chroma_flag,
                cu.intra_bdpcm_chroma_dir_flag,
                cu.intra_mip_flag,
                cu.intra_mip_transposed_flag,
                cu.intra_mip_mode,
                cu.intra_luma_ref_idx,
                cu.intra_subpartitions_mode_flag,
                cu.intra_subpartitions_split_flag,
                cu.get_intra_luma_not_planar_flag(),
                cu.intra_chroma_pred_mode,
                cu.general_merge_flag,
                cu.mvd_coding.clone(),
                cu.mvp_l0_flag,
                cu.mvp_l1_flag,
                cu.amvr_precision_idx,
                cu.inter_pred_idc,
                cu.inter_affine_flag,
                cu.affine_type_flag,
                cu.sym_mvd_flag,
                cu.ref_idx,
                cu.amvr_flag,
                cu.coded_flag,
                cu.merge_data.clone(),
                cu.sbt_flag,
                cu.sbt_quad_flag,
                cu.sbt_horizontal_flag,
                cu.sbt_pos_flag,
                cu.transform_tree.clone(),
                cu.lfnst_idx,
                cu.mts_idx,
                cu.bcw_idx,
            )
        };
        if sh.slice_type == SliceType::I && (width > 64 || height > 64) {
            debug_eprintln!("{}, {}", width, height);
            assert_eq!(mode_type, ModeType::MODE_TYPE_INTRA);
        }
        let ch_type = (tree_type == TreeType::DUAL_TREE_CHROMA) as usize;
        {
            let cu = &cu.lock().unwrap();
            if sh.slice_type != SliceType::I || sh.sps.ibc_enabled_flag {
                let ectx = &self.encoder_context;
                let mut ectx = ectx.lock().unwrap();
                if tree_type != TreeType::DUAL_TREE_CHROMA
                    && ((!(width == 4 && height == 4) && mode_type != ModeType::MODE_TYPE_INTRA)
                        || (sh.sps.ibc_enabled_flag && width <= 64 && height <= 64))
                {
                    debug_eprintln!("cu cu_skip_flag ");
                    self.coder.encode_cabac_cu(
                        bins,
                        skip_flag as usize,
                        CabacContext::CuSkipFlag,
                        cu,
                        sh,
                        &mut ectx,
                    );
                }
                if !skip_flag
                    && sh.slice_type != SliceType::I
                    && !(width == 4 && height == 4)
                    && mode_type == ModeType::MODE_TYPE_ALL
                {
                    debug_eprintln!("cu pred_mode_flag ");
                    self.coder.encode_cabac_cu(
                        bins,
                        pred_mode_flag as usize,
                        CabacContext::PredModeFlag,
                        cu,
                        sh,
                        &mut ectx,
                    );
                }
                if ((sh.slice_type == SliceType::I && !skip_flag)
                    || (sh.slice_type != SliceType::I
                        && (pred_mode[ch_type] != ModeType::MODE_INTRA)
                        && !skip_flag))
                    && width <= 64
                    && height <= 64
                    && mode_type != ModeType::MODE_TYPE_INTER
                    && sh.sps.ibc_enabled_flag
                    && tree_type != TreeType::DUAL_TREE_CHROMA
                {
                    debug_eprintln!("cu pred_mode_ibc_flag ");
                    self.coder.encode_cabac_cu(
                        bins,
                        pred_mode_ibc_flag as usize,
                        CabacContext::PredModeIbcFlag,
                        cu,
                        sh,
                        &mut ectx,
                    );
                }
            }
            {
                let ectx = &self.encoder_context;
                let mut ectx = ectx.lock().unwrap();
                //if ectx.cu_pred_mode[ch_type][x][y] == ModeType::MODE_INTRA
                if pred_mode[ch_type] == ModeType::MODE_INTRA
                    && sh.sps.palette_enabled_flag
                    && width <= 64
                    && height <= 64
                    && !skip_flag
                    && mode_type != ModeType::MODE_TYPE_INTER
                    && width * height
                        > (if tree_type != TreeType::DUAL_TREE_CHROMA {
                            16
                        } else {
                            16 * ectx.sub_width_c * ectx.sub_height_c
                        })
                    && (mode_type != ModeType::MODE_TYPE_INTRA
                        || tree_type != TreeType::DUAL_TREE_CHROMA)
                {
                    debug_eprintln!("cu pred_mode_plt_flag ");
                    self.coder.encode_cabac_cu(
                        bins,
                        pred_mode_plt_flag as usize,
                        CabacContext::PredModePltFlag,
                        cu,
                        sh,
                        &mut ectx,
                    );
                }
            }
            //if ectx.cu_pred_mode[ch_type][x][y] == ModeType::MODE_INTRA
            if pred_mode[ch_type] == ModeType::MODE_INTRA
                && sh.sps.act_enabled_flag
                && tree_type == TreeType::SINGLE_TREE
            {
                let ectx = &self.encoder_context;
                let mut ectx = ectx.lock().unwrap();
                debug_eprintln!("cu cu_act_enabled_flag ");
                self.coder.encode_cabac_cu(
                    bins,
                    cu_act_enabled_flag as usize,
                    CabacContext::CuActEnabledFlag,
                    cu,
                    sh,
                    &mut ectx,
                );
            }
            //if ectx.cu_pred_mode[ch_type][x][y] == ModeType::MODE_INTRA
            if pred_mode[ch_type] == ModeType::MODE_INTRA
                || pred_mode[ch_type] == ModeType::MODE_PLT
            {
                let ectx = &self.encoder_context;
                let mut ectx = ectx.lock().unwrap();
                if tree_type == TreeType::SINGLE_TREE || tree_type == TreeType::DUAL_TREE_LUMA {
                    if pred_mode_plt_flag {
                        // encode_palette_coding
                    } else {
                        if sh.sps.bdpcm_enabled_flag
                            && width <= ectx.max_ts_size
                            && height <= ectx.max_ts_size
                        {
                            debug_eprintln!("cu intra_bdpcm_luma_flag ");
                            self.coder.encode_cabac_cu(
                                bins,
                                intra_bdpcm_luma_flag as usize,
                                CabacContext::IntraBdpcmLumaFlag,
                                cu,
                                sh,
                                &mut ectx,
                            );
                        }
                        if intra_bdpcm_luma_flag {
                            debug_eprintln!("cu intra_bdpcm_luma_dir_flag ");
                            self.coder.encode_cabac_cu(
                                bins,
                                intra_bdpcm_luma_dir_flag as usize,
                                CabacContext::IntraBdpcmLumaDirFlag,
                                cu,
                                sh,
                                &mut ectx,
                            );
                        } else {
                            if sh.sps.mip_enabled_flag {
                                debug_eprintln!("cu intra_mip_flag ");
                                self.coder.encode_cabac_cu(
                                    bins,
                                    intra_mip_flag as usize,
                                    CabacContext::IntraMipFlag,
                                    cu,
                                    sh,
                                    &mut ectx,
                                );
                            }
                            if intra_mip_flag {
                                debug_eprintln!("cu intra_mip_transposed_flag ");
                                self.coder.encode_cabac_cu(
                                    bins,
                                    intra_mip_transposed_flag as usize,
                                    CabacContext::IntraMipTransposedFlag,
                                    cu,
                                    sh,
                                    &mut ectx,
                                );
                                debug_eprintln!("cu intra_mip_mode ");
                                self.coder.encode_cabac_cu(
                                    bins,
                                    intra_mip_mode,
                                    CabacContext::IntraMipMode,
                                    cu,
                                    sh,
                                    &mut ectx,
                                );
                            } else {
                                if sh.sps.mrl_enabled_flag && y % ectx.ctb_size_y > 0 {
                                    debug_eprintln!("cu intra_luma_ref_idx ");
                                    self.coder.encode_cabac_cu(
                                        bins,
                                        intra_luma_ref_idx,
                                        CabacContext::IntraLumaRefIdx,
                                        cu,
                                        sh,
                                        &mut ectx,
                                    );
                                }
                                if sh.sps.isp_enabled_flag
                                    && intra_luma_ref_idx == 0
                                    && (width <= ectx.max_tb_size_y && height <= ectx.max_tb_size_y)
                                    && (width * height > ectx.min_tb_size_y * ectx.min_tb_size_y)
                                    && !cu_act_enabled_flag
                                {
                                    debug_eprintln!("cu intra_subpartitions_mode_flag ");
                                    self.coder.encode_cabac_cu(
                                        bins,
                                        intra_subpartitions_mode_flag as usize,
                                        CabacContext::IntraSubpartitionsModeFlag,
                                        cu,
                                        sh,
                                        &mut ectx,
                                    );
                                }
                                if intra_subpartitions_mode_flag {
                                    debug_eprintln!("cu intra_subpartitions_split_flag ");
                                    self.coder.encode_cabac_cu(
                                        bins,
                                        intra_subpartitions_split_flag as usize,
                                        CabacContext::IntraSubpartitionsSplitFlag,
                                        cu,
                                        sh,
                                        &mut ectx,
                                    );
                                }
                                let (
                                    intra_luma_mpm_flag,
                                    intra_luma_mpm_idx,
                                    intra_luma_mpm_remainder,
                                ) = cu.get_intra_luma_mpm_flag_and_idx_and_remainder();
                                if intra_luma_ref_idx == 0 {
                                    debug_eprintln!("cu intra_luma_mpm_flag ");
                                    self.coder.encode_cabac_cu(
                                        bins,
                                        intra_luma_mpm_flag as usize,
                                        CabacContext::IntraLumaMpmFlag,
                                        cu,
                                        sh,
                                        &mut ectx,
                                    );
                                }
                                if intra_luma_mpm_flag {
                                    if intra_luma_ref_idx == 0 {
                                        debug_eprintln!("cu intra_luma_not_planar_flag ");
                                        self.coder.encode_cabac_cu(
                                            bins,
                                            intra_luma_not_planar_flag as usize,
                                            CabacContext::IntraLumaNotPlanarFlag,
                                            cu,
                                            sh,
                                            &mut ectx,
                                        );
                                    }
                                    if intra_luma_not_planar_flag {
                                        debug_eprintln!("cu intra_luma_mpm_idx ");
                                        self.coder.encode_cabac_cu(
                                            bins,
                                            intra_luma_mpm_idx,
                                            CabacContext::IntraLumaMpmIdx,
                                            cu,
                                            sh,
                                            &mut ectx,
                                        );
                                    }
                                } else {
                                    debug_eprintln!("cu intra_luma_mpm_remainer ");
                                    self.coder.encode_cabac_cu(
                                        bins,
                                        intra_luma_mpm_remainder,
                                        CabacContext::IntraLumaMpmRemainder,
                                        cu,
                                        sh,
                                        &mut ectx,
                                    );
                                }
                            }
                        }
                    }
                }
                if (tree_type == TreeType::SINGLE_TREE || tree_type == TreeType::DUAL_TREE_CHROMA)
                    && sh.sps.chroma_format != ChromaFormat::Monochrome
                {
                    if pred_mode_plt_flag && tree_type == TreeType::DUAL_TREE_CHROMA {
                        // TODO encode_palette_coding
                    } else if !pred_mode_plt_flag && !cu_act_enabled_flag {
                        if width / ectx.sub_width_c <= ectx.max_ts_size
                            && height / ectx.sub_height_c <= ectx.max_ts_size
                            && sh.sps.bdpcm_enabled_flag
                        {
                            debug_eprintln!("cu intra_bdpcm_chroma_flag ");
                            self.coder.encode_cabac_cu(
                                bins,
                                intra_bdpcm_chroma_flag as usize,
                                CabacContext::IntraBdpcmChromaFlag,
                                cu,
                                sh,
                                &mut ectx,
                            );
                        }
                        if intra_bdpcm_chroma_flag {
                            debug_eprintln!("cu intra_bdpcm_chroma_dir_flag ");
                            self.coder.encode_cabac_cu(
                                bins,
                                intra_bdpcm_chroma_dir_flag as usize,
                                CabacContext::IntraBdpcmChromaDirFlag,
                                cu,
                                sh,
                                &mut ectx,
                            );
                        } else {
                            if cu.is_cclm_enabled(sh, &ectx) {
                                debug_eprintln!("cu cclm_mode_flag ");
                                self.coder.encode_cabac_cu(
                                    bins,
                                    cu.get_cclm_mode_flag() as usize,
                                    CabacContext::CclmModeFlag,
                                    cu,
                                    sh,
                                    &mut ectx,
                                );
                            }
                            if cu.get_cclm_mode_flag() {
                                debug_eprintln!("cu cclm_mode_idx ");
                                self.coder.encode_cabac_cu(
                                    bins,
                                    cu.get_cclm_mode_idx(),
                                    CabacContext::CclmModeIdx,
                                    cu,
                                    sh,
                                    &mut ectx,
                                );
                            } else {
                                debug_eprintln!(
                                    "cu intra_chroma_pred_mode {}",
                                    intra_chroma_pred_mode
                                );
                                self.coder.encode_cabac_cu(
                                    bins,
                                    intra_chroma_pred_mode,
                                    CabacContext::IntraChromaPredMode,
                                    cu,
                                    sh,
                                    &mut ectx,
                                );
                            }
                        }
                    }
                }
            } else if tree_type != TreeType::DUAL_TREE_CHROMA {
                let ectx = self.encoder_context.clone();
                let mut ectx = ectx.lock().unwrap();
                if skip_flag {
                    debug_eprintln!("cu general_merge_flag ");
                    self.coder.encode_cabac_cu(
                        bins,
                        general_merge_flag as usize,
                        CabacContext::GeneralMergeFlag,
                        cu,
                        sh,
                        &mut ectx,
                    );
                }
                if general_merge_flag {
                    // TODO encode_merge_data
                } else if pred_mode[ch_type] == ModeType::MODE_IBC {
                    self.encode_mvd(bins, &mvd_coding[0][0], cu, sh);
                    if ectx.max_num_ibc_merge_cand > 1 {
                        debug_eprintln!("cu mvp_l0_flag ");
                        self.coder.encode_cabac_cu(
                            bins,
                            mvp_l0_flag as usize,
                            CabacContext::MvpL0Flag,
                            cu,
                            sh,
                            &mut ectx,
                        );
                    }
                    if sh.sps.amvr_enabled_flag && ectx.mvd_l0 != (0, 0) {
                        debug_eprintln!("cu amvr_precision_idx ");
                        self.coder.encode_cabac_cu(
                            bins,
                            amvr_precision_idx,
                            CabacContext::AmvrPrecisionIdx,
                            cu,
                            sh,
                            &mut ectx,
                        );
                    }
                } else {
                    if sh.slice_type == SliceType::B {
                        debug_eprintln!("cu inter_pred_idc ");
                        self.coder.encode_cabac_cu(
                            bins,
                            inter_pred_idc,
                            CabacContext::InterPredIdc,
                            cu,
                            sh,
                            &mut ectx,
                        );
                    }
                    if sh.sps.affine_enabled_flag && width >= 16 && height >= 16 {
                        debug_eprintln!("cu inter_affine_flag ");
                        self.coder.encode_cabac_cu(
                            bins,
                            inter_affine_flag as usize,
                            CabacContext::InterAffineFlag,
                            cu,
                            sh,
                            &mut ectx,
                        );
                        if sh.sps.six_param_affine_enabled_flag && inter_affine_flag {
                            debug_eprintln!("cu cu_affine_type_flag ");
                            self.coder.encode_cabac_cu(
                                bins,
                                affine_type_flag as usize,
                                CabacContext::CuAffineTypeFlag,
                                cu,
                                sh,
                                &mut ectx,
                            );
                        }
                    }
                    if sh.sps.smvd_enabled_flag
                        && !sh.ph.as_ref().unwrap().mvd_l1_zero_flag
                        && inter_pred_idc == InterPredMode::PRED_BI as usize
                        && !inter_affine_flag
                        && ectx.ref_idx_sym_l0 > -1
                        && ectx.ref_idx_sym_l1 > -1
                    {
                        debug_eprintln!("cu sym_mvd_flag ");
                        self.coder.encode_cabac_cu(
                            bins,
                            sym_mvd_flag as usize,
                            CabacContext::SymMvdFlag,
                            cu,
                            sh,
                            &mut ectx,
                        );
                    }
                    if inter_pred_idc != InterPredMode::PRED_L1 as usize {
                        if ectx.num_ref_idx_active[0] > 1 && !sym_mvd_flag {
                            debug_eprintln!("cu ref_idx_l0 ");
                            self.coder.encode_cabac_cu(
                                bins,
                                ref_idx[0],
                                CabacContext::RefIdxL0,
                                cu,
                                sh,
                                &mut ectx,
                            );
                        }
                        self.encode_mvd(bins, &mvd_coding[0][0], cu, sh);
                        if ectx.motion_model_idc > 0 {
                            self.encode_mvd(bins, &mvd_coding[0][1], cu, sh);
                        }
                        if ectx.motion_model_idc > 1 {
                            self.encode_mvd(bins, &mvd_coding[0][2], cu, sh);
                        }
                        debug_eprintln!("cu mvp_l0_flag ");
                        self.coder.encode_cabac_cu(
                            bins,
                            mvp_l0_flag as usize,
                            CabacContext::MvpL0Flag,
                            cu,
                            sh,
                            &mut ectx,
                        );
                    } else {
                        ectx.mvd_l0 = (0, 0);
                    }
                    if inter_pred_idc != InterPredMode::PRED_L0 as usize {
                        if ectx.num_ref_idx_active[1] > 1 && !sym_mvd_flag {
                            debug_eprintln!("cu ref_idx_l1 ");
                            self.coder.encode_cabac_cu(
                                bins,
                                ref_idx[1],
                                CabacContext::RefIdxL1,
                                cu,
                                sh,
                                &mut ectx,
                            );
                        }
                        if sh.ph.as_ref().unwrap().mvd_l1_zero_flag
                            && inter_pred_idc == InterPredMode::PRED_BI as usize
                        {
                            ectx.mvd_l1 = (0, 0);
                            ectx.mvd_cp_l1 = [(0, 0); 3];
                        } else {
                            if sym_mvd_flag {
                                ectx.mvd_l1 = (-ectx.mvd_l0.0, -ectx.mvd_l0.1);
                            } else {
                                self.encode_mvd(bins, &mvd_coding[1][0], cu, sh);
                            }
                            if ectx.motion_model_idc > 0 {
                                self.encode_mvd(bins, &mvd_coding[1][1], cu, sh);
                            }
                            if ectx.motion_model_idc > 1 {
                                self.encode_mvd(bins, &mvd_coding[1][2], cu, sh);
                            }
                        }
                        debug_eprintln!("cu mvp_l1_flag ");
                        self.coder.encode_cabac_cu(
                            bins,
                            mvp_l1_flag as usize,
                            CabacContext::MvpL1Flag,
                            cu,
                            sh,
                            &mut ectx,
                        );
                    } else {
                        ectx.mvd_l1 = (0, 0);
                    }
                    if sh.sps.amvr_enabled_flag
                        && !inter_affine_flag
                        && (ectx.mvd_l0 != (0, 0) || ectx.mvd_l1 != (0, 0))
                        || (sh.sps.affine_amvr_enabled_flag
                            && inter_affine_flag
                            && (ectx.mvd_cp_l0 != [(0, 0); 3] || ectx.mvd_cp_l1 != [(0, 0); 3]))
                    {
                        debug_eprintln!("cu amvr_flag ");
                        self.coder.encode_cabac_cu(
                            bins,
                            amvr_flag as usize,
                            CabacContext::AmvrFlag,
                            cu,
                            sh,
                            &mut ectx,
                        );
                        if amvr_flag {
                            debug_eprintln!("cu amvr_precision_idx ");
                            self.coder.encode_cabac_cu(
                                bins,
                                amvr_precision_idx,
                                CabacContext::AmvrPrecisionIdx,
                                cu,
                                sh,
                                &mut ectx,
                            );
                        }
                    }
                    if sh.sps.bcw_enabled_flag
                        && inter_pred_idc == InterPredMode::PRED_BI as usize
                        && !sh
                            .ph
                            .as_ref()
                            .unwrap()
                            .pred_weight_table
                            .as_ref()
                            .unwrap()
                            .luma_weight_l0_flag[ref_idx[0]]
                        && !sh
                            .ph
                            .as_ref()
                            .unwrap()
                            .pred_weight_table
                            .as_ref()
                            .unwrap()
                            .luma_weight_l1_flag[ref_idx[1]]
                        && !sh
                            .ph
                            .as_ref()
                            .unwrap()
                            .pred_weight_table
                            .as_ref()
                            .unwrap()
                            .chroma_weight_l0_flag[ref_idx[0]]
                        && !sh
                            .ph
                            .as_ref()
                            .unwrap()
                            .pred_weight_table
                            .as_ref()
                            .unwrap()
                            .chroma_weight_l1_flag[ref_idx[1]]
                        && width * height >= 256
                    {
                        debug_eprintln!("cu bcw_idx ");
                        self.coder.encode_cabac_cu(
                            bins,
                            bcw_idx,
                            CabacContext::BcwIdx,
                            cu,
                            sh,
                            &mut ectx,
                        );
                    }
                }
            }
            //if ectx.cu_pred_mode[ch_type][x][y] != ModeType::MODE_INTRA
            if pred_mode[ch_type] != ModeType::MODE_INTRA
                && !pred_mode_plt_flag
                && !general_merge_flag
            {
                debug_eprintln!("cu cu_coded_flag ");
                let ectx = &self.encoder_context;
                let mut ectx = ectx.lock().unwrap();
                self.coder.encode_cabac_cu(
                    bins,
                    coded_flag as usize,
                    CabacContext::CuCodedFlag,
                    cu,
                    sh,
                    &mut ectx,
                );
            }
        }
        if coded_flag {
            debug_eprintln!("coded_flag = true");
            {
                let cu = &cu.lock().unwrap();
                let ectx = &self.encoder_context;
                let mut ectx = ectx.lock().unwrap();
                //if ectx.cu_pred_mode[ch_type][x][y] == ModeType::MODE_INTRA
                if pred_mode[ch_type] == ModeType::MODE_INTRA
                    && sh.sps.sbt_enabled_flag
                    && !merge_data.as_ref().unwrap().ciip_flag
                    && width <= ectx.max_tb_size_y
                    && height <= ectx.max_tb_size_y
                {
                    let allow_sbt_ver_h = width >= 8;
                    let allow_sbt_ver_q = width >= 16;
                    let allow_sbt_hor_h = height >= 8;
                    let allow_sbt_hor_q = height >= 16;
                    if allow_sbt_ver_h || allow_sbt_hor_h {
                        debug_eprintln!("cu cu_sbt_flag ");
                        self.coder.encode_cabac_cu(
                            bins,
                            sbt_flag as usize,
                            CabacContext::CuSbtFlag,
                            cu,
                            sh,
                            &mut ectx,
                        );
                    }
                    if sbt_flag {
                        if (allow_sbt_ver_h || allow_sbt_hor_h)
                            && (allow_sbt_ver_q || allow_sbt_hor_q)
                        {
                            debug_eprintln!("cu cu_sbt_quad_flag ");
                            self.coder.encode_cabac_cu(
                                bins,
                                sbt_quad_flag as usize,
                                CabacContext::CuSbtQuadFlag,
                                cu,
                                sh,
                                &mut ectx,
                            );
                        }
                        if (sbt_quad_flag && allow_sbt_ver_q && allow_sbt_hor_q)
                            || (!sbt_quad_flag && allow_sbt_ver_h && allow_sbt_hor_h)
                        {
                            debug_eprintln!("cu cu_sbt_horizontal_flag ");
                            self.coder.encode_cabac_cu(
                                bins,
                                sbt_horizontal_flag as usize,
                                CabacContext::CuSbtHorizontalFlag,
                                cu,
                                sh,
                                &mut ectx,
                            );
                        }
                        debug_eprintln!("cu cu_sbt_pos_flag ");
                        self.coder.encode_cabac_cu(
                            bins,
                            sbt_pos_flag as usize,
                            CabacContext::CuSbtPosFlag,
                            cu,
                            sh,
                            &mut ectx,
                        );
                    }
                }
                if sh.sps.act_enabled_flag
                //&& ectx.cu_pred_mode[ch_type][x][y] != ModeType::MODE_INTRA
                && pred_mode[ch_type] != ModeType::MODE_INTRA
                && tree_type == TreeType::SINGLE_TREE
                {
                    debug_eprintln!("cu cu_act_enabled_flag ");
                    self.coder.encode_cabac_cu(
                        bins,
                        cu_act_enabled_flag as usize,
                        CabacContext::CuActEnabledFlag,
                        cu,
                        sh,
                        &mut ectx,
                    );
                }
                ectx.lfnst_dc_only = true;
                ectx.lfnst_zero_out_sig_coeff_flag = true;
                ectx.mts_dc_only = true;
                ectx.mts_zero_out_sig_coeff_flag = true;
            }
            {
                let tt = transform_tree.as_ref().unwrap();
                self.encode_transform_tree(bins, tt.clone(), sh);
            }
            let (lfnst_width, lfnst_height) = {
                let ectx = &self.encoder_context;
                let ectx = ectx.lock().unwrap();
                (
                    if tree_type == TreeType::DUAL_TREE_CHROMA {
                        width / ectx.sub_width_c
                    } else if ectx.intra_subpartitions_split_type
                        == IntraSubpartitionsSplitType::ISP_VER_SPLIT
                    {
                        width / ectx.num_intra_subpartitions
                    } else {
                        width
                    },
                    if tree_type == TreeType::DUAL_TREE_CHROMA {
                        height / ectx.sub_height_c
                    } else if ectx.intra_subpartitions_split_type
                        == IntraSubpartitionsSplitType::ISP_VER_SPLIT
                    {
                        height / ectx.num_intra_subpartitions
                    } else {
                        height
                    },
                )
            };
            let lfnst_not_ts_flag = {
                let first_tu = {
                    let tt = transform_tree.as_ref().unwrap();
                    let tt = tt.lock().unwrap();
                    tt.first_tu()
                };
                let first_tu = first_tu.borrow();
                (tree_type == TreeType::DUAL_TREE_CHROMA
                    || !first_tu.get_y_coded_flag()
                    || !first_tu.transform_skip_flag[0])
                    && (tree_type == TreeType::DUAL_TREE_LUMA
                        || ((!first_tu.get_cb_coded_flag() || !first_tu.transform_skip_flag[1])
                            && (!first_tu.get_cr_coded_flag() || !first_tu.transform_skip_flag[2])))
            };
            {
                let cu = &cu.lock().unwrap();
                let ectx = &self.encoder_context;
                let mut ectx = ectx.lock().unwrap();
                if lfnst_width.min(lfnst_height) >= 4
                && sh.sps.lfnst_enabled_flag
                //&& ectx.cu_pred_mode[ch_type][x][y] == ModeType::MODE_INTRA
                && pred_mode[ch_type] == ModeType::MODE_INTRA
                && lfnst_not_ts_flag
                && (tree_type == TreeType::DUAL_TREE_CHROMA
                    || !ectx.intra_mip_flag[x][y]
                    || lfnst_width.min(lfnst_height) >= 16)
                && width.max(height) <= ectx.max_tb_size_y
                    && (ectx.intra_subpartitions_split_type
                        != IntraSubpartitionsSplitType::ISP_NO_SPLIT
                        || !ectx.lfnst_dc_only)
                        && ectx.lfnst_zero_out_sig_coeff_flag
                {
                    debug_eprintln!("cu lfnst_idx ");
                    self.coder.encode_cabac_cu(
                        bins,
                        lfnst_idx,
                        CabacContext::LfnstIdx,
                        cu,
                        sh,
                        &mut ectx,
                    );
                }
                let transform_skip_flag = {
                    let tt = transform_tree.as_ref().unwrap();
                    let tt = tt.lock().unwrap();
                    let first_tu = tt.first_tu();
                    let first_tu = first_tu.borrow();
                    first_tu.transform_skip_flag[0]
                };
                if tree_type != TreeType::DUAL_TREE_CHROMA
                    && lfnst_idx == 0
                    && !transform_skip_flag
                    && width.max(height) <= 32
                    && ectx.intra_subpartitions_split_type
                        == IntraSubpartitionsSplitType::ISP_NO_SPLIT
                    && !sbt_flag
                    && ectx.mts_zero_out_sig_coeff_flag
                    && !ectx.mts_dc_only
                {
                    debug_eprintln!("cu mts_idx ");
                    self.coder.encode_cabac_cu(
                        bins,
                        mts_idx,
                        CabacContext::MtsIdx,
                        cu,
                        sh,
                        &mut ectx,
                    );
                }
            }
        }
    }

    // TODO encode_palette_coding
    // TODO encode_merge_data

    pub fn encode_mvd(
        &mut self,
        bins: &mut Bins,
        mvd: &MvdCoding,
        cu: &CodingUnit,
        sh: &SliceHeader,
    ) {
        let ectx = &self.encoder_context;
        let mut ectx = ectx.lock().unwrap();
        for i in 0..=1 {
            debug_eprintln!("mvd abs_mvd_greater0_flag ");
            self.coder.encode_cabac_cu(
                bins,
                mvd.abs_mvd_greater0_flag[i] as usize,
                CabacContext::AbsMvdGreater0Flag,
                cu,
                sh,
                &mut ectx,
            );
        }
        for i in 0..=1 {
            if mvd.abs_mvd_greater0_flag[i] {
                debug_eprintln!("mvd abs_mvd_greater1_flag ");
                self.coder.encode_cabac_cu(
                    bins,
                    mvd.abs_mvd_greater1_flag[i] as usize,
                    CabacContext::AbsMvdGreater1Flag,
                    cu,
                    sh,
                    &mut ectx,
                );
            }
        }
        for i in 0..=1 {
            if mvd.abs_mvd_greater0_flag[i] {
                if mvd.abs_mvd_greater1_flag[i] {
                    debug_eprintln!("mvd abs_mvd ");
                    self.coder.encode_cabac_cu(
                        bins,
                        mvd.abs_mvd[i] - 2,
                        CabacContext::AbsMvd,
                        cu,
                        sh,
                        &mut ectx,
                    );
                }
                debug_eprintln!("mvd sign_flag ");
                self.coder.encode_cabac_cu(
                    bins,
                    mvd.sign_flag[i] as usize,
                    CabacContext::MvdSignFlag,
                    cu,
                    sh,
                    &mut ectx,
                );
            }
        }
    }

    pub fn encode_transform_tree(
        &mut self,
        bins: &mut Bins,
        tt: Arc<Mutex<TransformTree>>,
        sh: &SliceHeader,
    ) {
        debug_eprintln!("start transform_tree");
        {
            let ectx = &self.encoder_context;
            let mut ectx = ectx.lock().unwrap();
            ectx.infer_tu_cbf_luma = true;
        }
        let tts = {
            let tt = tt.lock().unwrap();
            tt.tts.clone()
        };
        for tt in tts.iter() {
            self.encode_transform_tree(bins, tt.clone(), sh);
        }
        let tus = {
            let tt = tt.lock().unwrap();
            tt.tus.clone()
        };
        for tu in tus.iter() {
            self.encode_transform_unit(bins, tu.clone(), sh);
        }
        debug_eprintln!("end transform_tree");
    }

    pub fn encode_transform_unit(
        &mut self,
        bins: &mut Bins,
        tu: Rc<RefCell<TransformUnit>>,
        sh: &SliceHeader,
    ) {
        debug_eprintln!("start transform_unit");
        {
            let tu = &mut tu.borrow_mut();
            // FIXME should not be here
            let pred_mode_flag = tu.cu_pred_mode_flag;
            let mut intra_predictor = IntraPredictor::new();
            let ectx = &self.encoder_context;
            let ectx = &mut ectx.lock().unwrap();
            let mut transformer = Transformer::new();
            let mut quantizer = Quantizer::new(ectx);
            for c_idx in 0..3 {
                if tu.is_component_active(c_idx) {
                    if pred_mode_flag {
                        intra_predictor.predict(tu, c_idx, sh.sps, sh.pps, ectx);
                        ectx.enable_print = false;
                    }
                    transformer.transform(tu, c_idx, sh.sps, sh.ph.as_ref().unwrap(), ectx);
                    quantizer.quantize(tu, c_idx, true, sh, ectx);
                    quantizer.dequantize(tu, c_idx, sh, ectx);
                    transformer.inverse_transform(tu, c_idx, sh.sps, sh.ph.as_ref().unwrap(), ectx);
                    let tile = tu.get_tile();
                    let tile = &mut tile.lock().unwrap();
                    let (tx, ty) = tu.get_component_pos(c_idx);
                    let (tw, th) = tu.get_component_size(c_idx);
                    let pred_pixels = &tile.pred_pixels.borrow()[c_idx];
                    let reconst_pixels = &mut tile.reconst_pixels.borrow_mut()[c_idx];
                    for y in ty..ty + th {
                        let pred_pixels = &pred_pixels[y];
                        let reconst_pixels = &mut reconst_pixels[y];
                        let it = &tu.itransformed_coeffs[c_idx][y - ty];
                        for x in tx..tx + tw {
                            let pred = pred_pixels[x];
                            let res = it[x - tx];
                            let rec = (pred as i16 + res).clamp(0, 255) as u8;
                            reconst_pixels[x] = rec;
                        }
                    }
                }
            }

            ectx.qp_y = tu.qp;
        }

        let tu = &tu.borrow();

        let (
            width,
            height,
            ch_type,
            tree_type,
            sub_tu_index,
            y_coded_flag,
            cb_coded_flag,
            cr_coded_flag,
            cu_qp_delta_abs,
            cu_qp_delta_sign_flag,
            cu_chroma_qp_offset_flag,
            cu_chroma_qp_offset_idx,
            joint_cbcr_residual_flag,
            transform_skip_flag,
        ) = {
            let (cu_qp_delta_abs, cu_qp_delta_sign_flag) = {
                let ectx = &self.encoder_context;
                let ectx = ectx.lock().unwrap();
                let cu_qp_delta = tu.get_cu_qp_delta(sh.sps, sh.pps, &ectx);
                (cu_qp_delta.abs(), cu_qp_delta < 0)
            };
            (
                tu.width,
                tu.height,
                tu.ch_type,
                tu.tree_type,
                tu.sub_tu_index,
                tu.get_y_coded_flag(),
                tu.get_cb_coded_flag(),
                tu.get_cr_coded_flag(),
                cu_qp_delta_abs,
                cu_qp_delta_sign_flag,
                tu.cu_chroma_qp_offset_flag,
                tu.cu_chroma_qp_offset_idx,
                tu.joint_cbcr_residual_flag,
                tu.transform_skip_flag,
            )
        };
        let (sbt_flag, sbt_pos_flag, act_enabled_flag, cu_pred_mode, cb_width, cb_height) = {
            let cu = tu.get_cu();
            let cu = cu.lock().unwrap();
            (
                cu.sbt_flag,
                cu.sbt_pos_flag,
                cu.act_enabled_flag,
                cu.pred_mode,
                cu.width,
                cu.height,
            )
        };
        let (w_c, h_c) = {
            let ectx = &self.encoder_context;
            let ectx = ectx.lock().unwrap();
            if ectx.intra_subpartitions_split_type != IntraSubpartitionsSplitType::ISP_NO_SPLIT
                && tree_type == TreeType::SINGLE_TREE
                && sub_tu_index == ectx.num_intra_subpartitions - 1
            {
                (cb_width / ectx.sub_width_c, cb_height / ectx.sub_height_c)
            } else {
                (width / ectx.sub_width_c, height / ectx.sub_height_c)
            }
        };
        let chroma_available = {
            let ectx = &self.encoder_context;
            let ectx = ectx.lock().unwrap();
            tree_type != TreeType::DUAL_TREE_LUMA
                && sh.sps.chroma_format != ChromaFormat::Monochrome
                && (ectx.intra_subpartitions_split_type
                    == IntraSubpartitionsSplitType::ISP_NO_SPLIT
                    || (ectx.intra_subpartitions_split_type
                        != IntraSubpartitionsSplitType::ISP_NO_SPLIT
                        && sub_tu_index == ectx.num_intra_subpartitions - 1))
        };
        {
            let ectx = &self.encoder_context;
            let mut ectx = ectx.lock().unwrap();
            if (tree_type == TreeType::SINGLE_TREE || tree_type == TreeType::DUAL_TREE_CHROMA)
                && sh.sps.chroma_format != ChromaFormat::Monochrome
                && ((ectx.intra_subpartitions_split_type
                    == IntraSubpartitionsSplitType::ISP_NO_SPLIT
                    && !(sbt_flag
                        && ((sub_tu_index == 0 && sbt_pos_flag)
                            || (sub_tu_index == 1 && !sbt_pos_flag))))
                    || (ectx.intra_subpartitions_split_type
                        != IntraSubpartitionsSplitType::ISP_NO_SPLIT
                        && (sub_tu_index == ectx.num_intra_subpartitions - 1)))
            {
                debug_eprintln!("tu cb_coded_flag ");
                self.coder.encode_cabac_tu(
                    bins,
                    cb_coded_flag as usize,
                    0,
                    CabacContext::TuCbCodedFlag,
                    tu,
                    sh,
                    &mut ectx,
                );
                debug_eprintln!("tu cr_coded_flag ");
                self.coder.encode_cabac_tu(
                    bins,
                    cr_coded_flag as usize,
                    0,
                    CabacContext::TuCrCodedFlag,
                    tu,
                    sh,
                    &mut ectx,
                );
            }
            if tree_type == TreeType::SINGLE_TREE || tree_type == TreeType::DUAL_TREE_LUMA {
                if (ectx.intra_subpartitions_split_type
                    == IntraSubpartitionsSplitType::ISP_NO_SPLIT
                    && !(sbt_flag
                        && ((sub_tu_index == 0 && sbt_pos_flag)
                            || (sub_tu_index == 1 && !sbt_pos_flag)))
                    && ((cu_pred_mode[ch_type] == ModeType::MODE_INTRA && !act_enabled_flag)
                        || (chroma_available && (cb_coded_flag || cr_coded_flag))
                        || cb_width > ectx.max_tb_size_y
                        || cb_height > ectx.max_tb_size_y))
                    || (ectx.intra_subpartitions_split_type
                        != IntraSubpartitionsSplitType::ISP_NO_SPLIT
                        && (sub_tu_index < ectx.num_intra_subpartitions - 1
                            || !ectx.infer_tu_cbf_luma))
                {
                    debug_eprintln!("tu y_coded_flag ");
                    self.coder.encode_cabac_tu(
                        bins,
                        y_coded_flag as usize,
                        0,
                        CabacContext::TuYCodedFlag,
                        tu,
                        sh,
                        &mut ectx,
                    );
                }
                if ectx.intra_subpartitions_split_type != IntraSubpartitionsSplitType::ISP_NO_SPLIT
                {
                    ectx.infer_tu_cbf_luma = ectx.infer_tu_cbf_luma && !y_coded_flag;
                }
            }
            if (cb_width > 64
                || cb_height > 64
                || y_coded_flag
                || (chroma_available && (cb_coded_flag || cr_coded_flag)))
                && tree_type != TreeType::DUAL_TREE_CHROMA
                && sh.pps.cu_qp_delta_enabled_flag
                && !ectx.is_cu_qp_delta_coded
            {
                debug_eprintln!("tu qp_delta_abs {}", cu_qp_delta_abs);
                self.coder.encode_cabac_tu(
                    bins,
                    cu_qp_delta_abs as usize,
                    0,
                    CabacContext::CuQpDeltaAbs,
                    tu,
                    sh,
                    &mut ectx,
                );
                if cu_qp_delta_abs > 0 {
                    debug_eprintln!("tu qp_delta_sign_flag {}", cu_qp_delta_sign_flag);
                    self.coder.encode_cabac_tu(
                        bins,
                        cu_qp_delta_sign_flag as usize,
                        0,
                        CabacContext::CuQpDeltaSignFlag,
                        tu,
                        sh,
                        &mut ectx,
                    );
                }
                ectx.is_cu_qp_delta_coded = true;
                ectx.cu_qp_delta_val = cu_qp_delta_abs * (1 - 2 * cu_qp_delta_sign_flag as isize);
            }
            if (cb_width > 64
                || cb_height > 64
                || (chroma_available && (cb_coded_flag || cr_coded_flag)))
                && tree_type != TreeType::DUAL_TREE_LUMA
                && sh.cu_chroma_qp_offset_enabled_flag
                && !ectx.is_cu_chroma_qp_offset_coded
            {
                debug_eprintln!("tu cu_chroma_qp_offset_flag ");
                self.coder.encode_cabac_tu(
                    bins,
                    cu_chroma_qp_offset_flag as usize,
                    0,
                    CabacContext::CuChromaQpOffsetFlag,
                    tu,
                    sh,
                    &mut ectx,
                );
                if cu_chroma_qp_offset_flag
                    && sh.pps.chroma_tool_offsets.chroma_qp_offset_list_len > 1
                {
                    debug_eprintln!("tu cu_chroma_qp_offset_idx ");
                    self.coder.encode_cabac_tu(
                        bins,
                        cu_chroma_qp_offset_idx,
                        0,
                        CabacContext::CuChromaQpOffsetIdx,
                        tu,
                        sh,
                        &mut ectx,
                    );
                }
            }
            if sh.sps.joint_cbcr_enabled_flag
                && ((cu_pred_mode[ch_type] == ModeType::MODE_INTRA
                    && (cb_coded_flag || cr_coded_flag))
                    || (cb_coded_flag && cr_coded_flag))
                && chroma_available
            {
                debug_eprintln!("tu joint_cbcr_residual_flag ");
                self.coder.encode_cabac_tu(
                    bins,
                    joint_cbcr_residual_flag as usize,
                    0,
                    CabacContext::TuJointCbcrResidualFlag,
                    tu,
                    sh,
                    &mut ectx,
                );
            }
        }
        if y_coded_flag && tree_type != TreeType::DUAL_TREE_CHROMA {
            {
                let ectx = &self.encoder_context;
                let mut ectx = ectx.lock().unwrap();
                let bdpcm_flag = tu.cu_bdpcm_flag[0];
                if sh.sps.transform_skip_enabled_flag
                    && !bdpcm_flag
                    && width <= ectx.max_ts_size
                    && height <= ectx.max_ts_size
                    && (ectx.intra_subpartitions_split_type
                        == IntraSubpartitionsSplitType::ISP_NO_SPLIT)
                    && !sbt_flag
                {
                    debug_eprintln!("tu transform_skip_flag ");
                    self.coder.encode_cabac_tu(
                        bins,
                        transform_skip_flag[0] as usize,
                        0,
                        CabacContext::TransformSkipFlag,
                        tu,
                        sh,
                        &mut ectx,
                    );
                }
            }
            if !transform_skip_flag[0] || sh.ts_residual_coding_disabled_flag {
                self.encode_residual(bins, tu, 0, sh);
            } else {
                self.encode_residual_ts(bins, 0, tu, sh);
            }
        }
        if cb_coded_flag && tree_type != TreeType::DUAL_TREE_LUMA {
            let (bdpcm_flag, max_ts_size) = {
                let ectx = &self.encoder_context;
                let ectx = ectx.lock().unwrap();
                let bdpcm_flag = tu.cu_bdpcm_flag[1];
                (bdpcm_flag, ectx.max_ts_size)
            };
            if sh.sps.transform_skip_enabled_flag
                && !bdpcm_flag
                && w_c <= max_ts_size
                && h_c <= max_ts_size
                && !sbt_flag
            {
                let ectx = &self.encoder_context;
                let mut ectx = ectx.lock().unwrap();
                debug_eprintln!("tu transform_skip_flag ");
                self.coder.encode_cabac_tu(
                    bins,
                    transform_skip_flag[1] as usize,
                    1,
                    CabacContext::TransformSkipFlag,
                    tu,
                    sh,
                    &mut ectx,
                );
            }
            if !transform_skip_flag[1] || sh.ts_residual_coding_disabled_flag {
                self.encode_residual(bins, tu, 1, sh);
            } else {
                self.encode_residual_ts(bins, 1, tu, sh);
            }
        }
        if cr_coded_flag && tree_type != TreeType::DUAL_TREE_LUMA {
            let (bdpcm_flag, max_ts_size) = {
                let ectx = &self.encoder_context;
                let ectx = ectx.lock().unwrap();
                let bdpcm_flag = tu.cu_bdpcm_flag[2];
                (bdpcm_flag, ectx.max_ts_size)
            };
            if sh.sps.transform_skip_enabled_flag
                && !bdpcm_flag
                && w_c <= max_ts_size
                && h_c <= max_ts_size
                && !sbt_flag
            {
                debug_eprintln!("tu transform_skip_flag ");
                let ectx = &self.encoder_context;
                let mut ectx = ectx.lock().unwrap();
                self.coder.encode_cabac_tu(
                    bins,
                    transform_skip_flag[2] as usize,
                    2,
                    CabacContext::TransformSkipFlag,
                    tu,
                    sh,
                    &mut ectx,
                );
            }
            if !transform_skip_flag[2] || sh.ts_residual_coding_disabled_flag {
                self.encode_residual(bins, tu, 2, sh);
            } else {
                self.encode_residual_ts(bins, 2, tu, sh);
            }
        }
        debug_eprintln!("end transform_unit");
    }

    pub fn encode_residual(
        &mut self,
        bins: &mut Bins,
        tu: &TransformUnit,
        c_idx: usize,
        sh: &SliceHeader,
    ) {
        {
            debug_eprintln!("start encode_residual tu.x={}, tu.y={}", tu.x, tu.y);
        }
        let ectx = &self.encoder_context;
        let mut ectx = ectx.lock().unwrap();

        let ((mut log2_tb_width, mut log2_tb_height), transform_skip_flag) =
            { (tu.get_log2_tb_size(c_idx), tu.transform_skip_flag) };

        let (tw, th) = (1 << log2_tb_width, 1 << log2_tb_height);

        for y in 0..th {
            for x in 0..tw {
                ectx.abs_level[y][x] = 0;
                ectx.abs_level_pass1[y][x] = 0;
                ectx.abs_level_pass2[y][x] = 0;
            }
        }

        let (log2_zo_tb_width, log2_zo_tb_height) = { tu.get_log2_zo_tb_size(sh.sps, c_idx) };

        let (last_sig_coeff_x, last_sig_coeff_y) = { tu.get_last_sig_coeff_pos(c_idx) };
        ectx.last_significant_coeff_x = last_sig_coeff_x;
        ectx.last_significant_coeff_y = last_sig_coeff_y;

        let (last_sig_coeff_x_prefix, last_sig_coeff_x_suffix) = {
            if last_sig_coeff_x <= 3 {
                (last_sig_coeff_x, 0)
            } else {
                let (mut prefix, mut suffix);
                let mut suffix_bits = 1;
                while {
                    prefix = last_sig_coeff_x >> suffix_bits;
                    suffix = last_sig_coeff_x - (prefix << suffix_bits);
                    prefix >= 4
                } {
                    suffix_bits += 1;
                }
                (((suffix_bits + 1) << 1) + (prefix & 1), suffix)
            }
        };
        let (last_sig_coeff_y_prefix, last_sig_coeff_y_suffix) = {
            if last_sig_coeff_y <= 3 {
                (last_sig_coeff_y, 0)
            } else {
                let (mut prefix, mut suffix);
                let mut suffix_bits = 1;
                while {
                    prefix = last_sig_coeff_y >> suffix_bits;
                    suffix = last_sig_coeff_y - (prefix << suffix_bits);
                    prefix >= 4
                } {
                    suffix_bits += 1;
                }
                (((suffix_bits + 1) << 1) + (prefix & 1), suffix)
            }
        };
        debug_eprintln!(
            "last_x_prefix={}, last_x_suffix={}, last_y_prefix={}, last_y_suffix={}",
            last_sig_coeff_x_prefix,
            last_sig_coeff_x_suffix,
            last_sig_coeff_y_prefix,
            last_sig_coeff_y_suffix
        );
        {
            debug_eprintln!(
                "last_x={}, last_y={}, tu.x={}, tu.y={}, c_idx={}",
                last_sig_coeff_x,
                last_sig_coeff_y,
                tu.x,
                tu.y,
                c_idx
            );
        }
        debug_eprintln!(
            "log2_tb_width={}, log2_tb_height={}, c_idx={}",
            log2_tb_width,
            log2_tb_height,
            c_idx
        );

        if log2_tb_width > 0 {
            debug_eprintln!("res last_sig_coeff_x_prefix ");
            self.coder.encode_cabac_last_sig_coeff_x_prefix(
                bins,
                last_sig_coeff_x_prefix,
                tu,
                c_idx,
                sh,
            );
        }
        if log2_tb_height > 0 {
            debug_eprintln!("res last_sig_coeff_y_prefix ");
            self.coder.encode_cabac_last_sig_coeff_y_prefix(
                bins,
                last_sig_coeff_y_prefix,
                tu,
                c_idx,
                sh,
            );
        }
        if last_sig_coeff_x_prefix > 3 {
            debug_eprintln!("res last_sig_coeff_x_suffix ");
            self.coder.encode_cabac_for_last_sig_coeff_x_suffix(
                bins,
                last_sig_coeff_x_suffix,
                last_sig_coeff_x_prefix,
                CabacContext::LastSigCoeffXSuffix,
                sh,
            );
        }
        if last_sig_coeff_y_prefix > 3 {
            debug_eprintln!("res last_sig_coeff_y_suffix ");
            self.coder.encode_cabac_for_last_sig_coeff_y_suffix(
                bins,
                last_sig_coeff_y_suffix,
                last_sig_coeff_y_prefix,
                CabacContext::LastSigCoeffYSuffix,
                sh,
            );
        }

        (log2_tb_width, log2_tb_height) = (log2_zo_tb_width, log2_zo_tb_height);

        let mut rem_bins_pass1 = ((1 << (log2_tb_width + log2_tb_height)) * 7) >> 2;
        let (log2_sb_w, log2_sb_h) = { tu.get_log2_zo_sb_size(sh.sps, c_idx) };
        //println!("log2_sb_w={}, log2_sb_h={}", log2_sb_w, log2_sb_h);
        let num_sb_coeff = 1 << (log2_sb_w + log2_sb_h);
        let mut last_scan_pos = num_sb_coeff;
        let mut last_subblock =
            (1 << (log2_tb_width + log2_tb_height - (log2_sb_w + log2_sb_h))) - 1;
        let (mut x_c, mut y_c);
        let (mut x_s, mut y_s);
        while {
            if last_scan_pos == 0 {
                last_scan_pos = num_sb_coeff;
                last_subblock -= 1;
            }
            last_scan_pos -= 1;
            // FIXME speedup somehow
            (x_s, y_s) = DIAG_SCAN_ORDER[log2_tb_width - log2_sb_w][log2_tb_height - log2_sb_h]
                [last_subblock];
            x_c = (x_s << log2_sb_w) + DIAG_SCAN_ORDER[log2_sb_w][log2_sb_h][last_scan_pos].0;
            y_c = (y_s << log2_sb_h) + DIAG_SCAN_ORDER[log2_sb_w][log2_sb_h][last_scan_pos].1;
            x_c != last_sig_coeff_x || y_c != last_sig_coeff_y
        } {}
        if last_subblock == 0
            && log2_tb_width >= 2
            && log2_tb_height >= 2
            && !transform_skip_flag[c_idx]
            && last_scan_pos > 0
        {
            ectx.lfnst_dc_only = false;
        }
        if (last_subblock > 0 && log2_tb_width >= 2 && log2_tb_height >= 2)
            || (last_scan_pos > 7
                && (log2_tb_width == 2 || log2_tb_width == 3)
                && log2_tb_width == log2_tb_height)
        {
            ectx.lfnst_zero_out_sig_coeff_flag = false;
        }
        if (last_subblock > 0 || last_scan_pos > 0) && c_idx == 0 {
            ectx.mts_dc_only = false;
        }
        ectx.q_state = 0;
        let last_sig_coeff_pos = { tu.get_last_sig_coeff_pos(c_idx) };
        let sb_order = &DIAG_SCAN_ORDER[log2_tb_width - log2_sb_w][log2_tb_height - log2_sb_h];
        for i in (0..=last_subblock).rev() {
            let start_q_state_sb = ectx.q_state;
            (x_s, y_s) = sb_order[i];
            //let mut first_abs_remainder = true;
            let mut abs_levels = vec![0; num_sb_coeff];
            let order = &DIAG_SCAN_ORDER[log2_sb_w][log2_sb_h];
            let x_offset = x_s << log2_sb_w;
            let y_offset = y_s << log2_sb_h;
            if sh.dep_quant_used_flag {
                let mut q_state = ectx.q_state;
                let quantized_transformed_coeffs = &tu.quantized_transformed_coeffs[c_idx];
                for n in (0..num_sb_coeff).rev() {
                    x_c = x_offset + order[n].0;
                    y_c = y_offset + order[n].1;
                    if quantized_transformed_coeffs[y_c][x_c] != 0 {
                        assert_eq!(
                            quantized_transformed_coeffs[y_c][x_c].unsigned_abs() as usize & 1,
                            (q_state > 1) as usize
                        );
                    }
                    abs_levels[n] = (quantized_transformed_coeffs[y_c][x_c].unsigned_abs()
                        as usize
                        + (q_state > 1) as usize)
                        / 2;
                    q_state = ectx.q_state_trans_table[q_state][abs_levels[n] & 1];
                }
            } else {
                let quantized_transformed_coeffs = &tu.quantized_transformed_coeffs[c_idx];
                for n in (0..num_sb_coeff).rev() {
                    x_c = x_offset + order[n].0;
                    y_c = y_offset + order[n].1;
                    abs_levels[n] = quantized_transformed_coeffs[y_c][x_c].unsigned_abs() as usize;
                }
            }
            let mut pass1_abs_levels = abs_levels.clone();
            let mut infer_sb_dc_sig_coeff_flag = false;
            let sb_coded_flag = { tu.get_sb_coded_flag(c_idx, x_s, y_s) || (x_s, y_s) == (0, 0) };
            if i < last_subblock && i > 0 {
                debug_eprintln!("res sb_coded_flag ");
                self.coder.encode_cabac_for_sb_coded_flag(
                    bins,
                    sb_coded_flag,
                    tu,
                    c_idx,
                    x_s,
                    y_s,
                    sh,
                );
                infer_sb_dc_sig_coeff_flag = true;
            }
            if sb_coded_flag && (x_s > 3 || y_s > 3) && c_idx == 0 {
                ectx.mts_zero_out_sig_coeff_flag = false;
            }
            let mut first_sig_scan_pos_sb = num_sb_coeff;
            let mut last_sig_scan_pos_sb: isize = -1;
            let first_pos_mode0 = if i == last_subblock {
                last_scan_pos
            } else {
                num_sb_coeff - 1
            };
            let mut first_pos_mode1 = first_pos_mode0 as isize;
            for n in (0..=first_pos_mode0).rev() {
                if rem_bins_pass1 < 4 {
                    break;
                }
                x_c = x_offset + order[n].0;
                y_c = y_offset + order[n].1;
                let sig_coeff_flag = {
                    tu.get_sig_coeff_flag(c_idx, x_c, y_c)
                        || (if !transform_skip_flag[c_idx] || sh.ts_residual_coding_disabled_flag {
                            (x_c, y_c) == (last_sig_coeff_x, last_sig_coeff_y)
                                || (((x_c & ((1 << log2_sb_w) - 1), y_c & ((1 << log2_sb_h) - 1))
                                    == (0, 0))
                                    && infer_sb_dc_sig_coeff_flag
                                    && sb_coded_flag)
                        } else {
                            (x_c & ((1 << log2_sb_w) - 1), y_c & ((1 << log2_sb_h) - 1))
                                == ((1 << log2_sb_w) - 1, (1 << log2_sb_h) - 1)
                                && infer_sb_dc_sig_coeff_flag
                                && sb_coded_flag
                        })
                };
                if sb_coded_flag
                    && (n > 0 || !infer_sb_dc_sig_coeff_flag)
                    && (x_c != last_sig_coeff_x || y_c != last_sig_coeff_y)
                {
                    debug_eprintln!("res sig_coeff_flag ");
                    self.coder.encode_cabac_for_sig_coeff_flag(
                        bins,
                        sig_coeff_flag as usize,
                        x_c,
                        y_c,
                        CabacContext::SigCoeffFlag,
                        tu,
                        c_idx,
                        sh,
                        &mut ectx,
                    );
                    rem_bins_pass1 -= 1;
                    if sig_coeff_flag {
                        infer_sb_dc_sig_coeff_flag = false;
                    }
                }
                let (abs_level_gtx_flag0, abs_level_gtx_flag1, par_level_flag) = {
                    (
                        abs_levels[n] > 1,
                        abs_levels[n] > (1 << 1) + 1,
                        abs_levels[n] > 1 && abs_levels[n] % 2 == 1,
                    )
                };
                if sig_coeff_flag {
                    debug_eprintln!("res abs_level_gtx_flag0 ");
                    self.coder
                        .encode_cabac_for_par_level_flag_and_abs_level_gtx_flag(
                            bins,
                            abs_level_gtx_flag0,
                            CabacContext::AbsLevelGtxFlag,
                            0,
                            tu,
                            c_idx,
                            x_c,
                            y_c,
                            last_sig_coeff_pos,
                            sh,
                            &ectx,
                        );
                    rem_bins_pass1 -= 1;
                    if abs_level_gtx_flag0 {
                        debug_eprintln!("res par_level_flag ");
                        self.coder
                            .encode_cabac_for_par_level_flag_and_abs_level_gtx_flag(
                                bins,
                                par_level_flag,
                                CabacContext::ParLevelFlag,
                                0,
                                tu,
                                c_idx,
                                x_c,
                                y_c,
                                last_sig_coeff_pos,
                                sh,
                                &ectx,
                            );
                        rem_bins_pass1 -= 1;
                        debug_eprintln!("res abs_level_gtx_flag1 ");
                        self.coder
                            .encode_cabac_for_par_level_flag_and_abs_level_gtx_flag(
                                bins,
                                abs_level_gtx_flag1,
                                CabacContext::AbsLevelGtxFlag,
                                1,
                                tu,
                                c_idx,
                                x_c,
                                y_c,
                                last_sig_coeff_pos,
                                sh,
                                &ectx,
                            );
                        rem_bins_pass1 -= 1;
                    }
                    if last_sig_scan_pos_sb == -1 {
                        last_sig_scan_pos_sb = n as isize;
                    }
                    first_sig_scan_pos_sb = n;
                }
                let abs_level_pass1 = sig_coeff_flag as usize
                    + par_level_flag as usize
                    + abs_level_gtx_flag0 as usize
                    + 2 * abs_level_gtx_flag1 as usize;
                ectx.abs_level_pass1[y_c][x_c] = abs_level_pass1;
                pass1_abs_levels[n] = abs_levels[n] - abs_level_pass1;
                if sh.dep_quant_used_flag {
                    assert_eq!(abs_level_pass1 & 1, abs_levels[n] & 1);
                    ectx.q_state = ectx.q_state_trans_table[ectx.q_state][abs_level_pass1 & 1];
                }
                first_pos_mode1 = n as isize - 1;
            }
            for n in (first_pos_mode1 + 1..=first_pos_mode0 as isize).rev() {
                let n = n as usize;
                x_c = x_offset + order[n].0;
                y_c = y_offset + order[n].1;
                let (abs_level_gtx_flag1, abs_remainder) = {
                    (
                        abs_levels[n] > (1 << 1) + 1,
                        if abs_levels[n] > (1 << 1) + 1 {
                            pass1_abs_levels[n] / 2
                        } else {
                            0
                        },
                    )
                };
                if abs_level_gtx_flag1 {
                    debug_eprintln!("res abs_remainder ");
                    self.coder.encode_cabac_for_abs_remainder(
                        bins,
                        abs_remainder,
                        x_c,
                        y_c,
                        //first_abs_remainder,
                        n,
                        CabacContext::AbsRemainder,
                        tu,
                        c_idx,
                        sh,
                        &mut ectx,
                    );
                    //first_abs_remainder = false;
                }
                ectx.abs_level[y_c][x_c] = ectx.abs_level_pass1[y_c][x_c] + 2 * abs_remainder;
                assert_eq!(ectx.abs_level[y_c][x_c], abs_levels[n]);
            }
            for n in (0..=first_pos_mode1).rev() {
                let n = n as usize;
                x_c = x_offset + order[n].0;
                y_c = y_offset + order[n].1;
                ectx.abs_level[y_c][x_c] = abs_levels[n];
                if sb_coded_flag {
                    debug_eprintln!("res dec_abs_level ");
                    let dec_abs_level = {
                        tu.get_dec_abs_level(
                            abs_levels[n] as i16,
                            c_idx,
                            x_c,
                            y_c,
                            ectx.q_state,
                            &ectx,
                        )
                    };
                    debug_eprintln!("dec_abs_level={}", dec_abs_level);
                    self.coder.encode_cabac_for_dec_abs_level(
                        bins,
                        dec_abs_level,
                        x_c,
                        y_c,
                        n,
                        CabacContext::DecAbsLevel,
                        tu,
                        c_idx,
                        sh,
                        &mut ectx,
                    );
                }
                if ectx.abs_level[y_c][x_c] > 0 {
                    if last_sig_scan_pos_sb == -1 {
                        last_sig_scan_pos_sb = n as isize;
                    }
                    first_sig_scan_pos_sb = n;
                }
                if sh.dep_quant_used_flag {
                    ectx.q_state = ectx.q_state_trans_table[ectx.q_state][abs_levels[n] & 1];
                }
            }
            let sign_hidden_flag = sh.sign_data_hiding_used_flag
                && last_sig_scan_pos_sb - first_sig_scan_pos_sb as isize > 3;
            for n in (0..num_sb_coeff).rev() {
                x_c = x_offset + order[n].0;
                y_c = y_offset + order[n].1;
                assert_eq!(ectx.abs_level[y_c][x_c], abs_levels[n]);
                let coeff_sign_flag = { tu.quantized_transformed_coeffs[c_idx][y_c][x_c] < 0 };
                let abs_level = abs_levels[n];
                if abs_level > 0 && (!sign_hidden_flag || n != first_sig_scan_pos_sb) {
                    debug_eprintln!("res coeff_sign_flag ");
                    self.coder.encode_cabac_for_coeff_sign_flag(
                        bins,
                        coeff_sign_flag,
                        0,
                        n,
                        tu,
                        c_idx,
                        x_c,
                        y_c,
                        sh,
                        &ectx,
                    );
                }
                {
                    debug_eprintln!("coeff={}", tu.quantized_transformed_coeffs[c_idx][y_c][x_c]);
                }
            }
            for n in (0..num_sb_coeff).rev() {
                if abs_levels[n] > 0 {
                    debug_eprintln!(
                        "abs={} @ {:?}, {:?}, c={}",
                        abs_levels[n],
                        tu.get_component_pos(c_idx),
                        tu.get_component_size(c_idx),
                        c_idx
                    );
                }
            }
            if sh.dep_quant_used_flag {
                ectx.q_state = start_q_state_sb;
                for n in (0..num_sb_coeff).rev() {
                    x_c = x_offset + order[n].0;
                    y_c = y_offset + order[n].1;
                    if abs_levels[n] != 0 {
                        let t = 2 * abs_levels[n] - (ectx.q_state > 1) as usize;
                        assert_eq!(
                            t,
                            tu.quantized_transformed_coeffs[c_idx][y_c][x_c].unsigned_abs()
                                as usize
                        );
                    }
                    ectx.q_state = ectx.q_state_trans_table[ectx.q_state][abs_levels[n] & 1];
                }
            }
        }
        debug_eprintln!("end encode_residual");
    }

    pub fn encode_residual_ts(
        &mut self,
        bins: &mut Bins,
        c_idx: usize,
        tu: &TransformUnit,
        sh: &SliceHeader,
    ) {
        let ectx = &self.encoder_context;
        let mut ectx = ectx.lock().unwrap();

        let (log2_tb_width, log2_tb_height) = tu.get_log2_tb_size(c_idx);

        let (tw, th) = (1 << log2_tb_width, 1 << log2_tb_height);

        for y in 0..th {
            for x in 0..tw {
                ectx.abs_level[y][x] = 0;
                ectx.abs_level_pass1[y][x] = 0;
                ectx.abs_level_pass2[y][x] = 0;
            }
        }

        let mut log2_sb_w = if log2_tb_width.min(log2_tb_height) < 2 {
            1
        } else {
            2
        };
        let mut log2_sb_h = log2_sb_w;
        if log2_tb_width + log2_tb_height > 3 {
            if log2_tb_width < 2 {
                log2_sb_w = log2_tb_width;
                log2_sb_h = 4 - log2_sb_w;
            } else if log2_tb_height < 2 {
                log2_sb_h = log2_tb_height;
                log2_sb_w = 4 - log2_sb_h;
            }
        }
        let num_sb_coeff = 1 << (log2_sb_w + log2_sb_h);
        let last_subblock = (1 << (log2_tb_width + log2_tb_height - (log2_sb_w + log2_sb_h))) - 1;
        let mut infer_sb_cbf = true;
        ectx.rem_ccbs = ((1 << (log2_tb_width + log2_tb_height)) * 7) >> 2;
        let bdpcm_flag = tu.cu_bdpcm_flag[c_idx];
        let last_sig_coeff_pos = tu.get_last_sig_coeff_pos(c_idx);
        for i in 0..=last_subblock {
            //let mut first_abs_remainder = true;
            let x_s = DIAG_SCAN_ORDER[log2_tb_width - log2_sb_w][log2_tb_height - log2_sb_h][i].0;
            let y_s = DIAG_SCAN_ORDER[log2_tb_width - log2_sb_w][log2_tb_height - log2_sb_h][i].1;
            let sb_coded_flag = tu.get_sb_coded_flag(c_idx, x_s, y_s);
            if i != last_subblock || !infer_sb_cbf {
                debug_eprintln!("res_ts sb_coded_flag ");
                self.coder.encode_cabac_for_sb_coded_flag(
                    bins,
                    sb_coded_flag,
                    tu,
                    c_idx,
                    x_s,
                    y_s,
                    sh,
                );
            }
            if sb_coded_flag && i < last_subblock {
                infer_sb_cbf = false;
            }
            // first scan pass
            let mut infer_sb_sig_coeff_flag = true;
            let mut last_scan_pos_pass1 = -1;
            for n in 0..num_sb_coeff {
                if ectx.rem_ccbs < 4 {
                    break;
                }
                let x_c = (x_s << log2_sb_w) + DIAG_SCAN_ORDER[log2_sb_w][log2_sb_h][n].0;
                let y_c = (y_s << log2_sb_h) + DIAG_SCAN_ORDER[log2_sb_w][log2_sb_h][n].1;
                let pass1_abs_level = if !bdpcm_flag {
                    let abs_left_coeff = if x_c > 0 {
                        tu.quantized_transformed_coeffs[c_idx][y_c][x_c - 1].abs()
                    } else {
                        0
                    };
                    let abs_above_coeff = if y_c > 0 {
                        tu.quantized_transformed_coeffs[c_idx][y_c - 1][x_c].abs()
                    } else {
                        0
                    };
                    let pred_coeff = abs_left_coeff.max(abs_above_coeff);
                    let mut abs_level = tu.quantized_transformed_coeffs[c_idx][y_c][x_c].abs();
                    if abs_level == pred_coeff && pred_coeff > 0 {
                        abs_level = 1;
                    } else if abs_level < pred_coeff {
                        abs_level += 1;
                    }
                    abs_level
                } else {
                    tu.quantized_transformed_coeffs[c_idx][y_c][x_c].abs()
                };
                last_scan_pos_pass1 = n as isize;
                let sig_coeff_flag = tu.get_sig_coeff_flag(c_idx, x_c, y_c);
                if sb_coded_flag && (n != num_sb_coeff - 1 || !infer_sb_sig_coeff_flag) {
                    debug_eprintln!("res_ts sig_coeff_flag ");
                    self.coder.encode_cabac_for_sig_coeff_flag(
                        bins,
                        sig_coeff_flag as usize,
                        x_c,
                        y_c,
                        CabacContext::SigCoeffFlag,
                        tu,
                        c_idx,
                        sh,
                        &mut ectx,
                    );
                    ectx.rem_ccbs -= 1;
                    if sig_coeff_flag {
                        infer_sb_sig_coeff_flag = false;
                    }
                }
                ectx.coeff_sign_level[x_c][y_c] = 0;
                let (abs_level_gtx_flag0, par_level_flag) = (
                    pass1_abs_level > 1,
                    pass1_abs_level > 1 && pass1_abs_level % 2 == 1,
                );
                if sig_coeff_flag {
                    debug_eprintln!("res_ts coeff_sign_flag ");
                    let coeff_sign_flag = tu.quantized_transformed_coeffs[c_idx][y_c][x_c] < 0;
                    self.coder.encode_cabac_for_coeff_sign_flag(
                        bins,
                        coeff_sign_flag,
                        last_scan_pos_pass1,
                        n,
                        tu,
                        c_idx,
                        x_c,
                        y_c,
                        sh,
                        &ectx,
                    );
                    ectx.rem_ccbs -= 1;
                    ectx.coeff_sign_level[x_c][y_c] = if coeff_sign_flag { -1 } else { 1 };
                    debug_eprintln!("res_ts abs_level_gtx_flag ");
                    self.coder
                        .encode_cabac_for_par_level_flag_and_abs_level_gtx_flag(
                            bins,
                            abs_level_gtx_flag0,
                            CabacContext::AbsLevelGtxFlag,
                            0,
                            tu,
                            c_idx,
                            x_c,
                            y_c,
                            last_sig_coeff_pos,
                            sh,
                            &ectx,
                        );
                    ectx.rem_ccbs -= 1;
                    if abs_level_gtx_flag0 {
                        debug_eprintln!("res_ts par_level_flag ");
                        self.coder
                            .encode_cabac_for_par_level_flag_and_abs_level_gtx_flag(
                                bins,
                                par_level_flag,
                                CabacContext::ParLevelFlag,
                                0,
                                tu,
                                c_idx,
                                x_c,
                                y_c,
                                last_sig_coeff_pos,
                                sh,
                                &ectx,
                            );
                        ectx.rem_ccbs -= 1;
                    }
                }
                ectx.abs_level_pass1[y_c][x_c] = sig_coeff_flag as usize
                    + par_level_flag as usize
                    + abs_level_gtx_flag0 as usize;
            }
            // greater than X scan pass (num_gt_x_flags=5)
            let mut last_scan_pos_pass2: isize = -1;
            for n in 0..num_sb_coeff {
                if ectx.rem_ccbs < 4 {
                    break;
                }
                let x_c = (x_s << log2_sb_w) + DIAG_SCAN_ORDER[log2_sb_w][log2_sb_h][n].0;
                let y_c = (y_s << log2_sb_h) + DIAG_SCAN_ORDER[log2_sb_w][log2_sb_h][n].1;
                let abs_level = if !bdpcm_flag && n as isize <= last_scan_pos_pass1 {
                    let abs_left_coeff = if x_c > 0 {
                        tu.quantized_transformed_coeffs[c_idx][y_c][x_c - 1].abs()
                    } else {
                        0
                    };
                    let abs_above_coeff = if y_c > 0 {
                        tu.quantized_transformed_coeffs[c_idx][y_c - 1][x_c].abs()
                    } else {
                        0
                    };
                    let pred_coeff = abs_left_coeff.max(abs_above_coeff);
                    let mut abs_level = tu.quantized_transformed_coeffs[c_idx][y_c][x_c].abs();
                    if abs_level == pred_coeff && pred_coeff > 0 {
                        abs_level = 1;
                    } else if abs_level < pred_coeff {
                        abs_level += 1;
                    }
                    abs_level
                } else {
                    tu.quantized_transformed_coeffs[c_idx][y_c][x_c].abs()
                };
                ectx.abs_level_pass2[y_c][x_c] = ectx.abs_level_pass1[y_c][x_c];
                for j in 1..5 {
                    let (abs_level_gtx_flag_jm1, abs_level_gtx_flag_j) =
                        (abs_level > ((j - 1) << 1) + 1, abs_level > (j << 1) + 1);
                    if abs_level_gtx_flag_jm1 {
                        debug_eprintln!("res_ts abs_level_gtx_flag ");
                        self.coder
                            .encode_cabac_for_par_level_flag_and_abs_level_gtx_flag(
                                bins,
                                abs_level_gtx_flag_j,
                                CabacContext::AbsLevelGtxFlag,
                                j as usize,
                                tu,
                                c_idx,
                                x_c,
                                y_c,
                                last_sig_coeff_pos,
                                sh,
                                &ectx,
                            );
                        ectx.rem_ccbs -= 1;
                    }
                    ectx.abs_level_pass2[y_c][x_c] += 2 * abs_level_gtx_flag_j as usize;
                }
                last_scan_pos_pass2 = n as isize;
            }
            // remainder scan pass
            for n in 0..num_sb_coeff {
                let _x_c = (x_s << log2_sb_w) + DIAG_SCAN_ORDER[log2_sb_w][log2_sb_h][n].0;
                let _y_c = (y_s << log2_sb_h) + DIAG_SCAN_ORDER[log2_sb_w][log2_sb_h][n].1;
                debug_eprint!(
                    "{}, ",
                    tu.quantized_transformed_coeffs[c_idx][_y_c][_x_c].abs()
                );
            }
            debug_eprintln!();
            for n in 0..num_sb_coeff {
                let x_c = (x_s << log2_sb_w) + DIAG_SCAN_ORDER[log2_sb_w][log2_sb_h][n].0;
                let y_c = (y_s << log2_sb_h) + DIAG_SCAN_ORDER[log2_sb_w][log2_sb_h][n].1;
                let sb_coded_flag = tu.get_sb_coded_flag(c_idx, x_s, y_s);
                let abs_level = if !bdpcm_flag && n as isize <= last_scan_pos_pass1 {
                    let abs_left_coeff = if x_c > 0 {
                        tu.quantized_transformed_coeffs[c_idx][y_c][x_c - 1].abs()
                    } else {
                        0
                    };
                    let abs_above_coeff = if y_c > 0 {
                        tu.quantized_transformed_coeffs[c_idx][y_c - 1][x_c].abs()
                    } else {
                        0
                    };
                    let pred_coeff = abs_left_coeff.max(abs_above_coeff);
                    debug_eprintln!("pred={}", pred_coeff);
                    let mut abs_level = tu.quantized_transformed_coeffs[c_idx][y_c][x_c].abs();
                    if abs_level == pred_coeff && pred_coeff > 0 {
                        abs_level = 1;
                    } else if abs_level < pred_coeff {
                        abs_level += 1;
                    }
                    abs_level
                } else {
                    tu.quantized_transformed_coeffs[c_idx][y_c][x_c].abs()
                };
                let abs_remainder = {
                    (if n as isize <= last_scan_pos_pass2 {
                        (abs_level as i32 - ectx.abs_level_pass2[y_c][x_c] as i32) / 2
                    } else if n as isize <= last_scan_pos_pass1 {
                        (abs_level as i32 - ectx.abs_level_pass1[y_c][x_c] as i32) / 2
                    } else {
                        abs_level as i32
                    }) as i16
                };
                {
                    debug_eprintln!(
                        "abs_remainder={}, abs_level={}, pass1={}, pass2={}, res={}",
                        abs_remainder,
                        abs_level,
                        ectx.abs_level_pass1[y_c][x_c],
                        ectx.abs_level_pass2[y_c][x_c],
                        tu.quantized_transformed_coeffs[c_idx][y_c][x_c]
                    );
                }
                assert!(abs_remainder >= 0);
                if (n as isize <= last_scan_pos_pass2 && ectx.abs_level_pass2[y_c][x_c] >= 10)
                    || (n as isize > last_scan_pos_pass2
                        && n as isize <= last_scan_pos_pass1
                        && ectx.abs_level_pass1[y_c][x_c] >= 2)
                    || (n as isize > last_scan_pos_pass1 && sb_coded_flag)
                {
                    debug_eprintln!("res_ts abs_remainder ");
                    self.coder.encode_cabac_for_abs_remainder(
                        bins,
                        abs_remainder as usize,
                        x_c,
                        y_c,
                        //first_abs_remainder,
                        n,
                        CabacContext::AbsRemainder,
                        tu,
                        c_idx,
                        sh,
                        &mut ectx,
                    );
                    //first_abs_remainder = false;
                }
                let coeff_sign_flag = { tu.quantized_transformed_coeffs[c_idx][y_c][x_c] < 0 };
                if n as isize > last_scan_pos_pass2
                    && n as isize > last_scan_pos_pass1
                    && abs_remainder > 0
                {
                    debug_eprintln!("res_ts coeff_sign_flag ");
                    debug_eprintln!(
                        "n={}, pos1={}, pos2={}, rem={}",
                        n,
                        last_scan_pos_pass1,
                        last_scan_pos_pass2,
                        abs_remainder
                    );
                    self.coder.encode_cabac_for_coeff_sign_flag(
                        bins,
                        coeff_sign_flag,
                        last_scan_pos_pass1,
                        n,
                        tu,
                        c_idx,
                        x_c,
                        y_c,
                        sh,
                        &ectx,
                    );
                }
            }
        }
    }

    pub fn encode_sao(
        &mut self,
        bins: &mut Bins,
        sao: Rc<SAO>,
        sh: &SliceHeader,
        rx: usize,
        ry: usize,
    ) {
        let ectx = &self.encoder_context;
        let mut ectx = ectx.lock().unwrap();
        if rx > 0 {
            let left_ctb_available = rx != ectx.ctb_to_tile_col_bd[rx];
            if left_ctb_available {
                debug_eprintln!("sao alf_sao_merge_left_flag ");
                self.coder.encode_cabac_ctu(
                    bins,
                    sao.merge_left_flag as usize,
                    CabacContext::AlfSaoMergeLeftFlag,
                    sh,
                    &mut ectx,
                );
            }
        }
        if ry > 0 && !sao.merge_left_flag {
            let up_ctb_available =
                ry != ectx.ctb_to_tile_col_bd[ry] && !ectx.first_ctb_row_in_slice;
            if up_ctb_available {
                debug_eprintln!("sao alf_sao_merge_up_flag ");
                self.coder.encode_cabac_ctu(
                    bins,
                    sao.merge_up_flag as usize,
                    CabacContext::AlfSaoMergeUpFlag,
                    sh,
                    &mut ectx,
                );
            }
        }
        if !sao.merge_up_flag && !sao.merge_left_flag {
            for c_idx in 0..if sh.sps.chroma_format != ChromaFormat::Monochrome {
                3
            } else {
                1
            } {
                if (sh.sao_luma_used_flag && c_idx == 0) || (sh.sao_chroma_used_flag && c_idx > 0) {
                    if c_idx == 0 {
                        debug_eprintln!("sao alf_sao_type_idx_luma ");
                        self.coder.encode_cabac_ctu(
                            bins,
                            sao.type_idx_luma,
                            CabacContext::AlfSaoTypeIdxLuma,
                            sh,
                            &mut ectx,
                        );
                    } else if c_idx == 1 {
                        debug_eprintln!("sao alf_sao_type_idx_chroma ");
                        self.coder.encode_cabac_ctu(
                            bins,
                            sao.type_idx_chroma,
                            CabacContext::AlfSaoTypeIdxChroma,
                            sh,
                            &mut ectx,
                        );
                    }
                    if ectx.sao_type_idx[c_idx][rx][ry] != 0 {
                        for i in 0..4 {
                            debug_eprintln!("sao offset_abs ");
                            self.coder.encode_cabac_ctu(
                                bins,
                                sao.offset_abs[c_idx][rx][ry][i],
                                CabacContext::SaoOffsetAbs,
                                sh,
                                &mut ectx,
                            );
                        }
                        if ectx.sao_type_idx[c_idx][rx][ry] == 1 {
                            for i in 0..4 {
                                if sao.offset_abs[c_idx][rx][ry][i] != 0 {
                                    debug_eprintln!("sao offset_sign_flag ");
                                    self.coder.encode_cabac_ctu(
                                        bins,
                                        sao.offset_sign_flag[c_idx][rx][ry][i] as usize,
                                        CabacContext::SaoOffsetSignFlag,
                                        sh,
                                        &mut ectx,
                                    );
                                }
                            }
                            debug_eprintln!("sao band_position ");
                            self.coder.encode_cabac_ctu(
                                bins,
                                sao.band_position[c_idx][rx][ry],
                                CabacContext::SaoBandPosition,
                                sh,
                                &mut ectx,
                            );
                        } else if c_idx == 0 {
                            debug_eprintln!("sao eo_class_luma ");
                            self.coder.encode_cabac_ctu(
                                bins,
                                sao.eo_class_luma,
                                CabacContext::SaoEoClassLuma,
                                sh,
                                &mut ectx,
                            );
                        } else if c_idx == 1 {
                            debug_eprintln!("sao eo_class_chroma ");
                            self.coder.encode_cabac_ctu(
                                bins,
                                sao.eo_class_chroma,
                                CabacContext::SaoEoClassChroma,
                                sh,
                                &mut ectx,
                            );
                        }
                    }
                }
            }
        }
    }
}

use super::cabac_contexts::*;
use super::common::*;
use super::encoder_context::*;
use super::pps::*;
use super::slice_header::*;
use super::sps::*;
use super::tile::*;
//use debug_print::*;
use lazy_static::lazy_static;
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::{Arc, Mutex};

lazy_static! {
    pub static ref DIAG_SCAN_ORDER: Vec<Vec<Vec<(usize, usize)>>> = {
        let mut diag_scan_order = vec![
            vec![
                vec![(0, 0)],
                vec![(0, 0); 2],
                vec![(0, 0); 4],
                vec![(0, 0); 8],
                vec![(0, 0); 16],
            ],
            vec![
                vec![(0, 0); 2],
                vec![(0, 0); 4],
                vec![(0, 0); 8],
                vec![(0, 0); 16],
                vec![(0, 0); 32],
            ],
            vec![
                vec![(0, 0); 4],
                vec![(0, 0); 8],
                vec![(0, 0); 16],
                vec![(0, 0); 32],
                vec![(0, 0); 64],
            ],
            vec![
                vec![(0, 0); 8],
                vec![(0, 0); 16],
                vec![(0, 0); 32],
                vec![(0, 0); 64],
                vec![(0, 0); 128],
            ],
            vec![
                vec![(0, 0); 16],
                vec![(0, 0); 32],
                vec![(0, 0); 64],
                vec![(0, 0); 128],
                vec![(0, 0); 256],
            ],
        ];
        for (log2_block_height, order_h) in diag_scan_order.iter_mut().enumerate().take(4 + 1) {
            let block_height = 1 << log2_block_height;
            for (log2_block_width, order) in order_h.iter_mut().enumerate().take(4 + 1) {
                let block_width = 1 << log2_block_width;
                let mut i = 0;
                let mut x = 0;
                let mut y: isize = 0;
                let mut stop_loop = false;
                while !stop_loop {
                    while y >= 0 {
                        if x < block_width && y < block_height {
                            order[i].0 = x as usize;
                            order[i].1 = y as usize;
                            i += 1;
                        }
                        y -= 1;
                        x += 1;
                    }
                    y = x;
                    x = 0;
                    if i as isize >= block_width * block_height {
                        stop_loop = true;
                    }
                }
            }
        }
        diag_scan_order
    };
}

#[allow(clippy::upper_case_acronyms)]
pub struct SAO {
    pub merge_left_flag: bool,
    pub merge_up_flag: bool,
    pub type_idx_luma: usize,
    pub type_idx_chroma: usize,
    pub offset_abs: Vec<Vec<Vec<Vec<usize>>>>,
    pub offset_sign_flag: Vec<Vec<Vec<Vec<bool>>>>,
    pub band_position: Vec<Vec<Vec<usize>>>,
    pub eo_class_luma: usize,
    pub eo_class_chroma: usize,
}

impl SAO {
    pub fn new() -> SAO {
        SAO {
            merge_left_flag: false,
            merge_up_flag: false,
            type_idx_luma: 0,
            type_idx_chroma: 0,
            offset_abs: vec![],
            offset_sign_flag: vec![],
            band_position: vec![],
            eo_class_luma: 0,
            eo_class_chroma: 0,
        }
    }
}

#[allow(clippy::upper_case_acronyms)]
pub struct ALF {
    pub ctb_flag: Vec<bool>,
    pub use_aps_flag: bool,
    pub luma_prev_filter_idx: usize,
    pub luma_fixed_filter_idx: usize,
    pub ctb_filter_alt_idx: Vec<usize>,
    pub ctb_cc_cb_idc: usize,
    pub ctb_cc_cr_idc: usize,
}

impl ALF {
    pub fn new() -> ALF {
        ALF {
            ctb_flag: vec![],
            use_aps_flag: false,
            luma_prev_filter_idx: 0,
            luma_fixed_filter_idx: 0,
            ctb_filter_alt_idx: vec![],
            ctb_cc_cb_idc: 0,
            ctb_cc_cr_idc: 0,
        }
    }
}

pub struct CodingTreeUnit {
    pub x: usize,
    pub y: usize,
    pub width: usize,
    pub height: usize,
    pub pixels: Vec<Vec2d<u8>>,
    pub sao: Rc<SAO>,
    pub alf: ALF,
    pub dtiqs: Vec<Arc<Mutex<DTIQS>>>,
    pub ct: Vec<Arc<Mutex<CodingTree>>>,
    pub tile: Option<Arc<Mutex<Tile>>>,
    pub x_tile: usize,
    pub y_tile: usize,
    pub width_tile: usize,
    pub height_tile: usize,
}

impl CodingTreeUnit {
    pub fn new_arc_mutex(
        x: usize,
        y: usize,
        log2_width: usize,
        log2_height: usize,
        tile: Option<Arc<Mutex<Tile>>>,
        fixed_qp: Option<usize>,
    ) -> Arc<Mutex<CodingTreeUnit>> {
        let (x_tile, y_tile, width_tile, height_tile) = if let Some(tile) = &tile {
            let tile = tile.lock().unwrap();
            (
                tile.ctu_col << tile.log2_ctu_size,
                tile.ctu_row << tile.log2_ctu_size,
                tile.num_ctu_cols << tile.log2_ctu_size,
                tile.num_ctu_rows << tile.log2_ctu_size,
            )
        } else {
            (0, 0, 0, 0)
        };
        let ctu = Arc::new(Mutex::new(CodingTreeUnit {
            x,
            y,
            width: 1 << log2_width,
            height: 1 << log2_height,
            pixels: vec![],
            sao: Rc::new(SAO::new()),
            alf: ALF::new(),
            dtiqs: vec![],
            ct: vec![],
            tile: tile.clone(),
            x_tile,
            y_tile,
            width_tile,
            height_tile,
        }));
        {
            let ct = CodingTree::new_arc_mutex(
                x,
                y,
                log2_width,
                log2_height,
                0,
                0,
                0,
                0,
                true,
                true,
                None,
                Some(ctu.clone()),
                fixed_qp,
                MttSplitMode::SPLIT_NONE,
                TreeType::SINGLE_TREE,
                ModeType::MODE_TYPE_ALL,
                tile,
            );
            let tmp = &mut ctu.lock().unwrap();
            tmp.ct = vec![ct];
        }
        ctu
    }

    pub fn set_tile(&mut self, tile: Arc<Mutex<Tile>>) {
        self.tile = Some(tile.clone());
        for dtiq in self.dtiqs.iter() {
            let dtiq = &mut dtiq.lock().unwrap();
            dtiq.set_tile(tile.clone());
        }
        for t in self.ct.iter() {
            let t = &mut t.lock().unwrap();
            t.set_tile(tile.clone());
        }
        let tile = tile.lock().unwrap();
        self.x_tile = tile.ctu_col << tile.log2_ctu_size;
        self.y_tile = tile.ctu_row << tile.log2_ctu_size;
        self.width_tile = tile.num_ctu_cols << tile.log2_ctu_size;
        self.height_tile = tile.num_ctu_rows << tile.log2_ctu_size;
    }

    pub fn _left_ctu_in_tile(&self) -> Option<Arc<Mutex<CodingTreeUnit>>> {
        if let Some(tile) = &self.tile {
            let tile = tile.lock().unwrap();
            let row = self.y / self.height - tile.ctu_row;
            let col = self.x / self.width - tile.ctu_col;
            if col > 0 {
                let ctus = tile.ctus.lock().unwrap();
                Some(ctus[row][col - 1].clone())
            } else {
                None
            }
        } else {
            None
        }
    }

    pub fn _above_ctu_in_tile(&self) -> Option<Arc<Mutex<CodingTreeUnit>>> {
        if let Some(tile) = &self.tile {
            let tile = tile.lock().unwrap();
            let row = self.y / self.height - tile.ctu_row;
            let col = self.x / self.width - tile.ctu_col;
            if row > 0 {
                let ctus = tile.ctus.lock().unwrap();
                Some(ctus[row - 1][col].clone())
            } else {
                None
            }
        } else {
            None
        }
    }

    pub fn get_cu(&self, x: usize, y: usize) -> Option<Arc<Mutex<CodingUnit>>> {
        if x < self.x || x >= self.x + self.width || y < self.y || y >= self.y + self.height {
            None
        } else if !self.dtiqs.is_empty() {
            if let Some(dtiq) = self.dtiqs.first() {
                let dtiq = dtiq.lock().unwrap();
                return dtiq.get_cu(x, y);
            }
            panic!();
        } else {
            if let Some(ct) = self.ct.first() {
                let ct = ct.lock().unwrap();
                return ct.get_cu(x, y);
            }
            panic!();
        }
    }
}

pub struct PaletteCoding {
    pub palette_predictor_run: Vec<usize>,
    pub num_signalled_palette_entries: usize,
    pub new_palette_entries: Vec<Vec<usize>>,
    pub palette_escape_val_present_flag: bool,
    pub palette_transpose_flag: bool,
    pub cu_qp_delta_abs: usize,
    pub cu_qp_delta_sign_flag: bool,
    pub cu_chroma_qp_offset_flag: bool,
    pub cu_chroma_qp_offset_idx: usize,
    pub run_copy_flag: Vec<Vec<bool>>,
    pub copy_above_palette_indices_flag: Vec<Vec<bool>>,
    pub palette_idx_idc: Vec<Vec<usize>>,
    pub palette_escape_val: Vec<usize>,
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct MergeData {
    pub merge_idx: usize,
    pub merge_subblock_flag: bool,
    pub merge_subblock_idx: usize,
    pub regular_merge_flag: bool,
    pub mmvd_merge_flag: bool,
    pub mmvd_cand_flag: bool,
    pub mmvd_distance_idx: usize,
    pub mmvd_direction_idx: usize,
    pub ciip_flag: bool,
    pub merge_gpm_partition_idx: usize,
    pub merge_gpm_idx0: usize,
    pub merge_gpm_idx1: usize,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct MvdCoding {
    pub abs_mvd_greater0_flag: [bool; 2],
    pub abs_mvd_greater1_flag: [bool; 2],
    pub abs_mvd: [usize; 2],
    pub sign_flag: [bool; 2],
}

pub struct TransformUnit {
    pub qp: usize,
    pub cu_chroma_qp_offset_flag: bool,
    pub cu_chroma_qp_offset_idx: usize,
    pub joint_cbcr_residual_flag: bool,
    pub transform_skip_flag: [bool; 3],
    pub x: usize,
    pub y: usize,
    pub width: usize,
    pub height: usize,
    pub log2_tb_width: usize,
    pub log2_tb_height: usize,
    pub parent: Arc<Mutex<TransformTree>>,
    pub sub_tu_index: usize,
    pub tree_type: TreeType,
    pub ch_type: usize,
    pub quantized_transformed_coeffs: Vec<Vec2d<i16>>,
    pub dequantized_transformed_coeffs: Vec<Vec2d<i16>>,
    pub transformed_coeffs: Vec<Vec2d<i16>>,
    pub itransformed_coeffs: Vec<Vec2d<i16>>,
    pub residuals: Vec<Vec2d<i16>>,
    pub tile: Option<Arc<Mutex<Tile>>>,
    pub ctu: Arc<Mutex<CodingTreeUnit>>,
    pub cu: Arc<Mutex<CodingUnit>>,
    pub above_right_available: RefCell<Option<bool>>,
    pub below_left_available: RefCell<Option<bool>>,
    pub x_tile: usize,
    pub y_tile: usize,
    pub width_tile: usize,
    pub height_tile: usize,
    pub ls_cache: [Option<Vec2d<i32>>; 3],
    pub bd_shift_cache: [Option<usize>; 3],
    pub derive_qp_cache: Option<(usize, usize, usize, usize)>, // TODO qp-wise cache?
    pub cu_bdpcm_flag: [bool; 3],
    pub cu_intra_luma_ref_idx: usize,
    pub cu_intra_subpartitions_mode_flag: bool,
    pub cu_intra_subpartitions_split_flag: bool,
    pub cu_intra_pred_mode: [IntraPredMode; 3],
    pub cu_size: [(usize, usize); 3],
    pub cu_pos: [(usize, usize); 3],
    pub cu_pred_mode_flag: bool,
    pub cu_act_enabled_flag: bool,
    pub log2_component_tb_size: [(usize, usize); 3],
}

impl TransformUnit {
    pub fn new_rc_refcell(
        x: usize,
        y: usize,
        log2_tb_width: usize,
        log2_tb_height: usize,
        ch_type: usize,
        parent: Arc<Mutex<TransformTree>>,
        fixed_qp: Option<usize>,
        tree_type: TreeType,
        tile: Option<Arc<Mutex<Tile>>>,
        cu: Arc<Mutex<CodingUnit>>,
    ) -> Rc<RefCell<TransformUnit>> {
        let qp = if let Some(qp) = fixed_qp { qp } else { 26 };
        let luma_width = 1 << log2_tb_width;
        let luma_height = 1 << log2_tb_height;
        let chroma_width = 1 << (log2_tb_width - 1);
        let chroma_height = 1 << (log2_tb_height - 1);
        let (x_tile, y_tile, width_tile, height_tile) = if let Some(tile) = &tile {
            let tile = tile.lock().unwrap();
            (
                tile.ctu_col << tile.log2_ctu_size,
                tile.ctu_row << tile.log2_ctu_size,
                tile.num_ctu_cols << tile.log2_ctu_size,
                tile.num_ctu_rows << tile.log2_ctu_size,
            )
        } else {
            (0, 0, 0, 0)
        };
        let (
            ctu,
            cu_bdpcm_flag,
            cu_intra_luma_ref_idx,
            cu_intra_subpartitions_mode_flag,
            cu_intra_subpartitions_split_flag,
            cu_intra_pred_mode,
            cu_size,
            cu_pos,
            cu_pred_mode_flag,
            cu_act_enabled_flag,
        ) = {
            let cu = cu.lock().unwrap();
            (
                cu.get_ctu(),
                [
                    cu.get_bdpcm_flag(0),
                    cu.get_bdpcm_flag(1),
                    cu.get_bdpcm_flag(2),
                ],
                cu.intra_luma_ref_idx,
                cu.intra_subpartitions_mode_flag,
                cu.intra_subpartitions_split_flag,
                cu.intra_pred_mode,
                [
                    cu.get_component_size(0),
                    cu.get_component_size(1),
                    cu.get_component_size(2),
                ],
                [
                    cu.get_component_pos(0),
                    cu.get_component_pos(1),
                    cu.get_component_pos(2),
                ],
                cu.pred_mode_flag,
                cu.act_enabled_flag,
            )
        };
        let v = match tree_type {
            TreeType::SINGLE_TREE => vec![
                vec2d![0; luma_height; luma_width],
                vec2d![0; chroma_height; chroma_width],
                vec2d![0; chroma_height; chroma_width],
            ],
            TreeType::DUAL_TREE_LUMA => vec![vec2d![0; luma_height; luma_width]],
            TreeType::DUAL_TREE_CHROMA => vec![
                vec2d![0; 1; 1],
                vec2d![0; chroma_height; chroma_width],
                vec2d![0; chroma_height; chroma_width],
            ],
        };
        Rc::new(RefCell::new(TransformUnit {
            width: luma_width,
            height: luma_height,
            qp,
            cu_chroma_qp_offset_flag: false,
            cu_chroma_qp_offset_idx: 0,
            joint_cbcr_residual_flag: false,
            transform_skip_flag: [false; 3],
            x,
            y,
            log2_tb_width,
            log2_tb_height,
            parent,
            sub_tu_index: 0,
            tree_type,
            ch_type,
            quantized_transformed_coeffs: v.clone(),
            dequantized_transformed_coeffs: v.clone(),
            transformed_coeffs: v.clone(),
            itransformed_coeffs: v.clone(),
            residuals: v,
            tile,
            cu,
            above_right_available: RefCell::new(None),
            below_left_available: RefCell::new(None),
            x_tile,
            y_tile,
            width_tile,
            height_tile,
            ctu,
            ls_cache: [None, None, None],
            bd_shift_cache: [None, None, None],
            derive_qp_cache: None,
            cu_bdpcm_flag,
            log2_component_tb_size: [
                (log2_tb_width, log2_tb_height),
                (log2_tb_width - 1, log2_tb_height - 1),
                (log2_tb_width - 1, log2_tb_height - 1),
            ],
            cu_intra_luma_ref_idx,
            cu_intra_subpartitions_mode_flag,
            cu_intra_subpartitions_split_flag,
            cu_intra_pred_mode,
            cu_size,
            cu_pos,
            cu_pred_mode_flag,
            cu_act_enabled_flag,
        }))
    }

    pub fn is_component_active(&self, c_idx: usize) -> bool {
        match self.tree_type {
            TreeType::DUAL_TREE_LUMA => c_idx == 0,
            TreeType::DUAL_TREE_CHROMA => c_idx != 0,
            _ => true,
        }
    }

    pub fn set_tile(&mut self, tile: Arc<Mutex<Tile>>) {
        (self.x_tile, self.y_tile, self.width_tile, self.height_tile) = {
            let tile = tile.lock().unwrap();
            (
                tile.ctu_col << tile.log2_ctu_size,
                tile.ctu_row << tile.log2_ctu_size,
                tile.num_ctu_cols << tile.log2_ctu_size,
                tile.num_ctu_rows << tile.log2_ctu_size,
            )
        };
        self.tile = Some(tile.clone());
        let tile = tile.lock().unwrap();
        self.x_tile = tile.ctu_col << tile.log2_ctu_size;
        self.y_tile = tile.ctu_row << tile.log2_ctu_size;
        self.width_tile = tile.num_ctu_cols << tile.log2_ctu_size;
        self.height_tile = tile.num_ctu_rows << tile.log2_ctu_size;
    }

    pub fn is_below_left_available(&self) -> bool {
        if let Some(available) = *self.below_left_available.borrow() {
            return available;
        }
        {
            if self.y + self.height >= self.y_tile + self.height_tile {
                *self.below_left_available.borrow_mut() = Some(false);
                return false;
            }
        }
        let tt = self.parent.lock().unwrap();
        if tt.tus.len() > 1 {
            if tt.x < self.x {
                *self.below_left_available.borrow_mut() = Some(false);
                false
            } else if self.y + self.height < tt.y + tt.height {
                let available = self.x_tile < self.x;
                *self.below_left_available.borrow_mut() = Some(false);
                available
            } else {
                let available = tt.is_below_left_available();
                *self.below_left_available.borrow_mut() = Some(available);
                available
            }
        } else {
            let available = tt.is_below_left_available();
            *self.below_left_available.borrow_mut() = Some(available);
            available
        }
    }

    pub fn is_above_right_available(&self) -> bool {
        if let Some(available) = *self.above_right_available.borrow() {
            return available;
        }
        {
            if self.x + self.width >= self.x_tile + self.width_tile {
                *self.above_right_available.borrow_mut() = Some(false);
                return false;
            }
        }
        let tt = self.parent.lock().unwrap();
        if tt.tus.len() > 1 {
            if tt.width > tt.height {
                if self.x + self.width < tt.x + tt.width {
                    let available = self.y_tile < self.y;
                    *self.above_right_available.borrow_mut() = Some(available);
                    available
                } else {
                    let available = tt.is_above_right_available();
                    *self.above_right_available.borrow_mut() = Some(available);
                    available
                }
            } else if self.y == tt.y {
                let available = tt.is_above_right_available();
                *self.above_right_available.borrow_mut() = Some(available);
                available
            } else {
                *self.above_right_available.borrow_mut() = Some(false);
                false
            }
        } else {
            let available = tt.is_above_right_available();
            *self.above_right_available.borrow_mut() = Some(available);
            available
        }
    }

    #[inline(always)]
    pub fn get_component_size(&self, c_idx: usize) -> (usize, usize) {
        if c_idx == 0 {
            (self.width, self.height)
        } else {
            // FIXME
            (self.width / 2, self.height / 2)
        }
    }

    #[inline(always)]
    pub fn get_component_pos(&self, c_idx: usize) -> (usize, usize) {
        if c_idx == 0 {
            (self.x, self.y)
        } else {
            // FIXME
            (self.x / 2, self.y / 2)
        }
    }

    #[inline(always)]
    pub fn get_cu(&self) -> Arc<Mutex<CodingUnit>> {
        self.cu.clone()
    }

    #[inline(always)]
    pub fn get_tile(&self) -> Arc<Mutex<Tile>> {
        let tile = self.tile.as_ref().unwrap();
        tile.clone()
    }

    #[inline(always)]
    pub fn is_in_first_qg_in_slice_or_tile(&self, ectx: &EncoderContext) -> bool {
        let x_qg = ectx.cu_qg_top_left_x;
        let y_qg = ectx.cu_qg_top_left_y;
        // FIXME check slice
        x_qg == self.x_tile && y_qg == self.y_tile
    }

    #[inline(always)]
    pub fn is_in_first_qg_in_ctb_row_in_tile(&self, ectx: &EncoderContext) -> bool {
        let x_qg = ectx.cu_qg_top_left_x;
        let tile = self.get_tile();
        let tile = tile.lock().unwrap();
        x_qg == tile.ctu_col << tile.log2_ctu_size
    }

    pub fn get_cu_qp_delta(
        &self,
        sps: &SequenceParameterSet,
        pps: &PictureParameterSet,
        ectx: &EncoderContext,
    ) -> isize {
        let is_in_first_qg_in_slice_or_tile = self.is_in_first_qg_in_slice_or_tile(ectx);
        let x_qg = ectx.cu_qg_top_left_x;
        let y_qg = ectx.cu_qg_top_left_y;
        let qp_y_prev = if is_in_first_qg_in_slice_or_tile {
            ectx.slice_qp_y as usize
        } else {
            ectx.qp_y
        };
        let (cu_x, cu_y) = self.cu_pos[0];
        let (cu_width, cu_height) = self.cu_size[0];
        let available_a = ectx.derive_neighbouring_block_availability(
            cu_x,
            cu_y,
            x_qg as isize - 1,
            y_qg as isize,
            cu_width,
            cu_height,
            false,
            false,
            false,
            sps,
            pps,
        );
        let tile = self.get_tile();
        let tile = tile.lock().unwrap();
        let qp_y_a = if !available_a
            || (x_qg as isize - 1) >> ectx.ctb_log2_size_y != cu_x as isize >> ectx.ctb_log2_size_y
            || y_qg >> ectx.ctb_log2_size_y != cu_y >> ectx.ctb_log2_size_y
        {
            qp_y_prev
        } else {
            let left_qg_cu = tile.get_cu(x_qg as isize - 1, y_qg as isize);
            let left_qg_cu = left_qg_cu.as_ref().unwrap();
            let left_qg_cu = left_qg_cu.lock().unwrap();
            left_qg_cu.qp_y // qp of the CodingUnit containing the luma coding block covering (x_qg-1, y_qg)
        };
        let available_b = ectx.derive_neighbouring_block_availability(
            cu_x,
            cu_y,
            x_qg as isize,
            y_qg as isize - 1,
            cu_width,
            cu_height,
            false,
            false,
            false,
            sps,
            pps,
        );
        let qp_y_b = if !available_b
            || x_qg >> ectx.ctb_log2_size_y != cu_x >> ectx.ctb_log2_size_y
            || (y_qg as isize - 1) >> ectx.ctb_log2_size_y != cu_y as isize >> ectx.ctb_log2_size_y
        {
            qp_y_prev
        } else {
            let above_qg_cu = tile.get_cu(x_qg as isize, y_qg as isize - 1);
            let above_qg_cu = above_qg_cu.as_ref().unwrap();
            let above_qg_cu = above_qg_cu.lock().unwrap();
            above_qg_cu.qp_y
        };
        let qp_y_pred = if available_a && is_in_first_qg_in_slice_or_tile {
            let above_qg_cu = tile.get_cu(x_qg as isize, y_qg as isize - 1);
            let above_qg_cu = above_qg_cu.as_ref().unwrap();
            let above_qg_cu = above_qg_cu.lock().unwrap();
            above_qg_cu.qp_y
        } else {
            (qp_y_a + qp_y_b + 1) >> 1
        };
        let cu_qp_delta_val = self.qp as isize - ectx.qp_bd_offset - qp_y_pred as isize;
        debug_assert!(cu_qp_delta_val == 0);
        cu_qp_delta_val
    }

    #[inline(always)]
    pub fn get_sig_coeff_flag(&self, c_idx: usize, x_c: usize, y_c: usize) -> bool {
        self.quantized_transformed_coeffs[c_idx][y_c][x_c] != 0
    }

    pub fn get_sb_coded_flag(&self, c_idx: usize, x_s: usize, y_s: usize) -> bool {
        let (log2_w_s, log2_h_s) = self.get_log2_sb_size(c_idx);
        let w_s = 1 << log2_w_s;
        let h_s = 1 << log2_h_s;
        for y in y_s << log2_h_s..(y_s << log2_h_s) + h_s {
            let qtc = &self.quantized_transformed_coeffs[c_idx][y];
            for qtcx in qtc.iter().skip(x_s << log2_w_s).take(w_s) {
                if qtcx != &0 {
                    return true;
                }
            }
        }
        false
    }

    pub fn get_dec_abs_level(
        &self,
        abs_level: i16,
        c_idx: usize,
        x_c: usize,
        y_c: usize,
        q_state: usize,
        ectx: &EncoderContext,
    ) -> usize {
        //let v = self.quantized_transformed_coeffs[c_idx][y_c][x_c].abs();
        let v = abs_level;
        let (log2_tb_width, log2_tb_height) = self.get_log2_tb_size(c_idx);
        let base_level = 0;
        let mut loc_sum_abs = 0;
        if x_c < (1 << log2_tb_width) - 1 {
            loc_sum_abs += ectx.abs_level[y_c][x_c + 1];
            if x_c < (1 << log2_tb_width) - 2 {
                //loc_sum_abs += self.quantized_transformed_coeffs[c_idx][y_c][x_c + 2].abs();
                loc_sum_abs += ectx.abs_level[y_c][x_c + 2];
            }
            if y_c < (1 << log2_tb_height) - 1 {
                //loc_sum_abs += self.quantized_transformed_coeffs[c_idx][y_c + 1][x_c + 1].abs();
                loc_sum_abs += ectx.abs_level[y_c + 1][x_c + 1];
            }
        }
        if y_c < (1 << log2_tb_height) - 1 {
            //loc_sum_abs += self.quantized_transformed_coeffs[c_idx][y_c + 1][x_c].abs();
            loc_sum_abs += ectx.abs_level[y_c + 1][x_c];
            if y_c < (1 << log2_tb_height) - 2 {
                //loc_sum_abs += self.quantized_transformed_coeffs[c_idx][y_c + 2][x_c].abs();
                loc_sum_abs += ectx.abs_level[y_c + 2][x_c];
            }
        }
        loc_sum_abs = (loc_sum_abs - base_level * 5).clamp(0, 31);
        let c_rice_param = c_rice_params[loc_sum_abs];
        let zero_pos = (if q_state < 2 { 1 } else { 2 }) << c_rice_param;
        (if v == 0 {
            zero_pos
        } else if zero_pos >= v {
            v - 1
        } else {
            v
        }) as usize
    }

    #[inline(always)]
    pub fn get_log2_tb_size(&self, c_idx: usize) -> (usize, usize) {
        // FIXME
        self.log2_component_tb_size[c_idx]
        //if c_idx == 0 {
        //(self.log2_tb_width, self.log2_tb_height)
        //} else {
        //let (dw, dh) = match sps.chroma_format {
        //ChromaFormat::Monochrome => panic!(),
        //ChromaFormat::YCbCr420 => (1, 1),
        //ChromaFormat::YCbCr422 => (1, 0),
        //ChromaFormat::YCbCr444 => (0, 0),
        //};
        //(self.log2_tb_width - dw, self.log2_tb_height - dh)
        //}
    }

    pub fn get_log2_zo_tb_size(&self, sps: &SequenceParameterSet, c_idx: usize) -> (usize, usize) {
        let cu = self.get_cu();
        let cu = cu.lock().unwrap();
        let log2_tb_width: usize = if sps.mts_enabled_flag
            && cu.sbt_flag
            && c_idx == 0
            && self.get_log2_tb_size(c_idx).0 == 5
            && self.get_log2_tb_size(c_idx).1 < 6
        {
            4
        } else {
            self.get_log2_tb_size(c_idx).0.min(5)
        };
        let log2_tb_height: usize = if sps.mts_enabled_flag
            && cu.sbt_flag
            && c_idx == 0
            && self.get_log2_tb_size(c_idx).1 == 5
            && self.get_log2_tb_size(c_idx).0 < 6
        {
            4
        } else {
            self.get_log2_tb_size(c_idx).1.min(5)
        };
        (log2_tb_width, log2_tb_height)
    }

    pub fn get_log2_sb_size(&self, c_idx: usize) -> (usize, usize) {
        let (log2_tb_width, log2_tb_height) = self.get_log2_tb_size(c_idx);
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
        (log2_sb_w, log2_sb_h)
    }

    pub fn get_log2_zo_sb_size(&self, sps: &SequenceParameterSet, c_idx: usize) -> (usize, usize) {
        let (log2_tb_width, log2_tb_height) = self.get_log2_zo_tb_size(sps, c_idx);
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
        (log2_sb_w, log2_sb_h)
    }

    pub fn get_last_sig_coeff_pos(&self, c_idx: usize) -> (usize, usize) {
        let (log2_tb_width, log2_tb_height) = self.get_log2_tb_size(c_idx);
        let (log2_sb_w, log2_sb_h) = self.get_log2_sb_size(c_idx);
        let num_sb_coeff = 1 << (log2_sb_w + log2_sb_h);
        let mut last_scan_pos = num_sb_coeff;
        let mut last_sub_block =
            (1 << (log2_tb_width + log2_tb_height - (log2_sb_w + log2_sb_h))) - 1;
        let mut last_significant_coeff_x;
        let mut last_significant_coeff_y;
        let coeff_order = &DIAG_SCAN_ORDER[log2_sb_h][log2_sb_w];
        let sb_order = &DIAG_SCAN_ORDER[log2_tb_height - log2_sb_h][log2_tb_width - log2_sb_w];
        let q = &self.quantized_transformed_coeffs[c_idx];
        let (mut x_s, mut y_s) = sb_order[last_sub_block];
        let (mut x_0, mut y_0) = (x_s << log2_sb_w, y_s << log2_sb_h);
        let mut is_not_first_sub_block = last_sub_block > 0;
        while {
            if last_scan_pos == 0 {
                last_scan_pos = num_sb_coeff;
                last_sub_block -= 1;
                is_not_first_sub_block = last_sub_block > 0;
                (x_s, y_s) = sb_order[last_sub_block];
                (x_0, y_0) = (x_s << log2_sb_w, y_s << log2_sb_h);
            }
            last_scan_pos -= 1;
            let x_c = x_0 + coeff_order[last_scan_pos].0;
            let y_c = y_0 + coeff_order[last_scan_pos].1;
            last_significant_coeff_x = x_c;
            last_significant_coeff_y = y_c;
            let is_zero = q[y_c][x_c] == 0;
            is_zero && (last_scan_pos > 0 || is_not_first_sub_block)
        } {}
        (last_significant_coeff_x, last_significant_coeff_y)
    }

    #[inline(always)]
    pub fn get_y_coded_flag(&self) -> bool {
        if self.tree_type == TreeType::DUAL_TREE_CHROMA {
            return false;
        }
        let qtc = &self.quantized_transformed_coeffs[0];
        for y in 0..self.height {
            let qtc = &qtc[y];
            for qtcx in qtc.iter().take(self.width) {
                if qtcx != &0 {
                    return true;
                }
            }
        }
        false
    }

    #[inline(always)]
    pub fn get_cb_coded_flag(&self) -> bool {
        if self.tree_type == TreeType::DUAL_TREE_LUMA {
            return false;
        }
        let qtc = &self.quantized_transformed_coeffs[1];
        for y in 0..self.height / 2 {
            let qtc = &qtc[y];
            for qtcx in qtc.iter().take(self.width / 2) {
                if qtcx != &0 {
                    return true;
                }
            }
        }
        false
    }

    #[inline(always)]
    pub fn get_cr_coded_flag(&self) -> bool {
        if self.tree_type == TreeType::DUAL_TREE_LUMA {
            return false;
        }
        let qtc = &self.quantized_transformed_coeffs[2];
        for y in 0..self.height / 2 {
            let qtc = &qtc[y];
            for qtcx in qtc.iter().take(self.width / 2) {
                if qtcx != &0 {
                    return true;
                }
            }
        }
        false
    }

    #[inline(always)]
    pub fn first_in_cu(&self) -> bool {
        self.prev_tu_in_cu().is_none()
    }

    #[inline(always)]
    pub fn prev_tu_in_cu(&self) -> Option<Rc<RefCell<TransformUnit>>> {
        let parent = self.parent.lock().unwrap();
        parent.prev_tu()
    }
}

pub struct TransformTree {
    pub x: usize,
    pub y: usize,
    pub width: usize,
    pub height: usize,
    pub tts: Vec<Arc<Mutex<TransformTree>>>,
    pub tus: Vec<Rc<RefCell<TransformUnit>>>,
    pub parent: Option<Arc<Mutex<TransformTree>>>,
    pub part_idx: usize,
    pub cu: Arc<Mutex<CodingUnit>>,
    pub tile: Option<Arc<Mutex<Tile>>>,
    pub above_right_available: RefCell<Option<bool>>,
    pub below_left_available: RefCell<Option<bool>>,
    pub x_tile: usize,
    pub y_tile: usize,
    pub width_tile: usize,
    pub height_tile: usize,
    pub tree_type: TreeType,
}

impl TransformTree {
    pub fn new_arc_mutex(
        x: usize,
        y: usize,
        log2_width: usize,
        log2_height: usize,
        ch_type: usize,
        parent: Option<Arc<Mutex<TransformTree>>>,
        cu: Arc<Mutex<CodingUnit>>,
        fixed_qp: Option<usize>,
        tree_type: TreeType,
        tile: Option<Arc<Mutex<Tile>>>,
    ) -> Arc<Mutex<TransformTree>> {
        let (x_tile, y_tile, width_tile, height_tile) = if let Some(tile) = &tile {
            let tile = tile.lock().unwrap();
            (
                tile.ctu_col << tile.log2_ctu_size,
                tile.ctu_row << tile.log2_ctu_size,
                tile.num_ctu_cols << tile.log2_ctu_size,
                tile.num_ctu_rows << tile.log2_ctu_size,
            )
        } else {
            (0, 0, 0, 0)
        };
        let tt = Arc::new(Mutex::new(TransformTree {
            x,
            y,
            width: 1 << log2_width,
            height: 1 << log2_height,
            tts: vec![],
            tus: vec![],
            parent,
            part_idx: 0,
            cu: cu.clone(),
            tile: tile.clone(),
            above_right_available: RefCell::new(None),
            below_left_available: RefCell::new(None),
            x_tile,
            y_tile,
            width_tile,
            height_tile,
            tree_type,
        }));
        let cur = tt.clone();
        {
            let tmp = &mut tt.lock().unwrap();
            tmp.tus = vec![TransformUnit::new_rc_refcell(
                x,
                y,
                log2_width,
                log2_height,
                ch_type,
                cur,
                fixed_qp,
                tree_type,
                tile,
                cu,
            )];
        }
        tt
    }

    pub fn set_tile(&mut self, tile: Arc<Mutex<Tile>>) {
        self.tile = Some(tile.clone());
        for tt in self.tts.iter() {
            let tt = &mut tt.lock().unwrap();
            tt.set_tile(tile.clone());
        }
        for tu in self.tus.iter() {
            let mut tu = tu.borrow_mut();
            tu.set_tile(tile.clone());
        }
        let tile = tile.lock().unwrap();
        self.x_tile = tile.ctu_col << tile.log2_ctu_size;
        self.y_tile = tile.ctu_row << tile.log2_ctu_size;
        self.width_tile = tile.num_ctu_cols << tile.log2_ctu_size;
        self.height_tile = tile.num_ctu_rows << tile.log2_ctu_size;
    }

    pub fn set_cu_intra_pred_mode(&mut self, intra_pred_mode: [IntraPredMode; 3]) {
        if self.tts.is_empty() {
            for tu in self.tus.iter() {
                let tu = &mut tu.borrow_mut();
                tu.cu_intra_pred_mode = intra_pred_mode;
            }
        } else {
            for tt in self.tts.iter() {
                let tt = &mut tt.lock().unwrap();
                tt.set_cu_intra_pred_mode(intra_pred_mode);
            }
        }
    }

    pub fn is_below_left_available(&self) -> bool {
        if let Some(available) = *self.below_left_available.borrow() {
            return available;
        }
        {
            if self.y + self.height >= self.y_tile + self.height_tile {
                *self.below_left_available.borrow_mut() = Some(false);
                return false;
            }
        }
        let tt = self.parent.as_ref();
        if let Some(tt) = tt {
            let tt = tt.lock().unwrap();
            if tt.tts.len() > 1 {
                if tt.x < self.x {
                    *self.below_left_available.borrow_mut() = Some(false);
                    false
                } else if self.y + self.height < tt.y + tt.height {
                    let available = self.x_tile < self.x;
                    *self.below_left_available.borrow_mut() = Some(false);
                    available
                } else {
                    let available = tt.is_below_left_available();
                    *self.below_left_available.borrow_mut() = Some(available);
                    available
                }
            } else {
                let available = tt.is_below_left_available();
                *self.below_left_available.borrow_mut() = Some(available);
                available
            }
        } else {
            let cu = self.cu.lock().unwrap();
            let available = cu.is_below_left_available();
            *self.below_left_available.borrow_mut() = Some(available);
            available
        }
    }

    pub fn is_above_right_available(&self) -> bool {
        if let Some(available) = *self.above_right_available.borrow() {
            return available;
        }
        if self.x + self.width >= self.x_tile + self.width_tile {
            *self.above_right_available.borrow_mut() = Some(false);
            return false;
        }
        let tt = self.parent.as_ref();
        if let Some(tt) = tt {
            let tt = tt.lock().unwrap();
            if tt.width > tt.height {
                if self.x + self.width < tt.x + tt.width {
                    let available = self.y_tile < self.y;
                    *self.above_right_available.borrow_mut() = Some(available);
                    available
                } else {
                    let available = tt.is_above_right_available();
                    *self.above_right_available.borrow_mut() = Some(available);
                    available
                }
            } else if self.y == tt.y {
                let available = tt.is_above_right_available();
                *self.above_right_available.borrow_mut() = Some(available);
                available
            } else {
                *self.above_right_available.borrow_mut() = Some(false);
                false
            }
        } else {
            let cu = self.cu.lock().unwrap();
            let available = cu.is_above_right_available();
            *self.above_right_available.borrow_mut() = Some(available);
            available
        }
    }

    #[inline(always)]
    pub fn first_tu(&self) -> Rc<RefCell<TransformUnit>> {
        if !self.tts.is_empty() {
            let tt = self.tts[0].lock().unwrap();
            tt.first_tu()
        } else {
            self.tus[0].clone()
        }
    }

    #[inline(always)]
    pub fn last_tu(&self) -> Rc<RefCell<TransformUnit>> {
        if !self.tts.is_empty() {
            let tt = self.tts.last().unwrap().lock().unwrap();
            tt.last_tu()
        } else {
            self.tus[0].clone()
        }
    }

    #[inline(always)]
    pub fn prev_tu(&self) -> Option<Rc<RefCell<TransformUnit>>> {
        match &self.parent {
            Some(parent) => {
                let parent = parent.lock().unwrap();
                if self.part_idx > 0 {
                    let tt = parent.tts[self.part_idx - 1].lock().unwrap();
                    Some(tt.last_tu())
                } else {
                    parent.prev_tu()
                }
            }
            None => None,
        }
    }
}

pub struct CodingUnit {
    pub skip_flag: bool,
    pub pred_mode: [ModeType; 2],
    pub pred_mode_flag: bool,
    pub pred_mode_ibc_flag: bool,
    pub pred_mode_plt_flag: bool,
    pub act_enabled_flag: bool,
    pub palette_coding: Vec<PaletteCoding>,
    pub intra_bdpcm_luma_flag: bool,
    pub intra_bdpcm_luma_dir_flag: bool,
    pub intra_mip_flag: bool,
    pub intra_mip_transposed_flag: bool,
    pub intra_mip_mode: usize,
    pub intra_luma_ref_idx: usize,
    pub intra_subpartitions_mode_flag: bool,
    pub intra_subpartitions_split_flag: bool,
    pub intra_luma_mpm_flag: bool,
    pub intra_luma_mpm_idx: usize,
    pub intra_luma_mpm_remainer: usize,
    pub intra_bdpcm_chroma_flag: bool,
    pub intra_bdpcm_chroma_dir_flag: bool,
    pub intra_chroma_pred_mode: usize,
    pub general_merge_flag: bool,
    pub merge_data: Option<MergeData>,
    pub mvd_coding: Vec<Vec<MvdCoding>>,
    pub mvp_l0_flag: bool,
    pub mvp_l1_flag: bool,
    pub amvr_precision_idx: usize,
    pub inter_pred_idc: usize,
    pub inter_affine_flag: bool,
    pub affine_type_flag: bool,
    pub sym_mvd_flag: bool,
    pub ref_idx: [usize; 2],
    pub amvr_flag: bool,
    pub bcw_idx: usize,
    pub coded_flag: bool,
    pub sbt_flag: bool,
    pub sbt_quad_flag: bool,
    pub sbt_horizontal_flag: bool,
    pub sbt_pos_flag: bool,
    pub transform_tree: Option<Arc<Mutex<TransformTree>>>,
    pub lfnst_idx: usize,
    pub mts_idx: usize,
    pub width: usize,
    pub height: usize,
    pub idx: usize,
    pub x: usize,
    pub y: usize,
    pub mode_type: ModeType,
    pub tree_type: TreeType,
    pub parent: Arc<Mutex<CodingTree>>,
    pub intra_pred_mode: [IntraPredMode; 3],
    pub qp_y: usize,
    pub tile: Option<Arc<Mutex<Tile>>>,
    pub above_right_available: RefCell<Option<bool>>,
    pub below_left_available: RefCell<Option<bool>>,
    pub ctu: Option<Arc<Mutex<CodingTreeUnit>>>,
    pub x_tile: usize,
    pub y_tile: usize,
    pub width_tile: usize,
    pub height_tile: usize,
}

impl CodingUnit {
    pub fn new_arc_mutex(
        x: usize,
        y: usize,
        log2_width: usize,
        log2_height: usize,
        ch_type: usize,
        parent: Arc<Mutex<CodingTree>>,
        fixed_qp: Option<usize>,
        tree_type: TreeType,
        tile: Option<Arc<Mutex<Tile>>>,
    ) -> Arc<Mutex<CodingUnit>> {
        let qp = if let Some(qp) = fixed_qp { qp } else { 26 };
        let (x_tile, y_tile, width_tile, height_tile) = if let Some(tile) = &tile {
            let tile = tile.lock().unwrap();
            (
                tile.ctu_col << tile.log2_ctu_size,
                tile.ctu_row << tile.log2_ctu_size,
                tile.num_ctu_cols << tile.log2_ctu_size,
                tile.num_ctu_rows << tile.log2_ctu_size,
            )
        } else {
            (0, 0, 0, 0)
        };
        let cu = Arc::new(Mutex::new(CodingUnit {
            skip_flag: false,
            pred_mode: [ModeType::MODE_INTRA; 2],
            pred_mode_flag: true,
            pred_mode_ibc_flag: false,
            pred_mode_plt_flag: false,
            act_enabled_flag: false,
            palette_coding: vec![],
            intra_bdpcm_luma_flag: false,
            intra_bdpcm_luma_dir_flag: false,
            intra_mip_flag: false,
            intra_mip_transposed_flag: false,
            intra_mip_mode: 0,
            intra_luma_ref_idx: 0,
            intra_subpartitions_mode_flag: false,
            intra_subpartitions_split_flag: false,
            intra_luma_mpm_flag: true,
            intra_luma_mpm_idx: 0,
            intra_luma_mpm_remainer: 0,
            intra_bdpcm_chroma_flag: false,
            intra_bdpcm_chroma_dir_flag: false,
            intra_chroma_pred_mode: 4,
            general_merge_flag: false,
            merge_data: None,
            mvd_coding: vec![],
            mvp_l0_flag: false,
            mvp_l1_flag: false,
            amvr_precision_idx: 0,
            inter_pred_idc: 0,
            inter_affine_flag: false,
            affine_type_flag: false,
            sym_mvd_flag: false,
            ref_idx: [0, 1],
            amvr_flag: false,
            bcw_idx: 0,
            coded_flag: true,
            sbt_flag: false,
            sbt_quad_flag: false,
            sbt_horizontal_flag: false,
            sbt_pos_flag: false,
            transform_tree: None,
            lfnst_idx: 0,
            mts_idx: 0,
            width: 1 << log2_width,
            height: 1 << log2_height,
            idx: 0,
            x,
            y,
            mode_type: ModeType::MODE_INTRA,
            tree_type,
            parent,
            intra_pred_mode: [IntraPredMode::PLANAR; 3],
            qp_y: qp,
            tile: tile.clone(),
            above_right_available: RefCell::new(None),
            below_left_available: RefCell::new(None),
            ctu: None,
            x_tile,
            y_tile,
            width_tile,
            height_tile,
        }));
        let tt = TransformTree::new_arc_mutex(
            x,
            y,
            log2_width,
            log2_height,
            ch_type,
            None,
            cu.clone(),
            fixed_qp,
            tree_type,
            tile,
        );
        {
            let cu = &mut cu.lock().unwrap();
            let ctu = cu.get_ctu();
            cu.ctu = Some(ctu);
            cu.transform_tree = Some(tt);
        }
        cu
    }

    pub fn set_tile(&mut self, tile: Arc<Mutex<Tile>>) {
        self.tile = Some(tile.clone());
        let tt = self.transform_tree.as_ref().unwrap();
        let tt = &mut tt.lock().unwrap();
        tt.set_tile(tile.clone());
        let tile = tile.lock().unwrap();
        self.x_tile = tile.ctu_col << tile.log2_ctu_size;
        self.y_tile = tile.ctu_row << tile.log2_ctu_size;
        self.width_tile = tile.num_ctu_cols << tile.log2_ctu_size;
        self.height_tile = tile.num_ctu_rows << tile.log2_ctu_size;
    }

    pub fn set_intra_pred_mode(&mut self, intra_pred_mode: [IntraPredMode; 3]) {
        self.intra_pred_mode = intra_pred_mode;
        let (intra_chroma_pred_mode, _) =
            self.get_intra_chroma_pred_mode_and_mip_chroma_direct_mode_flag();
        self.intra_pred_mode[1] = intra_chroma_pred_mode;
        self.intra_pred_mode[2] = intra_chroma_pred_mode;
        let tt = self.transform_tree.as_ref().unwrap();
        let tt = &mut tt.lock().unwrap();
        tt.set_cu_intra_pred_mode(intra_pred_mode);
    }

    pub fn is_cclm_enabled(&self, sh: &SliceHeader, ectx: &EncoderContext) -> bool {
        // cross-component chroma intra prediction mode checking process (8.4.4)
        if sh.sps.cclm_enabled_flag {
            if !sh.sps.partition_constraints.qtbtt_dual_tree_intra_flag
                || sh.slice_type != SliceType::I
                || ectx.ctb_log2_size_y < 6
            {
                true
            } else {
                // FIXME
                panic!();
                #[allow(unreachable_code)]
                false
                // TODO
                //let ct = self.parent.lock().unwrap();
                //(self.width / 2 == 64 && self.height / 2 == 64)
                //|| (ct.cqt_depth == ectx.ctb_log2_size_y - 6
                //&& ct.split_mode == MttSplitMode::SPLIT_BT_HOR
                //&& self.width / 2 == 64
                //&& self.height / 2 == 32)
                //|| (ct.cqt_depth > ectx.ctb_log2_size_y - 6)
                //|| (ct.cqt_depth==ectx.ctb_log2_size_y-6 && ct.split_mode==MttSplitMode::SPLIT_BT_HOR&&ct.)
            }
        } else {
            false
        }
    }

    pub fn get_cclm_mode_flag(&self) -> bool {
        matches!(
            self.intra_pred_mode[1],
            IntraPredMode::LT_CCLM | IntraPredMode::T_CCLM | IntraPredMode::L_CCLM
        )
    }

    pub fn get_cclm_mode_idx(&self) -> usize {
        if self.get_cclm_mode_flag() {
            self.intra_pred_mode[1] as usize - IntraPredMode::LT_CCLM as usize
        } else {
            0
        }
    }

    pub fn is_below_left_available(&self) -> bool {
        if let Some(available) = *self.below_left_available.borrow() {
            return available;
        }
        if self.y + self.height >= self.y_tile + self.height_tile {
            *self.below_left_available.borrow_mut() = Some(false);
            return false;
        }
        let ct = self.parent.lock().unwrap();
        if ct.cus.len() > 1 {
            // FIXME unreachable?
            if ct.x < self.x {
                *self.below_left_available.borrow_mut() = Some(false);
                false
            } else if self.y + self.height < ct.y + ct.height {
                let available = self.x_tile < self.x;
                *self.below_left_available.borrow_mut() = Some(false);
                available
            } else {
                let available = ct.is_below_left_available();
                *self.below_left_available.borrow_mut() = Some(available);
                available
            }
        } else {
            let available = ct.is_below_left_available();
            *self.below_left_available.borrow_mut() = Some(available);
            available
        }
    }

    pub fn is_above_right_available(&self) -> bool {
        if let Some(available) = *self.above_right_available.borrow() {
            return available;
        }
        if self.x + self.width >= self.x_tile + self.width_tile {
            *self.above_right_available.borrow_mut() = Some(false);
            return false;
        }
        let ct = self.parent.lock().unwrap();
        if ct.cus.len() > 1 {
            // FIXME unreachable?
            if ct.width > ct.height {
                if self.x + self.width < ct.x + ct.width {
                    let available = self.y_tile < self.y;
                    *self.above_right_available.borrow_mut() = Some(available);
                    available
                } else {
                    let available = ct.is_above_right_available();
                    *self.above_right_available.borrow_mut() = Some(available);
                    available
                }
            } else if self.y == ct.y {
                let available = ct.is_above_right_available();
                *self.above_right_available.borrow_mut() = Some(available);
                available
            } else {
                *self.above_right_available.borrow_mut() = Some(false);
                false
            }
        } else {
            let available = ct.is_above_right_available();
            *self.above_right_available.borrow_mut() = Some(available);
            available
        }
    }

    #[inline(always)]
    pub fn get_intra_luma_not_planar_flag(&self) -> bool {
        self.intra_pred_mode[0] != IntraPredMode::PLANAR
    }

    #[inline(always)]
    pub fn get_intra_luma_mpm_flag_and_idx_and_remainder(&self) -> (bool, usize, usize) {
        if self.intra_pred_mode[0] == IntraPredMode::PLANAR {
            return (true, 0, 0);
        }
        let tile = self.tile.as_ref().unwrap();
        let tile = tile.lock().unwrap();
        let left_cu = tile.get_cu(self.x as isize - 1, (self.y + self.height - 1) as isize);
        let above_cu = tile.get_cu((self.x + self.width - 1) as isize, self.y as isize - 1);
        let left_cand_pred_mode = if let Some(cu) = &left_cu {
            let cu = cu.lock().unwrap();
            if cu.pred_mode[0] != ModeType::MODE_INTRA || cu.intra_mip_flag {
                IntraPredMode::PLANAR
            } else {
                cu.intra_pred_mode[0]
            }
        } else {
            IntraPredMode::PLANAR
        };
        let above_cand_pred_mode = if let Some(cu) = &above_cu {
            let cu = cu.lock().unwrap();
            if cu.pred_mode[0] != ModeType::MODE_INTRA
                || cu.intra_mip_flag
                || self.y as isize - 1
                    < ((self.y >> tile.log2_ctu_size) << tile.log2_ctu_size) as isize
            {
                IntraPredMode::PLANAR
            } else {
                cu.intra_pred_mode[0]
            }
        } else {
            IntraPredMode::PLANAR
        };
        let mut cand_mode_list = if left_cand_pred_mode == above_cand_pred_mode
            && left_cand_pred_mode as usize > IntraPredMode::DC as usize
        {
            let mode = left_cand_pred_mode as usize;
            [
                mode,
                2 + (mode + 61) % 64,
                2 + (mode - 1) % 64,
                2 + (mode + 60) % 64,
                2 + mode % 64,
            ]
        } else if left_cand_pred_mode != above_cand_pred_mode
            && (left_cand_pred_mode as usize > IntraPredMode::DC as usize
                || above_cand_pred_mode as usize > IntraPredMode::DC as usize)
        {
            let min_left_right = (left_cand_pred_mode as usize).min(above_cand_pred_mode as usize);
            let max_left_right = (left_cand_pred_mode as usize).max(above_cand_pred_mode as usize);
            if min_left_right > IntraPredMode::DC as usize {
                let d = max_left_right - min_left_right;
                let left_mode = left_cand_pred_mode as usize;
                let above_mode = above_cand_pred_mode as usize;
                if d == 1 {
                    [
                        left_mode,
                        above_mode,
                        2 + (min_left_right + 61) % 64,
                        2 + (max_left_right - 1) % 64,
                        2 + (min_left_right + 60) % 64,
                    ]
                } else if d >= 62 {
                    [
                        left_mode,
                        above_mode,
                        2 + (min_left_right - 1) % 64,
                        2 + (max_left_right + 61) % 64,
                        2 + min_left_right % 64,
                    ]
                } else if d == 2 {
                    [
                        left_mode,
                        above_mode,
                        2 + (min_left_right - 1) % 64,
                        2 + (min_left_right + 61) % 64,
                        2 + (max_left_right - 1) % 64,
                    ]
                } else {
                    [
                        left_mode,
                        above_mode,
                        2 + (min_left_right + 61) % 64,
                        2 + (min_left_right - 1) % 64,
                        2 + (max_left_right + 61) % 64,
                    ]
                }
            } else {
                [
                    max_left_right,
                    2 + (max_left_right + 61) % 64,
                    2 + (max_left_right - 1) % 64,
                    2 + (max_left_right + 60) % 64,
                    2 + max_left_right % 64,
                ]
            }
        } else {
            [
                IntraPredMode::DC as usize,
                IntraPredMode::ANGULAR50 as usize,
                IntraPredMode::ANGULAR18 as usize,
                IntraPredMode::ANGULAR46 as usize,
                IntraPredMode::ANGULAR54 as usize,
            ]
        };
        let mode = self.intra_pred_mode[0] as usize;
        if cand_mode_list.contains(&mode) {
            let intra_luma_mpm_flag = true;
            let intra_luma_mpm_idx = cand_mode_list.iter().position(|&x| x == mode).unwrap();
            let intra_luma_mpm_remainer = 0;
            (
                intra_luma_mpm_flag,
                intra_luma_mpm_idx,
                intra_luma_mpm_remainer,
            )
        } else {
            cand_mode_list.sort();
            let intra_luma_mpm_flag = false;
            let intra_luma_mpm_idx = 0;
            let intra_luma_mpm_remainder = if mode > cand_mode_list[4] {
                mode - 6
            } else if mode > cand_mode_list[3] {
                mode - 5
            } else if mode > cand_mode_list[2] {
                mode - 4
            } else if mode > cand_mode_list[1] {
                mode - 3
            } else if mode > cand_mode_list[0] {
                mode - 2
            } else {
                mode - 1
            };
            (
                intra_luma_mpm_flag,
                intra_luma_mpm_idx,
                intra_luma_mpm_remainder,
            )
        }
    }

    pub fn get_intra_chroma_pred_mode_and_mip_chroma_direct_mode_flag(
        &self,
        //sps: &SequenceParameterSet,
    ) -> (IntraPredMode, bool) {
        // derivation process for chroma intra prediction mode (8.4.3)
        //if self.tree_type == TreeType::SINGLE_TREE
        ////&& sps.chroma_format == ChromaFormat::YCbCr444
        //&& false // FIXME
        //&& self.intra_chroma_pred_mode == 4
        //{
        //let mip_chroma_direct_mode_flag = true;
        //let intra_chroma_pred_mode = self.intra_pred_mode[0];
        //(intra_chroma_pred_mode, mip_chroma_direct_mode_flag)
        //} else
        {
            let mip_chroma_direct_mode_flag = false;
            let cu = &self; // FIXME?
            let intra_luma_pred_mode = if cu.intra_mip_flag {
                IntraPredMode::PLANAR
            } else if cu.pred_mode[0] == ModeType::MODE_IBC || cu.pred_mode[0] == ModeType::MODE_PLT
            {
                IntraPredMode::DC
            } else {
                cu.intra_pred_mode[0]
            };
            let intra_chroma_pred_mode = if self.act_enabled_flag {
                intra_luma_pred_mode
            } else if self.get_bdpcm_flag(1) {
                if self.get_bdpcm_dir(1) {
                    IntraPredMode::ANGULAR50
                } else {
                    IntraPredMode::ANGULAR18
                }
            } else {
                // Table 20
                let pred_mode_idx = if self.get_cclm_mode_flag() {
                    match self.get_cclm_mode_idx() {
                        0 => match intra_luma_pred_mode as usize {
                            0 => 81,
                            50 => 81,
                            18 => 81,
                            1 => 81,
                            _ => 81,
                        },
                        1 => match intra_luma_pred_mode as usize {
                            0 => 82,
                            50 => 82,
                            18 => 82,
                            1 => 82,
                            _ => 82,
                        },
                        2 => match intra_luma_pred_mode as usize {
                            0 => 83,
                            50 => 83,
                            18 => 83,
                            1 => 83,
                            _ => 83,
                        },
                        _ => panic!(),
                    }
                } else {
                    match self.intra_chroma_pred_mode {
                        0 => match intra_luma_pred_mode as usize {
                            0 => 66,
                            50 => 0,
                            18 => 0,
                            1 => 0,
                            _ => 0,
                        },
                        1 => match intra_luma_pred_mode as usize {
                            0 => 50,
                            50 => 66,
                            18 => 50,
                            1 => 50,
                            _ => 50,
                        },
                        2 => match intra_luma_pred_mode as usize {
                            0 => 18,
                            50 => 18,
                            18 => 66,
                            1 => 18,
                            _ => 18,
                        },
                        3 => match intra_luma_pred_mode as usize {
                            0 => 1,
                            50 => 1,
                            18 => 1,
                            1 => 66,
                            _ => 1,
                        },
                        4 => match intra_luma_pred_mode as usize {
                            0 => 0,
                            50 => 50,
                            18 => 18,
                            1 => 1,
                            n => n,
                        },
                        _ => panic!(),
                    }
                };
                num::FromPrimitive::from_usize(pred_mode_idx).unwrap()
            };
            (intra_chroma_pred_mode, mip_chroma_direct_mode_flag)
        }
    }

    #[inline(always)]
    pub fn get_ctu(&self) -> Arc<Mutex<CodingTreeUnit>> {
        if let Some(ctu) = &self.ctu {
            ctu.clone()
        } else {
            let ct = self.parent.lock().unwrap();
            let ctu = ct.ctu.as_ref().unwrap();
            ctu.clone()
        }
    }

    #[inline(always)]
    pub fn get_bdpcm_flag(&self, c_idx: usize) -> bool {
        if c_idx == 0 {
            self.intra_bdpcm_luma_flag
        } else {
            self.intra_bdpcm_chroma_flag
        }
    }

    #[inline(always)]
    pub fn get_bdpcm_dir(&self, c_idx: usize) -> bool {
        if c_idx == 0 {
            self.intra_bdpcm_luma_dir_flag
        } else {
            self.intra_bdpcm_chroma_dir_flag
        }
    }

    #[inline(always)]
    pub fn get_component_size(&self, c_idx: usize) -> (usize, usize) {
        if c_idx == 0 {
            (self.width, self.height)
        } else {
            // FIXME
            (self.width / 2, self.height / 2)
        }
    }

    #[inline(always)]
    pub fn get_component_pos(&self, c_idx: usize) -> (usize, usize) {
        if c_idx == 0 {
            (self.x, self.y)
        } else {
            // FIXME
            (self.x / 2, self.y / 2)
        }
    }
}

#[derive(Clone)]
pub struct CodingTree {
    pub mtt_split_cu_binary_flag: bool,
    pub non_inter_flag: bool,
    pub cts: Vec<Arc<Mutex<CodingTree>>>,
    pub cus: Vec<Arc<Mutex<CodingUnit>>>,
    pub width: usize,
    pub height: usize,
    pub depth: usize,
    pub cqt_depth: usize,
    pub tree_type: TreeType,
    pub mode_type: ModeType,
    pub x: usize,
    pub y: usize,
    pub depth_offset: usize,
    pub part_idx: usize,
    pub split_mode: MttSplitMode,
    pub qg_on_y: bool,
    pub qg_on_c: bool,
    pub parent: Option<Arc<Mutex<CodingTree>>>,
    pub ctu: Option<Arc<Mutex<CodingTreeUnit>>>,
    pub tile: Option<Arc<Mutex<Tile>>>,
    pub above_right_available: RefCell<Option<bool>>,
    pub below_left_available: RefCell<Option<bool>>,
    pub x_tile: usize,
    pub y_tile: usize,
    pub width_tile: usize,
    pub height_tile: usize,
}

impl CodingTree {
    pub fn new_arc_mutex(
        x: usize,
        y: usize,
        log2_width: usize,
        log2_height: usize,
        depth: usize,
        cqt_depth: usize,
        depth_offset: usize,
        part_idx: usize,
        qg_on_y: bool,
        qg_on_c: bool,
        parent: Option<Arc<Mutex<CodingTree>>>,
        ctu: Option<Arc<Mutex<CodingTreeUnit>>>,
        fixed_qp: Option<usize>,
        split_mode: MttSplitMode,
        tree_type: TreeType,
        mode_type: ModeType,
        tile: Option<Arc<Mutex<Tile>>>,
    ) -> Arc<Mutex<CodingTree>> {
        let (x_tile, y_tile, width_tile, height_tile) = if let Some(tile) = &tile {
            let tile = tile.lock().unwrap();
            (
                tile.ctu_col << tile.log2_ctu_size,
                tile.ctu_row << tile.log2_ctu_size,
                tile.num_ctu_cols << tile.log2_ctu_size,
                tile.num_ctu_rows << tile.log2_ctu_size,
            )
        } else {
            (0, 0, 0, 0)
        };
        let ct = Arc::new(Mutex::new(CodingTree {
            mtt_split_cu_binary_flag: false,
            non_inter_flag: false,
            cts: vec![],
            cus: vec![],
            width: 1 << log2_width,
            height: 1 << log2_height,
            depth,
            cqt_depth,
            tree_type,
            mode_type,
            x,
            y,
            depth_offset,
            part_idx,
            split_mode,
            qg_on_y,
            qg_on_c,
            parent,
            ctu,
            tile: tile.clone(),
            above_right_available: RefCell::new(None),
            below_left_available: RefCell::new(None),
            x_tile,
            y_tile,
            width_tile,
            height_tile,
        }));
        let parent = ct.clone();
        {
            // FIXME move to cu constructor
            let ch_type = (tree_type == TreeType::DUAL_TREE_CHROMA) as usize;
            let cu = CodingUnit::new_arc_mutex(
                x,
                y,
                log2_width,
                log2_height,
                ch_type,
                parent,
                fixed_qp,
                tree_type,
                tile,
            );
            let tmp = &mut ct.lock().unwrap();
            tmp.cus = vec![cu];
        }
        ct
    }

    #[inline(always)]
    pub fn get_component_size(&self, c_idx: usize) -> (usize, usize) {
        if c_idx == 0 {
            (self.width, self.height)
        } else {
            // FIXME
            (self.width / 2, self.height / 2)
        }
    }

    #[inline(always)]
    pub fn get_component_pos(&self, c_idx: usize) -> (usize, usize) {
        if c_idx == 0 {
            (self.x, self.y)
        } else {
            // FIXME
            (self.x / 2, self.y / 2)
        }
    }

    pub fn get_mode_type_condition(&self, sh: &SliceHeader) -> usize {
        if (sh.slice_type == SliceType::I
            && sh.sps.partition_constraints.qtbtt_dual_tree_intra_flag)
            || self.mode_type != ModeType::MODE_TYPE_ALL
            || sh.sps.chroma_format == ChromaFormat::Monochrome
            || sh.sps.chroma_format == ChromaFormat::YCbCr444
        {
            0
        } else if (self.width * self.height == 64 && self.get_split_qt_flag())
            || (self.width * self.height == 64
                && !self.get_split_qt_flag()
                && (self.split_mode == MttSplitMode::SPLIT_TT_HOR
                    || self.split_mode == MttSplitMode::SPLIT_TT_VER))
            || (self.width * self.height == 32
                && (self.split_mode == MttSplitMode::SPLIT_BT_HOR
                    || self.split_mode == MttSplitMode::SPLIT_BT_VER))
        {
            1
        } else if (self.width * self.height == 64
            && (self.split_mode == MttSplitMode::SPLIT_BT_HOR
                || self.split_mode == MttSplitMode::SPLIT_BT_VER)
            && sh.sps.chroma_format == ChromaFormat::YCbCr420)
            || (self.width * self.height == 128
                && (self.split_mode == MttSplitMode::SPLIT_TT_HOR
                    || self.split_mode == MttSplitMode::SPLIT_TT_VER)
                && sh.sps.chroma_format == ChromaFormat::YCbCr420)
            || (self.width == 8 && self.split_mode == MttSplitMode::SPLIT_BT_VER)
            || (self.width == 16
                && !self.get_split_qt_flag()
                && self.split_mode == MttSplitMode::SPLIT_TT_VER)
        {
            1 + (sh.slice_type != SliceType::I) as usize
        } else {
            0
        }
    }

    pub fn split(
        &mut self,
        split_mode: MttSplitMode,
        self_arc_mutex: Arc<Mutex<CodingTree>>,
        sh: &SliceHeader,
        ectx: &EncoderContext,
    ) {
        self.split_mode = split_mode;
        self.cus = vec![];
        let mode_type_condition = self.get_mode_type_condition(sh);
        let log2_width = self.width.ilog2() as usize;
        let log2_height = self.height.ilog2() as usize;
        let depth = self.depth + 1;
        let fixed_qp = if ectx.fixed_qp.is_some() {
            ectx.fixed_qp
        } else {
            Some(ectx.slice_qp_y as usize)
        };
        let parent = self_arc_mutex;
        let mode_type = match mode_type_condition {
            1 => ModeType::MODE_TYPE_INTRA,
            2 => {
                if self.non_inter_flag {
                    ModeType::MODE_TYPE_INTRA
                } else {
                    ModeType::MODE_TYPE_INTER
                }
            }
            _ => self.mode_type,
        };
        let mut tree_type = if mode_type == ModeType::MODE_TYPE_INTRA {
            TreeType::DUAL_TREE_LUMA
        } else {
            self.tree_type
        };
        if self.width == 8 && self.height == 8 {
            assert!(tree_type != TreeType::SINGLE_TREE);
        }
        match split_mode {
            MttSplitMode::SPLIT_QT => {
                self.cts = (0..4)
                    .map(|i| {
                        CodingTree::new_arc_mutex(
                            self.x + (i % 2) * ((1 << log2_width) / 2),
                            self.y + (i / 2) * ((1 << log2_height) / 2),
                            log2_width - 1,
                            log2_height - 1,
                            depth,
                            self.cqt_depth + 1,
                            self.depth_offset,
                            i,
                            self.qg_on_y,
                            self.qg_on_c,
                            Some(parent.clone()),
                            self.ctu.clone(),
                            fixed_qp,
                            MttSplitMode::SPLIT_NONE,
                            tree_type,
                            mode_type,
                            self.tile.clone(),
                        )
                    })
                    .collect();
            }
            // TODO
            MttSplitMode::SPLIT_TT_VER => {}
            MttSplitMode::SPLIT_TT_HOR => {}
            MttSplitMode::SPLIT_BT_VER => {}
            MttSplitMode::SPLIT_BT_HOR => {}
            MttSplitMode::SPLIT_NONE => panic!(),
        }
        if self.mode_type == ModeType::MODE_TYPE_ALL && mode_type == ModeType::MODE_TYPE_INTRA {
            tree_type = TreeType::DUAL_TREE_CHROMA;
            match split_mode {
                MttSplitMode::SPLIT_QT => {
                    // FIXME separate variable for dual trees
                    self.cts.push(CodingTree::new_arc_mutex(
                        self.x,
                        self.y,
                        log2_width,
                        log2_height,
                        self.depth,
                        self.cqt_depth,
                        self.depth_offset,
                        0,
                        self.qg_on_y,
                        self.qg_on_c,
                        Some(parent),
                        self.ctu.clone(),
                        fixed_qp,
                        MttSplitMode::SPLIT_NONE,
                        tree_type,
                        mode_type,
                        self.tile.clone(),
                    ));
                }
                // TODO
                MttSplitMode::SPLIT_TT_VER => {}
                MttSplitMode::SPLIT_TT_HOR => {}
                MttSplitMode::SPLIT_BT_VER => {}
                MttSplitMode::SPLIT_BT_HOR => {}
                MttSplitMode::SPLIT_NONE => panic!(),
            }
        }
    }

    pub fn set_tile(&mut self, tile: Arc<Mutex<Tile>>) {
        self.tile = Some(tile.clone());
        for ct in self.cts.iter() {
            let ct = &mut ct.lock().unwrap();
            ct.set_tile(tile.clone());
        }
        for cu in self.cus.iter() {
            let cu = &mut cu.lock().unwrap();
            cu.set_tile(tile.clone());
        }
        let tile = tile.lock().unwrap();
        self.x_tile = tile.ctu_col << tile.log2_ctu_size;
        self.y_tile = tile.ctu_row << tile.log2_ctu_size;
        self.width_tile = tile.num_ctu_cols << tile.log2_ctu_size;
        self.height_tile = tile.num_ctu_rows << tile.log2_ctu_size;
    }

    pub fn is_below_left_available(&self) -> bool {
        if let Some(available) = *self.below_left_available.borrow() {
            return available;
        }
        {
            if self.y + self.height >= self.y_tile + self.height_tile {
                *self.below_left_available.borrow_mut() = Some(false);
                return false;
            }
        }
        let ct = self.parent.as_ref();
        if let Some(ct) = ct {
            let ct = ct.lock().unwrap();
            if ct.cts.len() > 1 {
                if ct.x < self.x {
                    *self.below_left_available.borrow_mut() = Some(false);
                    false
                } else if self.y + self.height < ct.y + ct.height {
                    let available = self.x_tile < self.x;
                    *self.below_left_available.borrow_mut() = Some(available);
                    available
                } else {
                    let available = ct.is_below_left_available();
                    *self.below_left_available.borrow_mut() = Some(available);
                    available
                }
            } else {
                let available = ct.is_below_left_available();
                *self.below_left_available.borrow_mut() = Some(available);
                available
            }
        } else {
            *self.below_left_available.borrow_mut() = Some(false);
            false
        }
    }

    pub fn is_above_right_available(&self) -> bool {
        if let Some(available) = *self.above_right_available.borrow() {
            return available;
        }
        if self.x + self.width >= self.x_tile + self.width_tile {
            *self.above_right_available.borrow_mut() = Some(false);
            return false;
        }
        let ct = self.parent.as_ref();
        if let Some(ct) = ct {
            let ct = ct.lock().unwrap();
            if self.width == ct.width && self.height == ct.height {
                let available = ct.is_above_right_available();
                *self.above_right_available.borrow_mut() = Some(available);
                available
            } else if ct.cts.len() > 1 {
                match ct.split_mode {
                    MttSplitMode::SPLIT_BT_VER | MttSplitMode::SPLIT_TT_VER => {
                        if self.x + self.width < ct.x + ct.width {
                            let available = self.y_tile < self.y;
                            *self.above_right_available.borrow_mut() = Some(available);
                            available
                        } else {
                            let available = ct.is_above_right_available();
                            *self.above_right_available.borrow_mut() = Some(available);
                            available
                        }
                    }
                    MttSplitMode::SPLIT_BT_HOR | MttSplitMode::SPLIT_TT_HOR => {
                        if self.y == ct.y {
                            let available = ct.is_above_right_available();
                            *self.above_right_available.borrow_mut() = Some(available);
                            available
                        } else {
                            *self.above_right_available.borrow_mut() = Some(false);
                            false
                        }
                    }
                    MttSplitMode::SPLIT_QT => {
                        if self.x == ct.x && self.y == ct.y {
                            let available = self.y_tile < self.y;
                            *self.above_right_available.borrow_mut() = Some(available);
                            available
                        } else if self.y == ct.y {
                            let available = ct.is_above_right_available();
                            *self.above_right_available.borrow_mut() = Some(available);
                            available
                        } else if self.x == ct.x {
                            *self.above_right_available.borrow_mut() = Some(true);
                            true
                        } else {
                            *self.above_right_available.borrow_mut() = Some(false);
                            false
                        }
                    }
                    _ => panic!(),
                }
            } else {
                let available = ct.is_above_right_available();
                *self.above_right_available.borrow_mut() = Some(available);
                available
            }
        } else {
            let available =
                self.y_tile < self.y && self.x + self.width < self.x_tile + self.width_tile;
            *self.above_right_available.borrow_mut() = Some(available);
            available
        }
    }

    #[inline(always)]
    pub fn get_cb_subdiv(&self) -> usize {
        let ctu = self.ctu.as_ref();
        let ctu = ctu.unwrap();
        let ctu = ctu.lock().unwrap();
        ((ctu.width.ilog2() - self.width.ilog2()) + (ctu.height.ilog2() - self.height.ilog2()))
            as usize
    }

    #[inline(always)]
    pub fn get_split_cu_flag(&self) -> bool {
        self.split_mode != MttSplitMode::SPLIT_NONE
    }

    #[inline(always)]
    pub fn get_split_qt_flag(&self) -> bool {
        self.split_mode == MttSplitMode::SPLIT_QT
    }

    pub fn left_ct(&self) -> Option<Arc<Mutex<CodingTree>>> {
        let cu = {
            let ctu = self.ctu.as_ref().unwrap();
            let tile = {
                let ctu = ctu.lock().unwrap();
                let tile = ctu.tile.as_ref().unwrap();
                tile.clone()
            };
            let tile = tile.lock().unwrap();
            tile.get_cu(self.x as isize - 1, self.y as isize)
        };
        if let Some(cu) = cu {
            let cu = cu.lock().unwrap();
            Some(cu.parent.clone())
        } else {
            None
        }
    }

    pub fn above_ct(&self) -> Option<Arc<Mutex<CodingTree>>> {
        let cu = {
            let ctu = self.ctu.as_ref().unwrap();
            let tile = {
                let ctu = ctu.lock().unwrap();
                let tile = ctu.tile.as_ref().unwrap();
                tile.clone()
            };
            let tile = tile.lock().unwrap();
            tile.get_cu(self.x as isize, self.y as isize - 1)
        };
        if let Some(cu) = cu {
            let cu = cu.lock().unwrap();
            Some(cu.parent.clone())
        } else {
            None
        }
    }

    #[inline(always)]
    pub fn allow_split_qt(&self, ectx: &EncoderContext) -> bool {
        ectx.derive_allow_split_qt(
            self.width,
            self.depth - self.cqt_depth,
            self.tree_type,
            self.mode_type,
        )
    }

    #[inline(always)]
    pub fn allow_split_bt(
        &self,
        split_mode: MttSplitMode,
        pps: &PictureParameterSet,
        ectx: &EncoderContext,
    ) -> bool {
        debug_assert!(
            split_mode == MttSplitMode::SPLIT_BT_VER || split_mode == MttSplitMode::SPLIT_BT_HOR
        );
        let (min_qt_size, max_bt_size, max_mtt_depth) =
            if self.tree_type == TreeType::DUAL_TREE_CHROMA {
                (
                    ectx.min_qt_size_c,
                    ectx.max_bt_size_c,
                    ectx.max_mtt_depth_c + self.depth_offset,
                )
            } else {
                (
                    ectx.min_qt_size_y,
                    ectx.max_bt_size_y,
                    ectx.max_mtt_depth_y + self.depth_offset,
                )
            };
        ectx.derive_allow_split_bt(
            split_mode,
            self.width,
            self.height,
            self.x,
            self.y,
            self.depth - self.cqt_depth,
            max_mtt_depth,
            max_bt_size,
            min_qt_size,
            self.part_idx,
            self.tree_type,
            self.mode_type,
            pps,
        )
    }

    #[inline(always)]
    pub fn allow_split_tt(
        &self,
        split_mode: MttSplitMode,
        pps: &PictureParameterSet,
        ectx: &EncoderContext,
    ) -> bool {
        debug_assert!(
            split_mode == MttSplitMode::SPLIT_TT_VER || split_mode == MttSplitMode::SPLIT_TT_HOR
        );
        let (min_qt_size, max_tt_size, max_mtt_depth) =
            if self.tree_type == TreeType::DUAL_TREE_CHROMA {
                (
                    ectx.min_qt_size_c,
                    ectx.max_tt_size_c,
                    ectx.max_mtt_depth_c + self.depth_offset,
                )
            } else {
                (
                    ectx.min_qt_size_y,
                    ectx.max_tt_size_y,
                    ectx.max_mtt_depth_y + self.depth_offset,
                )
            };
        ectx.derive_allow_split_bt(
            split_mode,
            self.width,
            self.height,
            self.x,
            self.y,
            self.depth - self.cqt_depth,
            max_mtt_depth,
            max_tt_size,
            min_qt_size,
            self.part_idx,
            self.tree_type,
            self.mode_type,
            pps,
        )
    }

    #[inline(always)]
    pub fn ch_type(&self) -> usize {
        (self.tree_type == TreeType::DUAL_TREE_CHROMA) as usize
    }

    #[inline(always)]
    pub fn mtt_split_cu_vertical_flag(&self) -> bool {
        self.split_mode == MttSplitMode::SPLIT_BT_VER
            || self.split_mode == MttSplitMode::SPLIT_TT_VER
    }

    #[inline(always)]
    pub fn mtt_split_cu_binary_flag(&self) -> bool {
        self.split_mode == MttSplitMode::SPLIT_BT_VER
            || self.split_mode == MttSplitMode::SPLIT_BT_HOR
    }

    #[inline(always)]
    pub fn _prev_mode_type(&self) -> ModeType {
        // FIXME
        ModeType::MODE_TYPE_ALL
    }

    #[inline(always)]
    pub fn _prev_tree_type(&self, c_idx: usize) -> TreeType {
        // FIXME
        if c_idx == 0 {
            TreeType::DUAL_TREE_LUMA
        } else {
            TreeType::DUAL_TREE_CHROMA
        }
    }

    pub fn get_cu(&self, x: usize, y: usize) -> Option<Arc<Mutex<CodingUnit>>> {
        if x < self.x || x >= self.x + self.width || y < self.y || y >= self.y + self.height {
            None
        } else if !self.cts.is_empty() {
            for ct in self.cts.iter() {
                let ct = ct.lock().unwrap();
                if x >= ct.x && x < ct.x + ct.width && y >= ct.y && y < ct.y + ct.height {
                    return ct.get_cu(x, y);
                }
            }
            panic!();
        } else {
            for cu in self.cus.iter() {
                let cu = cu.clone();
                let (cu_x, cu_y, cu_width, cu_height) = {
                    let cu = cu.lock().unwrap();
                    (cu.x, cu.y, cu.width, cu.height)
                };
                if x >= cu_x && x < cu_x + cu_width && y >= cu_y && y < cu_y + cu_height {
                    return Some(cu);
                }
            }
            panic!();
        }
    }
}

#[allow(clippy::upper_case_acronyms)]
pub struct DTIQS {
    pub x: usize,
    pub y: usize,
    pub width: usize,
    pub height: usize,
    pub dtiqss: Vec<Arc<Mutex<DTIQS>>>,
    pub cts: Vec<Arc<Mutex<CodingTree>>>,
    pub parent: Option<Arc<Mutex<DTIQS>>>,
    pub tile: Option<Arc<Mutex<Tile>>>,
}

impl DTIQS {
    pub fn _new_arc_mutex(
        x: usize,
        y: usize,
        width: usize,
        height: usize,
        parent: Option<Arc<Mutex<DTIQS>>>,
        tile: Option<Arc<Mutex<Tile>>>,
    ) -> Arc<Mutex<DTIQS>> {
        Arc::new(Mutex::new(DTIQS {
            x,
            y,
            width,
            height,
            dtiqss: vec![],
            cts: vec![],
            parent,
            tile,
        }))
    }

    pub fn set_tile(&mut self, tile: Arc<Mutex<Tile>>) {
        self.tile = Some(tile.clone());
        for dtiqs in self.dtiqss.iter() {
            let dtiqs = &mut dtiqs.lock().unwrap();
            dtiqs.set_tile(tile.clone());
        }
        for ct in self.cts.iter() {
            let ct = &mut ct.lock().unwrap();
            ct.set_tile(tile.clone());
        }
    }

    pub fn get_cu(&self, x: usize, y: usize) -> Option<Arc<Mutex<CodingUnit>>> {
        for ct in self.cts.iter() {
            let ct = ct.lock().unwrap();
            if x >= ct.x && x < ct.x + ct.width && y >= ct.y && y < ct.y + ct.height {
                return ct.get_cu(x, y);
            }
        }
        panic!();
    }
}

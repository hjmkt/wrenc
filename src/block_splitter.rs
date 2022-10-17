use super::common::*;
use super::ctu::*;
use super::encoder_context::*;
use super::intra_predictor::*;
use super::quantizer::*;
use super::slice_header::*;
use super::transformer::*;
use std::sync::{Arc, Mutex};

pub struct BlockSplitter {
    intra_predictor: IntraPredictor,
    transformer: Transformer,
    quantizer: Quantizer,
    lv_table: [i64; 1024],
    lv_dq_table: [i64; 1024],
    lv_dq_trellis_table: [i64; 1024],
}

impl BlockSplitter {
    pub fn new(ectx: &EncoderContext) -> BlockSplitter {
        let lv_pow = match ectx.extra_params.get("lv_pow_dq") {
            Some(lv_pow) => lv_pow.parse::<f64>().unwrap(),
            _ => 0.5,
        };
        let lv_pow_dq = match ectx.extra_params.get("lv_pow_dq") {
            Some(lv_pow) => lv_pow.parse::<f64>().unwrap(),
            _ => 0.5850246891437862,
        };
        let lv_pow_dq_trellis = match ectx.extra_params.get("lv_pow_dq_trellis") {
            Some(lv_pow) => lv_pow.parse::<f64>().unwrap(),
            _ => 0.48592678233563835,
        };
        let lv_offset = match ectx.extra_params.get("lv_offset") {
            Some(lv_offset) => lv_offset.parse::<f64>().unwrap(),
            _ => 0.671_961_67,
        };
        let lv_offset_dq = match ectx.extra_params.get("lv_offset_dq") {
            Some(lv_offset) => lv_offset.parse::<f64>().unwrap(),
            _ => 0.13731084642527322,
        };
        let lv_offset_dq_trellis = match ectx.extra_params.get("lv_offset_dq_trellis") {
            Some(lv_offset) => lv_offset.parse::<f64>().unwrap(),
            _ => 0.15150746310196822,
        };
        let mut lv_table = [0i64; 1024];
        let mut lv_dq_table = [0i64; 1024];
        let mut lv_dq_trellis_table = [0i64; 1024];
        for i in 0..1024 {
            lv_table[i] = ((i as f64 + lv_offset).powf(lv_pow) * 16384.0) as i64;
            lv_dq_table[i] = ((i as f64 + lv_offset_dq).powf(lv_pow_dq) * 16384.0) as i64;
            lv_dq_trellis_table[i] =
                ((i as f64 + lv_offset_dq_trellis).powf(lv_pow_dq_trellis) * 16384.0) as i64;
        }
        BlockSplitter {
            intra_predictor: IntraPredictor::new(),
            transformer: Transformer::new(),
            quantizer: Quantizer::new(ectx),
            lv_table,
            lv_dq_table,
            lv_dq_trellis_table,
        }
    }

    pub fn get_intra_pred_aux_cost(
        &mut self,
        intra_pred_mode: [IntraPredMode; 3],
        ct: &mut Arc<Mutex<CodingTree>>,
        sh: &SliceHeader,
        ectx: &mut EncoderContext,
    ) -> f32 {
        let tu = {
            let cu = {
                let ct = ct.lock().unwrap();
                ct.cus[0].clone()
            };
            let cu = &mut cu.lock().unwrap();
            cu.set_intra_pred_mode(intra_pred_mode);
            let tt = cu.transform_tree.as_ref().unwrap();
            let tt = tt.lock().unwrap();
            // FIXME multiple transform units
            tt.tus[0].clone()
        };
        let mut tu = tu.borrow_mut();
        let mut sad: usize = 0;
        for c_idx in 0..3 {
            if tu.is_component_active(c_idx) {
                self.intra_predictor
                    .predict(&mut tu, c_idx, sh.sps, sh.pps, ectx);
                let tile = tu.get_tile();
                let tile = &mut tile.lock().unwrap();
                let (tx, ty) = tu.get_component_pos(c_idx);
                let (tw, th) = tu.get_component_size(c_idx);
                // FIXME SIMD?
                let pred_pixels = &tile.pred_pixels.borrow()[c_idx];
                let original_pixels = &tile.original_pixels.borrow()[c_idx];
                for y in ty..ty + th {
                    let pred_pixels = &pred_pixels[y][tx..];
                    let original_pixels = &original_pixels[y][tx..];
                    for x in 0..tw {
                        let pred = pred_pixels[x];
                        let d = pred as i32 - original_pixels[x] as i32;
                        sad += d.unsigned_abs() as usize;
                    }
                }
            }
        }
        sad as f32
    }

    pub fn get_intra_pred_cost(
        &mut self,
        intra_pred_mode: [IntraPredMode; 3],
        ct: &mut Arc<Mutex<CodingTree>>,
        trellis: bool,
        sh: &SliceHeader,
        ectx: &mut EncoderContext,
    ) -> f32 {
        let (
            tu,
            non_planar_flag,
            (mpm_flag, mpm_idx, mpm_remainder),
            cclm_mode_flag,
            cclm_mode_idx,
            tree_type,
        ) = {
            let (cu, tree_type) = {
                let ct = ct.lock().unwrap();
                (ct.cus[0].clone(), ct.tree_type)
            };
            let cu = &mut cu.lock().unwrap();
            cu.set_intra_pred_mode(intra_pred_mode);
            let tt = cu.transform_tree.as_ref().unwrap();
            let tt = tt.lock().unwrap();
            // FIXME multiple transform units
            (
                tt.tus[0].clone(),
                cu.get_intra_luma_not_planar_flag(),
                cu.get_intra_luma_mpm_flag_and_idx_and_remainder(),
                cu.get_cclm_mode_flag(),
                cu.get_cclm_mode_idx(),
                tree_type,
            )
        };
        let mut tu = tu.borrow_mut();
        let mut ssd: usize = 0;
        for c_idx in 0..3 {
            if tu.is_component_active(c_idx) {
                self.intra_predictor
                    .predict(&mut tu, c_idx, sh.sps, sh.pps, ectx);
                self.transformer
                    .transform(&mut tu, c_idx, sh.sps, sh.ph.as_ref().unwrap(), ectx);
                self.quantizer.quantize(&mut tu, c_idx, trellis, sh, ectx);
                self.quantizer.dequantize(&mut tu, c_idx, sh, ectx);
                self.transformer.inverse_transform(
                    &mut tu,
                    c_idx,
                    sh.sps,
                    sh.ph.as_ref().unwrap(),
                    ectx,
                );
                let tile = tu.get_tile();
                let tile = &mut tile.lock().unwrap();
                let (tx, ty) = tu.get_component_pos(c_idx);
                let (tw, th) = tu.get_component_size(c_idx);
                // FIXME SIMD?
                let pred_pixels = &tile.pred_pixels.borrow()[c_idx];
                let reconst_pixels = &mut tile.reconst_pixels.borrow_mut()[c_idx];
                let original_pixels = &tile.original_pixels.borrow()[c_idx];
                let it = &tu.itransformed_coeffs[c_idx];
                for y in ty..ty + th {
                    let pred_pixels = &pred_pixels[y][tx..];
                    let reconst_pixels = &mut reconst_pixels[y][tx..];
                    let original_pixels = &original_pixels[y][tx..];
                    let it = &it[y - ty];
                    for x in 0..tw {
                        let pred = pred_pixels[x];
                        let res = it[x];
                        let rec = (pred as i16 + res).clamp(0, 255) as u8;
                        reconst_pixels[x] = rec;
                        let d = rec as i32 - original_pixels[x] as i32;
                        ssd += (d * d) as usize;
                    }
                }
            }
        }

        let non_planar_offset = if !sh.dep_quant_used_flag {
            match ectx.extra_params.get("non_planar_offset") {
                Some(non_planar_offset) => non_planar_offset.parse::<f32>().unwrap(),
                _ => 2.495_123_1,
            }
        } else if trellis {
            match ectx.extra_params.get("non_planar_offset_dq_trellis") {
                Some(non_planar_offset) => non_planar_offset.parse::<f32>().unwrap(),
                _ => 2.215_359_7,
            }
        } else {
            match ectx.extra_params.get("non_planar_offset_dq") {
                Some(non_planar_offset) => non_planar_offset.parse::<f32>().unwrap(),
                _ => 2.600_296_5,
            }
        };
        let mpm_idx_offset = if !sh.dep_quant_used_flag {
            match ectx.extra_params.get("mpm_idx_offset") {
                Some(mpm_idx_offset) => mpm_idx_offset.parse::<f32>().unwrap(),
                _ => 1.321_590_3,
            }
        } else if trellis {
            match ectx.extra_params.get("mpm_idx_offset_dq_trellis") {
                Some(mpm_idx_offset) => mpm_idx_offset.parse::<f32>().unwrap(),
                _ => 1.366_022_1,
            }
        } else {
            match ectx.extra_params.get("mpm_idx_offset_dq") {
                Some(mpm_idx_offset) => mpm_idx_offset.parse::<f32>().unwrap(),
                _ => 1.506_942_6,
            }
        };
        let mpm_remainder_mult = if !sh.dep_quant_used_flag {
            match ectx.extra_params.get("mpm_remainder_mult") {
                Some(mpm_remainder_mult) => mpm_remainder_mult.parse::<f32>().unwrap(),
                _ => 0.673_733_23,
            }
        } else if trellis {
            match ectx.extra_params.get("mpm_remainder_mult_dq_trellis") {
                Some(mpm_remainder_mult) => mpm_remainder_mult.parse::<f32>().unwrap(),
                _ => 0.500_718_2,
            }
        } else {
            match ectx.extra_params.get("mpm_remainder_mult_dq") {
                Some(mpm_remainder_mult) => mpm_remainder_mult.parse::<f32>().unwrap(),
                _ => 0.456_410_26,
            }
        };
        let mpm_remainder_offset = if !sh.dep_quant_used_flag {
            match ectx.extra_params.get("mpm_remainder_offset") {
                Some(mpm_remainder_offset) => mpm_remainder_offset.parse::<f32>().unwrap(),
                _ => 2.694_721_2,
            }
        } else if trellis {
            match ectx.extra_params.get("mpm_remainder_offset_dq_trellis") {
                Some(mpm_remainder_offset) => mpm_remainder_offset.parse::<f32>().unwrap(),
                _ => 2.297_330_4,
            }
        } else {
            match ectx.extra_params.get("mpm_remainder_offset_dq") {
                Some(mpm_remainder_offset) => mpm_remainder_offset.parse::<f32>().unwrap(),
                _ => 2.352_948,
            }
        };
        // FIXME
        let planar_offset = if !sh.dep_quant_used_flag {
            match ectx.extra_params.get("planer_offset") {
                Some(planer_offset) => planer_offset.parse::<f32>().unwrap(),
                _ => 0.596_190_8,
            }
        } else if trellis {
            match ectx.extra_params.get("planer_offset_dq_trellis") {
                Some(planer_offset) => planer_offset.parse::<f32>().unwrap(),
                _ => 0.962_686_4,
            }
        } else {
            match ectx.extra_params.get("planer_offset_dq") {
                Some(planer_offset) => planer_offset.parse::<f32>().unwrap(),
                _ => 0.962_686_4,
            }
        };
        let header_bits = if !sh.dep_quant_used_flag {
            match ectx.extra_params.get("header_bits") {
                Some(planer_offset) => planer_offset.parse::<f32>().unwrap(),
                _ => 1.762_286_1,
            }
        } else if trellis {
            match ectx.extra_params.get("header_bits_dq_trellis") {
                Some(planer_offset) => planer_offset.parse::<f32>().unwrap(),
                _ => 1.177_287_2,
            }
        } else {
            match ectx.extra_params.get("header_bits_dq") {
                Some(planer_offset) => planer_offset.parse::<f32>().unwrap(),
                _ => 0.982_125_64,
            }
        };
        let qp_div = if !sh.dep_quant_used_flag {
            match ectx.extra_params.get("qp_div") {
                Some(qp_div) => qp_div.parse::<f32>().unwrap(),
                _ => 7.0,
            }
        } else if trellis {
            match ectx.extra_params.get("qp_div_dq_trellis") {
                Some(qp_div) => qp_div.parse::<f32>().unwrap(),
                _ => 4.404_366_5,
            }
        } else {
            match ectx.extra_params.get("qp_div_dq") {
                Some(qp_div) => qp_div.parse::<f32>().unwrap(),
                _ => 3.970_736,
            }
        };
        let lambda_mul = if !sh.dep_quant_used_flag {
            match ectx.extra_params.get("lambda_mul") {
                Some(lambda_mul) => lambda_mul.parse::<f32>().unwrap(),
                _ => 7.915_166,
            }
        } else if trellis {
            match ectx.extra_params.get("lambda_mul_dq_trellis") {
                Some(lambda_mul) => lambda_mul.parse::<f32>().unwrap(),
                _ => 1.128_258_1,
            }
        } else {
            match ectx.extra_params.get("lambda_mul_dq") {
                Some(lambda_mul) => lambda_mul.parse::<f32>().unwrap(),
                _ => 1.343_928_7,
            }
        };
        let cclm_pow = match ectx.extra_params.get("cclm_pow") {
            Some(cclm_pow) => cclm_pow.parse::<f32>().unwrap(),
            _ => 0.458_765_1,
        };
        let mpm_idx_pow = match ectx.extra_params.get("mpm_idx_pow") {
            Some(mpm_idx_pow) => mpm_idx_pow.parse::<f32>().unwrap(),
            _ => 0.402_712_85,
        };
        let mpm_remainder_pow = match ectx.extra_params.get("mpm_remainder_pow") {
            Some(mpm_remainder_pow) => mpm_remainder_pow.parse::<f32>().unwrap(),
            _ => 0.343_850_94,
        };
        let cclm_mode_idx_offset = if !sh.dep_quant_used_flag {
            match ectx.extra_params.get("cclm_mode_idx_offset") {
                Some(cclm_mode_idx_offset) => cclm_mode_idx_offset.parse::<f32>().unwrap(),
                _ => 1.944_860_6,
            }
        } else if trellis {
            match ectx.extra_params.get("cclm_mode_idx_offset_dq_trellis") {
                Some(cclm_mode_idx_offset) => cclm_mode_idx_offset.parse::<f32>().unwrap(),
                _ => 2.1,
            }
        } else {
            match ectx.extra_params.get("cclm_mode_idx_offset_dq") {
                Some(cclm_mode_idx_offset) => cclm_mode_idx_offset.parse::<f32>().unwrap(),
                _ => 2.1,
            }
        };
        let non_cclm_offset = if !sh.dep_quant_used_flag {
            match ectx.extra_params.get("non_cclm_offset") {
                Some(non_cclm_offset) => non_cclm_offset.parse::<f32>().unwrap(),
                _ => 0.979_434_97,
            }
        } else if trellis {
            match ectx.extra_params.get("non_cclm_offset_dq_trellis") {
                Some(non_cclm_offset) => non_cclm_offset.parse::<f32>().unwrap(),
                _ => 0.89,
            }
        } else {
            match ectx.extra_params.get("non_cclm_offset_dq") {
                Some(non_cclm_offset) => non_cclm_offset.parse::<f32>().unwrap(),
                _ => 0.89,
            }
        };
        let cclm_offset = if !sh.dep_quant_used_flag {
            match ectx.extra_params.get("cclm_offset") {
                Some(cclm_offset) => cclm_offset.parse::<f32>().unwrap(),
                _ => 0.1,
            }
        } else if trellis {
            match ectx.extra_params.get("cclm_offset_dq_trellis") {
                Some(cclm_offset) => cclm_offset.parse::<f32>().unwrap(),
                _ => 0.53,
            }
        } else {
            match ectx.extra_params.get("cclm_offset_dq") {
                Some(cclm_offset) => cclm_offset.parse::<f32>().unwrap(),
                _ => 0.53,
            }
        };

        let cclm_bits = if sh.sps.cclm_enabled_flag {
            if cclm_mode_flag {
                cclm_offset + (cclm_mode_idx as f32 + cclm_mode_idx_offset).powf(cclm_pow)
            } else if tree_type == TreeType::DUAL_TREE_LUMA {
                0.0
            } else {
                non_cclm_offset
            }
        } else {
            0.0
        };
        let mode_bits = if non_planar_flag {
            non_planar_offset
                + if mpm_flag {
                    (mpm_idx as f32 + mpm_idx_offset).powf(mpm_idx_pow)
                } else {
                    mpm_remainder_mult
                        * (mpm_remainder as f32 + mpm_remainder_offset).powf(mpm_remainder_pow)
                }
        } else {
            planar_offset
        } + cclm_bits;
        // FIXME estimate additional header bits for coding units
        let header_bits = ({
            match tree_type {
                TreeType::SINGLE_TREE => header_bits + mode_bits,
                TreeType::DUAL_TREE_LUMA => header_bits / 3.0 + mode_bits,
                TreeType::DUAL_TREE_CHROMA => cclm_bits,
            }
        } * 16384.0) as i64;
        let lv_table = if !sh.dep_quant_used_flag {
            &self.lv_table
        } else if trellis {
            &self.lv_dq_trellis_table
        } else {
            &self.lv_dq_table
        };
        let q_state_trans_table = &ectx.q_state_trans_table;
        let level: i64 = if sh.dep_quant_used_flag {
            let mut sum = 0;
            for c_idx in 0..3 {
                if !tu.is_component_active(c_idx) {
                    continue;
                }
                let mut q_state = 0;
                let (log2_tb_width, log2_tb_height) = tu.get_log2_tb_size(c_idx);
                let (log2_sb_w, log2_sb_h) = tu.get_log2_sb_size(c_idx);
                let num_sb_coeff = 1 << (log2_sb_w + log2_sb_h);
                let mut last_scan_pos = num_sb_coeff;
                let mut last_sub_block =
                    (1 << (log2_tb_width + log2_tb_height - (log2_sb_w + log2_sb_h))) - 1;
                let coeff_order = &DIAG_SCAN_ORDER[log2_sb_h][log2_sb_w];
                let sb_order =
                    &DIAG_SCAN_ORDER[log2_tb_height - log2_sb_h][log2_tb_width - log2_sb_w];
                let q = &tu.quantized_transformed_coeffs[c_idx];
                let (mut x_s, mut y_s) = sb_order[last_sub_block];
                let (mut x_0, mut y_0) = (x_s << log2_sb_w, y_s << log2_sb_h);
                let mut is_not_first_sub_block = last_sub_block > 0;
                let mut is_trailing_zeros = true;
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
                    let qc = q[y_c][x_c].unsigned_abs() as usize;
                    if qc == 0 {
                        sum += if is_trailing_zeros { 0 } else { lv_table[0] };
                        q_state = q_state_trans_table[q_state][0];
                    } else {
                        let a = (qc + (q_state > 1) as usize) / 2;
                        sum += lv_table[a];
                        q_state = q_state_trans_table[q_state][a & 1];
                    }
                    is_trailing_zeros &= qc == 0;
                    last_scan_pos > 0 || is_not_first_sub_block
                } {}
            }
            sum
        } else {
            tu.quantized_transformed_coeffs
                .iter()
                .flat_map(|a| {
                    a.data.iter().map(|v| {
                        let v = v.unsigned_abs() as usize;
                        lv_table[v]
                    })
                })
                .sum::<i64>()
        } + header_bits;
        let lambda = (2.0f32).powf(tu.qp as f32 / qp_div) * lambda_mul;
        ssd as f32 + lambda * (level as f32 / 16384.0)
    }

    pub fn get_chroma_intra_pred_aux_cost(
        &mut self,
        intra_pred_mode: IntraPredMode,
        ct: &mut Arc<Mutex<CodingTree>>,
        sh: &SliceHeader,
        ectx: &mut EncoderContext,
    ) -> f32 {
        let tu = {
            let cu = {
                let ct = ct.lock().unwrap();
                ct.cus[0].clone()
            };
            let cu = &mut cu.lock().unwrap();
            let mut mode = cu.intra_pred_mode;
            mode[1] = intra_pred_mode;
            mode[2] = intra_pred_mode;
            cu.set_intra_pred_mode(mode);
            let tt = cu.transform_tree.as_ref().unwrap();
            let tt = tt.lock().unwrap();
            // FIXME multiple transform units
            tt.tus[0].clone()
        };
        let mut tu = tu.borrow_mut();
        let mut sad: usize = 0;
        for c_idx in 1..3 {
            if tu.is_component_active(c_idx) {
                self.intra_predictor
                    .predict(&mut tu, c_idx, sh.sps, sh.pps, ectx);
                let tile = tu.get_tile();
                let tile = &mut tile.lock().unwrap();
                let (tx, ty) = tu.get_component_pos(c_idx);
                let (tw, th) = tu.get_component_size(c_idx);
                let pred_pixels = &tile.pred_pixels.borrow()[c_idx];
                let original_pixels = &tile.original_pixels.borrow()[c_idx];
                for y in ty..ty + th {
                    let pred_pixels = &pred_pixels[y][tx..];
                    let original_pixels = &original_pixels[y][tx..];
                    for x in 0..tw {
                        let pred = pred_pixels[x];
                        let d = pred as i32 - original_pixels[x] as i32;
                        sad += d.unsigned_abs() as usize;
                    }
                }
            }
        }
        sad as f32
    }

    pub fn get_chroma_intra_pred_cost(
        &mut self,
        intra_pred_mode: IntraPredMode,
        ct: &mut Arc<Mutex<CodingTree>>,
        trellis: bool,
        sh: &SliceHeader,
        ectx: &mut EncoderContext,
    ) -> f32 {
        let (tu, cclm_mode_flag, cclm_mode_idx) = {
            let cu = {
                let ct = ct.lock().unwrap();
                ct.cus[0].clone()
            };
            let cu = &mut cu.lock().unwrap();
            let mut mode = cu.intra_pred_mode;
            mode[1] = intra_pred_mode;
            mode[2] = intra_pred_mode;
            cu.set_intra_pred_mode(mode);
            let tt = cu.transform_tree.as_ref().unwrap();
            let tt = tt.lock().unwrap();
            // FIXME multiple transform units
            (
                tt.tus[0].clone(),
                cu.get_cclm_mode_flag(),
                cu.get_cclm_mode_idx(),
            )
        };
        let mut tu = tu.borrow_mut();
        let mut ssd: usize = 0;
        for c_idx in 1..3 {
            if tu.is_component_active(c_idx) {
                self.intra_predictor
                    .predict(&mut tu, c_idx, sh.sps, sh.pps, ectx);
                self.transformer
                    .transform(&mut tu, c_idx, sh.sps, sh.ph.as_ref().unwrap(), ectx);
                self.quantizer.quantize(&mut tu, c_idx, trellis, sh, ectx);
                self.quantizer.dequantize(&mut tu, c_idx, sh, ectx);
                self.transformer.inverse_transform(
                    &mut tu,
                    c_idx,
                    sh.sps,
                    sh.ph.as_ref().unwrap(),
                    ectx,
                );
                let tile = tu.get_tile();
                let tile = &mut tile.lock().unwrap();
                let (tx, ty) = tu.get_component_pos(c_idx);
                let (tw, th) = tu.get_component_size(c_idx);
                // FIXME SIMD?
                let pred_pixels = &tile.pred_pixels.borrow()[c_idx];
                let reconst_pixels = &mut tile.reconst_pixels.borrow_mut()[c_idx];
                let original_pixels = &tile.original_pixels.borrow()[c_idx];
                let it = &tu.itransformed_coeffs[c_idx];
                for y in ty..ty + th {
                    let pred_pixels = &pred_pixels[y][tx..];
                    let reconst_pixels = &mut reconst_pixels[y][tx..];
                    let original_pixels = &original_pixels[y][tx..];
                    let it = &it[y - ty];
                    for x in 0..tw {
                        let pred = pred_pixels[x];
                        let res = it[x];
                        let rec = (pred as i16 + res).clamp(0, 255) as u8;
                        reconst_pixels[x] = rec;
                        let d = rec as i32 - original_pixels[x] as i32;
                        ssd += (d * d) as usize;
                    }
                }
            }
        }

        let cclm_offset = if !sh.dep_quant_used_flag {
            match ectx.extra_params.get("cclm_offset") {
                Some(cclm_offset) => cclm_offset.parse::<f32>().unwrap(),
                _ => 0.1,
            }
        } else if trellis {
            match ectx.extra_params.get("cclm_offset_dq_trellis") {
                Some(cclm_offset) => cclm_offset.parse::<f32>().unwrap(),
                _ => 0.53,
            }
        } else {
            match ectx.extra_params.get("cclm_offset_dq") {
                Some(cclm_offset) => cclm_offset.parse::<f32>().unwrap(),
                _ => 0.53,
            }
        };
        let non_cclm_offset = if !sh.dep_quant_used_flag {
            match ectx.extra_params.get("non_cclm_offset") {
                Some(non_cclm_offset) => non_cclm_offset.parse::<f32>().unwrap(),
                _ => 0.979_434_97,
            }
        } else if trellis {
            match ectx.extra_params.get("non_cclm_offset_dq_trellis") {
                Some(non_cclm_offset) => non_cclm_offset.parse::<f32>().unwrap(),
                _ => 0.89,
            }
        } else {
            match ectx.extra_params.get("non_cclm_offset_dq") {
                Some(non_cclm_offset) => non_cclm_offset.parse::<f32>().unwrap(),
                _ => 0.89,
            }
        };
        let chroma_header_bits = if !sh.dep_quant_used_flag {
            match ectx.extra_params.get("chroma_header_bits") {
                Some(chroma_header_bits) => chroma_header_bits.parse::<f32>().unwrap(),
                _ => 1.180_406_8,
            }
        } else if trellis {
            match ectx.extra_params.get("chroma_header_bits_dq_trellis") {
                Some(chroma_header_bits) => chroma_header_bits.parse::<f32>().unwrap(),
                _ => 1.309_252,
            }
        } else {
            match ectx.extra_params.get("chroma_header_bits_dq") {
                Some(chroma_header_bits) => chroma_header_bits.parse::<f32>().unwrap(),
                _ => 1.122_390_6,
            }
        };
        let qp_div = if !sh.dep_quant_used_flag {
            match ectx.extra_params.get("qp_div") {
                Some(qp_div) => qp_div.parse::<f32>().unwrap(),
                _ => 7.0,
            }
        } else if trellis {
            match ectx.extra_params.get("qp_div_dq_trellis") {
                Some(qp_div) => qp_div.parse::<f32>().unwrap(),
                _ => 4.404_366_5,
            }
        } else {
            match ectx.extra_params.get("qp_div_dq") {
                Some(qp_div) => qp_div.parse::<f32>().unwrap(),
                _ => 3.970_736,
            }
        };
        let lambda_mul = if !sh.dep_quant_used_flag {
            match ectx.extra_params.get("lambda_mul") {
                Some(lambda_mul) => lambda_mul.parse::<f32>().unwrap(),
                _ => 7.915_166,
            }
        } else if trellis {
            match ectx.extra_params.get("lambda_mul_dq_trellis") {
                Some(lambda_mul) => lambda_mul.parse::<f32>().unwrap(),
                _ => 1.128_258_1,
            }
        } else {
            match ectx.extra_params.get("lambda_mul_dq") {
                Some(lambda_mul) => lambda_mul.parse::<f32>().unwrap(),
                _ => 1.343_928_7,
            }
        };
        let cclm_pow = match ectx.extra_params.get("cclm_pow") {
            Some(cclm_pow) => cclm_pow.parse::<f32>().unwrap(),
            _ => 0.458_765_1,
        };
        let cclm_mode_idx_offset = if !sh.dep_quant_used_flag {
            match ectx.extra_params.get("cclm_mode_idx_offset") {
                Some(cclm_mode_idx_offset) => cclm_mode_idx_offset.parse::<f32>().unwrap(),
                _ => 1.944_860_6,
            }
        } else if trellis {
            match ectx.extra_params.get("cclm_mode_idx_offset_dq_trellis") {
                Some(cclm_mode_idx_offset) => cclm_mode_idx_offset.parse::<f32>().unwrap(),
                _ => 2.1,
            }
        } else {
            match ectx.extra_params.get("cclm_mode_idx_offset_dq") {
                Some(cclm_mode_idx_offset) => cclm_mode_idx_offset.parse::<f32>().unwrap(),
                _ => 2.1,
            }
        };

        let mode_bits = if sh.sps.cclm_enabled_flag {
            if cclm_mode_flag {
                cclm_offset + (cclm_mode_idx as f32 + cclm_mode_idx_offset).powf(cclm_pow)
            } else {
                non_cclm_offset
            }
        } else {
            0.0
        };
        // FIXME estimate additional header bits for coding units
        let header_bits = ({
            let ct = ct.lock().unwrap();
            match ct.tree_type {
                TreeType::SINGLE_TREE => chroma_header_bits + mode_bits,
                TreeType::DUAL_TREE_LUMA => panic!(),
                TreeType::DUAL_TREE_CHROMA => chroma_header_bits + mode_bits,
            }
        } * 16384.0) as i64;
        let lv_table = if !sh.dep_quant_used_flag {
            &self.lv_table
        } else if trellis {
            &self.lv_dq_trellis_table
        } else {
            &self.lv_dq_table
        };
        let q_state_trans_table = &ectx.q_state_trans_table;
        let (log2_tb_width, log2_tb_height) = tu.get_log2_tb_size(1);
        let (log2_sb_w, log2_sb_h) = tu.get_log2_sb_size(1);
        let num_sb_coeff = 1 << (log2_sb_w + log2_sb_h);
        let coeff_order = &DIAG_SCAN_ORDER[log2_sb_h][log2_sb_w];
        let sb_order = &DIAG_SCAN_ORDER[log2_tb_height - log2_sb_h][log2_tb_width - log2_sb_w];

        let level: i64 = if sh.dep_quant_used_flag {
            let mut sum = 0;
            for c_idx in 1..3 {
                let mut is_trailing_zeros = true;
                let mut q_state = 0;
                let mut last_scan_pos = num_sb_coeff;
                let q = &tu.quantized_transformed_coeffs[c_idx];
                let mut last_sub_block =
                    (1 << (log2_tb_width + log2_tb_height - (log2_sb_w + log2_sb_h))) - 1;
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
                    let qc = q[y_c][x_c].unsigned_abs() as usize;
                    if qc == 0 {
                        sum += if is_trailing_zeros { 0 } else { lv_table[0] };
                        q_state = q_state_trans_table[q_state][0];
                    } else {
                        let a = (qc + (q_state > 1) as usize) / 2;
                        sum += lv_table[a];
                        q_state = q_state_trans_table[q_state][a & 1];
                    }
                    is_trailing_zeros &= qc == 0;
                    last_scan_pos > 0 || is_not_first_sub_block
                } {}
            }
            sum
        } else {
            tu.quantized_transformed_coeffs[1..]
                .iter()
                .flat_map(|a| {
                    a.data.iter().map(|v| {
                        let v = v.unsigned_abs() as usize;
                        lv_table[v]
                    })
                })
                .sum::<i64>()
        } + header_bits;
        let lambda = match ectx.extra_params.get("a") {
            Some(alpha) => (2.0f32).powf(tu.qp as f32 / qp_div) * alpha.parse::<f32>().unwrap(),
            _ => (2.0f32).powf(tu.qp as f32 / qp_div) * lambda_mul,
        };
        ssd as f32 + lambda * (level as f32 / 16384.0)
    }

    pub fn split_ct(
        &mut self,
        ct: &mut Arc<Mutex<CodingTree>>,
        max_depth: usize,
        sh: &SliceHeader,
        ectx: &mut EncoderContext,
    ) -> f32 {
        if max_depth == 0 {
            let tree_type = {
                let ct = ct.lock().unwrap();
                ct.tree_type
            };
            if tree_type == TreeType::DUAL_TREE_CHROMA {
                let luma_cu = {
                    let ct = ct.lock().unwrap();
                    let ct = ct.parent.as_ref().unwrap();
                    let ct = ct.lock().unwrap();
                    ct.get_cu(ct.x + ct.width / 2, ct.y + ct.height / 2)
                };
                let (chroma_pred_mode, _) = {
                    let luma_cu = luma_cu.as_ref().unwrap();
                    let luma_cu = luma_cu.lock().unwrap();
                    luma_cu.get_intra_chroma_pred_mode_and_mip_chroma_direct_mode_flag()
                };
                if sh.sps.cclm_enabled_flag {
                    let cache_reconsts = |ct: &Arc<Mutex<CodingTree>>| -> Vec<Vec2d<u8>> {
                        let ct = ct.lock().unwrap();
                        let tile = ct.tile.as_ref().unwrap();
                        let tile = tile.lock().unwrap();
                        let mut reconsts = vec![];
                        for c_idx in 1..3 {
                            let (cx, cy) = ct.get_component_pos(c_idx);
                            let (cw, ch) = ct.get_component_size(c_idx);
                            let mut reconst = vec2d![0; ch; cw];
                            let tile_reconst = &tile.reconst_pixels.borrow()[c_idx];
                            for y in cy..cy + ch {
                                for x in cx..cx + cw {
                                    reconst[y - cy][x - cx] = tile_reconst[y][x];
                                }
                            }
                            reconsts.push(reconst);
                        }
                        reconsts
                    };
                    let restore_reconsts = |ct: &Arc<Mutex<CodingTree>>, cache: &Vec<Vec2d<u8>>| {
                        let ct = ct.lock().unwrap();
                        let tile = ct.tile.as_ref().unwrap();
                        let tile = &mut tile.lock().unwrap();
                        for c_idx in 1..3 {
                            let (cx, cy) = ct.get_component_pos(c_idx);
                            let (cw, ch) = ct.get_component_size(c_idx);
                            let tile_reconst = &mut tile.reconst_pixels.borrow_mut()[c_idx];
                            for y in cy..cy + ch {
                                for x in cx..cx + cw {
                                    tile_reconst[y][x] = cache[c_idx - 1][y - cy][x - cx];
                                }
                            }
                        }
                    };
                    let cclm_lt_cost =
                        self.get_chroma_intra_pred_aux_cost(IntraPredMode::LT_CCLM, ct, sh, ectx);
                    let cclm_t_cost =
                        self.get_chroma_intra_pred_aux_cost(IntraPredMode::T_CCLM, ct, sh, ectx);
                    let cclm_l_cost =
                        self.get_chroma_intra_pred_aux_cost(IntraPredMode::L_CCLM, ct, sh, ectx);
                    let (cclm_mode, _cclm_cost) =
                        if cclm_lt_cost <= cclm_t_cost && cclm_lt_cost <= cclm_l_cost {
                            (IntraPredMode::LT_CCLM, cclm_lt_cost)
                        } else if cclm_t_cost <= cclm_l_cost {
                            (IntraPredMode::T_CCLM, cclm_t_cost)
                        } else {
                            (IntraPredMode::L_CCLM, cclm_l_cost)
                        };
                    let cclm_cost = self.get_chroma_intra_pred_cost(cclm_mode, ct, true, sh, ectx);
                    let cclm_reconsts = cache_reconsts(ct);
                    let current_cost =
                        self.get_chroma_intra_pred_cost(chroma_pred_mode, ct, true, sh, ectx);
                    let chroma_cand_costs = [current_cost, cclm_cost];
                    let chroma_min_cost = chroma_cand_costs.iter().fold(f32::MAX, |m, v| v.min(m));
                    let chroma_min_cost_idx = chroma_cand_costs
                        .iter()
                        .position(|x| x == &chroma_min_cost)
                        .unwrap();
                    let cu = {
                        let ct = ct.lock().unwrap();
                        ct.cus[0].clone()
                    };
                    if chroma_min_cost_idx == 1 {
                        let cu = &mut cu.lock().unwrap();
                        cu.set_intra_pred_mode([cclm_mode; 3]);
                        restore_reconsts(ct, &cclm_reconsts);
                    }
                    chroma_min_cost
                } else {
                    let current_cost =
                        self.get_chroma_intra_pred_cost(chroma_pred_mode, ct, true, sh, ectx);
                    let cu = {
                        let ct = ct.lock().unwrap();
                        ct.cus[0].clone()
                    };
                    let cu = &mut cu.lock().unwrap();
                    cu.set_intra_pred_mode([chroma_pred_mode; 3]);
                    current_cost
                }
            } else {
                let cand_modes = [0, 1, 2, 7, 13, 18, 23, 29, 34, 39, 45, 50, 55, 60, 66];
                let cand_costs = cand_modes
                    .iter()
                    .map(|m| {
                        let mode = num::FromPrimitive::from_usize(*m).unwrap();
                        if mode as usize <= 1 {
                            self.get_intra_pred_cost([mode; 3], ct, true, sh, ectx)
                        } else {
                            self.get_intra_pred_aux_cost([mode; 3], ct, sh, ectx)
                        }
                    })
                    .collect::<Vec<f32>>();
                let min_dir_cost = cand_costs[2..].iter().fold(f32::MAX, |m, v| v.min(m));
                let min_dir_cost_idx = cand_costs[2..]
                    .iter()
                    .position(|x| x == &min_dir_cost)
                    .unwrap()
                    + 2;
                let mut step_search = |current_mode: usize,
                                       step: usize,
                                       current_cost: f32,
                                       aux: bool|
                 -> (usize, f32) {
                    let (mut current_mode, mut step, mut current_cost) = if aux {
                        (current_mode, step, current_cost)
                    } else {
                        let current_cost = self.get_intra_pred_cost(
                            [num::FromPrimitive::from_usize(current_mode).unwrap(); 3],
                            ct,
                            true,
                            sh,
                            ectx,
                        );
                        (current_mode, step, current_cost)
                    };
                    while step > 0 {
                        let trellis = true;
                        let cost0 = if current_mode < 2 + step {
                            f32::MAX
                        } else if aux {
                            self.get_intra_pred_aux_cost(
                                [num::FromPrimitive::from_usize(current_mode - step).unwrap(); 3],
                                ct,
                                sh,
                                ectx,
                            )
                        } else {
                            self.get_intra_pred_cost(
                                [num::FromPrimitive::from_usize(current_mode - step).unwrap(); 3],
                                ct,
                                trellis,
                                sh,
                                ectx,
                            )
                        };
                        let cost1 = if current_mode + step > 66 {
                            f32::MAX
                        } else if aux {
                            self.get_intra_pred_aux_cost(
                                [num::FromPrimitive::from_usize(current_mode + step).unwrap(); 3],
                                ct,
                                sh,
                                ectx,
                            )
                        } else {
                            self.get_intra_pred_cost(
                                [num::FromPrimitive::from_usize(current_mode + step).unwrap(); 3],
                                ct,
                                trellis,
                                sh,
                                ectx,
                            )
                        };
                        let min_cost = current_cost.min(cost0).min(cost1);
                        (current_mode, current_cost) = if current_cost == min_cost {
                            (current_mode, current_cost)
                        } else if cost0 == min_cost {
                            (current_mode - step, cost0)
                        } else {
                            (current_mode + step, cost1)
                        };
                        step /= 2;
                    }
                    (current_mode, current_cost)
                };
                let (dir_mode, _dir_cost) =
                    step_search(cand_modes[min_dir_cost_idx], 2, min_dir_cost, true);
                let (dir_mode, dir_cost) = step_search(dir_mode, 1, min_dir_cost, false);
                let cand_modes = [0, 1, dir_mode];
                let cand_costs = [cand_costs[0], cand_costs[1], dir_cost];
                let mut min_cost = cand_costs.iter().fold(f32::MAX, |m, v| v.min(m));
                let min_cost_idx = cand_costs.iter().position(|x| x == &min_cost).unwrap();
                let cu = {
                    let ct = ct.lock().unwrap();
                    ct.cus[0].clone()
                };
                {
                    let cu = &mut cu.lock().unwrap();
                    cu.set_intra_pred_mode(
                        [num::FromPrimitive::from_usize(cand_modes[min_cost_idx]).unwrap(); 3],
                    );
                }
                {
                    let c_idx = 0;
                    let tu = {
                        let cu = &mut cu.lock().unwrap();
                        let tt = cu.transform_tree.as_ref().unwrap();
                        let tt = tt.lock().unwrap();
                        // FIXME multiple transform units
                        tt.tus[0].clone()
                    };
                    let mut tu = tu.borrow_mut();
                    if tu.is_component_active(c_idx) {
                        self.intra_predictor
                            .predict(&mut tu, c_idx, sh.sps, sh.pps, ectx);
                        self.transformer.transform(
                            &mut tu,
                            c_idx,
                            sh.sps,
                            sh.ph.as_ref().unwrap(),
                            ectx,
                        );
                        self.quantizer.quantize(&mut tu, c_idx, true, sh, ectx);
                        self.quantizer.dequantize(&mut tu, c_idx, sh, ectx);
                        self.transformer.inverse_transform(
                            &mut tu,
                            c_idx,
                            sh.sps,
                            sh.ph.as_ref().unwrap(),
                            ectx,
                        );
                        let tile = tu.get_tile();
                        let tile = &mut tile.lock().unwrap();
                        let (tx, ty) = tu.get_component_pos(c_idx);
                        let (tw, th) = tu.get_component_size(c_idx);
                        let pred_pixels = &tile.pred_pixels.borrow()[c_idx];
                        let reconst_pixels = &mut tile.reconst_pixels.borrow_mut()[c_idx];
                        let it = &tu.itransformed_coeffs[c_idx];
                        for y in ty..ty + th {
                            let pred_pixels = &pred_pixels[y][tx..];
                            let reconst_pixels = &mut reconst_pixels[y][tx..];
                            let it = &it[y - ty];
                            for x in 0..tw {
                                let pred = pred_pixels[x];
                                let res = it[x];
                                let rec = (pred as i16 + res).clamp(0, 255) as u8;
                                reconst_pixels[x] = rec;
                            }
                        }
                    }
                }
                let mode = num::FromPrimitive::from_usize(cand_modes[min_cost_idx]).unwrap();
                if sh.sps.cclm_enabled_flag && tree_type != TreeType::DUAL_TREE_LUMA {
                    let current_cost = self.get_chroma_intra_pred_cost(mode, ct, true, sh, ectx);
                    let cclm_lt_cost =
                        self.get_chroma_intra_pred_aux_cost(IntraPredMode::LT_CCLM, ct, sh, ectx);
                    let cclm_t_cost =
                        self.get_chroma_intra_pred_aux_cost(IntraPredMode::T_CCLM, ct, sh, ectx);
                    let cclm_l_cost =
                        self.get_chroma_intra_pred_aux_cost(IntraPredMode::L_CCLM, ct, sh, ectx);
                    let (cclm_mode, _cclm_cost) =
                        if cclm_lt_cost <= cclm_t_cost && cclm_lt_cost <= cclm_l_cost {
                            (IntraPredMode::LT_CCLM, cclm_lt_cost)
                        } else if cclm_t_cost <= cclm_l_cost {
                            (IntraPredMode::T_CCLM, cclm_t_cost)
                        } else {
                            (IntraPredMode::L_CCLM, cclm_l_cost)
                        };
                    let cclm_cost = self.get_chroma_intra_pred_cost(cclm_mode, ct, true, sh, ectx);
                    let chroma_cand_costs = [current_cost, cclm_cost];
                    let chroma_min_cost = chroma_cand_costs.iter().fold(f32::MAX, |m, v| v.min(m));
                    let chroma_min_cost_idx = chroma_cand_costs
                        .iter()
                        .position(|x| x == &chroma_min_cost)
                        .unwrap();
                    if chroma_min_cost_idx == 0 {
                        let modes = [mode; 3];
                        {
                            let cu = &mut cu.lock().unwrap();
                            cu.set_intra_pred_mode(modes);
                        }
                        min_cost = self.get_intra_pred_cost(modes, ct, true, sh, ectx);
                    } else {
                        let modes = [mode, cclm_mode, cclm_mode];
                        min_cost = self.get_intra_pred_cost(modes, ct, true, sh, ectx);
                    }
                } else if cand_modes[min_cost_idx] <= 1 {
                    let modes = [mode; 3];
                    min_cost = self.get_intra_pred_cost(modes, ct, true, sh, ectx);
                }
                min_cost
            }
        } else {
            let no_split_cost = self.split_ct(ct, 0, sh, ectx);
            let split_ct = {
                let ct = ct.lock().unwrap();
                Arc::new(Mutex::new(ct.clone()))
            };
            let no_split_reconsts = {
                let ct = ct.lock().unwrap();
                let tile = ct.tile.as_ref().unwrap();
                let tile = tile.lock().unwrap();
                let mut reconsts = vec![];
                for c_idx in 0..3 {
                    if ct.tree_type == TreeType::DUAL_TREE_LUMA && c_idx > 0 {
                        break;
                    } else if ct.tree_type == TreeType::DUAL_TREE_CHROMA && c_idx == 0 {
                        reconsts.push(vec2d![0; 1; 1]);
                        continue;
                    }
                    let (cx, cy) = ct.get_component_pos(c_idx);
                    let (cw, ch) = ct.get_component_size(c_idx);
                    let mut reconst = vec2d![0; ch; cw];
                    let tile_reconst = &tile.reconst_pixels.borrow()[c_idx];
                    for y in cy..cy + ch {
                        for x in cx..cx + cw {
                            reconst[y - cy][x - cx] = tile_reconst[y][x];
                        }
                    }
                    reconsts.push(reconst);
                }
                reconsts
            };
            let n = {
                let parent = split_ct.clone();
                let split_ct = &mut split_ct.lock().unwrap();
                split_ct.split(MttSplitMode::SPLIT_QT, parent, sh, ectx);
                split_ct.cts.len()
            };
            let mut split_cost = 0.0;
            for i in 0..n {
                let mut ct = {
                    let split_ct = &mut split_ct.lock().unwrap();
                    split_ct.cts[i].clone()
                };
                split_cost += self.split_ct(&mut ct, max_depth - 1, sh, ectx);
            }
            //println!("split={split_cost}, no={no_split_cost}");
            let cost = if split_cost > no_split_cost {
                let ct = ct.lock().unwrap();
                let tile = ct.tile.as_ref().unwrap();
                let tile = &mut tile.lock().unwrap();
                #[allow(clippy::needless_range_loop)]
                for c_idx in 0..3 {
                    if ct.tree_type == TreeType::DUAL_TREE_LUMA && c_idx > 0 {
                        break;
                    } else if ct.tree_type == TreeType::DUAL_TREE_CHROMA && c_idx == 0 {
                        continue;
                    }
                    let (cx, cy) = ct.get_component_pos(c_idx);
                    let (cw, ch) = ct.get_component_size(c_idx);
                    let tile_reconst = &mut tile.reconst_pixels.borrow_mut()[c_idx];
                    for y in cy..cy + ch {
                        for x in cx..cx + cw {
                            tile_reconst[y][x] = no_split_reconsts[c_idx][y - cy][x - cx];
                        }
                    }
                }
                no_split_cost
            } else {
                let mut ct = ct.lock().unwrap();
                let split_ct = split_ct.lock().unwrap();
                *ct = split_ct.clone();
                split_cost
            };
            cost
        }
    }
}

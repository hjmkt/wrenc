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
}

impl BlockSplitter {
    pub fn new() -> BlockSplitter {
        BlockSplitter {
            intra_predictor: IntraPredictor::new(),
            transformer: Transformer::new(),
            quantizer: Quantizer::new(),
        }
    }

    pub fn get_intra_pred_cost(
        &mut self,
        intra_pred_mode: [IntraPredMode; 3],
        ct: &mut Arc<Mutex<CodingTree>>,
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
                self.quantizer.quantize(&mut tu, c_idx, sh, ectx);
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

        let l0 = match ectx.extra_params.get("l0") {
            Some(l0) => l0.parse::<f32>().unwrap(),
            _ => 2.495_123_1,
        };
        let l1 = match ectx.extra_params.get("l1") {
            Some(l1) => l1.parse::<f32>().unwrap(),
            _ => 1.321_590_3,
        };
        let l2 = match ectx.extra_params.get("l2") {
            Some(l2) => l2.parse::<f32>().unwrap(),
            _ => 0.673_733_23,
        };
        let l3 = match ectx.extra_params.get("l3") {
            Some(l3) => l3.parse::<f32>().unwrap(),
            _ => 2.694_721_2,
        };
        let l4 = match ectx.extra_params.get("l4") {
            Some(l4) => l4.parse::<f32>().unwrap(),
            _ => 0.596_190_8,
        };
        let l5 = match ectx.extra_params.get("l5") {
            Some(l5) => l5.parse::<f32>().unwrap(),
            _ => 1.762_286_1,
        };
        let l6 = match ectx.extra_params.get("l6") {
            Some(l6) => l6.parse::<f32>().unwrap(),
            _ => 0.671_961_67,
        };
        let l7 = match ectx.extra_params.get("l7") {
            Some(l7) => l7.parse::<f32>().unwrap(),
            _ => 1.158_880_6,
        };
        let l8 = match ectx.extra_params.get("l8") {
            Some(l8) => l8.parse::<f32>().unwrap(),
            _ => 7.915_166,
        };
        let l9 = match ectx.extra_params.get("l9") {
            Some(l9) => l9.parse::<f32>().unwrap(),
            _ => 1.944_860_6,
        };
        let l10 = match ectx.extra_params.get("l10") {
            Some(l10) => l10.parse::<f32>().unwrap(),
            _ => 0.979_434_97,
        };

        let mode_bits = if non_planar_flag {
            l0 + if mpm_flag {
                (mpm_idx as f32 + l1).log2()
            } else {
                l2 * (mpm_remainder as f32 + l3).log2()
            }
        } else {
            l4
        } + if cclm_mode_flag {
            (cclm_mode_idx as f32 + l9).log2()
        } else if tree_type == TreeType::DUAL_TREE_LUMA {
            0.0
        } else {
            l10
        };
        let header_bits = match ectx.extra_params.get("b") {
            Some(beta) => beta.parse::<f32>().unwrap(),
            _ => l5,
        }; // FIXME estimate additional header bits for coding units
        let header_bits = {
            match tree_type {
                TreeType::SINGLE_TREE => header_bits + mode_bits,
                TreeType::DUAL_TREE_LUMA => header_bits / 3.0 + mode_bits,
                TreeType::DUAL_TREE_CHROMA => header_bits / 1.5,
            }
        };
        let level: f32 = tu
            .quantized_transformed_coeffs
            .iter()
            .flat_map(|a| {
                a.data.iter().map(|v| {
                    let v = v.unsigned_abs() as f32;
                    (v + l6).log2()
                })
            })
            .sum::<f32>() as f32
            + header_bits;
        let gamma = match ectx.extra_params.get("c") {
            Some(gamma) => gamma.parse::<f32>().unwrap(),
            _ => l7,
        };
        let d = 6.0 * gamma;
        let lambda = match ectx.extra_params.get("a") {
            Some(alpha) => (2.0f32).powf(tu.qp as f32 / d) * alpha.parse::<f32>().unwrap(),
            _ => (2.0f32).powf(tu.qp as f32 / d) * l8,
        };
        ssd as f32 + lambda * level
    }

    pub fn _get_luma_intra_pred_cost(
        &mut self,
        intra_pred_mode: IntraPredMode,
        ct: &mut Arc<Mutex<CodingTree>>,
        sh: &SliceHeader,
        ectx: &mut EncoderContext,
    ) -> f32 {
        let (tu, non_planar_flag, (mpm_flag, mpm_idx, mpm_remainder)) = {
            let cu = {
                let ct = ct.lock().unwrap();
                ct.cus[0].clone()
            };
            let cu = &mut cu.lock().unwrap();
            let mut mode = cu.intra_pred_mode;
            mode[0] = intra_pred_mode;
            cu.set_intra_pred_mode(mode);
            let tt = cu.transform_tree.as_ref().unwrap();
            let tt = tt.lock().unwrap();
            // FIXME multiple transform units
            (
                tt.tus[0].clone(),
                cu.get_intra_luma_not_planar_flag(),
                cu.get_intra_luma_mpm_flag_and_idx_and_remainder(),
            )
        };
        let mut tu = tu.borrow_mut();
        let mut ssd: usize = 0;
        let c_idx = 0;
        if tu.is_component_active(c_idx) {
            self.intra_predictor
                .predict(&mut tu, c_idx, sh.sps, sh.pps, ectx);
            self.transformer
                .transform(&mut tu, c_idx, sh.sps, sh.ph.as_ref().unwrap(), ectx);
            self.quantizer.quantize(&mut tu, c_idx, sh, ectx);
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
        let mode_bits = if non_planar_flag {
            2.6 + if mpm_flag {
                (mpm_idx as f32 + 1.45).log2()
            } else {
                0.65 * (mpm_remainder as f32 + 2.6).log2()
            }
        } else {
            0.6
        };
        let header_bits = match ectx.extra_params.get("b") {
            Some(beta) => beta.parse::<f32>().unwrap(),
            _ => 1.76,
        }; // FIXME estimate additional header bits for coding units
        let header_bits = {
            let ct = ct.lock().unwrap();
            match ct.tree_type {
                TreeType::SINGLE_TREE => header_bits + mode_bits,
                TreeType::DUAL_TREE_LUMA => header_bits / 3.0 + mode_bits,
                TreeType::DUAL_TREE_CHROMA => header_bits / 1.5,
            }
        };
        let level: f32 = tu
            .quantized_transformed_coeffs
            .iter()
            .flat_map(|a| {
                a.data.iter().map(|v| {
                    let v = v.unsigned_abs() as f32;
                    (v + 0.65).log2()
                })
            })
            .sum::<f32>() as f32
            + header_bits;
        let gamma = match ectx.extra_params.get("c") {
            Some(gamma) => gamma.parse::<f32>().unwrap(),
            _ => 0.93,
        };
        let d = 6.0 * gamma;
        let lambda = match ectx.extra_params.get("a") {
            Some(alpha) => (2.0f32).powf(tu.qp as f32 / d) * alpha.parse::<f32>().unwrap(),
            _ => (2.0f32).powf(tu.qp as f32 / d) * 7.73,
        };
        ssd as f32 + lambda * level
    }

    pub fn get_chroma_intra_pred_cost(
        &mut self,
        intra_pred_mode: IntraPredMode,
        ct: &mut Arc<Mutex<CodingTree>>,
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
                self.quantizer.quantize(&mut tu, c_idx, sh, ectx);
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

        let p0 = match ectx.extra_params.get("p0") {
            Some(p0) => p0.parse::<f32>().unwrap(),
            _ => 2.668_370_7,
        };
        let p1 = match ectx.extra_params.get("p1") {
            Some(p1) => p1.parse::<f32>().unwrap(),
            _ => 1.922_151_4,
        };
        let p2 = match ectx.extra_params.get("p2") {
            Some(p2) => p2.parse::<f32>().unwrap(),
            _ => 0.559_554_4,
        };
        let p3 = match ectx.extra_params.get("p3") {
            Some(p3) => p3.parse::<f32>().unwrap(),
            _ => 1.180_406_8,
        };
        let p4 = match ectx.extra_params.get("p4") {
            Some(p4) => p4.parse::<f32>().unwrap(),
            _ => 0.747_049_75,
        };
        let p5 = match ectx.extra_params.get("p5") {
            Some(p5) => p5.parse::<f32>().unwrap(),
            _ => 1.029_335,
        };
        let p6 = match ectx.extra_params.get("p6") {
            Some(p6) => p6.parse::<f32>().unwrap(),
            _ => 7.775_918,
        };

        let mode_bits = if cclm_mode_flag {
            p0 + (cclm_mode_idx as f32 + p1).log2()
        } else {
            p2
        };
        let header_bits = match ectx.extra_params.get("b") {
            Some(beta) => beta.parse::<f32>().unwrap(),
            _ => p3,
        }; // FIXME estimate additional header bits for coding units
        let header_bits = {
            let ct = ct.lock().unwrap();
            match ct.tree_type {
                TreeType::SINGLE_TREE => header_bits + mode_bits,
                TreeType::DUAL_TREE_LUMA => panic!(),
                TreeType::DUAL_TREE_CHROMA => header_bits + mode_bits,
            }
        };
        let level: f32 = tu.quantized_transformed_coeffs[1..]
            .iter()
            .flat_map(|a| {
                a.data.iter().map(|v| {
                    let v = v.unsigned_abs() as f32;
                    (v + p4).log2()
                })
            })
            .sum::<f32>() as f32
            + header_bits;
        let gamma = match ectx.extra_params.get("c") {
            Some(gamma) => gamma.parse::<f32>().unwrap(),
            _ => p5,
        };
        let d = 6.0 * gamma;
        let lambda = match ectx.extra_params.get("a") {
            Some(alpha) => (2.0f32).powf(tu.qp as f32 / d) * alpha.parse::<f32>().unwrap(),
            _ => (2.0f32).powf(tu.qp as f32 / d) * p6,
        };
        ssd as f32 + lambda * level
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
                let cache_reconsts = |ct: &Arc<Mutex<CodingTree>>| -> Vec<Vec2d<u8>> {
                    let ct = ct.lock().unwrap();
                    let tile = ct.tile.as_ref().unwrap();
                    let tile = tile.lock().unwrap();
                    let mut reconsts = vec![];
                    for c_idx in 1..3 {
                        let mut reconst = vec2d![0; ct.height/2; ct.width/2];
                        let tile_reconst = &tile.reconst_pixels.borrow()[c_idx];
                        for y in ct.y / 2..ct.y / 2 + ct.height / 2 {
                            for x in ct.x / 2..ct.x / 2 + ct.width / 2 {
                                reconst[y - ct.y / 2][x - ct.x / 2] = tile_reconst[y][x];
                            }
                        }
                        reconsts.push(reconst);
                    }
                    reconsts
                };
                let restore_reconsts = |ct: &Arc<Mutex<CodingTree>>, cache: Vec<Vec2d<u8>>| {
                    let ct = ct.lock().unwrap();
                    let tile = ct.tile.as_ref().unwrap();
                    let tile = &mut tile.lock().unwrap();
                    for c_idx in 1..3 {
                        let tile_reconst = &mut tile.reconst_pixels.borrow_mut()[c_idx];
                        for y in ct.y / 2..ct.y / 2 + ct.height / 2 {
                            for x in ct.x / 2..ct.x / 2 + ct.width / 2 {
                                tile_reconst[y][x] = cache[c_idx - 1][y - ct.y / 2][x - ct.x / 2];
                            }
                        }
                    }
                };
                let current_cost = self.get_chroma_intra_pred_cost(chroma_pred_mode, ct, sh, ectx);
                let current_reconsts = cache_reconsts(ct);
                let cclm_lt_cost =
                    self.get_chroma_intra_pred_cost(IntraPredMode::LT_CCLM, ct, sh, ectx);
                let lt_reconsts = cache_reconsts(ct);
                let cclm_t_cost =
                    self.get_chroma_intra_pred_cost(IntraPredMode::T_CCLM, ct, sh, ectx);
                let t_reconsts = cache_reconsts(ct);
                let cclm_l_cost =
                    self.get_chroma_intra_pred_cost(IntraPredMode::L_CCLM, ct, sh, ectx);
                let l_reconsts = cache_reconsts(ct);
                let chroma_cand_costs = [current_cost, cclm_lt_cost, cclm_t_cost, cclm_l_cost];
                let chroma_min_cost = chroma_cand_costs.iter().fold(f32::MAX, |m, v| v.min(m));
                let chroma_min_cost_idx = chroma_cand_costs
                    .iter()
                    .position(|x| x == &chroma_min_cost)
                    .unwrap();
                let cu = {
                    let ct = ct.lock().unwrap();
                    ct.cus[0].clone()
                };
                if chroma_min_cost_idx == 0 {
                    let cu = &mut cu.lock().unwrap();
                    cu.set_intra_pred_mode([chroma_pred_mode; 3]);
                    restore_reconsts(ct, current_reconsts);
                } else if chroma_min_cost_idx == 1 {
                    let cu = &mut cu.lock().unwrap();
                    cu.set_intra_pred_mode([IntraPredMode::LT_CCLM; 3]);
                    restore_reconsts(ct, lt_reconsts);
                } else if chroma_min_cost_idx == 2 {
                    let cu = &mut cu.lock().unwrap();
                    cu.set_intra_pred_mode([IntraPredMode::T_CCLM; 3]);
                    restore_reconsts(ct, t_reconsts);
                } else if chroma_min_cost_idx == 3 {
                    let cu = &mut cu.lock().unwrap();
                    cu.set_intra_pred_mode([IntraPredMode::L_CCLM; 3]);
                    restore_reconsts(ct, l_reconsts);
                }
                chroma_min_cost
            } else {
                let cand_modes = [0, 1, 2, 10, 18, 26, 34, 42, 50, 58, 66];
                let cand_costs = cand_modes
                    .iter()
                    .map(|m| {
                        let mode = num::FromPrimitive::from_usize(*m).unwrap();
                        self.get_intra_pred_cost([mode; 3], ct, sh, ectx)
                    })
                    .collect::<Vec<f32>>();
                let min_dir_cost = cand_costs[2..].iter().fold(f32::MAX, |m, v| v.min(m));
                let min_dir_cost_idx = cand_costs[2..]
                    .iter()
                    .position(|x| x == &min_dir_cost)
                    .unwrap()
                    + 2;
                let mut step_search =
                    |current_mode: usize, step: usize, current_cost: f32| -> (usize, f32) {
                        let (mut current_mode, mut step, mut current_cost) =
                            (current_mode, step, current_cost);
                        while step > 0 {
                            if current_mode >= 2 + step {
                                if current_mode + step <= 66 {
                                    let cost0 = self.get_intra_pred_cost(
                                        [num::FromPrimitive::from_usize(current_mode - step)
                                            .unwrap(); 3],
                                        ct,
                                        sh,
                                        ectx,
                                    );
                                    let cost1 = self.get_intra_pred_cost(
                                        [num::FromPrimitive::from_usize(current_mode + step)
                                            .unwrap(); 3],
                                        ct,
                                        sh,
                                        ectx,
                                    );
                                    let min_cost = current_cost.min(cost0).min(cost1);
                                    (current_mode, current_cost) = if current_cost == min_cost {
                                        (current_mode, current_cost)
                                    } else if cost0 == min_cost {
                                        (current_mode - step, cost0)
                                    } else {
                                        (current_mode + step, cost1)
                                    };
                                } else {
                                    let cost = self.get_intra_pred_cost(
                                        [num::FromPrimitive::from_usize(current_mode - step)
                                            .unwrap(); 3],
                                        ct,
                                        sh,
                                        ectx,
                                    );
                                    let min_cost = current_cost.min(cost);
                                    (current_mode, current_cost) = if current_cost == min_cost {
                                        (current_mode, current_cost)
                                    } else {
                                        (current_mode - step, cost)
                                    };
                                }
                            } else if current_mode + step <= 66 {
                                let cost = self.get_intra_pred_cost(
                                    [num::FromPrimitive::from_usize(current_mode + step).unwrap();
                                        3],
                                    ct,
                                    sh,
                                    ectx,
                                );
                                let min_cost = current_cost.min(cost);
                                (current_mode, current_cost) = if current_cost == min_cost {
                                    (current_mode, current_cost)
                                } else {
                                    (current_mode + step, cost)
                                };
                            }
                            step /= 2;
                        }
                        (current_mode, current_cost)
                    };
                let (dir_mode, dir_cost) =
                    step_search(cand_modes[min_dir_cost_idx], 4, min_dir_cost);
                let cand_modes = [0, 1, dir_mode];
                let cand_costs = [cand_costs[0], cand_costs[1], dir_cost];
                let mut min_cost = cand_costs.iter().fold(f32::MAX, |m, v| v.min(m));
                let min_cost_idx = cand_costs.iter().position(|x| x == &min_cost).unwrap();
                let cu = {
                    let ct = ct.lock().unwrap();
                    ct.cus[0].clone()
                };
                if cand_modes[min_cost_idx] == 0 {
                    let cu = &mut cu.lock().unwrap();
                    cu.set_intra_pred_mode([IntraPredMode::PLANAR; 3]);
                } else if cand_modes[min_cost_idx] == 1 {
                    let cu = &mut cu.lock().unwrap();
                    cu.set_intra_pred_mode([IntraPredMode::DC; 3]);
                } else {
                    let cu = &mut cu.lock().unwrap();
                    cu.set_intra_pred_mode([num::FromPrimitive::from_usize(dir_mode).unwrap(); 3]);
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
                        self.quantizer.quantize(&mut tu, c_idx, sh, ectx);
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
                if cand_modes[min_cost_idx] == 0 {
                    if sh.sps.cclm_enabled_flag && tree_type != TreeType::DUAL_TREE_LUMA {
                        let current_cost =
                            self.get_chroma_intra_pred_cost(IntraPredMode::PLANAR, ct, sh, ectx);
                        let cclm_lt_cost =
                            self.get_chroma_intra_pred_cost(IntraPredMode::LT_CCLM, ct, sh, ectx);
                        let cclm_t_cost =
                            self.get_chroma_intra_pred_cost(IntraPredMode::T_CCLM, ct, sh, ectx);
                        let cclm_l_cost =
                            self.get_chroma_intra_pred_cost(IntraPredMode::L_CCLM, ct, sh, ectx);
                        let chroma_cand_costs =
                            [current_cost, cclm_lt_cost, cclm_t_cost, cclm_l_cost];
                        let chroma_min_cost =
                            chroma_cand_costs.iter().fold(f32::MAX, |m, v| v.min(m));
                        let chroma_min_cost_idx = chroma_cand_costs
                            .iter()
                            .position(|x| x == &chroma_min_cost)
                            .unwrap();
                        if chroma_min_cost_idx == 0 {
                            let modes = [IntraPredMode::PLANAR; 3];
                            {
                                let cu = &mut cu.lock().unwrap();
                                cu.set_intra_pred_mode(modes);
                            }
                            min_cost = self.get_intra_pred_cost(modes, ct, sh, ectx);
                        } else if chroma_min_cost_idx == 1 {
                            let modes = [
                                IntraPredMode::PLANAR,
                                IntraPredMode::LT_CCLM,
                                IntraPredMode::LT_CCLM,
                            ];
                            min_cost = self.get_intra_pred_cost(modes, ct, sh, ectx);
                            let cu = &mut cu.lock().unwrap();
                            cu.set_intra_pred_mode(modes);
                        } else if chroma_min_cost_idx == 2 {
                            let modes = [
                                IntraPredMode::PLANAR,
                                IntraPredMode::T_CCLM,
                                IntraPredMode::T_CCLM,
                            ];
                            min_cost = self.get_intra_pred_cost(modes, ct, sh, ectx);
                            let cu = &mut cu.lock().unwrap();
                            cu.set_intra_pred_mode(modes);
                        } else if chroma_min_cost_idx == 3 {
                            let modes = [
                                IntraPredMode::PLANAR,
                                IntraPredMode::L_CCLM,
                                IntraPredMode::L_CCLM,
                            ];
                            min_cost = self.get_intra_pred_cost(modes, ct, sh, ectx);
                            let cu = &mut cu.lock().unwrap();
                            cu.set_intra_pred_mode(modes);
                        }
                    } else {
                        let modes = [IntraPredMode::PLANAR; 3];
                        min_cost = self.get_intra_pred_cost(modes, ct, sh, ectx);
                    }
                    min_cost
                } else if cand_modes[min_cost_idx] == 1 {
                    if sh.sps.cclm_enabled_flag && tree_type != TreeType::DUAL_TREE_LUMA {
                        let current_cost =
                            self.get_chroma_intra_pred_cost(IntraPredMode::DC, ct, sh, ectx);
                        let cclm_lt_cost =
                            self.get_chroma_intra_pred_cost(IntraPredMode::LT_CCLM, ct, sh, ectx);
                        let cclm_t_cost =
                            self.get_chroma_intra_pred_cost(IntraPredMode::T_CCLM, ct, sh, ectx);
                        let cclm_l_cost =
                            self.get_chroma_intra_pred_cost(IntraPredMode::L_CCLM, ct, sh, ectx);
                        let chroma_cand_costs =
                            [current_cost, cclm_lt_cost, cclm_t_cost, cclm_l_cost];
                        let chroma_min_cost =
                            chroma_cand_costs.iter().fold(f32::MAX, |m, v| v.min(m));
                        let chroma_min_cost_idx = chroma_cand_costs
                            .iter()
                            .position(|x| x == &chroma_min_cost)
                            .unwrap();
                        if chroma_min_cost_idx == 0 {
                            let modes = [IntraPredMode::DC; 3];
                            {
                                let cu = &mut cu.lock().unwrap();
                                cu.set_intra_pred_mode(modes);
                            }
                            min_cost = self.get_intra_pred_cost(modes, ct, sh, ectx);
                        } else if chroma_min_cost_idx == 1 {
                            let modes = [
                                IntraPredMode::DC,
                                IntraPredMode::LT_CCLM,
                                IntraPredMode::LT_CCLM,
                            ];
                            min_cost = self.get_intra_pred_cost(modes, ct, sh, ectx);
                            let cu = &mut cu.lock().unwrap();
                            cu.set_intra_pred_mode(modes);
                        } else if chroma_min_cost_idx == 2 {
                            let modes = [
                                IntraPredMode::DC,
                                IntraPredMode::T_CCLM,
                                IntraPredMode::T_CCLM,
                            ];
                            min_cost = self.get_intra_pred_cost(modes, ct, sh, ectx);
                            let cu = &mut cu.lock().unwrap();
                            cu.set_intra_pred_mode(modes);
                        } else if chroma_min_cost_idx == 3 {
                            let modes = [
                                IntraPredMode::DC,
                                IntraPredMode::L_CCLM,
                                IntraPredMode::L_CCLM,
                            ];
                            min_cost = self.get_intra_pred_cost(modes, ct, sh, ectx);
                            let cu = &mut cu.lock().unwrap();
                            cu.set_intra_pred_mode(modes);
                        }
                    } else {
                        let modes = [IntraPredMode::DC; 3];
                        min_cost = self.get_intra_pred_cost(modes, ct, sh, ectx);
                    }
                    min_cost
                } else {
                    let (best_mode, mut min_cost) = (dir_mode, dir_cost);
                    if sh.sps.cclm_enabled_flag && tree_type != TreeType::DUAL_TREE_LUMA {
                        let mode = num::FromPrimitive::from_usize(best_mode).unwrap();
                        let current_cost = self.get_chroma_intra_pred_cost(mode, ct, sh, ectx);
                        let cclm_lt_cost =
                            self.get_chroma_intra_pred_cost(IntraPredMode::LT_CCLM, ct, sh, ectx);
                        let cclm_t_cost =
                            self.get_chroma_intra_pred_cost(IntraPredMode::T_CCLM, ct, sh, ectx);
                        let cclm_l_cost =
                            self.get_chroma_intra_pred_cost(IntraPredMode::L_CCLM, ct, sh, ectx);
                        let chroma_cand_costs =
                            [current_cost, cclm_lt_cost, cclm_t_cost, cclm_l_cost];
                        let chroma_min_cost =
                            chroma_cand_costs.iter().fold(f32::MAX, |m, v| v.min(m));
                        let chroma_min_cost_idx = chroma_cand_costs
                            .iter()
                            .position(|x| x == &chroma_min_cost)
                            .unwrap();
                        if chroma_min_cost_idx == 0 {
                            let modes = [mode; 3];
                            min_cost = self.get_intra_pred_cost(modes, ct, sh, ectx);
                        } else if chroma_min_cost_idx == 1 {
                            let modes = [mode, IntraPredMode::LT_CCLM, IntraPredMode::LT_CCLM];
                            min_cost = self.get_intra_pred_cost(modes, ct, sh, ectx);
                        } else if chroma_min_cost_idx == 2 {
                            let modes = [mode, IntraPredMode::T_CCLM, IntraPredMode::T_CCLM];
                            min_cost = self.get_intra_pred_cost(modes, ct, sh, ectx);
                        } else if chroma_min_cost_idx == 3 {
                            let modes = [mode, IntraPredMode::L_CCLM, IntraPredMode::L_CCLM];
                            min_cost = self.get_intra_pred_cost(modes, ct, sh, ectx);
                        }
                    } else {
                        let modes = [num::FromPrimitive::from_usize(best_mode).unwrap(); 3];
                        min_cost = self.get_intra_pred_cost(modes, ct, sh, ectx);
                    }
                    min_cost
                }
            }
        } else {
            let no_split_cost = self.split_ct(ct, 0, sh, ectx);
            let split_ct = {
                let ct = ct.lock().unwrap();
                Arc::new(Mutex::new(ct.clone()))
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

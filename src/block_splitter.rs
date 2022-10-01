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
        let (tu, non_planar_flag, (mpm_flag, _mpm_idx, mpm_remainder)) = {
            let cu = {
                let ct = ct.lock().unwrap();
                ct.cus[0].clone()
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
        let mode_bits = if non_planar_flag {
            2.0 + if mpm_flag {
                5.0f32.log2()
            } else {
                0.6 * (mpm_remainder as f32 + 2.3).log2()
            }
        } else {
            1.0
        };
        let header_bits = match ectx.extra_params.get("b") {
            Some(beta) => beta.parse::<f32>().unwrap(),
            _ => 1.753,
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
                    let v = v.unsigned_abs() as usize;
                    let b = if v == 0 { 0 } else { v.ilog2() + 1 };
                    //let b = v.ilog2() + 1;
                    b as usize
                })
            })
            .sum::<usize>() as f32
            + header_bits;
        let gamma = match ectx.extra_params.get("c") {
            Some(gamma) => gamma.parse::<f32>().unwrap(),
            _ => 0.954,
        };
        let d = 6.0 * gamma;
        let lambda = match ectx.extra_params.get("a") {
            Some(alpha) => (2.0f32).powf(tu.qp as f32 / d) * alpha.parse::<f32>().unwrap(),
            _ => (2.0f32).powf(tu.qp as f32 / d) * 7.73,
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
                let luma_cu = luma_cu.as_ref().unwrap();
                let luma_cu = luma_cu.lock().unwrap();
                //let luma_pred_mode = luma_cu.intra_pred_mode[0];
                // FIXME
                let (chroma_pred_mode, _) =
                    luma_cu.get_intra_chroma_pred_mode_and_mip_chroma_direct_mode_flag();
                match chroma_pred_mode {
                    IntraPredMode::PLANAR => {
                        let modes = {
                            let cu = {
                                let ct = ct.lock().unwrap();
                                ct.cus[0].clone()
                            };
                            let cu = &mut cu.lock().unwrap();
                            let luma_pred_mode = cu.intra_pred_mode[0];
                            let modes =
                                [luma_pred_mode, IntraPredMode::PLANAR, IntraPredMode::PLANAR];
                            cu.set_intra_pred_mode(modes);
                            modes
                        };
                        self.get_intra_pred_cost(modes, ct, sh, ectx)
                    }
                    IntraPredMode::DC => {
                        let modes = {
                            let cu = {
                                let ct = ct.lock().unwrap();
                                ct.cus[0].clone()
                            };
                            let cu = &mut cu.lock().unwrap();
                            let luma_pred_mode = cu.intra_pred_mode[0];
                            let modes = [luma_pred_mode, IntraPredMode::DC, IntraPredMode::DC];
                            cu.set_intra_pred_mode(modes);
                            modes
                        };
                        self.get_intra_pred_cost(modes, ct, sh, ectx)
                    }
                    pred_mode => {
                        let m = pred_mode as usize;
                        if m <= 66 {
                            let modes = {
                                let cu = {
                                    let ct = ct.lock().unwrap();
                                    ct.cus[0].clone()
                                };
                                let cu = &mut cu.lock().unwrap();
                                let luma_pred_mode = cu.intra_pred_mode[0];
                                let modes = [luma_pred_mode, pred_mode, pred_mode];
                                cu.set_intra_pred_mode(modes);
                                modes
                            };
                            self.get_intra_pred_cost(modes, ct, sh, ectx)
                        } else {
                            panic!();
                        }
                    }
                }
            } else {
                let angular2_cost =
                    self.get_intra_pred_cost([IntraPredMode::ANGULAR2; 3], ct, sh, ectx);
                let angular10_cost =
                    self.get_intra_pred_cost([IntraPredMode::ANGULAR10; 3], ct, sh, ectx);
                let angular18_cost =
                    self.get_intra_pred_cost([IntraPredMode::ANGULAR18; 3], ct, sh, ectx);
                let angular26_cost =
                    self.get_intra_pred_cost([IntraPredMode::ANGULAR26; 3], ct, sh, ectx);
                let angular34_cost =
                    self.get_intra_pred_cost([IntraPredMode::ANGULAR34; 3], ct, sh, ectx);
                let angular42_cost =
                    self.get_intra_pred_cost([IntraPredMode::ANGULAR42; 3], ct, sh, ectx);
                let angular50_cost =
                    self.get_intra_pred_cost([IntraPredMode::ANGULAR50; 3], ct, sh, ectx);
                let angular58_cost =
                    self.get_intra_pred_cost([IntraPredMode::ANGULAR58; 3], ct, sh, ectx);
                let angular66_cost =
                    self.get_intra_pred_cost([IntraPredMode::ANGULAR66; 3], ct, sh, ectx);
                let dc_cost = self.get_intra_pred_cost([IntraPredMode::DC; 3], ct, sh, ectx);
                let planar_cost =
                    self.get_intra_pred_cost([IntraPredMode::PLANAR; 3], ct, sh, ectx);
                let min_cost = [
                    angular2_cost,
                    angular10_cost,
                    angular18_cost,
                    angular26_cost,
                    angular34_cost,
                    angular42_cost,
                    angular50_cost,
                    angular58_cost,
                    angular66_cost,
                    dc_cost,
                    planar_cost,
                ]
                .iter()
                .fold(f32::MAX, |m, v| v.min(m));
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
                if planar_cost == min_cost {
                    // skip updating cu as its intra pred mode is already set as planar
                    //{
                    //let cu = {
                    //let ct = ct.lock().unwrap();
                    //ct.cus[0].clone()
                    //};
                    //let cu = &mut cu.lock().unwrap();
                    //cu.set_intra_pred_mode([IntraPredMode::PLANAR; 3]);
                    //}
                    planar_cost
                } else if angular2_cost == min_cost {
                    let (best_mode, min_cost) = step_search(2, 4, min_cost);
                    {
                        let cu = {
                            let ct = ct.lock().unwrap();
                            ct.cus[0].clone()
                        };
                        let cu = &mut cu.lock().unwrap();
                        cu.set_intra_pred_mode(
                            [num::FromPrimitive::from_usize(best_mode).unwrap(); 3],
                        );
                    }
                    min_cost
                } else if angular10_cost == min_cost {
                    let (best_mode, min_cost) = step_search(10, 4, min_cost);
                    {
                        let cu = {
                            let ct = ct.lock().unwrap();
                            ct.cus[0].clone()
                        };
                        let cu = &mut cu.lock().unwrap();
                        cu.set_intra_pred_mode(
                            [num::FromPrimitive::from_usize(best_mode).unwrap(); 3],
                        );
                    }
                    min_cost
                } else if angular18_cost == min_cost {
                    let (best_mode, min_cost) = step_search(18, 4, min_cost);
                    {
                        let cu = {
                            let ct = ct.lock().unwrap();
                            ct.cus[0].clone()
                        };
                        let cu = &mut cu.lock().unwrap();
                        cu.set_intra_pred_mode(
                            [num::FromPrimitive::from_usize(best_mode).unwrap(); 3],
                        );
                    }
                    min_cost
                } else if angular26_cost == min_cost {
                    let (best_mode, min_cost) = step_search(26, 4, min_cost);
                    {
                        let cu = {
                            let ct = ct.lock().unwrap();
                            ct.cus[0].clone()
                        };
                        let cu = &mut cu.lock().unwrap();
                        cu.set_intra_pred_mode(
                            [num::FromPrimitive::from_usize(best_mode).unwrap(); 3],
                        );
                    }
                    min_cost
                } else if angular34_cost == min_cost {
                    let (best_mode, min_cost) = step_search(34, 4, min_cost);
                    {
                        let cu = {
                            let ct = ct.lock().unwrap();
                            ct.cus[0].clone()
                        };
                        let cu = &mut cu.lock().unwrap();
                        cu.set_intra_pred_mode(
                            [num::FromPrimitive::from_usize(best_mode).unwrap(); 3],
                        );
                    }
                    min_cost
                } else if angular42_cost == min_cost {
                    let (best_mode, min_cost) = step_search(42, 4, min_cost);
                    {
                        let cu = {
                            let ct = ct.lock().unwrap();
                            ct.cus[0].clone()
                        };
                        let cu = &mut cu.lock().unwrap();
                        cu.set_intra_pred_mode(
                            [num::FromPrimitive::from_usize(best_mode).unwrap(); 3],
                        );
                    }
                    min_cost
                } else if angular50_cost == min_cost {
                    let (best_mode, min_cost) = step_search(50, 4, min_cost);
                    {
                        let cu = {
                            let ct = ct.lock().unwrap();
                            ct.cus[0].clone()
                        };
                        let cu = &mut cu.lock().unwrap();
                        cu.set_intra_pred_mode(
                            [num::FromPrimitive::from_usize(best_mode).unwrap(); 3],
                        );
                    }
                    min_cost
                } else if angular58_cost == min_cost {
                    let (best_mode, min_cost) = step_search(58, 4, min_cost);
                    {
                        let cu = {
                            let ct = ct.lock().unwrap();
                            ct.cus[0].clone()
                        };
                        let cu = &mut cu.lock().unwrap();
                        cu.set_intra_pred_mode(
                            [num::FromPrimitive::from_usize(best_mode).unwrap(); 3],
                        );
                    }
                    min_cost
                } else if angular66_cost == min_cost {
                    let (best_mode, min_cost) = step_search(66, 4, min_cost);
                    {
                        let cu = {
                            let ct = ct.lock().unwrap();
                            ct.cus[0].clone()
                        };
                        let cu = &mut cu.lock().unwrap();
                        cu.set_intra_pred_mode(
                            [num::FromPrimitive::from_usize(best_mode).unwrap(); 3],
                        );
                    }
                    min_cost
                } else {
                    {
                        let cu = {
                            let ct = ct.lock().unwrap();
                            ct.cus[0].clone()
                        };
                        let cu = &mut cu.lock().unwrap();
                        cu.set_intra_pred_mode([IntraPredMode::DC; 3]);
                    }
                    dc_cost
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

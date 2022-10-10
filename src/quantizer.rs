use super::common::*;
use super::ctu::*;
use super::encoder_context::*;
use super::slice_header::*;
use super::transformer::*;
//use debug_print::*;

const LEVEL_SCALE: [[i32; 6]; 2] = [[40, 45, 51, 57, 64, 72], [57, 64, 72, 80, 90, 102]];

pub struct Quantizer {
    pub dq_table: Vec<Vec<f32>>,
}

impl Quantizer {
    pub fn new(ectx: &EncoderContext) -> Quantizer {
        let q0 = match ectx.extra_params.get("q0") {
            Some(q0) => q0.parse::<f32>().unwrap(),
            _ => 5.479_013_4,
        };
        let q1 = match ectx.extra_params.get("q1") {
            Some(q1) => q1.parse::<f32>().unwrap(),
            _ => 1.120_687_8,
        };
        let q2 = match ectx.extra_params.get("q2") {
            Some(q2) => q2.parse::<f32>().unwrap(),
            _ => 2.531_597_4,
        };
        let delta = q2;
        let mut dq_table = vec![];
        for qp in 0..64 {
            let lambda = (2.0f32).powf(qp as f32 / q0) * q1;
            let mut table = vec![];
            for a in 0..100 {
                let t = lambda * (a as f32 + delta).log2();
                table.push(t);
            }
            dq_table.push(table);
        }
        Quantizer { dq_table }
    }

    #[inline(always)]
    pub fn get_dq_cost(&self, qp: usize, a: usize, lambda: f32, delta: f32) -> f32 {
        let table = &self.dq_table[qp];
        if a < table.len() {
            table[a]
        } else {
            lambda * (a as f32 + delta).log2()
        }
    }

    pub fn get_scaling_matrix_id(pred_mode: ModeType, c_idx: usize, max_tb_size: usize) -> usize {
        match pred_mode {
            ModeType::MODE_INTRA => match c_idx {
                0 => match max_tb_size {
                    4 => 2,
                    8 => 8,
                    16 => 14,
                    32 => 20,
                    64 => 26,
                    _ => panic!(),
                },
                1 => match max_tb_size {
                    4 => 3,
                    8 => 9,
                    16 => 15,
                    32 => 21,
                    64 => 21,
                    _ => panic!(),
                },
                2 => match max_tb_size {
                    4 => 4,
                    8 => 10,
                    16 => 16,
                    32 => 22,
                    64 => 22,
                    _ => panic!(),
                },
                _ => panic!(),
            },
            ModeType::MODE_INTER | ModeType::MODE_IBC => match c_idx {
                0 => match max_tb_size {
                    4 => 5,
                    8 => 11,
                    16 => 17,
                    32 => 23,
                    64 => 27,
                    _ => panic!(),
                },
                1 => match max_tb_size {
                    2 => 0,
                    4 => 6,
                    8 => 12,
                    16 => 18,
                    32 => 24,
                    64 => 24,
                    _ => panic!(),
                },
                2 => match max_tb_size {
                    2 => 1,
                    4 => 7,
                    8 => 13,
                    16 => 19,
                    32 => 25,
                    64 => 25,
                    _ => panic!(),
                },
                _ => panic!(),
            },
            _ => panic!(),
        }
    }

    pub fn derive_qp(
        &self,
        tu: &mut TransformUnit,
        sh: &SliceHeader,
        ectx: &EncoderContext,
    ) -> (usize, usize, usize, usize) {
        if let Some(qps) = tu.derive_qp_cache {
            return qps;
        }
        let is_in_first_qg_in_slice_or_tile = tu.is_in_first_qg_in_slice_or_tile(ectx);
        let is_in_first_qg_in_ctb_row_in_tile = tu.is_in_first_qg_in_ctb_row_in_tile(ectx);
        let cu_qp_delta = tu.get_cu_qp_delta(sh.sps, sh.pps, ectx);
        let ((x_cb, y_cb), (w_cb, h_cb)) = (tu.cu_pos[0], tu.cu_size[0]);
        let x_qg = ectx.cu_qg_top_left_x;
        let y_qg = ectx.cu_qg_top_left_y;
        //debug_eprintln!("x_qg={}, y_qg={}", x_qg, y_qg);
        let tile = tu.get_tile();
        let tile = tile.lock().unwrap();
        let mut qp_y =
            if tu.tree_type == TreeType::SINGLE_TREE || tu.tree_type == TreeType::DUAL_TREE_LUMA {
                let qp_y_prev = if is_in_first_qg_in_slice_or_tile {
                    ectx.slice_qp_y as usize
                } else {
                    ectx.qp_y
                };
                let (tw, th) = tu.get_component_size(0);
                //debug_eprintln!("qp_y_prev={}", qp_y_prev);
                let is_above_right_available = tu.is_above_right_available();
                let is_below_left_available = tu.is_below_left_available();
                let available_a = ectx.derive_neighbouring_block_availability(
                    x_cb,
                    y_cb,
                    x_qg as isize - 1,
                    y_qg as isize,
                    tw,
                    th,
                    is_above_right_available,
                    is_below_left_available,
                    false,
                    sh.sps,
                    sh.pps,
                );
                let qp_y_a = if !available_a
                    || (x_qg as isize - 1) >> ectx.ctb_log2_size_y
                        != x_cb as isize >> ectx.ctb_log2_size_y
                    || y_qg >> ectx.ctb_log2_size_y != y_cb >> ectx.ctb_log2_size_y
                {
                    qp_y_prev
                } else {
                    let left_qg_cu = tile.get_cu(x_qg as isize - 1, y_qg as isize);
                    let left_qg_cu = left_qg_cu.as_ref().unwrap();
                    let left_qg_cu = left_qg_cu.lock().unwrap();
                    left_qg_cu.qp_y // qp of the CU containing the luma coding block covering (x_qg-1, y_qg)
                };
                let available_b = ectx.derive_neighbouring_block_availability(
                    x_cb,
                    y_cb,
                    x_qg as isize,
                    y_qg as isize - 1,
                    tw,
                    th,
                    is_above_right_available,
                    is_below_left_available,
                    false,
                    sh.sps,
                    sh.pps,
                );
                let qp_y_b = if !available_b
                    || x_qg >> ectx.ctb_log2_size_y != x_cb >> ectx.ctb_log2_size_y
                    || (y_qg as isize - 1) >> ectx.ctb_log2_size_y
                        != y_cb as isize >> ectx.ctb_log2_size_y
                {
                    qp_y_prev
                } else {
                    let above_qg_cu = tile.get_cu(x_qg as isize, y_qg as isize - 1);
                    let above_qg_cu = above_qg_cu.as_ref().unwrap();
                    let above_qg_cu = above_qg_cu.lock().unwrap();
                    above_qg_cu.qp_y
                };
                let qp_y_pred = if available_b && is_in_first_qg_in_ctb_row_in_tile {
                    let above_qg_cu = tile.get_cu(x_qg as isize, y_qg as isize - 1);
                    let above_qg_cu = above_qg_cu.as_ref().unwrap();
                    let above_qg_cu = above_qg_cu.lock().unwrap();
                    above_qg_cu.qp_y
                } else {
                    (qp_y_a + qp_y_b + 1) >> 1
                };
                let qp_y = (qp_y_pred as isize + cu_qp_delta + 64 + 2 * ectx.qp_bd_offset)
                    % (64 + ectx.qp_bd_offset)
                    - ectx.qp_bd_offset;
                //debug_eprintln!("delta={}, qp_y={}", cu_qp_delta, qp_y);
                qp_y + ectx.qp_bd_offset
            } else {
                0
            };
        let (qp_cb, qp_cr, qp_cb_cr) = if sh.sps.chroma_format != ChromaFormat::Monochrome
            && (tu.tree_type == TreeType::SINGLE_TREE || tu.tree_type == TreeType::DUAL_TREE_CHROMA)
        {
            if tu.tree_type == TreeType::DUAL_TREE_CHROMA {
                let cu = tile.get_cu((x_cb + w_cb / 2) as isize, (y_cb + h_cb / 2) as isize);
                let cu = cu.as_ref().unwrap();
                let cu = cu.lock().unwrap();
                qp_y = cu.qp_y as isize;
            }
            let qp_chroma = qp_y.clamp(-ectx.qp_bd_offset, 63);
            let qp_cb = ectx.chroma_qp_table[0][qp_chroma as usize];
            let qp_cr = ectx.chroma_qp_table[1][qp_chroma as usize];
            let qp_cb_cr = ectx.chroma_qp_table[2][qp_chroma as usize];
            (
                (qp_cb
                    + sh.pps.chroma_tool_offsets.cb_qp_offset
                    + sh.cb_qp_offset
                    + ectx.cu_qp_offset_cb as isize)
                    .clamp(-ectx.qp_bd_offset, 63)
                    + ectx.qp_bd_offset,
                (qp_cr
                    + sh.pps.chroma_tool_offsets.cr_qp_offset
                    + sh.cr_qp_offset
                    + ectx.cu_qp_offset_cr as isize)
                    .clamp(-ectx.qp_bd_offset, 63)
                    + ectx.qp_bd_offset,
                (qp_cb_cr
                    + sh.pps.chroma_tool_offsets.joint_cbcr_qp_offset_value
                    + sh.joint_cbcr_qp_offset
                    + ectx.cu_qp_offset_cbcr as isize)
                    .clamp(-ectx.qp_bd_offset, 63)
                    + ectx.qp_bd_offset,
            )
        } else {
            (0, 0, 0)
        };
        let qps = (
            qp_y as usize,
            qp_cb as usize,
            qp_cr as usize,
            qp_cb_cr as usize,
        );
        tu.derive_qp_cache = Some(qps);
        qps
    }

    pub fn derive_ls(
        &mut self,
        tu: &mut TransformUnit,
        c_idx: usize,
        ls: &mut Vec2d<i32>,
        m: &Vec2d<i32>,
        scale: i32,
        shift: usize,
    ) {
        let (tw, th) = tu.get_component_size(c_idx);
        if is_x86_feature_detected!("avx2") {
            use core::arch::x86_64::*;
            match tw {
                4 => {
                    for y in 0..th {
                        let lsy = &mut ls[y];
                        let my = &m[y];
                        for x in 0..4 {
                            lsy[x] = (my[x] * scale) << shift;
                        }
                    }
                }
                8 => unsafe {
                    for y in 0..th {
                        let lsy = &mut ls[y];
                        let my = &m[y];
                        let my = _mm256_lddqu_si256(my.as_ptr() as *const _);
                        let scale = _mm256_set1_epi32(scale);
                        let m = _mm256_mullo_epi32(my, scale);
                        let shift = _mm_set_epi64x(0, shift as i64);
                        let m = _mm256_sll_epi32(m, shift);
                        _mm256_storeu_si256(lsy.as_mut_ptr() as *mut _, m);
                    }
                },
                16 => unsafe {
                    for y in 0..th {
                        let lsy = &mut ls[y];
                        let my = &m[y];
                        let myv = _mm256_lddqu_si256(my.as_ptr() as *const _);
                        let scale = _mm256_set1_epi32(scale);
                        let m = _mm256_mullo_epi32(myv, scale);
                        let shift = _mm_set_epi64x(0, shift as i64);
                        let m = _mm256_sll_epi32(m, shift);
                        _mm256_storeu_si256(lsy.as_mut_ptr() as *mut _, m);

                        let myv = _mm256_lddqu_si256(my[8..].as_ptr() as *const _);
                        let m = _mm256_mullo_epi32(myv, scale);
                        let m = _mm256_sll_epi32(m, shift);
                        _mm256_storeu_si256(lsy[8..].as_mut_ptr() as *mut _, m);
                    }
                },
                32 => unsafe {
                    for y in 0..th {
                        let lsy = &mut ls[y];
                        let my = &m[y];
                        let myv = _mm256_lddqu_si256(my.as_ptr() as *const _);
                        let scale = _mm256_set1_epi32(scale);
                        let m = _mm256_mullo_epi32(myv, scale);
                        let shift = _mm_set_epi64x(0, shift as i64);
                        let m = _mm256_sll_epi32(m, shift);
                        _mm256_storeu_si256(lsy.as_mut_ptr() as *mut _, m);

                        for offset in (8..32).step_by(8) {
                            let myv = _mm256_lddqu_si256(my[offset..].as_ptr() as *const _);
                            let m = _mm256_mullo_epi32(myv, scale);
                            let m = _mm256_sll_epi32(m, shift);
                            _mm256_storeu_si256(lsy[offset..].as_mut_ptr() as *mut _, m);
                        }
                    }
                },
                _ => unsafe {
                    for y in 0..th {
                        let lsy = &mut ls[y];
                        let my = &m[y];
                        let myv = _mm256_lddqu_si256(my.as_ptr() as *const _);
                        let scale = _mm256_set1_epi32(scale);
                        let m = _mm256_mullo_epi32(myv, scale);
                        let shift = _mm_set_epi64x(0, shift as i64);
                        let m = _mm256_sll_epi32(m, shift);
                        _mm256_storeu_si256(lsy.as_mut_ptr() as *mut _, m);

                        for offset in (8..64).step_by(8) {
                            let myv = _mm256_lddqu_si256(my[offset..].as_ptr() as *const _);
                            let m = _mm256_mullo_epi32(myv, scale);
                            let m = _mm256_sll_epi32(m, shift);
                            _mm256_storeu_si256(lsy[offset..].as_mut_ptr() as *mut _, m);
                        }
                    }
                },
            }
        } else {
            for y in 0..th {
                let lsy = &mut ls[y];
                let my = &m[y];
                for x in 0..tw {
                    lsy[x] = (my[x] * scale) << shift;
                }
            }
        }
    }

    pub fn search_dq(
        &self,
        t: &Vec2d<i16>,
        log2_sb_w: usize,
        log2_sb_h: usize,
        num_sb_coeff: usize,
        q_state: usize,
        sb_order: &Vec<(usize, usize)>,
        coeff_order: &Vec<(usize, usize)>,
        last_scan_pos: usize,
        last_sub_block: usize,
        ls: &Vec2d<i32>,
        bd_shift: usize,
        bd_offset: i32,
        depth: usize,
        qp: usize,
        lambda: f32,
        delta: f32,
        is_trailing_zeros: bool,
        trellis_table: &mut Vec2d<[(usize, i16, f32); 4]>,
        ectx: &EncoderContext,
    ) -> (usize, i16, f32) {
        if trellis_table[last_sub_block][last_scan_pos][q_state].2 != f32::MIN {
            return trellis_table[last_sub_block][last_scan_pos][q_state];
        }
        let (x_s, y_s) = sb_order[last_sub_block];
        let (x_0, y_0) = (x_s << log2_sb_w, y_s << log2_sb_h);
        let x_c = x_0 + coeff_order[last_scan_pos].0;
        let y_c = y_0 + coeff_order[last_scan_pos].1;
        let table = &self.dq_table[qp];
        let (a, q, cost) = if depth == 0 || (last_scan_pos == 0 && last_sub_block == 0) {
            if t[y_c][x_c] == 0 {
                let cost = if is_trailing_zeros {
                    0.0
                } else {
                    self.get_dq_cost(qp, 0, lambda, delta)
                };
                (0, 0, cost)
            } else if q_state > 1 {
                if t[y_c][x_c] > 0 {
                    let s = ((t[y_c][x_c] as i32) << bd_shift) - bd_offset;
                    if s * 2 < ls[y_c][x_c] {
                        let d = t[y_c][x_c].abs() as f32;
                        let cost = if is_trailing_zeros {
                            d
                        } else {
                            d + self.get_dq_cost(qp, 0, lambda, delta)
                        };
                        (0, 0, cost)
                    } else {
                        let a = 1 + (s / ls[y_c][x_c] / 2) as usize;
                        let q = (2 * a - 1) as i16;
                        let dq = (q as i32 * ls[y_c][x_c] + bd_offset) >> bd_shift;
                        let d = (t[y_c][x_c] as i32 - dq).abs() as f32;
                        let cost = d + self.get_dq_cost(qp, a, lambda, delta);
                        (a, q, cost)
                    }
                } else {
                    let s = ((t[y_c][x_c] as i32) << bd_shift) - bd_offset;
                    let s = -s;
                    if s * 2 < ls[y_c][x_c] {
                        let d = t[y_c][x_c].abs() as f32;
                        let cost = if is_trailing_zeros {
                            d
                        } else {
                            d + self.get_dq_cost(qp, 0, lambda, delta)
                        };
                        (0, 0, cost)
                    } else {
                        let a = 1 + (s / ls[y_c][x_c] / 2) as usize;
                        let q = -((2 * a - 1) as i16);
                        let dq = (q as i32 * ls[y_c][x_c] + bd_offset) >> bd_shift;
                        let d = (t[y_c][x_c] as i32 - dq).abs() as f32;
                        let cost = d + self.get_dq_cost(qp, a, lambda, delta);
                        (a, q, cost)
                    }
                }
            } else if t[y_c][x_c] > 0 {
                let s = ((t[y_c][x_c] as i32) << bd_shift) - bd_offset;
                let a0 = (s / ls[y_c][x_c] / 2) as usize;
                let q0 = (2 * a0) as i16;
                let dq0 = (q0 as i32 * ls[y_c][x_c] + bd_offset) >> bd_shift;
                let d0 = (t[y_c][x_c] as i32 - dq0).abs() as f32;
                let cost0 = if a0 == 0 && is_trailing_zeros {
                    d0
                } else {
                    d0 + self.get_dq_cost(qp, a0, lambda, delta)
                };
                let a1 = 1 + (s / ls[y_c][x_c] / 2) as usize;
                let q1 = (2 * a1) as i16;
                let dq1 = (q1 as i32 * ls[y_c][x_c] + bd_offset) >> bd_shift;
                let d1 = (t[y_c][x_c] as i32 - dq1).abs() as f32;
                let cost1 = d1 + self.get_dq_cost(qp, a1, lambda, delta);
                if cost0 <= cost1 {
                    (a0, q0, cost0)
                } else {
                    (a1, q1, cost1)
                }
            } else {
                let s = ((t[y_c][x_c] as i32) << bd_shift) - bd_offset;
                let s = -s;
                let a0 = (s / ls[y_c][x_c] / 2) as usize;
                let q0 = -(2 * a0 as isize) as i16;
                let dq0 = (q0 as i32 * ls[y_c][x_c] + bd_offset) >> bd_shift;
                let d0 = (t[y_c][x_c] as i32 - dq0).abs() as f32;
                let cost0 = if a0 == 0 && is_trailing_zeros {
                    d0
                } else {
                    d0 + self.get_dq_cost(qp, a0, lambda, delta)
                };
                let a1 = 1 + (s / ls[y_c][x_c] / 2) as usize;
                let q1 = -(2 * a1 as isize) as i16;
                let dq1 = (q1 as i32 * ls[y_c][x_c] + bd_offset) >> bd_shift;
                let d1 = (t[y_c][x_c] as i32 - dq1).abs() as f32;
                let cost1 = d1 + self.get_dq_cost(qp, a1, lambda, delta);
                if cost0 <= cost1 {
                    (a0, q0, cost0)
                } else {
                    (a1, q1, cost1)
                }
            }
        } else {
            let (next_scan_pos, next_sub_block) = if last_scan_pos == 0 {
                (num_sb_coeff - 1, last_sub_block - 1)
            } else {
                (last_scan_pos - 1, last_sub_block)
            };
            if t[y_c][x_c] == 0 {
                let nq_state = ectx.q_state_trans_table[q_state][0];
                let (_, _, cost) = self.search_dq(
                    t,
                    log2_sb_w,
                    log2_sb_h,
                    num_sb_coeff,
                    nq_state,
                    sb_order,
                    coeff_order,
                    next_scan_pos,
                    next_sub_block,
                    ls,
                    bd_shift,
                    bd_offset,
                    depth - 1,
                    qp,
                    lambda,
                    delta,
                    is_trailing_zeros,
                    trellis_table,
                    ectx,
                );
                let cost = cost + table[0];
                (0, 0, 0.0 + cost)
            } else if q_state > 1 {
                if t[y_c][x_c] > 0 {
                    let s = ((t[y_c][x_c] as i32) << bd_shift) - bd_offset;
                    if s < ls[y_c][x_c] {
                        let nq_state0 = ectx.q_state_trans_table[q_state][0];
                        let a0 = 0;
                        let q0 = 0;
                        let dq0 = (q0 * ls[y_c][x_c] + bd_offset) >> bd_shift;
                        let d0 = (t[y_c][x_c] as i32 - dq0).abs() as f32;
                        let cost0 = if is_trailing_zeros {
                            d0
                        } else {
                            d0 + self.get_dq_cost(qp, a0, lambda, delta)
                        };
                        let (_, _, ncost0) = self.search_dq(
                            t,
                            log2_sb_w,
                            log2_sb_h,
                            num_sb_coeff,
                            nq_state0,
                            sb_order,
                            coeff_order,
                            next_scan_pos,
                            next_sub_block,
                            ls,
                            bd_shift,
                            bd_offset,
                            depth - 1,
                            qp,
                            lambda,
                            delta,
                            is_trailing_zeros,
                            trellis_table,
                            ectx,
                        );
                        let nq_state1 = ectx.q_state_trans_table[q_state][1];
                        let a1 = 1;
                        let q1 = 2 * a1 - 1;
                        let dq1 = (q1 as i32 * ls[y_c][x_c] + bd_offset) >> bd_shift;
                        let d1 = (t[y_c][x_c] as i32 - dq1).abs() as f32;
                        let cost1 = d1 + self.get_dq_cost(qp, a1, lambda, delta);
                        let (_, _, ncost1) = self.search_dq(
                            t,
                            log2_sb_w,
                            log2_sb_h,
                            num_sb_coeff,
                            nq_state1,
                            sb_order,
                            coeff_order,
                            next_scan_pos,
                            next_sub_block,
                            ls,
                            bd_shift,
                            bd_offset,
                            depth - 1,
                            qp,
                            lambda,
                            delta,
                            false,
                            trellis_table,
                            ectx,
                        );
                        if cost0 + ncost0 <= cost1 + ncost1 {
                            (a0, q0 as i16, cost0 + ncost0)
                        } else {
                            (a1, q1 as i16, cost1 + ncost1)
                        }
                    } else {
                        let a0 = ((s + ls[y_c][x_c]) / ls[y_c][x_c] / 2) as usize;
                        let nq_state0 = ectx.q_state_trans_table[q_state][a0 & 1];
                        let q0 = 2 * a0 - 1;
                        let dq0 = (q0 as i32 * ls[y_c][x_c] + bd_offset) >> bd_shift;
                        let d0 = (t[y_c][x_c] as i32 - dq0).abs() as f32;
                        let cost0 = if a0 == 0 && is_trailing_zeros {
                            d0
                        } else {
                            d0 + self.get_dq_cost(qp, a0, lambda, delta)
                        };
                        let (_, _, ncost0) = self.search_dq(
                            t,
                            log2_sb_w,
                            log2_sb_h,
                            num_sb_coeff,
                            nq_state0,
                            sb_order,
                            coeff_order,
                            next_scan_pos,
                            next_sub_block,
                            ls,
                            bd_shift,
                            bd_offset,
                            depth - 1,
                            qp,
                            lambda,
                            delta,
                            is_trailing_zeros && a0 == 0,
                            trellis_table,
                            ectx,
                        );
                        let a1 = 1 + ((s + ls[y_c][x_c]) / ls[y_c][x_c] / 2) as usize;
                        let nq_state1 = ectx.q_state_trans_table[q_state][a1 & 1];
                        let q1 = 2 * a1 - 1;
                        let dq1 = (q1 as i32 * ls[y_c][x_c] + bd_offset) >> bd_shift;
                        let d1 = (t[y_c][x_c] as i32 - dq1).abs() as f32;
                        let cost1 = d1 + self.get_dq_cost(qp, a1, lambda, delta);
                        let (_, _, ncost1) = self.search_dq(
                            t,
                            log2_sb_w,
                            log2_sb_h,
                            num_sb_coeff,
                            nq_state1,
                            sb_order,
                            coeff_order,
                            next_scan_pos,
                            next_sub_block,
                            ls,
                            bd_shift,
                            bd_offset,
                            depth - 1,
                            qp,
                            lambda,
                            delta,
                            false,
                            trellis_table,
                            ectx,
                        );
                        if cost0 + ncost0 <= cost1 + ncost1 {
                            (a0, q0 as i16, cost0 + ncost0)
                        } else {
                            (a1, q1 as i16, cost1 + ncost1)
                        }
                    }
                } else {
                    let s = ((t[y_c][x_c] as i32) << bd_shift) - bd_offset;
                    let s = -s;
                    if s < ls[y_c][x_c] {
                        let nq_state0 = ectx.q_state_trans_table[q_state][0];
                        let a0 = 0;
                        let q0 = 0;
                        let dq0 = (q0 * ls[y_c][x_c] + bd_offset) >> bd_shift;
                        let d0 = (t[y_c][x_c] as i32 - dq0).abs() as f32;
                        let cost0 = if is_trailing_zeros {
                            d0
                        } else {
                            d0 + self.get_dq_cost(qp, a0, lambda, delta)
                        };
                        let (_, _, ncost0) = self.search_dq(
                            t,
                            log2_sb_w,
                            log2_sb_h,
                            num_sb_coeff,
                            nq_state0,
                            sb_order,
                            coeff_order,
                            next_scan_pos,
                            next_sub_block,
                            ls,
                            bd_shift,
                            bd_offset,
                            depth - 1,
                            qp,
                            lambda,
                            delta,
                            is_trailing_zeros,
                            trellis_table,
                            ectx,
                        );
                        let nq_state1 = ectx.q_state_trans_table[q_state][1];
                        let a1 = 1;
                        let q1 = -(2 * a1 as isize - 1);
                        let dq1 = (q1 as i32 * ls[y_c][x_c] + bd_offset) >> bd_shift;
                        let d1 = (t[y_c][x_c] as i32 - dq1).abs() as f32;
                        let cost1 = d1 + self.get_dq_cost(qp, a1, lambda, delta);
                        let (_, _, ncost1) = self.search_dq(
                            t,
                            log2_sb_w,
                            log2_sb_h,
                            num_sb_coeff,
                            nq_state1,
                            sb_order,
                            coeff_order,
                            next_scan_pos,
                            next_sub_block,
                            ls,
                            bd_shift,
                            bd_offset,
                            depth - 1,
                            qp,
                            lambda,
                            delta,
                            false,
                            trellis_table,
                            ectx,
                        );
                        if cost0 + ncost0 <= cost1 + ncost1 {
                            (a0, q0 as i16, cost0 + ncost0)
                        } else {
                            (a1, q1 as i16, cost1 + ncost1)
                        }
                    } else {
                        let a0 = ((s + ls[y_c][x_c]) / ls[y_c][x_c] / 2) as usize;
                        let nq_state0 = ectx.q_state_trans_table[q_state][a0 & 1];
                        let q0 = -(2 * a0 as isize - 1);
                        let dq0 = (q0 as i32 * ls[y_c][x_c] + bd_offset) >> bd_shift;
                        let d0 = (t[y_c][x_c] as i32 - dq0).abs() as f32;
                        let cost0 = if a0 == 0 && is_trailing_zeros {
                            d0
                        } else {
                            d0 + self.get_dq_cost(qp, a0, lambda, delta)
                        };
                        let (_, _, ncost0) = self.search_dq(
                            t,
                            log2_sb_w,
                            log2_sb_h,
                            num_sb_coeff,
                            nq_state0,
                            sb_order,
                            coeff_order,
                            next_scan_pos,
                            next_sub_block,
                            ls,
                            bd_shift,
                            bd_offset,
                            depth - 1,
                            qp,
                            lambda,
                            delta,
                            is_trailing_zeros && a0 == 0,
                            trellis_table,
                            ectx,
                        );
                        let a1 = 1 + ((s + ls[y_c][x_c]) / ls[y_c][x_c] / 2) as usize;
                        let nq_state1 = ectx.q_state_trans_table[q_state][a1 & 1];
                        let q1 = -(2 * a1 as isize - 1);
                        let dq1 = (q1 as i32 * ls[y_c][x_c] + bd_offset) >> bd_shift;
                        let d1 = (t[y_c][x_c] as i32 - dq1).abs() as f32;
                        let cost1 = d1 + self.get_dq_cost(qp, a1, lambda, delta);
                        let (_, _, ncost1) = self.search_dq(
                            t,
                            log2_sb_w,
                            log2_sb_h,
                            num_sb_coeff,
                            nq_state1,
                            sb_order,
                            coeff_order,
                            next_scan_pos,
                            next_sub_block,
                            ls,
                            bd_shift,
                            bd_offset,
                            depth - 1,
                            qp,
                            lambda,
                            delta,
                            false,
                            trellis_table,
                            ectx,
                        );
                        if cost0 + ncost0 <= cost1 + ncost1 {
                            (a0, q0 as i16, cost0 + ncost0)
                        } else {
                            (a1, q1 as i16, cost1 + ncost1)
                        }
                    }
                }
            } else {
                let s = ((t[y_c][x_c] as i32) << bd_shift) - bd_offset;
                if t[y_c][x_c] > 0 {
                    let a0 = (s / ls[y_c][x_c] / 2) as usize;
                    let nq_state0 = ectx.q_state_trans_table[q_state][a0 & 1];
                    let q0 = 2 * a0;
                    let dq0 = (q0 as i32 * ls[y_c][x_c] + bd_offset) >> bd_shift;
                    let d0 = (t[y_c][x_c] as i32 - dq0).abs() as f32;
                    let cost0 = if a0 == 0 && is_trailing_zeros {
                        d0
                    } else {
                        d0 + self.get_dq_cost(qp, a0, lambda, delta)
                    };
                    let (_, _, ncost0) = self.search_dq(
                        t,
                        log2_sb_w,
                        log2_sb_h,
                        num_sb_coeff,
                        nq_state0,
                        sb_order,
                        coeff_order,
                        next_scan_pos,
                        next_sub_block,
                        ls,
                        bd_shift,
                        bd_offset,
                        depth - 1,
                        qp,
                        lambda,
                        delta,
                        is_trailing_zeros && a0 == 0,
                        trellis_table,
                        ectx,
                    );
                    let a1 = 1 + (s / ls[y_c][x_c] / 2) as usize;
                    let nq_state1 = ectx.q_state_trans_table[q_state][a1 & 1];
                    let q1 = 2 * a1;
                    let dq1 = (q1 as i32 * ls[y_c][x_c] + bd_offset) >> bd_shift;
                    let d1 = (t[y_c][x_c] as i32 - dq1).abs() as f32;
                    let cost1 = d1 + self.get_dq_cost(qp, a1, lambda, delta);
                    let (_, _, ncost1) = self.search_dq(
                        t,
                        log2_sb_w,
                        log2_sb_h,
                        num_sb_coeff,
                        nq_state1,
                        sb_order,
                        coeff_order,
                        next_scan_pos,
                        next_sub_block,
                        ls,
                        bd_shift,
                        bd_offset,
                        depth - 1,
                        qp,
                        lambda,
                        delta,
                        false,
                        trellis_table,
                        ectx,
                    );
                    if cost0 + ncost0 <= cost1 + ncost1 {
                        (a0, q0 as i16, cost0 + ncost0)
                    } else {
                        (a1, q1 as i16, cost1 + ncost1)
                    }
                } else {
                    let s = -s;
                    let a0 = (s / ls[y_c][x_c] / 2) as usize;
                    let nq_state0 = ectx.q_state_trans_table[q_state][a0 & 1];
                    let q0 = -(2 * a0 as isize);
                    let dq0 = (q0 as i32 * ls[y_c][x_c] + bd_offset) >> bd_shift;
                    let d0 = (t[y_c][x_c] as i32 - dq0).abs() as f32;
                    let cost0 = if a0 == 0 && is_trailing_zeros {
                        d0
                    } else {
                        d0 + self.get_dq_cost(qp, a0, lambda, delta)
                    };
                    let (_, _, ncost0) = self.search_dq(
                        t,
                        log2_sb_w,
                        log2_sb_h,
                        num_sb_coeff,
                        nq_state0,
                        sb_order,
                        coeff_order,
                        next_scan_pos,
                        next_sub_block,
                        ls,
                        bd_shift,
                        bd_offset,
                        depth - 1,
                        qp,
                        lambda,
                        delta,
                        is_trailing_zeros && a0 == 0,
                        trellis_table,
                        ectx,
                    );
                    let a1 = 1 + (s / ls[y_c][x_c] / 2) as usize;
                    let nq_state1 = ectx.q_state_trans_table[q_state][a1 & 1];
                    let q1 = -(2 * a1 as isize);
                    let dq1 = (q1 as i32 * ls[y_c][x_c] + bd_offset) >> bd_shift;
                    let d1 = (t[y_c][x_c] as i32 - dq1).abs() as f32;
                    let cost1 = d1 + self.get_dq_cost(qp, a1, lambda, delta);
                    let (_, _, ncost1) = self.search_dq(
                        t,
                        log2_sb_w,
                        log2_sb_h,
                        num_sb_coeff,
                        nq_state1,
                        sb_order,
                        coeff_order,
                        next_scan_pos,
                        next_sub_block,
                        ls,
                        bd_shift,
                        bd_offset,
                        depth - 1,
                        qp,
                        lambda,
                        delta,
                        false,
                        trellis_table,
                        ectx,
                    );
                    if cost0 + ncost0 <= cost1 + ncost1 {
                        (a0, q0 as i16, cost0 + ncost0)
                    } else {
                        (a1, q1 as i16, cost1 + ncost1)
                    }
                }
            }
        };
        trellis_table[last_sub_block][last_scan_pos][q_state] = (a, q, cost);
        (a, q, cost)
    }

    pub fn quantize(
        &mut self,
        tu: &mut TransformUnit,
        c_idx: usize,
        sh: &SliceHeader,
        ectx: &EncoderContext,
    ) {
        let (log2_tw, log2_th) = tu.get_log2_tb_size(c_idx);
        let (tw, th) = (1 << log2_tw, 1 << log2_th);

        let (mut bd_shift, mut bd_offset) = if let Some(bd_shift) = tu.bd_shift_cache[c_idx] {
            (bd_shift, (1 << bd_shift) >> 1)
        } else {
            (0, 0)
        };

        let bdpcm_flag = tu.cu_bdpcm_flag[c_idx];
        let ls = if let Some(ls) = &tu.ls_cache[c_idx] {
            ls.clone()
        } else {
            // derivation process for quantization parameters (8.7.1)
            let qp_prime = self.derive_qp(tu, sh, ectx);

            let tu_c_res_mode = Transformer::get_tu_c_res_mode(tu);

            // scaling process for transform cofficients (8.7.3)
            let (qp_act_offset, mut qp) = if c_idx == 0 {
                (if tu.cu_act_enabled_flag { -5 } else { 0 }, qp_prime.0)
            } else if tu_c_res_mode == 2 {
                (tu.cu_act_enabled_flag as isize, qp_prime.3)
            } else if c_idx == 1 {
                (tu.cu_act_enabled_flag as isize, qp_prime.1)
            } else {
                (if tu.cu_act_enabled_flag { 3 } else { 0 }, qp_prime.2)
            };
            let rect_non_ts_flag = if !tu.transform_skip_flag[c_idx] {
                qp = (qp as isize + qp_act_offset).clamp(0, 63 + ectx.qp_bd_offset) as usize;
                let rect_non_ts_flag = (tu.log2_tb_width + tu.log2_tb_height) & 1;
                bd_shift = ectx.bit_depth + rect_non_ts_flag + (log2_tw + log2_th) / 2 - 5
                    + sh.dep_quant_used_flag as usize;
                rect_non_ts_flag
            } else {
                qp = (qp as isize + qp_act_offset)
                    .clamp(ectx.qp_prime_ts_min as isize, 63 + ectx.qp_bd_offset)
                    as usize;
                bd_shift = 10;
                0
            };
            tu.bd_shift_cache[c_idx] = Some(bd_shift);
            bd_offset = (1 << bd_shift) >> 1;

            let mut m = vec2d![16; th; tw];
            // FIXME cache?
            if !sh.explicit_scaling_list_used_flag
                || tu.transform_skip_flag[c_idx]
                || (sh.sps.scaling_matrix_for_lfnst_disabled_flag && ectx.apply_lfnst_flag[c_idx])
                || (sh
                    .sps
                    .scaling_matrix_for_alternative_colour_space_disabled_flag
                    && sh.sps.scaling_matrix_designated_colour_space_flag == tu.cu_act_enabled_flag)
            {
                //for y in 0..th {
                //m[y][0..tw].fill(16);
                //}
            } else {
                let cu = tu.get_cu();
                let cu = cu.lock().unwrap();
                let pred_mode = cu.pred_mode[(c_idx > 0) as usize];
                let max_tb_size = tu.width.max(th);
                let id = Self::get_scaling_matrix_id(pred_mode, c_idx, max_tb_size);
                let log2_matrix_size = if id < 2 {
                    1
                } else if id < 8 {
                    2
                } else {
                    3
                };
                {
                    let matrix = &ectx.scaling_matrix_rec[id];
                    let x_shift = log2_tw - log2_matrix_size;
                    let y_shift = log2_th - log2_matrix_size;
                    for y in 0..th {
                        let my = &mut m[y];
                        for (x, myx) in my.iter_mut().enumerate().take(tw) {
                            let i = x >> x_shift;
                            let j = y >> y_shift;
                            *myx = matrix[i][j];
                            panic!();
                        }
                    }
                }
                if id > 13 {
                    m[0][0] = ectx.scaling_matrix_dc_rec[id - 14];
                    panic!();
                }
            }

            let ls: Vec2d<i32> = if sh.dep_quant_used_flag && !tu.transform_skip_flag[c_idx] {
                let scale = LEVEL_SCALE[rect_non_ts_flag][(qp + 1) % 6];
                let shift = (qp + 1) / 6;
                let mut ls = vec2d![0i32; th; tw];
                self.derive_ls(tu, c_idx, &mut ls, &m, scale, shift);
                ls
            } else {
                let mut ls = vec2d![0i32; th; tw];
                let scale = LEVEL_SCALE[rect_non_ts_flag][qp % 6];
                let shift = qp / 6;
                self.derive_ls(tu, c_idx, &mut ls, &m, scale, shift);
                ls
            };
            tu.ls_cache[c_idx] = Some(ls.clone());
            ls
        };
        if sh.dep_quant_used_flag {
            let mut q_state = 0;
            let (log2_tb_width, log2_tb_height) = tu.get_log2_tb_size(c_idx);
            let (log2_sb_w, log2_sb_h) = tu.get_log2_sb_size(c_idx);
            let (tw, th) = tu.get_component_size(c_idx);
            let num_sb_coeff = 1 << (log2_sb_w + log2_sb_h);
            let mut last_scan_pos = num_sb_coeff;
            let mut last_sub_block =
                (1 << (log2_tb_width + log2_tb_height - (log2_sb_w + log2_sb_h))) - 1;
            let mut trellis_table = vec2d![[(0, 0, f32::MIN); 4];last_sub_block+1 ; num_sb_coeff];
            let coeff_order = &DIAG_SCAN_ORDER[log2_sb_h][log2_sb_w];
            let sb_order = &DIAG_SCAN_ORDER[log2_tb_height - log2_sb_h][log2_tb_width - log2_sb_w];
            let q = &mut tu.quantized_transformed_coeffs[c_idx];
            let t = &tu.transformed_coeffs[c_idx];
            let (mut x_s, mut y_s) = sb_order[last_sub_block];
            let (mut x_0, mut y_0) = (x_s << log2_sb_w, y_s << log2_sb_h);
            let mut is_not_first_sub_block = last_sub_block > 0;
            let q0 = match ectx.extra_params.get("q0") {
                Some(q0) => q0.parse::<f32>().unwrap(),
                _ => 5.479_013_4,
            };
            let q1 = match ectx.extra_params.get("q1") {
                Some(q1) => q1.parse::<f32>().unwrap(),
                _ => 1.120_687_8,
            };
            let q2 = match ectx.extra_params.get("q2") {
                Some(q2) => q2.parse::<f32>().unwrap(),
                _ => 2.531_597_4,
            };
            let lambda = (2.0f32).powf(tu.qp as f32 / q0) * q1;
            let delta = q2;
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
                let (na, nq, _ncost) = self.search_dq(
                    t,
                    log2_sb_w,
                    log2_sb_h,
                    num_sb_coeff,
                    q_state,
                    sb_order,
                    coeff_order,
                    last_scan_pos,
                    last_sub_block,
                    &ls,
                    bd_shift,
                    bd_offset,
                    tw * th,
                    tu.qp,
                    lambda,
                    delta,
                    is_trailing_zeros,
                    &mut trellis_table,
                    ectx,
                );
                is_trailing_zeros &= na == 0;
                q[y_c][x_c] = nq;
                q_state = ectx.q_state_trans_table[q_state][na & 1];
                last_scan_pos > 0 || is_not_first_sub_block
            } {}
        } else {
            for y in 0..th {
                let t = &tu.transformed_coeffs[c_idx][y];
                let q = &mut tu.quantized_transformed_coeffs[c_idx][y];
                let l = &ls[y];
                for x in 0..tw {
                    let tq = ((t[x] as i32) << bd_shift) - bd_offset;
                    if tq >= 0 {
                        q[x] = ((tq + l[x] / 2) / l[x]) as i16;
                    } else {
                        q[x] = (-((-tq + l[x] / 2) / l[x])) as i16;
                    }
                }
            }
        }
        // TODO rd sensitive quantization
        if bdpcm_flag {
            // TODO optimize if statement
            let cu = tu.get_cu();
            let cu = cu.lock().unwrap();
            let bdpcm_dir = if c_idx == 0 {
                cu.intra_bdpcm_luma_dir_flag
            } else {
                cu.intra_bdpcm_chroma_dir_flag
            };
            for y in 0..th {
                for x in 0..tw {
                    if !bdpcm_dir && x > 0 {
                        tu.quantized_transformed_coeffs[c_idx][y][x] -=
                            tu.quantized_transformed_coeffs[c_idx][y][x - 1];
                    } else if bdpcm_dir && y > 0 {
                        tu.quantized_transformed_coeffs[c_idx][y][x] -=
                            tu.quantized_transformed_coeffs[c_idx][y - 1][x];
                    }
                }
            }
        }
    }

    pub fn dequantize(
        &mut self,
        tu: &mut TransformUnit,
        c_idx: usize,
        sh: &SliceHeader,
        ectx: &EncoderContext,
    ) {
        let (log2_tw, log2_th) = tu.get_log2_tb_size(c_idx);
        let (tw, th) = (1 << log2_tw, 1 << log2_th);

        let (mut bd_shift, mut bd_offset) = if let Some(bd_shift) = tu.bd_shift_cache[c_idx] {
            (bd_shift, (1 << bd_shift) >> 1)
        } else {
            (0, 0)
        };
        let ls = if let Some(ls) = &tu.ls_cache[c_idx] {
            ls.clone()
        } else {
            // derivation process for quantization parameters (8.7.1)
            let qp_prime = self.derive_qp(tu, sh, ectx);

            // scaling and transformation process (8.7.2)
            let tu_c_res_mode = Transformer::get_tu_c_res_mode(tu);

            // scaling process for transform cofficients (8.7.3)
            let (qp_act_offset, mut qp) = if c_idx == 0 {
                (if tu.cu_act_enabled_flag { -5 } else { 0 }, qp_prime.0)
            } else if tu_c_res_mode == 2 {
                (tu.cu_act_enabled_flag as isize, qp_prime.3)
            } else if c_idx == 1 {
                (tu.cu_act_enabled_flag as isize, qp_prime.1)
            } else {
                (if tu.cu_act_enabled_flag { 3 } else { 0 }, qp_prime.2)
            };
            let rect_non_ts_flag = if !tu.transform_skip_flag[c_idx] {
                qp = (qp as isize + qp_act_offset).clamp(0, 63 + ectx.qp_bd_offset) as usize;
                let rect_non_ts_flag = (log2_tw + log2_th) & 1;
                bd_shift = ectx.bit_depth + rect_non_ts_flag + (log2_tw + log2_th) / 2 - 5
                    + sh.dep_quant_used_flag as usize;
                rect_non_ts_flag
            } else {
                qp = (qp as isize + qp_act_offset)
                    .clamp(ectx.qp_prime_ts_min as isize, 63 + ectx.qp_bd_offset)
                    as usize;
                bd_shift = 10;
                0
            };
            bd_offset = (1 << bd_shift) >> 1;

            let mut m = vec2d![16; th; tw];
            if !sh.explicit_scaling_list_used_flag
                || tu.transform_skip_flag[c_idx]
                || (sh.sps.scaling_matrix_for_lfnst_disabled_flag && ectx.apply_lfnst_flag[c_idx])
                || (sh
                    .sps
                    .scaling_matrix_for_alternative_colour_space_disabled_flag
                    && sh.sps.scaling_matrix_designated_colour_space_flag == tu.cu_act_enabled_flag)
            {
                //for y in 0..th {
                //m[y][0..tw].fill(16);
                //}
            } else {
                let cu = tu.get_cu();
                let cu = cu.lock().unwrap();
                let pred_mode = cu.pred_mode[(c_idx > 0) as usize];
                let max_tb_size = tw.max(th);
                let id = Self::get_scaling_matrix_id(pred_mode, c_idx, max_tb_size);
                let log2_matrix_size = if id < 2 {
                    1
                } else if id < 8 {
                    2
                } else {
                    3
                };
                let x_shift = log2_tw - log2_matrix_size;
                let y_shift = log2_th - log2_matrix_size;
                let matrix = &ectx.scaling_matrix_rec[id];
                for y in 0..th {
                    let my = &mut m[y];
                    for (x, myx) in my.iter_mut().enumerate().take(tw) {
                        let i = x >> x_shift;
                        let j = y >> y_shift;
                        *myx = matrix[i][j];
                    }
                }
                if id > 13 {
                    m[0][0] = ectx.scaling_matrix_dc_rec[id - 14];
                }
            }

            let mut ls: Vec2d<i32> = vec2d![0; th; tw];
            if sh.dep_quant_used_flag && !tu.transform_skip_flag[c_idx] {
                let scale = LEVEL_SCALE[rect_non_ts_flag][(qp + 1) % 6];
                let shift = (qp + 1) / 6;
                self.derive_ls(tu, c_idx, &mut ls, &m, scale, shift);
            } else {
                let scale = LEVEL_SCALE[rect_non_ts_flag][qp % 6];
                let shift = qp / 6;
                self.derive_ls(tu, c_idx, &mut ls, &m, scale, shift);
            };
            tu.ls_cache[c_idx] = Some(ls.clone());
            ls
        };
        let bdpcm_flag = tu.cu_bdpcm_flag[c_idx];
        let coeff_min = i16::MIN;
        let coeff_max = i16::MAX;
        let mut dz = tu.quantized_transformed_coeffs[c_idx].clone();
        if bdpcm_flag {
            let cu = tu.get_cu();
            let cu = cu.lock().unwrap();
            let bdpcm_dir = if c_idx == 0 {
                cu.intra_bdpcm_luma_dir_flag
            } else {
                cu.intra_bdpcm_chroma_dir_flag
            };
            if bdpcm_dir {
                for y in 1..th {
                    for x in 0..tw {
                        dz[y][x] = (dz[y - 1][x] + dz[y][x]).clamp(coeff_min, coeff_max);
                    }
                }
            } else {
                for y in 0..th {
                    for x in 1..tw {
                        dz[y][x] = (dz[y][x - 1] + dz[y][x]).clamp(coeff_min, coeff_max);
                    }
                }
            }
        }

        let coeff_min = coeff_min as i32;
        let coeff_max = coeff_max as i32;

        //if is_x86_feature_detected!("avx2") && false {
        //use core::arch::x86_64::*;
        //match tw {
        //4 => {
        //for y in 0..th {
        //let d = &mut tu.dequantized_transformed_coeffs[c_idx][y];
        //let dzy = &dz[y];
        //let lsy = &ls[y];
        //for x in 0..4 {
        //d[x] = ((((dzy[x] as i32) * lsy[x] + bd_offset) >> bd_shift)
        //.clamp(coeff_min, coeff_max))
        //as i16;
        //}
        //}
        //}
        //8 => unsafe {
        //for y in 0..th {
        //let d = &mut tu.dequantized_transformed_coeffs[c_idx][y];
        //let dzy = &dz[y];
        //let lsy = &ls[y];
        //let dzyv = _mm_lddqu_si128(dzy.as_ptr() as *const _);
        //let dzyv = _mm256_cvtepi16_epi32(dzyv);
        //let lsyv = _mm256_lddqu_si256(lsy.as_ptr() as *const _);
        //let v = _mm256_mullo_epi32(dzyv, lsyv);
        //let bd_offset = _mm256_set1_epi32(bd_offset);
        //let v = _mm256_add_epi32(v, bd_offset);
        //let v = _mm256_srai_epi32(v, 6);
        //let lb = _mm256_set1_epi32(coeff_min);
        //let v = _mm256_max_epi32(v, lb);
        //let ub = _mm256_set1_epi32(coeff_max);
        //let v = _mm256_min_epi32(v, ub);
        //let shuffle = _mm256_set_epi8(
        //0, 0, 0, 0, 0, 0, 0, 0, 13, 12, 9, 8, 5, 4, 1, 0, 0, 0, 0, 0, 0, 0, 0,
        //0, 13, 12, 9, 8, 5, 4, 1, 0,
        //);
        //let v = _mm256_shuffle_epi8(v, shuffle);
        //let v0 = _mm256_extract_epi64(v, 0);
        //let v1 = _mm256_extract_epi64(v, 2);
        //*(d.as_mut_ptr() as *mut i64) = v0;
        //*(d[4..].as_mut_ptr() as *mut i64) = v1;
        //}
        //},
        //16 => unsafe {
        //for y in 0..th {
        //let d = &mut tu.dequantized_transformed_coeffs[c_idx][y];
        //let dzy = &dz[y];
        //let lsy = &ls[y];
        //let dzyv = _mm_lddqu_si128(dzy.as_ptr() as *const _);
        //let dzyv = _mm256_cvtepi16_epi32(dzyv);
        //let lsyv = _mm256_lddqu_si256(lsy.as_ptr() as *const _);
        //let v = _mm256_mullo_epi32(dzyv, lsyv);
        //let bd_offset = _mm256_set1_epi32(bd_offset);
        //let v = _mm256_add_epi32(v, bd_offset);
        //let v = _mm256_srai_epi32(v, 7);
        //let lb = _mm256_set1_epi32(coeff_min);
        //let v = _mm256_max_epi32(v, lb);
        //let ub = _mm256_set1_epi32(coeff_max);
        //let v = _mm256_min_epi32(v, ub);
        //let shuffle = _mm256_set_epi8(
        //0, 0, 0, 0, 0, 0, 0, 0, 13, 12, 9, 8, 5, 4, 1, 0, 0, 0, 0, 0, 0, 0, 0,
        //0, 13, 12, 9, 8, 5, 4, 1, 0,
        //);
        //let v = _mm256_shuffle_epi8(v, shuffle);
        //let v0 = _mm256_extract_epi64(v, 0);
        //let v1 = _mm256_extract_epi64(v, 2);
        //*(d.as_mut_ptr() as *mut i64) = v0;
        //*(d[4..].as_mut_ptr() as *mut i64) = v1;

        //let dzyv = _mm_lddqu_si128(dzy[8..].as_ptr() as *const _);
        //let dzyv = _mm256_cvtepi16_epi32(dzyv);
        //let lsyv = _mm256_lddqu_si256(lsy[8..].as_ptr() as *const _);
        //let v = _mm256_mullo_epi32(dzyv, lsyv);
        //let v = _mm256_add_epi32(v, bd_offset);
        //let v = _mm256_srai_epi32(v, 7);
        //let v = _mm256_max_epi32(v, lb);
        //let v = _mm256_min_epi32(v, ub);
        //let v = _mm256_shuffle_epi8(v, shuffle);
        //let v0 = _mm256_extract_epi64(v, 0);
        //let v1 = _mm256_extract_epi64(v, 2);
        //*(d[8..].as_mut_ptr() as *mut i64) = v0;
        //*(d[12..].as_mut_ptr() as *mut i64) = v1;
        //}
        //},
        //32 => unsafe {
        //for y in 0..th {
        //let d = &mut tu.dequantized_transformed_coeffs[c_idx][y];
        //let dzy = &dz[y];
        //let lsy = &ls[y];
        //let dzyv = _mm_lddqu_si128(dzy.as_ptr() as *const _);
        //let dzyv = _mm256_cvtepi16_epi32(dzyv);
        //let lsyv = _mm256_lddqu_si256(lsy.as_ptr() as *const _);
        //let v = _mm256_mullo_epi32(dzyv, lsyv);
        //let bd_offset = _mm256_set1_epi32(bd_offset);
        //let v = _mm256_add_epi32(v, bd_offset);
        //let v = _mm256_srai_epi32(v, 8);
        //let lb = _mm256_set1_epi32(coeff_min);
        //let v = _mm256_max_epi32(v, lb);
        //let ub = _mm256_set1_epi32(coeff_max);
        //let v = _mm256_min_epi32(v, ub);
        //let shuffle = _mm256_set_epi8(
        //0, 0, 0, 0, 0, 0, 0, 0, 13, 12, 9, 8, 5, 4, 1, 0, 0, 0, 0, 0, 0, 0, 0,
        //0, 13, 12, 9, 8, 5, 4, 1, 0,
        //);
        //let v = _mm256_shuffle_epi8(v, shuffle);
        //let v0 = _mm256_extract_epi64(v, 0);
        //let v1 = _mm256_extract_epi64(v, 2);
        //*(d.as_mut_ptr() as *mut i64) = v0;
        //*(d[4..].as_mut_ptr() as *mut i64) = v1;

        //for offset in (8..32).step_by(8) {
        //let dzyv = _mm_lddqu_si128(dzy[offset..].as_ptr() as *const _);
        //let dzyv = _mm256_cvtepi16_epi32(dzyv);
        //let lsyv = _mm256_lddqu_si256(lsy[offset..].as_ptr() as *const _);
        //let v = _mm256_mullo_epi32(dzyv, lsyv);
        //let v = _mm256_add_epi32(v, bd_offset);
        //let v = _mm256_srai_epi32(v, 8);
        //let v = _mm256_max_epi32(v, lb);
        //let v = _mm256_min_epi32(v, ub);
        //let v = _mm256_shuffle_epi8(v, shuffle);
        //let v0 = _mm256_extract_epi64(v, 0);
        //let v1 = _mm256_extract_epi64(v, 2);
        //*(d[offset..].as_mut_ptr() as *mut i64) = v0;
        //*(d[offset + 4..].as_mut_ptr() as *mut i64) = v1;
        //}
        //}
        //},
        //_ => unsafe {
        //for y in 0..th {
        //let d = &mut tu.dequantized_transformed_coeffs[c_idx][y];
        //let dzy = &dz[y];
        //let lsy = &ls[y];
        //let dzyv = _mm_lddqu_si128(dzy.as_ptr() as *const _);
        //let dzyv = _mm256_cvtepi16_epi32(dzyv);
        //let lsyv = _mm256_lddqu_si256(lsy.as_ptr() as *const _);
        //let v = _mm256_mullo_epi32(dzyv, lsyv);
        //let bd_offset = _mm256_set1_epi32(bd_offset);
        //let v = _mm256_add_epi32(v, bd_offset);
        //let v = _mm256_srai_epi32(v, 9);
        //let lb = _mm256_set1_epi32(coeff_min);
        //let v = _mm256_max_epi32(v, lb);
        //let ub = _mm256_set1_epi32(coeff_max);
        //let v = _mm256_min_epi32(v, ub);
        //let shuffle = _mm256_set_epi8(
        //0, 0, 0, 0, 0, 0, 0, 0, 13, 12, 9, 8, 5, 4, 1, 0, 0, 0, 0, 0, 0, 0, 0,
        //0, 13, 12, 9, 8, 5, 4, 1, 0,
        //);
        //let v = _mm256_shuffle_epi8(v, shuffle);
        //let v0 = _mm256_extract_epi64(v, 0);
        //let v1 = _mm256_extract_epi64(v, 2);
        //*(d.as_mut_ptr() as *mut i64) = v0;
        //*(d[4..].as_mut_ptr() as *mut i64) = v1;

        //for offset in (8..64).step_by(8) {
        //let dzyv = _mm_lddqu_si128(dzy[offset..].as_ptr() as *const _);
        //let dzyv = _mm256_cvtepi16_epi32(dzyv);
        //let lsyv = _mm256_lddqu_si256(lsy[offset..].as_ptr() as *const _);
        //let v = _mm256_mullo_epi32(dzyv, lsyv);
        //let v = _mm256_add_epi32(v, bd_offset);
        //let v = _mm256_srai_epi32(v, 9);
        //let v = _mm256_max_epi32(v, lb);
        //let v = _mm256_min_epi32(v, ub);
        //let v = _mm256_shuffle_epi8(v, shuffle);
        //let v0 = _mm256_extract_epi64(v, 0);
        //let v1 = _mm256_extract_epi64(v, 2);
        //*(d[offset..].as_mut_ptr() as *mut i64) = v0;
        //*(d[offset + 4..].as_mut_ptr() as *mut i64) = v1;
        //}
        //}
        //},
        //}
        //} else {
        for y in 0..th {
            let d = &mut tu.dequantized_transformed_coeffs[c_idx][y];
            let dzy = &dz[y];
            let lsy = &ls[y];
            for x in 0..tw {
                d[x] = ((((dzy[x] as i32) * lsy[x] + bd_offset) >> bd_shift)
                    .clamp(coeff_min, coeff_max)) as i16;
            }
        }
        //}
    }
}

use super::common::*;
use super::ctu::*;
use super::encoder_context::*;
use super::pps::*;
use super::sps::*;
//use debug_print::*;
use std::cell::RefCell;
use std::rc::Rc;

pub struct IntraPredictor {
    pub ref_l: Vec2d<i16>,
    pub ref_t: Vec2d<i16>,
    pub left_ref_samples: [i16; 130],
    pub above_ref_samples: [i16; 129],
    pub left_ref_filtered_samples: Rc<RefCell<[i16; 130]>>,
    pub above_ref_filtered_samples: Rc<RefCell<[i16; 129]>>,
    pub pred_v: Vec2d<i16>,
    pub pred_h: Vec2d<i16>,
}

impl IntraPredictor {
    pub fn new() -> IntraPredictor {
        IntraPredictor {
            ref_l: vec2d![0; 64; 64],
            ref_t: vec2d![0; 64; 64],
            left_ref_samples: [0; 130],
            above_ref_samples: [0; 129],
            left_ref_filtered_samples: Rc::new(RefCell::new([0; 130])),
            above_ref_filtered_samples: Rc::new(RefCell::new([0; 129])),
            pred_v: vec2d![0; 64; 64],
            pred_h: vec2d![0; 64; 64],
        }
    }

    // [n_scale][idx]
    const PDPSF_WEIGHTS: [[i16; 64]; 3] = [
        [
            32, 8, 2, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0,
        ],
        [
            32, 16, 8, 4, 2, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0,
        ],
        [
            32, 32, 16, 16, 8, 8, 4, 4, 2, 2, 1, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0,
        ],
    ];

    const PDPSF_WEIGHTS_ZERO: [i16; 64] = [0; 64];

    pub fn predict(
        &mut self,
        tu: &mut TransformUnit,
        c_idx: usize,
        sps: &SequenceParameterSet,
        pps: &PictureParameterSet,
        ectx: &mut EncoderContext,
    ) {
        let (intra_luma_ref_line_idx, intra_pred_mode) =
            (tu.cu_intra_luma_ref_idx, tu.cu_intra_pred_mode[c_idx]);
        let ref_idx = if c_idx == 0 {
            intra_luma_ref_line_idx
        } else {
            0
        };
        let tile = tu.get_tile();
        let tile = &mut tile.lock().unwrap();
        let pred_pixels = &mut tile.pred_pixels.borrow_mut()[c_idx];
        let reconst_pixels = &tile.reconst_pixels.borrow()[c_idx];
        match intra_pred_mode {
            IntraPredMode::PLANAR => {
                self.predict_planar(
                    tu,
                    c_idx,
                    ref_idx,
                    pred_pixels,
                    reconst_pixels,
                    sps,
                    pps,
                    ectx,
                );
            }
            IntraPredMode::DC => {
                self.predict_dc(
                    tu,
                    c_idx,
                    ref_idx,
                    pred_pixels,
                    reconst_pixels,
                    sps,
                    pps,
                    ectx,
                );
            }
            intra_pred_mode => {
                let m = intra_pred_mode as usize;
                if m <= 66 {
                    self.predict_angular(
                        tu,
                        c_idx,
                        ref_idx,
                        pred_pixels,
                        reconst_pixels,
                        sps,
                        pps,
                        ectx,
                    );
                } else {
                    panic!()
                }
            }
        }
        let (tw, th) = tu.get_component_size(c_idx);
        let (tx, ty) = tu.get_component_pos(c_idx);
        let original_pixels = &tile.original_pixels.borrow()[c_idx];
        let residual_pixels = &mut tile.residual_pixels.borrow_mut()[c_idx];
        let residuals = &mut tu.residuals[c_idx];
        for y in ty..ty + th {
            let residuals = &mut residuals[y - ty];
            let original_pixels = &original_pixels[y][tx..];
            let pred_pixels = &pred_pixels[y][tx..];
            let residual_pixels = &mut residual_pixels[y][tx..];
            for x in 0..tw {
                let original = original_pixels[x];
                let pred = pred_pixels[x];
                let residual = original as i16 - pred as i16;
                residual_pixels[x] = residual;
                residuals[x] = residual;
            }
        }
    }

    pub fn set_left_and_above_ref_samples(
        &mut self,
        tu: &TransformUnit,
        c_idx: usize,
        ref_idx: usize,
        tile_reconst_pixels: &Vec2d<u8>,
        sps: &SequenceParameterSet,
        pps: &PictureParameterSet,
        ectx: &EncoderContext,
    ) {
        let (x_tb_cmp, y_tb_cmp) = tu.get_component_pos(c_idx);
        let (n_tb_w, n_tb_h) = tu.get_component_size(c_idx);
        let (
            (n_cb_w, n_cb_h),
            intra_subpartitions_mode_flag,
            intra_subpartitions_split_flag,
            intra_pred_mode,
        ) = (
            tu.cu_size[c_idx],
            tu.cu_intra_subpartitions_mode_flag,
            tu.cu_intra_subpartitions_split_flag,
            tu.cu_intra_pred_mode[c_idx],
        );

        // FIXME should be pre-calculated
        let intra_subpartitions_split_type = if intra_subpartitions_mode_flag {
            1 + intra_subpartitions_split_flag as usize
        } else {
            0
        };
        let (ref_w, ref_h) = if intra_subpartitions_split_type == 0 || c_idx != 0 {
            (n_tb_w * 2, n_tb_h * 2)
        } else {
            (n_cb_w + n_tb_w, n_cb_h + n_tb_h)
        };

        let intra_pred_mode = intra_pred_mode as isize;
        // TODO wide angle intra prediction mode mapping process (8.4.5.2.7)

        let ref_filter_flag = matches!(
            intra_pred_mode,
            0 | -14 | -12 | -10 | -6 | 2 | 34 | 66 | 72 | 76 | 78 | 80
        );
        let lx = -1 - (ref_idx as isize);
        let ay = -1 - (ref_idx as isize);
        let left_ref_samples = &mut self.left_ref_samples[..ref_h + ref_idx + 1];
        let above_ref_samples = &mut self.above_ref_samples[..ref_w + ref_idx];
        left_ref_samples.fill(-1);
        above_ref_samples.fill(-1);

        // reference sample avilability marking process (8.4.5.2.8)
        let sy = -1 - (ref_idx as isize);
        let ey = ref_h as isize - 1;
        let is_above_right_available = tu.is_above_right_available();
        let is_below_left_available = tu.is_below_left_available();
        let mut available = true;
        let chroma_shift = (c_idx != 0) as usize; // FIXME
        let x_nb_cmp = x_tb_cmp as isize + lx;
        let x_nb_y = x_nb_cmp << chroma_shift;
        for y in sy..=ey {
            let y_nb_cmp = y_tb_cmp as isize + y;
            let (x_tb_y, y_tb_y) = tu.get_component_pos(0);
            let y_nb_y = y_nb_cmp << chroma_shift;
            if y == sy || y % 4 == 0 {
                available = ectx.derive_neighbouring_block_availability(
                    x_tb_y,
                    y_tb_y,
                    x_nb_y,
                    y_nb_y,
                    tu.width,
                    tu.height,
                    is_above_right_available,
                    is_below_left_available,
                    false,
                    sps,
                    pps,
                );
            }
            if available {
                left_ref_samples[(y - sy) as usize] =
                    tile_reconst_pixels[y_nb_cmp as usize][x_nb_cmp as usize] as i16;
            }
        }

        let sx = -(ref_idx as isize);
        let ex = ref_w as isize - 1;
        let y_nb_cmp = y_tb_cmp as isize + ay;
        let y_nb_y = y_nb_cmp << chroma_shift;
        let reconst_pixels = if y_nb_cmp >= 0 {
            &tile_reconst_pixels[y_nb_cmp as usize]
        } else {
            &tile_reconst_pixels[0]
        };
        for x in sx..=ex {
            let x_nb_cmp = x_tb_cmp as isize + x;
            let (x_tb_y, y_tb_y) = tu.get_component_pos(0);
            let x_nb_y = x_nb_cmp << chroma_shift;
            if x == sx || x % 4 == 0 {
                available = ectx.derive_neighbouring_block_availability(
                    x_tb_y,
                    y_tb_y,
                    x_nb_y,
                    y_nb_y,
                    tu.width,
                    tu.height,
                    is_above_right_available,
                    is_below_left_available,
                    false,
                    sps,
                    pps,
                );
            }
            if available {
                above_ref_samples[(x - sx) as usize] = reconst_pixels[x_nb_cmp as usize] as i16;
            }
        }

        // reference sample filtering process (8.4.5.2.9)
        let left_all_unavailable = left_ref_samples.iter().all(|s| s < &0);
        let above_all_unavailable = above_ref_samples.iter().all(|s| s < &0);
        if left_all_unavailable && above_all_unavailable {
            left_ref_samples.fill(128); // TODO support bit depths other than 8
            above_ref_samples.fill(128); // TODO support bit depths other than 8
        } else {
            let left_bottom_sample = left_ref_samples.last().unwrap();
            if left_bottom_sample < &0 {
                let mut found = false;
                for i in (0..left_ref_samples.len() - 1).rev() {
                    if left_ref_samples[i] >= 0 {
                        *left_ref_samples.last_mut().unwrap() = left_ref_samples[i];
                        found = true;
                        break;
                    }
                }
                if !found {
                    for ars in above_ref_samples.iter() {
                        if *ars >= 0 {
                            *left_ref_samples.last_mut().unwrap() = *ars;
                            break;
                        }
                    }
                }
            }
            for y in (-1 - ref_idx as isize..=ref_h as isize - 2).rev() {
                if left_ref_samples[(y - sy) as usize] < 0 {
                    left_ref_samples[(y - sy) as usize] = left_ref_samples[(y - sy + 1) as usize];
                }
            }
        }
        if above_ref_samples[0] < 0 {
            above_ref_samples[0] = left_ref_samples[0];
        }
        for x in -(ref_idx as isize) + 1..=ref_w as isize - 1 {
            if above_ref_samples[(x - sx) as usize] < 0 {
                above_ref_samples[(x - sx) as usize] = above_ref_samples[(x - sx - 1) as usize];
            }
        }

        // reference sample filtering process (8.4.5.2.10)
        let filter_flag = ref_idx == 0
            && n_tb_w * n_tb_h > 32
            && c_idx == 0
            && intra_subpartitions_split_type == 0
            && ref_filter_flag;
        //let mut left_ref_filtered_samples = vec![0; left_ref_samples.len()];
        //let mut above_ref_filtered_samples = vec![0; above_ref_samples.len()];
        let left_ref_filtered_samples =
            &mut self.left_ref_filtered_samples.borrow_mut()[..left_ref_samples.len()];
        //let mut above_ref_filtered_samples = vec![0; above_ref_samples.len()];
        let above_ref_filtered_samples =
            &mut self.above_ref_filtered_samples.borrow_mut()[..above_ref_samples.len()];
        //left_ref_filtered_samples.fill(0);
        //above_ref_filtered_samples.fill(0);
        if filter_flag {
            left_ref_filtered_samples[ref_idx] = (left_ref_samples[1 + ref_idx]
                + 2 * left_ref_samples[ref_idx]
                + above_ref_samples[ref_idx]
                + 2)
                >> 2;
            let lrfs = &mut left_ref_filtered_samples[1 + ref_idx..];
            let lrs_m1 = &left_ref_samples[ref_idx..];
            let lrs = &left_ref_samples[1 + ref_idx..];
            let lrs_p1 = &left_ref_samples[2 + ref_idx..];
            for y in 0..ref_h - 1 {
                lrfs[y] = (lrs_p1[y] + 2 * lrs[y] + lrs_m1[y] + 2) >> 2;
            }
            lrfs[ref_h - 1] = lrs_m1[ref_h];
            above_ref_filtered_samples[0] =
                (left_ref_samples[0] + 2 * above_ref_samples[0] + above_ref_samples[1] + 2) >> 2;
            above_ref_filtered_samples[0] = (left_ref_samples[ref_idx]
                + 2 * above_ref_samples[ref_idx]
                + above_ref_samples[ref_idx + 1]
                + 2)
                >> 2;
            let arfs = &mut above_ref_filtered_samples[ref_idx + 1..];
            let ars = &above_ref_samples[ref_idx..];
            let ars_p1 = &above_ref_samples[ref_idx + 1..];
            let ars_p2 = &above_ref_samples[ref_idx + 2..];
            for x in 0..ref_w - 2 {
                arfs[x] = (ars[x] + 2 * ars_p1[x] + ars_p2[x] + 2) >> 2;
            }
            arfs[ref_w - 2] = ars[ref_w - 1];
        } else {
            left_ref_filtered_samples[..left_ref_samples.len()].copy_from_slice(left_ref_samples);
            above_ref_filtered_samples[..above_ref_samples.len()]
                .copy_from_slice(above_ref_samples);
        }
    }

    pub fn position_dependent_prediction_sample_filter(
        &mut self,
        above_ref_samples: &[i16],
        left_ref_samples: &[i16],
        above_left_ref_sample: i16,
        tile_pred_pixels: &mut Vec2d<u8>,
        tu: &mut TransformUnit,
        c_idx: usize,
        pred_mode: isize,
        inv_angle: isize,
        _ectx: &EncoderContext,
    ) {
        let (tw, th) = tu.get_component_size(c_idx);
        let (tx, ty) = tu.get_component_pos(c_idx);
        let n_scale = if pred_mode > 50 {
            (th.ilog2() as isize - (3 * inv_angle - 2).ilog2() as isize + 8).min(2)
        } else if pred_mode > 1 && pred_mode < 18 {
            (tw.ilog2() as isize - (3 * inv_angle - 2).ilog2() as isize + 8).min(2)
        } else {
            (tw.ilog2() as isize + th.ilog2() as isize - 2) >> 2
        };
        let alrs = above_left_ref_sample;
        let ref_l = &mut self.ref_l;
        let ref_t = &mut self.ref_t;
        let (w_l, w_t) = if pred_mode < 2 {
            for y in 0..th {
                let ref_ly = &mut ref_l[y];
                let ref_ty = &mut ref_t[y];
                let lrs = left_ref_samples[y];
                let ars = above_ref_samples;
                ref_ly[..tw].fill(lrs);
                ref_ty[..tw].clone_from_slice(&ars[..tw]);
            }
            (
                &Self::PDPSF_WEIGHTS[n_scale as usize][..tw],
                &Self::PDPSF_WEIGHTS[n_scale as usize][..th],
            )
        } else if pred_mode == 18 || pred_mode == 50 {
            for y in 0..th {
                let ref_ly = &mut ref_l[y];
                let ref_ty = &mut ref_t[y];
                let lrs = left_ref_samples[y];
                let ars = above_ref_samples;
                for x in 0..tw {
                    ref_ly[x] = lrs - alrs + tile_pred_pixels[ty + y][tx + x] as i16;
                    ref_ty[x] = ars[x] - alrs + tile_pred_pixels[ty + y][tx + x] as i16;
                }
            }
            (
                if pred_mode == 50 {
                    &Self::PDPSF_WEIGHTS[n_scale as usize][..tw]
                } else {
                    &Self::PDPSF_WEIGHTS_ZERO[..tw]
                },
                if pred_mode == 18 {
                    &Self::PDPSF_WEIGHTS[n_scale as usize][..th]
                } else {
                    &Self::PDPSF_WEIGHTS_ZERO[..th]
                },
            )
        } else if pred_mode < 18 && n_scale >= 0 {
            let dx_int: Vec<i16> = (0..th)
                .map(|y| (((y + 1) as i32 * inv_angle as i32 + 256) >> 9) as i16)
                .collect();
            let mut dx = vec2d![0; th; tw];
            for y in 0..th {
                for x in 0..tw {
                    dx[y][x] = x as i16 + dx_int[y];
                }
            }
            for y in 0..th {
                let ref_ly = &mut ref_l[y];
                let ref_ty = &mut ref_t[y];
                let ars = above_ref_samples;
                ref_ly[..tw].fill(0);
                for x in 0..tw {
                    ref_ty[x] = if y < (3 << n_scale) {
                        ars[dx[y][x] as usize]
                    } else {
                        0
                    };
                }
            }
            (
                &Self::PDPSF_WEIGHTS_ZERO[..th],
                &Self::PDPSF_WEIGHTS[n_scale as usize][..tw],
            )
        } else if pred_mode > 50 && n_scale >= 0 {
            let dy_int: Vec<i16> = (0..tw)
                .map(|x| (((x + 1) as i32 * inv_angle as i32 + 256) >> 9) as i16)
                .collect();
            let mut dy = vec2d![0; th; tw];
            for y in 0..th {
                for (x, dy_intx) in dy_int.iter().enumerate().take(tw) {
                    dy[y][x] = y as i16 + dy_intx;
                }
            }
            for y in 0..th {
                let ref_ly = &mut ref_l[y];
                let ref_ty = &mut ref_t[y];
                let lrs = left_ref_samples;
                ref_ty[..th].fill(0);
                for x in 0..tw {
                    ref_ly[x] = if x < (3 << n_scale) {
                        lrs[dy[y][x] as usize]
                    } else {
                        0
                    };
                }
            }
            (
                &Self::PDPSF_WEIGHTS[n_scale as usize][..th],
                &Self::PDPSF_WEIGHTS_ZERO[..tw],
            )
        } else {
            for y in 0..th {
                let ref_ly = &mut ref_l[y];
                let ref_ty = &mut ref_t[y];
                ref_ly[..tw].fill(0);
                ref_ty[..tw].fill(0);
            }
            (
                &Self::PDPSF_WEIGHTS_ZERO[..th],
                &Self::PDPSF_WEIGHTS_ZERO[..tw],
            )
        };
        // TODO should not rewrite tile directly here
        for y in 0..th {
            let ref_ly = &ref_l[y];
            let ref_ty = &ref_t[y];
            let w_ty = w_t[y];
            let neg_w_ty = 64 - w_ty;
            let tpp = &mut tile_pred_pixels[ty + y][tx..];
            // FIXME something is wrong with SIMD
            //if is_x86_feature_detected!("avx2") && false {
            //use core::arch::x86_64::*;
            //match tw {
            //4 => {
            //for x in 0..4 {
            //let pred = ((ref_ly[x] * w_l[x]
            //+ ref_ty[x] * w_ty
            //+ (neg_w_ty - w_l[x]) * tpp[x] as i16
            //+ 32)
            //>> 6)
            //.clamp(0, 255) as u8;
            //tpp[x] = pred;
            //}
            //}
            //8 => unsafe {
            //let ref_ly = _mm_lddqu_si128(ref_ly.as_ptr() as *const _);
            //let w_l = _mm_lddqu_si128(w_l.as_ptr() as *const _);
            //let ml = _mm_mullo_epi16(ref_ly, w_l);
            //let ref_ty = _mm_lddqu_si128(ref_ty.as_ptr() as *const _);
            //let w_ty = _mm_set1_epi16(w_ty as i16);
            //let mt = _mm_mullo_epi16(ref_ty, w_ty);
            //let m = _mm_add_epi16(ml, mt);
            //let neg_w_ty = _mm_set1_epi16(neg_w_ty as i16);
            //let d = _mm_sub_epi16(neg_w_ty, w_l);
            //let tppv: i64 = *(tpp.as_ptr() as *const _);
            //let tppv = _mm_set_epi64x(0, tppv);
            //let tppv = _mm_cvtepu8_epi16(tppv);
            //let d = _mm_mullo_epi16(d, tppv);
            //let s = _mm_add_epi16(m, d);
            //let d = _mm_set1_epi16(32);
            //let s = _mm_add_epi16(s, d);
            //let shift = _mm_set_epi64x(0, 6);
            //let s = _mm_srl_epi16(s, shift);
            //let ub = _mm_set1_epi16(255);
            //let s = _mm_min_epi16(s, ub);
            //let shuffle =
            //_mm_set_epi8(0, 0, 0, 0, 0, 0, 0, 0, 14, 12, 10, 8, 6, 4, 2, 0);
            //let s = _mm_shuffle_epi8(s, shuffle);
            //let s = _mm_extract_epi64(s, 0);
            //*(tpp.as_mut_ptr() as *mut i64) = s;
            //},
            //16 => unsafe {
            //let ref_lyv = _mm256_lddqu_si256(ref_ly.as_ptr() as *const _);
            //let w_lv = _mm256_lddqu_si256(w_l.as_ptr() as *const _);
            //let ml = _mm256_mullo_epi16(ref_lyv, w_lv);
            //let ref_tyv = _mm256_lddqu_si256(ref_ty.as_ptr() as *const _);
            //let w_ty = _mm256_set1_epi16(w_ty as i16);
            //let mt = _mm256_mullo_epi16(ref_tyv, w_ty);
            //let m = _mm256_add_epi16(ml, mt);
            //let neg_w_ty = _mm256_set1_epi16(neg_w_ty as i16);
            //let d = _mm256_sub_epi16(neg_w_ty, w_lv);
            //let tppv = _mm_lddqu_si128(tpp.as_ptr() as *const _);
            //let tppv = _mm256_cvtepu8_epi16(tppv);
            //let d = _mm256_mullo_epi16(d, tppv);
            //let s = _mm256_add_epi16(m, d);
            //let d = _mm256_set1_epi16(32);
            //let s = _mm256_add_epi16(s, d);
            //let shift = _mm_set_epi64x(0, 6);
            //let s = _mm256_srl_epi16(s, shift);
            //let ub = _mm256_set1_epi16(255);
            //let s = _mm256_min_epi16(s, ub);
            //let shuffle = _mm256_set_epi8(
            //0, 0, 0, 0, 0, 0, 0, 0, 14, 12, 10, 8, 6, 4, 2, 0, 0, 0, 0, 0, 0, 0, 0,
            //0, 14, 12, 10, 8, 6, 4, 2, 0,
            //);
            //let s = _mm256_shuffle_epi8(s, shuffle);
            //let s0 = _mm256_extract_epi64(s, 0);
            //let s1 = _mm256_extract_epi64(s, 2);
            //*(tpp.as_mut_ptr() as *mut i64) = s0;
            //*(tpp[8..].as_mut_ptr() as *mut i64) = s1;
            //},
            //32 => unsafe {
            //let ref_lyv = _mm256_lddqu_si256(ref_ly.as_ptr() as *const _);
            //let w_lv = _mm256_lddqu_si256(w_l.as_ptr() as *const _);
            //let ml = _mm256_mullo_epi16(ref_lyv, w_lv);
            //let ref_tyv = _mm256_lddqu_si256(ref_ty.as_ptr() as *const _);
            //let w_ty = _mm256_set1_epi16(w_ty as i16);
            //let mt = _mm256_mullo_epi16(ref_tyv, w_ty);
            //let m = _mm256_add_epi16(ml, mt);
            //let neg_w_ty = _mm256_set1_epi16(neg_w_ty as i16);
            //let d = _mm256_sub_epi16(neg_w_ty, w_lv);
            //let tppv = _mm_lddqu_si128(tpp.as_ptr() as *const _);
            //let tppv = _mm256_cvtepu8_epi16(tppv);
            //let d = _mm256_mullo_epi16(d, tppv);
            //let s = _mm256_add_epi16(m, d);
            //let d = _mm256_set1_epi16(32);
            //let s = _mm256_add_epi16(s, d);
            //let shift = _mm_set_epi64x(0, 6);
            //let s = _mm256_srl_epi16(s, shift);
            //let ub = _mm256_set1_epi16(255);
            //let s = _mm256_min_epi16(s, ub);
            //let shuffle = _mm256_set_epi8(
            //0, 0, 0, 0, 0, 0, 0, 0, 14, 12, 10, 8, 6, 4, 2, 0, 0, 0, 0, 0, 0, 0, 0,
            //0, 14, 12, 10, 8, 6, 4, 2, 0,
            //);
            //let s = _mm256_shuffle_epi8(s, shuffle);
            //let s0 = _mm256_extract_epi64(s, 0);
            //let s1 = _mm256_extract_epi64(s, 2);
            //*(tpp.as_mut_ptr() as *mut i64) = s0;
            //*(tpp[8..].as_mut_ptr() as *mut i64) = s1;

            //let ref_lyv = _mm256_lddqu_si256(ref_ly[16..].as_ptr() as *const _);
            //let w_lv = _mm256_lddqu_si256(w_l[16..].as_ptr() as *const _);
            //let ml = _mm256_mullo_epi16(ref_lyv, w_lv);
            //let ref_tyv = _mm256_lddqu_si256(ref_ty[16..].as_ptr() as *const _);
            //let mt = _mm256_mullo_epi16(ref_tyv, w_ty);
            //let m = _mm256_add_epi16(ml, mt);
            //let d = _mm256_sub_epi16(neg_w_ty, w_lv);
            //let tppv = _mm_lddqu_si128(tpp[16..].as_ptr() as *const _);
            //let tppv = _mm256_cvtepu8_epi16(tppv);
            //let d = _mm256_mullo_epi16(d, tppv);
            //let s = _mm256_add_epi16(m, d);
            //let d = _mm256_set1_epi16(32);
            //let s = _mm256_add_epi16(s, d);
            //let shift = _mm_set_epi64x(0, 6);
            //let s = _mm256_srl_epi16(s, shift);
            //let ub = _mm256_set1_epi16(255);
            //let s = _mm256_min_epi16(s, ub);
            //let shuffle = _mm256_set_epi8(
            //0, 0, 0, 0, 0, 0, 0, 0, 14, 12, 10, 8, 6, 4, 2, 0, 0, 0, 0, 0, 0, 0, 0,
            //0, 14, 12, 10, 8, 6, 4, 2, 0,
            //);
            //let s = _mm256_shuffle_epi8(s, shuffle);
            //let s0 = _mm256_extract_epi64(s, 0);
            //let s1 = _mm256_extract_epi64(s, 2);
            //*(tpp[16..].as_mut_ptr() as *mut i64) = s0;
            //*(tpp[24..].as_mut_ptr() as *mut i64) = s1;
            //},
            //_ => unsafe {
            //let ref_lyv = _mm256_lddqu_si256(ref_ly.as_ptr() as *const _);
            //let w_lv = _mm256_lddqu_si256(w_l.as_ptr() as *const _);
            //let ml = _mm256_mullo_epi16(ref_lyv, w_lv);
            //let ref_tyv = _mm256_lddqu_si256(ref_ty.as_ptr() as *const _);
            //let w_ty = _mm256_set1_epi16(w_ty as i16);
            //let mt = _mm256_mullo_epi16(ref_tyv, w_ty);
            //let m = _mm256_add_epi16(ml, mt);
            //let neg_w_ty = _mm256_set1_epi16(neg_w_ty as i16);
            //let d = _mm256_sub_epi16(neg_w_ty, w_lv);
            //let tppv = _mm_lddqu_si128(tpp.as_ptr() as *const _);
            //let tppv = _mm256_cvtepu8_epi16(tppv);
            //let d = _mm256_mullo_epi16(d, tppv);
            //let s = _mm256_add_epi16(m, d);
            //let d = _mm256_set1_epi16(32);
            //let s = _mm256_add_epi16(s, d);
            //let shift = _mm_set_epi64x(0, 6);
            //let s = _mm256_srl_epi16(s, shift);
            //let ub = _mm256_set1_epi16(255);
            //let s = _mm256_min_epi16(s, ub);
            //let shuffle = _mm256_set_epi8(
            //0, 0, 0, 0, 0, 0, 0, 0, 14, 12, 10, 8, 6, 4, 2, 0, 0, 0, 0, 0, 0, 0, 0,
            //0, 14, 12, 10, 8, 6, 4, 2, 0,
            //);
            //let s = _mm256_shuffle_epi8(s, shuffle);
            //let s0 = _mm256_extract_epi64(s, 0);
            //let s1 = _mm256_extract_epi64(s, 2);
            //*(tpp.as_mut_ptr() as *mut i64) = s0;
            //*(tpp[8..].as_mut_ptr() as *mut i64) = s1;

            //let ref_lyv = _mm256_lddqu_si256(ref_ly[16..].as_ptr() as *const _);
            //let w_lv = _mm256_lddqu_si256(w_l[16..].as_ptr() as *const _);
            //let ml = _mm256_mullo_epi16(ref_lyv, w_lv);
            //let ref_tyv = _mm256_lddqu_si256(ref_ty[16..].as_ptr() as *const _);
            //let mt = _mm256_mullo_epi16(ref_tyv, w_ty);
            //let m = _mm256_add_epi16(ml, mt);
            //let d = _mm256_sub_epi16(neg_w_ty, w_lv);
            //let tppv = _mm_lddqu_si128(tpp[16..].as_ptr() as *const _);
            //let tppv = _mm256_cvtepu8_epi16(tppv);
            //let d = _mm256_mullo_epi16(d, tppv);
            //let s = _mm256_add_epi16(m, d);
            //let d = _mm256_set1_epi16(32);
            //let s = _mm256_add_epi16(s, d);
            //let shift = _mm_set_epi64x(0, 6);
            //let s = _mm256_srl_epi16(s, shift);
            //let ub = _mm256_set1_epi16(255);
            //let s = _mm256_min_epi16(s, ub);
            //let shuffle = _mm256_set_epi8(
            //0, 0, 0, 0, 0, 0, 0, 0, 14, 12, 10, 8, 6, 4, 2, 0, 0, 0, 0, 0, 0, 0, 0,
            //0, 14, 12, 10, 8, 6, 4, 2, 0,
            //);
            //let s = _mm256_shuffle_epi8(s, shuffle);
            //let s0 = _mm256_extract_epi64(s, 0);
            //let s1 = _mm256_extract_epi64(s, 2);
            //*(tpp[16..].as_mut_ptr() as *mut i64) = s0;
            //*(tpp[24..].as_mut_ptr() as *mut i64) = s1;

            //let ref_lyv = _mm256_lddqu_si256(ref_ly[32..].as_ptr() as *const _);
            //let w_lv = _mm256_lddqu_si256(w_l[32..].as_ptr() as *const _);
            //let ml = _mm256_mullo_epi16(ref_lyv, w_lv);
            //let ref_tyv = _mm256_lddqu_si256(ref_ty[32..].as_ptr() as *const _);
            //let mt = _mm256_mullo_epi16(ref_tyv, w_ty);
            //let m = _mm256_add_epi16(ml, mt);
            //let d = _mm256_sub_epi16(neg_w_ty, w_lv);
            //let tppv = _mm_lddqu_si128(tpp[32..].as_ptr() as *const _);
            //let tppv = _mm256_cvtepu8_epi16(tppv);
            //let d = _mm256_mullo_epi16(d, tppv);
            //let s = _mm256_add_epi16(m, d);
            //let d = _mm256_set1_epi16(32);
            //let s = _mm256_add_epi16(s, d);
            //let shift = _mm_set_epi64x(0, 6);
            //let s = _mm256_srl_epi16(s, shift);
            //let ub = _mm256_set1_epi16(255);
            //let s = _mm256_min_epi16(s, ub);
            //let shuffle = _mm256_set_epi8(
            //0, 0, 0, 0, 0, 0, 0, 0, 14, 12, 10, 8, 6, 4, 2, 0, 0, 0, 0, 0, 0, 0, 0,
            //0, 14, 12, 10, 8, 6, 4, 2, 0,
            //);
            //let s = _mm256_shuffle_epi8(s, shuffle);
            //let s0 = _mm256_extract_epi64(s, 0);
            //let s1 = _mm256_extract_epi64(s, 2);
            //*(tpp[32..].as_mut_ptr() as *mut i64) = s0;
            //*(tpp[40..].as_mut_ptr() as *mut i64) = s1;

            //let ref_lyv = _mm256_lddqu_si256(ref_ly[48..].as_ptr() as *const _);
            //let w_lv = _mm256_lddqu_si256(w_l[48..].as_ptr() as *const _);
            //let ml = _mm256_mullo_epi16(ref_lyv, w_lv);
            //let ref_tyv = _mm256_lddqu_si256(ref_ty[48..].as_ptr() as *const _);
            //let mt = _mm256_mullo_epi16(ref_tyv, w_ty);
            //let m = _mm256_add_epi16(ml, mt);
            //let d = _mm256_sub_epi16(neg_w_ty, w_lv);
            //let tppv = _mm_lddqu_si128(tpp[48..].as_ptr() as *const _);
            //let tppv = _mm256_cvtepu8_epi16(tppv);
            //let d = _mm256_mullo_epi16(d, tppv);
            //let s = _mm256_add_epi16(m, d);
            //let d = _mm256_set1_epi16(32);
            //let s = _mm256_add_epi16(s, d);
            //let shift = _mm_set_epi64x(0, 6);
            //let s = _mm256_srl_epi16(s, shift);
            //let ub = _mm256_set1_epi16(255);
            //let s = _mm256_min_epi16(s, ub);
            //let shuffle = _mm256_set_epi8(
            //0, 0, 0, 0, 0, 0, 0, 0, 14, 12, 10, 8, 6, 4, 2, 0, 0, 0, 0, 0, 0, 0, 0,
            //0, 14, 12, 10, 8, 6, 4, 2, 0,
            //);
            //let s = _mm256_shuffle_epi8(s, shuffle);
            //let s0 = _mm256_extract_epi64(s, 0);
            //let s1 = _mm256_extract_epi64(s, 2);
            //*(tpp[48..].as_mut_ptr() as *mut i64) = s0;
            //*(tpp[56..].as_mut_ptr() as *mut i64) = s1;
            //},
            //}
            //} else {
            for x in 0..tw {
                let pred = ((ref_ly[x] * w_l[x]
                    + ref_ty[x] * w_ty
                    + (neg_w_ty - w_l[x]) * tpp[x] as i16
                    + 32)
                    >> 6)
                    .clamp(0, 255) as u8;
                tpp[x] = pred;
            }
            //}
        }
    }

    pub fn predict_planar(
        &mut self,
        tu: &mut TransformUnit,
        c_idx: usize,
        ref_idx: usize,
        tile_pred_pixels: &mut Vec2d<u8>,
        tile_reconst_pixels: &Vec2d<u8>,
        sps: &SequenceParameterSet,
        pps: &PictureParameterSet,
        ectx: &mut EncoderContext,
    ) {
        self.set_left_and_above_ref_samples(
            tu,
            c_idx,
            ref_idx,
            tile_reconst_pixels,
            sps,
            pps,
            ectx,
        );
        let (tw, th) = tu.get_component_size(c_idx);
        let (tx, ty) = tu.get_component_pos(c_idx);
        if ectx.enable_print {
            println!("pred planar {}x{} @ ({},{})", tw, th, tx, ty);
        }
        ectx.enable_print = false;
        let lrs = self.left_ref_filtered_samples.clone();
        let lrs = &lrs.borrow();
        let alrs = lrs[0];
        let lrs = &lrs[ref_idx + 1..];
        let ars = self.above_ref_filtered_samples.clone();
        let ars = &ars.borrow()[ref_idx..];

        if tw == th {
            let pred_v = &mut self.pred_v;
            let pred_h = &mut self.pred_h;
            let ars_r = ars[tw];
            let lrs_b = lrs[th];
            let tw_m1 = tw as i16 - 1;
            let th_m1 = th as i16 - 1;
            let rx: Vec<i16> = (0..tw as i16).map(|x| (x + 1) * ars_r).collect();
            if is_x86_feature_detected!("avx2") {
                use core::arch::x86_64::*;
                for y in 0..th {
                    let pred_vy = &mut pred_v[y];
                    let pred_hy = &mut pred_h[y];
                    let rv = th_m1 - y as i16;
                    let ry = (y as i16 + 1) * lrs_b;
                    let lrs = lrs[y];
                    match tw {
                        4 => {
                            for x in 0..4 {
                                pred_vy[x] = rv * ars[x] + ry;
                                pred_hy[x] = (3 - x as i16) * lrs + rx[x];
                            }
                        }
                        8 => unsafe {
                            let rv = _mm_set1_epi16(rv);
                            let ars = _mm_lddqu_si128(ars.as_ptr() as *const _);
                            let rv = _mm_mullo_epi16(rv, ars);
                            let ry = _mm_set1_epi16(ry);
                            let vy = _mm_add_epi16(rv, ry);
                            _mm_storeu_si128(pred_vy.as_mut_ptr() as *mut _, vy);
                            let m = _mm_set_epi16(0, 1, 2, 3, 4, 5, 6, 7);
                            let lrs = _mm_set1_epi16(lrs);
                            let hy = _mm_mullo_epi16(m, lrs);
                            let rx = _mm_lddqu_si128(rx.as_ptr() as *const _);
                            let hy = _mm_add_epi16(hy, rx);
                            _mm_storeu_si128(pred_hy.as_mut_ptr() as *mut _, hy);
                        },
                        16 => unsafe {
                            let rv = _mm256_set1_epi16(rv);
                            let ars = _mm256_lddqu_si256(ars.as_ptr() as *const _);
                            let rv = _mm256_mullo_epi16(rv, ars);
                            let ry = _mm256_set1_epi16(ry);
                            let vy = _mm256_add_epi16(rv, ry);
                            _mm256_storeu_si256(pred_vy.as_mut_ptr() as *mut _, vy);
                            let m = _mm256_set_epi16(
                                0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15,
                            );
                            let lrs = _mm256_set1_epi16(lrs as i16);
                            let hy = _mm256_mullo_epi16(m, lrs);
                            let rx = _mm256_lddqu_si256(rx.as_ptr() as *const _);
                            let hy = _mm256_add_epi16(hy, rx);
                            _mm256_storeu_si256(pred_hy.as_mut_ptr() as *mut _, hy);
                        },
                        32 => unsafe {
                            {
                                let rv = _mm256_set1_epi16(rv);
                                let arsv = _mm256_lddqu_si256(ars.as_ptr() as *const _);
                                let rvv = _mm256_mullo_epi16(rv, arsv);
                                let ry = _mm256_set1_epi16(ry);
                                let vy = _mm256_add_epi16(rvv, ry);
                                _mm256_storeu_si256(pred_vy.as_mut_ptr() as *mut _, vy);
                                let m = _mm256_set_epi16(
                                    16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 26, 27, 28, 29, 30, 31,
                                );
                                let lrs = _mm256_set1_epi16(lrs);
                                let hy = _mm256_mullo_epi16(m, lrs);
                                let rxv = _mm256_lddqu_si256(rx.as_ptr() as *const _);
                                let hy = _mm256_add_epi16(hy, rxv);
                                _mm256_storeu_si256(pred_hy.as_mut_ptr() as *mut _, hy);

                                let arsv = _mm256_lddqu_si256(ars[16..].as_ptr() as *const _);
                                let rvv = _mm256_mullo_epi16(rv, arsv);
                                let vy = _mm256_add_epi16(rvv, ry);
                                _mm256_storeu_si256(pred_vy[16..].as_mut_ptr() as *mut _, vy);
                                let m = _mm256_set_epi16(
                                    0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15,
                                );
                                let hy = _mm256_mullo_epi16(m, lrs);
                                let rxv = _mm256_lddqu_si256(rx[16..].as_ptr() as *const _);
                                let hy = _mm256_add_epi16(hy, rxv);
                                _mm256_storeu_si256(pred_hy[16..].as_mut_ptr() as *mut _, hy);
                            }
                        },
                        _ => unsafe {
                            let rv = _mm256_set1_epi16(rv);
                            let arsv = _mm256_lddqu_si256(ars.as_ptr() as *const _);
                            let rvv = _mm256_mullo_epi16(rv, arsv);
                            let ry = _mm256_set1_epi16(ry);
                            let vy = _mm256_add_epi16(rvv, ry);
                            _mm256_storeu_si256(pred_vy.as_mut_ptr() as *mut _, vy);
                            let m = _mm256_set_epi16(
                                48, 49, 50, 51, 52, 53, 54, 55, 56, 57, 58, 59, 60, 61, 62, 63,
                            );
                            let lrs = _mm256_set1_epi16(lrs);
                            let hy = _mm256_mullo_epi16(m, lrs);
                            let rxv = _mm256_lddqu_si256(rx.as_ptr() as *const _);
                            let hy = _mm256_add_epi16(hy, rxv);
                            _mm256_storeu_si256(pred_hy.as_mut_ptr() as *mut _, hy);

                            let arsv = _mm256_lddqu_si256(ars[16..].as_ptr() as *const _);
                            let rvv = _mm256_mullo_epi16(rv, arsv);
                            let vy = _mm256_add_epi16(rvv, ry);
                            _mm256_storeu_si256(pred_vy[16..].as_mut_ptr() as *mut _, vy);
                            let m = _mm256_set_epi16(
                                32, 33, 34, 35, 36, 37, 38, 39, 40, 41, 42, 43, 44, 45, 46, 47,
                            );
                            let hy = _mm256_mullo_epi16(m, lrs);
                            let rxv = _mm256_lddqu_si256(rx[16..].as_ptr() as *const _);
                            let hy = _mm256_add_epi16(hy, rxv);
                            _mm256_storeu_si256(pred_hy[16..].as_mut_ptr() as *mut _, hy);

                            let arsv = _mm256_lddqu_si256(ars[32..].as_ptr() as *const _);
                            let rvv = _mm256_mullo_epi16(rv, arsv);
                            let vy = _mm256_add_epi16(rvv, ry);
                            _mm256_storeu_si256(pred_vy[32..].as_mut_ptr() as *mut _, vy);
                            let m = _mm256_set_epi16(
                                16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 26, 27, 28, 29, 30, 31,
                            );
                            let hy = _mm256_mullo_epi16(m, lrs);
                            let rxv = _mm256_lddqu_si256(rx[32..].as_ptr() as *const _);
                            let hy = _mm256_add_epi16(hy, rxv);
                            _mm256_storeu_si256(pred_hy[32..].as_mut_ptr() as *mut _, hy);

                            let arsv = _mm256_lddqu_si256(ars[48..].as_ptr() as *const _);
                            let rvv = _mm256_mullo_epi16(rv, arsv);
                            let vy = _mm256_add_epi16(rvv, ry);
                            _mm256_storeu_si256(pred_vy[48..].as_mut_ptr() as *mut _, vy);
                            let m = _mm256_set_epi16(
                                0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15,
                            );
                            let hy = _mm256_mullo_epi16(m, lrs);
                            let rxv = _mm256_lddqu_si256(rx[48..].as_ptr() as *const _);
                            let hy = _mm256_add_epi16(hy, rxv);
                            _mm256_storeu_si256(pred_hy[48..].as_mut_ptr() as *mut _, hy);
                        },
                    }
                }
            } else {
                for y in 0..th {
                    let pred_vy = &mut pred_v[y];
                    let pred_hy = &mut pred_h[y];
                    let rv = th_m1 - y as i16;
                    let ry = (y as i16 + 1) * lrs_b;
                    let lrs = lrs[y];
                    for x in 0..tw {
                        pred_vy[x] = rv * ars[x] + ry;
                        pred_hy[x] = (tw_m1 - x as i16) * lrs + rx[x];
                    }
                }
            }

            let shift = tw.ilog2() + 1;
            let d = tw as i16;
            if is_x86_feature_detected!("avx2") {
                use core::arch::x86_64::*;
                match tw {
                    4 => {
                        for y in 0..th {
                            let pred_vy = &pred_v[y];
                            let pred_hy = &pred_h[y];
                            let tile_pred_pixels = &mut tile_pred_pixels[ty + y][tx..];
                            for x in 0..4 {
                                let planar_val = (pred_vy[x] + pred_hy[x] + d) >> 3;
                                tile_pred_pixels[x] = planar_val as u8;
                            }
                        }
                    }
                    8 => {
                        //let d = _mm_set1_epi16(8);
                        //for y in 0..th {
                        //let pred_vy = &pred_v[y];
                        //let pred_hy = &pred_h[y];
                        //let tile_pred_pixels = &mut tile_pred_pixels[ty + y][tx..];
                        //let pred_vy = _mm_lddqu_si128(pred_vy.as_ptr() as *const _);
                        //let pred_hy = _mm_lddqu_si128(pred_hy.as_ptr() as *const _);
                        //let planar_val = _mm_add_epi16(pred_vy, pred_hy);
                        //let planar_val = _mm_add_epi16(planar_val, d);
                        //let planar_val = _mm_srai_epi16(planar_val, 5);
                        //let planar_val = _mm_cvtepi16_epu8(planar_val);
                        //*(tile_pred_pixels.as_mut_ptr() as *mut i64) = planar_val;
                        ////for x in 0..8 {
                        ////let planar_val = (pred_vy[x] + pred_hy[x] + d) >> shift;
                        ////tile_pred_pixels[x] = planar_val as u8;
                        ////}
                        //}
                        for y in 0..th {
                            let pred_vy = &pred_v[y];
                            let pred_hy = &pred_h[y];
                            let tile_pred_pixels = &mut tile_pred_pixels[ty + y][tx..];
                            for x in 0..8 {
                                let planar_val = (pred_vy[x] + pred_hy[x] + d) >> 4;
                                tile_pred_pixels[x] = planar_val as u8;
                            }
                        }
                    }
                    16 => {
                        //let d = _mm256_set1_epi16(16);
                        //for y in 0..th {
                        //let pred_vy = &pred_v[y];
                        //let pred_hy = &pred_h[y];
                        //let tile_pred_pixels = &mut tile_pred_pixels[ty + y][tx..];
                        //let pred_vy = _mm256_lddqu_si256(pred_vy.as_ptr() as *const _);
                        //let pred_hy = _mm256_lddqu_si256(pred_hy.as_ptr() as *const _);
                        //let planar_val = _mm256_add_epi16(pred_vy, pred_hy);
                        //let planar_val = _mm256_add_epi16(planar_val, d);
                        //let planar_val = _mm256_srai_epi16(planar_val, 5);
                        //let planar_val = _mm256_cvtepi16_epu8(planar_val);
                        //_mm_storeu_si128(tile_pred_pixels.as_mut_ptr() as *mut _, planar_val);
                        ////for x in 0..8 {
                        ////let planar_val = (pred_vy[x] + pred_hy[x] + d) >> shift;
                        ////tile_pred_pixels[x] = planar_val as u8;
                        ////}
                        //}
                        for y in 0..th {
                            let pred_vy = &pred_v[y];
                            let pred_hy = &pred_h[y];
                            let tile_pred_pixels = &mut tile_pred_pixels[ty + y][tx..];
                            for x in 0..16 {
                                let planar_val = (pred_vy[x] + pred_hy[x] + d) >> 5;
                                tile_pred_pixels[x] = planar_val as u8;
                            }
                        }
                    }
                    32 => unsafe {
                        let d = _mm256_set1_epi16(32);
                        for y in 0..th {
                            let pred_vy = &pred_v[y];
                            let pred_hy = &pred_h[y];
                            let tile_pred_pixels = &mut tile_pred_pixels[ty + y][tx..];
                            let vpred_vy = _mm256_lddqu_si256(pred_vy.as_ptr() as *const _);
                            let vpred_hy = _mm256_lddqu_si256(pred_hy.as_ptr() as *const _);
                            let planar_val = _mm256_add_epi16(vpred_vy, vpred_hy);
                            let planar_val = _mm256_add_epi16(planar_val, d);
                            let planar_val = _mm256_srai_epi16(planar_val, 6);
                            let planar_val = _mm256_cvtepi16_epu8(planar_val);
                            _mm_storeu_si128(tile_pred_pixels.as_mut_ptr() as *mut _, planar_val);

                            let vpred_vy = _mm256_lddqu_si256(pred_vy[16..].as_ptr() as *const _);
                            let vpred_hy = _mm256_lddqu_si256(pred_hy[16..].as_ptr() as *const _);
                            let planar_val = _mm256_add_epi16(vpred_vy, vpred_hy);
                            let planar_val = _mm256_add_epi16(planar_val, d);
                            let planar_val = _mm256_srai_epi16(planar_val, 6);
                            let planar_val = _mm256_cvtepi16_epu8(planar_val);
                            _mm_storeu_si128(
                                tile_pred_pixels[16..].as_mut_ptr() as *mut _,
                                planar_val,
                            );
                        }
                    },
                    _ => unsafe {
                        let d = _mm256_set1_epi16(64);
                        for y in 0..th {
                            let pred_vy = &pred_v[y];
                            let pred_hy = &pred_h[y];
                            let tile_pred_pixels = &mut tile_pred_pixels[ty + y][tx..];
                            let vpred_vy = _mm256_lddqu_si256(pred_vy.as_ptr() as *const _);
                            let vpred_hy = _mm256_lddqu_si256(pred_hy.as_ptr() as *const _);
                            let planar_val = _mm256_add_epi16(vpred_vy, vpred_hy);
                            let planar_val = _mm256_add_epi16(planar_val, d);
                            let planar_val = _mm256_srai_epi16(planar_val, 7);
                            let planar_val = _mm256_cvtepi16_epu8(planar_val);
                            _mm_storeu_si128(tile_pred_pixels.as_mut_ptr() as *mut _, planar_val);

                            let vpred_vy = _mm256_lddqu_si256(pred_vy[16..].as_ptr() as *const _);
                            let vpred_hy = _mm256_lddqu_si256(pred_hy[16..].as_ptr() as *const _);
                            let planar_val = _mm256_add_epi16(vpred_vy, vpred_hy);
                            let planar_val = _mm256_add_epi16(planar_val, d);
                            let planar_val = _mm256_srai_epi16(planar_val, 7);
                            let planar_val = _mm256_cvtepi16_epu8(planar_val);
                            _mm_storeu_si128(
                                tile_pred_pixels[16..].as_mut_ptr() as *mut _,
                                planar_val,
                            );

                            let vpred_vy = _mm256_lddqu_si256(pred_vy[32..].as_ptr() as *const _);
                            let vpred_hy = _mm256_lddqu_si256(pred_hy[32..].as_ptr() as *const _);
                            let planar_val = _mm256_add_epi16(vpred_vy, vpred_hy);
                            let planar_val = _mm256_add_epi16(planar_val, d);
                            let planar_val = _mm256_srai_epi16(planar_val, 7);
                            let planar_val = _mm256_cvtepi16_epu8(planar_val);
                            _mm_storeu_si128(
                                tile_pred_pixels[32..].as_mut_ptr() as *mut _,
                                planar_val,
                            );

                            let vpred_vy = _mm256_lddqu_si256(pred_vy[48..].as_ptr() as *const _);
                            let vpred_hy = _mm256_lddqu_si256(pred_hy[48..].as_ptr() as *const _);
                            let planar_val = _mm256_add_epi16(vpred_vy, vpred_hy);
                            let planar_val = _mm256_add_epi16(planar_val, d);
                            let planar_val = _mm256_srai_epi16(planar_val, 7);
                            let planar_val = _mm256_cvtepi16_epu8(planar_val);
                            _mm_storeu_si128(
                                tile_pred_pixels[48..].as_mut_ptr() as *mut _,
                                planar_val,
                            );
                        }
                    },
                }
            } else {
                for y in 0..th {
                    let pred_vy = &pred_v[y];
                    let pred_hy = &pred_h[y];
                    let tile_pred_pixels = &mut tile_pred_pixels[ty + y][tx..];
                    for x in 0..tw {
                        let planar_val = (pred_vy[x] + pred_hy[x] + d) >> shift;
                        tile_pred_pixels[x] = planar_val as u8;
                    }
                }
            }
        } else {
            // TODO SIMD
            let pred_v = &mut self.pred_v;
            let pred_h = &mut self.pred_h;
            let ars_r = ars[tw] as i16;
            let lrs_b = lrs[th] as i16;
            let tw_m1 = tw - 1;
            let rx: Vec<i16> = (0..tw as i16).map(|x| (x + 1) * ars_r).collect();
            for y in 0..th {
                let pred_vy = &mut pred_v[y];
                let pred_hy = &mut pred_h[y];
                let rv = (th - 1 - y) as i16;
                let ry = (y as i16 + 1) * lrs_b;
                for x in 0..tw {
                    pred_vy[x] = (rv * ars[x] + ry) << tw.ilog2();
                    pred_hy[x] = ((tw_m1 - x) as i16 * lrs[y] + rx[x]) << th.ilog2();
                }
            }
            let d = (tw * th) as i16;
            let shift = tw.ilog2() + th.ilog2() + 1;
            for y in 0..th {
                let pred_vy = &pred_v[y];
                let pred_hy = &pred_h[y];
                let tile_pred_pixels = &mut tile_pred_pixels[ty + y][tx..];
                for x in 0..tw {
                    let planar_val = (pred_vy[x] + pred_hy[x] + d) >> shift;
                    tile_pred_pixels[x] = planar_val as u8;
                }
            }
        }

        let bdpcm_flag = tu.cu_bdpcm_flag[c_idx];
        if tw >= 4 && th >= 4 && ref_idx == 0 && !bdpcm_flag {
            self.position_dependent_prediction_sample_filter(
                &ars[..tw],
                &lrs[..th],
                alrs,
                tile_pred_pixels,
                tu,
                c_idx,
                IntraPredMode::PLANAR as isize,
                0,
                ectx,
            );
        }
    }

    pub fn predict_dc(
        &mut self,
        tu: &mut TransformUnit,
        c_idx: usize,
        ref_idx: usize,
        tile_pred_pixels: &mut Vec2d<u8>,
        tile_reconst_pixels: &Vec2d<u8>,
        sps: &SequenceParameterSet,
        pps: &PictureParameterSet,
        ectx: &mut EncoderContext,
    ) {
        self.set_left_and_above_ref_samples(
            tu,
            c_idx,
            ref_idx,
            tile_reconst_pixels,
            sps,
            pps,
            ectx,
        );
        let (tw, th) = tu.get_component_size(c_idx);
        let (tx, ty) = tu.get_component_pos(c_idx);
        if ectx.enable_print {
            println!("pred dc {}x{} @ ({},{})", tw, th, tx, ty);
        }
        ectx.enable_print = false;
        let lrs = self.left_ref_filtered_samples.clone();
        let lrs = &lrs.borrow();
        let alrs = lrs[0];
        let lrs = &lrs[ref_idx + 1..ref_idx + 1 + th];
        let ars = self.above_ref_filtered_samples.clone();
        let ars = &ars.borrow()[ref_idx..ref_idx + tw];
        // TODO SIMD
        let dc_val = if tw == th {
            let v = tw as i16
                + if is_x86_feature_detected!("avx2") {
                    //use core::arch::x86_64::*;
                    match tw {
                        4 => {
                            ((ars[0] + ars[1]) + (ars[2] + ars[3]))
                                + ((lrs[0] + lrs[1]) + (lrs[2] + lrs[3]))
                        }
                        8 => {
                            //let ars = _mm_lddqu_si128(ars.as_ptr() as *const _);
                            //let lrs = _mm_lddqu_si128(lrs.as_ptr() as *const _);
                            //let sum = _mm_add_epi16(ars, lrs);
                            //let sum = _mm_hadd_epi16(sum, sum);
                            //let sum = _mm_hadd_epi16(sum, sum);
                            //let s0 = _mm_extract_epi16(sum, 0) as u16;
                            //let s1 = _mm_extract_epi16(sum, 1) as u16;
                            //s0 + s1
                            (((ars[0] + ars[1]) + (ars[2] + ars[3]))
                                + ((ars[4] + ars[5]) + (ars[6] + ars[7])))
                                + (((lrs[0] + lrs[1]) + (lrs[2] + lrs[3]))
                                    + ((lrs[4] + lrs[5]) + (lrs[6] + lrs[7])))
                        }
                        //16 => unsafe {
                        //((((ars[0] + ars[1]) + (ars[2] + ars[3]))
                        //+ ((ars[4] + ars[5]) + (ars[6] + ars[7])))
                        //+ (((ars[8] + ars[9]) + (ars[10] + ars[11]))
                        //+ ((ars[12] + ars[13]) + (ars[14] + ars[15]))))
                        //+ ((((lrs[0] + lrs[1]) + (lrs[2] + lrs[3]))
                        //+ ((lrs[4] + lrs[5]) + (lrs[6] + lrs[7])))
                        //+ (((lrs[8] + lrs[9]) + (lrs[10] + lrs[11]))
                        //+ ((lrs[12] + lrs[13]) + (lrs[14] + lrs[15]))))
                        //},
                        _ => ars.iter().sum::<i16>() + lrs.iter().sum::<i16>(),
                        //16 => unsafe {
                        //let rs = _mm256_unordered_cvt2epi16_epu8(
                        //_mm256_lddqu_si256(ars.as_ptr() as *const _),
                        //_mm256_lddqu_si256(lrs.as_ptr() as *const _),
                        //);
                        //let zero = _mm256_setzero_si256();
                        //let sum = _mm256_sad_epu8(rs, zero);
                        //let s0 = _mm256_extract_epi16(sum, 0) as u16;
                        //let s1 = _mm256_extract_epi16(sum, 4) as u16;
                        //let s2 = _mm256_extract_epi16(sum, 8) as u16;
                        //let s3 = _mm256_extract_epi16(sum, 12) as u16;
                        //(s0 + s1) + (s2 + s3)
                        //},
                        //32 => unsafe {
                        //let rs = _mm256_unordered_cvt2epi16_epu8(
                        //_mm256_lddqu_si256(ars.as_ptr() as *const _),
                        //_mm256_lddqu_si256(lrs.as_ptr() as *const _),
                        //);
                        //let zero = _mm256_setzero_si256();
                        //let sum0 = _mm256_sad_epu8(rs, zero);

                        //let rs = _mm256_unordered_cvt2epi16_epu8(
                        //_mm256_lddqu_si256(ars[16..].as_ptr() as *const _),
                        //_mm256_lddqu_si256(lrs[16..].as_ptr() as *const _),
                        //);
                        //let sum1 = _mm256_sad_epu8(rs, zero);
                        //let sum = _mm256_add_epi16(sum0, sum1);

                        //let s0 = _mm256_extract_epi16(sum, 0) as u16;
                        //let s1 = _mm256_extract_epi16(sum, 4) as u16;
                        //let s2 = _mm256_extract_epi16(sum, 8) as u16;
                        //let s3 = _mm256_extract_epi16(sum, 12) as u16;
                        //(s0 + s1) + (s2 + s3)
                        //},
                        //_ => {
                        //ars.iter().map(|x| *x as u16).sum::<u16>()
                        //+ lrs.iter().map(|x| *x as u16).sum::<u16>()
                        //}
                    }
                } else {
                    ars.iter().sum::<i16>() + lrs.iter().sum::<i16>()
                };
            v >> (tw.ilog2() + 1)
        } else if tw > th {
            let v = (tw as i16 >> 1) + ars.iter().sum::<i16>();
            v >> tw.ilog2()
        } else {
            let v = (th as i16 >> 1) + lrs.iter().sum::<i16>();
            v >> th.ilog2()
        } as u8;

        for y in ty..ty + th {
            let tile_pred_pixels = &mut tile_pred_pixels[y];
            tile_pred_pixels[tx..tx + tw].fill(dc_val);
        }
        let bdpcm_flag = tu.cu_bdpcm_flag[c_idx];
        if tw >= 4 && th >= 4 && ref_idx == 0 && !bdpcm_flag {
            // position-dependent prediction sample filtering process (8.4.5.2.15)
            self.position_dependent_prediction_sample_filter(
                ars,
                lrs,
                alrs,
                tile_pred_pixels,
                tu,
                c_idx,
                IntraPredMode::DC as isize,
                0,
                ectx,
            );
        }
    }

    pub fn predict_angular(
        &mut self,
        tu: &mut TransformUnit,
        c_idx: usize,
        ref_idx: usize,
        tile_pred_pixels: &mut Vec2d<u8>,
        tile_reconst_pixels: &Vec2d<u8>,
        sps: &SequenceParameterSet,
        pps: &PictureParameterSet,
        ectx: &EncoderContext,
    ) {
        let (n_tb_w, n_tb_h) = tu.get_component_size(c_idx);
        let (
            (n_cb_w, n_cb_h),
            intra_subpartitions_mode_flag,
            intra_subpartitions_split_flag,
            mut intra_pred_mode,
        ) = (
            tu.cu_size[c_idx],
            tu.cu_intra_subpartitions_mode_flag,
            tu.cu_intra_subpartitions_split_flag,
            tu.cu_intra_pred_mode[c_idx] as isize,
        );

        let intra_subpartitions_split_type = if intra_subpartitions_mode_flag {
            1 + intra_subpartitions_split_flag as usize
        } else {
            0
        };
        let (ref_w, ref_h) = if intra_subpartitions_split_type == 0 || c_idx != 0 {
            (n_tb_w * 2, n_tb_h * 2)
        } else {
            (n_cb_w + n_tb_w, n_cb_h + n_tb_h)
        };

        // wide angle intra prediction mode mapping process (8.4.5.2.7)
        let (nw, nh) = if intra_subpartitions_split_type
            == IntraSubpartitionsSplitType::ISP_NO_SPLIT as usize
        {
            (n_tb_w, n_tb_h)
        } else {
            (n_cb_w, n_cb_h)
        };
        let wh_ratio = (nw.ilog2() as isize - nh.ilog2() as isize).abs() as isize;
        if nw != nh {
            if nw > nh
                && intra_pred_mode >= 2
                && intra_pred_mode < (if wh_ratio > 1 { 8 + 2 * wh_ratio } else { 8 })
            {
                intra_pred_mode += 65;
            } else if nh > nw
                && intra_pred_mode <= 66
                && intra_pred_mode > (if wh_ratio > 1 { 60 - 2 * wh_ratio } else { 60 })
            {
                intra_pred_mode -= 67;
            }
        }

        self.set_left_and_above_ref_samples(
            tu,
            c_idx,
            ref_idx,
            tile_reconst_pixels,
            sps,
            pps,
            ectx,
        );
        let (tw, th) = tu.get_component_size(c_idx);
        let (tx, ty) = tu.get_component_pos(c_idx);
        let lrs = self.left_ref_filtered_samples.clone();
        let lrs = &lrs.borrow();
        let alrs = lrs[0];
        //let lrs = &lrs[ref_idx + 1..];
        let ars = self.above_ref_filtered_samples.clone();
        let ars = &ars.borrow();
        //let ars = &ars.borrow()[ref_idx..];

        let n_tb_s = (tw.ilog2() + th.ilog2()) >> 1;
        let ref_filter_flag = matches!(
            intra_pred_mode,
            0 | -14 | -12 | -10 | -6 | 2 | 34 | 66 | 72 | 76 | 78 | 80
        );
        let filter_flag = if ref_filter_flag
            || ref_idx != 0
            || ectx.intra_subpartitions_split_type != IntraSubpartitionsSplitType::ISP_NO_SPLIT
        {
            false
        } else {
            let min_dist_ver_hor = (intra_pred_mode - 50)
                .abs()
                .min((intra_pred_mode - 18).abs());
            let intra_hor_ver_dist_thres = match n_tb_s {
                2 => 24,
                3 => 14,
                4 => 2,
                5 => 0,
                6 => 0,
                _ => panic!(),
            };
            min_dist_ver_hor > intra_hor_ver_dist_thres
        };

        let intra_pred_angle = INTRA_ANGLE_TABLE[(14 + intra_pred_mode) as usize];
        let inv_angle = if intra_pred_angle > 0 {
            (512 * 32 + intra_pred_angle / 2) / intra_pred_angle
        } else if intra_pred_angle < 0 {
            -((512 * 32 + (-intra_pred_angle) / 2) / -intra_pred_angle)
        } else {
            0
        }; // round
        if intra_pred_mode >= 34 {
            let mut refx = vec![0; tw + 1 + ref_idx + 1];
            refx[0] = alrs;
            for x in 0..=tw + ref_idx {
                refx[x + 1] = ars[x];
            }
            if intra_pred_angle < 0 {
                for x in -(n_tb_h as isize)..=-1 {
                    refx.push(lrs[((x * inv_angle + 256) >> 9).min(n_tb_h as isize) as usize]);
                }
            } else {
                for x in n_tb_w + 2 + ref_idx..ref_w + ref_idx {
                    refx.push(ars[x - 1]);
                }
                for _x in 1..=(n_tb_w / n_tb_h).max(1) * ref_idx + 3 {
                    refx.push(ars[ref_w + ref_idx - 1]);
                }
            }
            if ectx.enable_print {
                println!("ars {:?}", ars);
            }
            if ectx.enable_print {
                println!("refx {:?}", refx);
            }

            for y in 0..th {
                let tile_pred_pixels = &mut tile_pred_pixels[ty + y][tx..];
                let i_idx =
                    (((y + 1 + ref_idx) as isize * intra_pred_angle) >> 5) + ref_idx as isize;
                let i_fact = ((y + 1 + ref_idx) as isize * intra_pred_angle) & 31;
                if c_idx == 0 {
                    let f_t = if filter_flag {
                        F_G[i_fact as usize]
                    } else {
                        F_C[i_fact as usize]
                    };
                    for (x, tpp) in tile_pred_pixels.iter_mut().enumerate().take(tw) {
                        let s: isize = (0..=3)
                            .map(|i| {
                                let idx = x as isize + i_idx + i as isize;
                                let idx = if idx < 0 {
                                    refx.len() as isize + idx
                                } else {
                                    idx
                                };
                                f_t[i] * refx[idx as usize] as isize
                            })
                            .sum();
                        *tpp = ((s + 32) >> 6).clamp(0, 255) as u8;
                    }
                } else if i_fact != 0 {
                    for (x, tpp) in tile_pred_pixels.iter_mut().enumerate().take(tw) {
                        let idx0 = x as isize + i_idx + 1;
                        let idx0 = if idx0 < 0 {
                            refx.len() as isize + idx0
                        } else {
                            idx0
                        };
                        let idx1 = x as isize + i_idx + 2;
                        let idx1 = if idx1 < 0 {
                            refx.len() as isize + idx1
                        } else {
                            idx1
                        };
                        *tpp = (((32 - i_fact) * refx[idx0 as usize] as isize
                            + i_fact * refx[idx1 as usize] as isize
                            + 16)
                            >> 5) as u8;
                    }
                } else {
                    for (x, tpp) in tile_pred_pixels.iter_mut().enumerate().take(tw) {
                        let idx = x as isize + i_idx + 1;
                        let idx = if idx < 0 {
                            refx.len() as isize + idx
                        } else {
                            idx
                        };
                        *tpp = refx[idx as usize] as u8;
                    }
                }
            }
        } else {
            let mut refx = vec![0; n_tb_h + 2 + ref_idx];
            for x in 0..=n_tb_h + ref_idx + 1 {
                refx[x] = lrs[x];
            }
            if intra_pred_angle < 0 {
                for x in -(n_tb_w as isize)..=-1 {
                    let idx = ((x * inv_angle + 256) >> 9).min(n_tb_w as isize) as usize;
                    refx.push(if idx == 0 { alrs } else { ars[idx - 1] });
                }
            } else {
                for x in n_tb_h + 2 + ref_idx..=ref_h + ref_idx {
                    refx.push(lrs[x]);
                }
                // FIXME +2 not in spec?, o.w. overlow in the following process
                for _x in 1..=((n_tb_h / n_tb_w).max(1) * ref_idx + 2) {
                    refx.push(lrs[ref_h + ref_idx]);
                }
            }
            if ectx.enable_print {
                println!("ars {:?}", ars);
            }
            if ectx.enable_print {
                println!("refx {:?}", refx);
            }
            for x in 0..tw {
                let i_idx =
                    (((x + 1 + ref_idx) as isize * intra_pred_angle) >> 5) + ref_idx as isize;
                let i_fact = ((x + 1 + ref_idx) as isize * intra_pred_angle) & 31;
                if c_idx == 0 {
                    let f_t = if filter_flag {
                        F_G[i_fact as usize]
                    } else {
                        F_C[i_fact as usize]
                    };
                    for y in 0..th {
                        let s: isize = (0..=3)
                            .map(|i| {
                                let idx = y as isize + i_idx + i as isize;
                                let idx = if idx < 0 {
                                    refx.len() as isize + idx
                                } else {
                                    idx
                                };
                                f_t[i] * refx[idx as usize] as isize
                            })
                            .sum();
                        tile_pred_pixels[ty + y][tx + x] = ((s + 32) >> 6).clamp(0, 255) as u8;
                    }
                } else if i_fact != 0 {
                    for y in 0..th {
                        let idx0 = y as isize + i_idx + 1;
                        let idx0 = if idx0 < 0 {
                            refx.len() as isize + idx0
                        } else {
                            idx0
                        };
                        let idx1 = y as isize + i_idx + 2;
                        let idx1 = if idx1 < 0 {
                            refx.len() as isize + idx1
                        } else {
                            idx1
                        };
                        tile_pred_pixels[ty + y][tx + x] = (((32 - i_fact)
                            * refx[idx0 as usize] as isize
                            + i_fact * refx[idx1 as usize] as isize
                            + 16)
                            >> 5) as u8;
                    }
                } else {
                    for y in 0..th {
                        let idx = y as isize + i_idx + 1;
                        let idx = if idx < 0 {
                            refx.len() as isize + idx
                        } else {
                            idx
                        };
                        tile_pred_pixels[ty + y][tx + x] = refx[idx as usize] as u8;
                    }
                }
            }
        }
        if ectx.enable_print {
            println!("pre pdpsf pred ang {}x{} @ ({},{})", tw, th, tx, ty);
            for y in 0..th {
                for x in 0..th {
                    print!("{} ", tile_pred_pixels[ty + y][tx + x]);
                }
                println!();
            }
            println!();
        }
        let bdpcm_flag = tu.cu_bdpcm_flag[c_idx];
        if tw >= 4
            && th >= 4
            && ref_idx == 0
            && !bdpcm_flag
            && (intra_pred_mode <= IntraPredMode::ANGULAR18 as isize
                || (intra_pred_mode >= IntraPredMode::ANGULAR50 as isize
                    && intra_pred_mode < IntraPredMode::LT_CCLM as isize))
        {
            // position-dependent prediction sample filtering process (8.4.5.2.15)
            self.position_dependent_prediction_sample_filter(
                &ars[ref_idx..],
                &lrs[1 + ref_idx..],
                alrs,
                tile_pred_pixels,
                tu,
                c_idx,
                intra_pred_mode,
                inv_angle,
                ectx,
            );
        }
        if ectx.enable_print {
            println!("pred ang {}x{} @ ({},{})", tw, th, tx, ty);
            for y in 0..th {
                for x in 0..th {
                    print!("{} ", tile_pred_pixels[ty + y][tx + x]);
                }
                println!();
            }
            println!();
        }
    }
}

#![allow(non_camel_case_types, non_snake_case)]
#[allow(unused_imports)]
use num::{integer::Integer, FromPrimitive};
use std::ops::{Index, IndexMut};
use std::sync::{Arc, Mutex};

#[macro_export]
macro_rules! hashmap {
    () => { std::collections::HashMap::new() };
    ($( $key: expr => $val: expr ),*) => {{
         let mut map = ::std::collections::HashMap::new();
         $( map.insert($key, $val); )*
         map
    }};
}

pub type ArcMutex<T> = Arc<Mutex<T>>;

#[derive(Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)]
pub enum ChromaFormat {
    Monochrome = 0,
    YCbCr420 = 1,
    YCbCr422 = 2,
    YCbCr444 = 3,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[allow(clippy::upper_case_acronyms)]
pub enum MttSplitMode {
    SPLIT_NONE,
    SPLIT_BT_VER,
    SPLIT_BT_HOR,
    SPLIT_TT_VER,
    SPLIT_TT_HOR,
    SPLIT_QT,
}

impl MttSplitMode {
    pub fn _is_ver(&self) -> bool {
        self == &Self::SPLIT_BT_VER || self == &Self::SPLIT_TT_VER
    }

    pub fn _is_hor(&self) -> bool {
        self == &Self::SPLIT_BT_HOR || self == &Self::SPLIT_TT_HOR
    }

    pub fn is_bt(&self) -> bool {
        self == &Self::SPLIT_BT_VER || self == &Self::SPLIT_BT_HOR
    }

    pub fn is_tt(&self) -> bool {
        self == &Self::SPLIT_TT_VER || self == &Self::SPLIT_TT_HOR
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[allow(dead_code)]
#[allow(clippy::upper_case_acronyms)]
pub enum ModeType {
    MODE_INTRA = 0,
    MODE_IBC = 1,
    MODE_PLT = 2,
    MODE_INTER = 3,
    MODE_TYPE_ALL = 4,
    MODE_TYPE_INTRA = 6,
    MODE_TYPE_INTER = 7,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, FromPrimitive)]
#[allow(clippy::upper_case_acronyms)]
pub enum IntraPredMode {
    PLANAR = 0,
    DC = 1,
    ANGULAR2 = 2,
    ANGULAR3 = 3,
    ANGULAR4 = 4,
    ANGULAR5 = 5,
    ANGULAR6 = 6,
    ANGULAR7 = 7,
    ANGULAR8 = 8,
    ANGULAR9 = 9,
    ANGULAR10 = 10,
    ANGULAR11 = 11,
    ANGULAR12 = 12,
    ANGULAR13 = 13,
    ANGULAR14 = 14,
    ANGULAR15 = 15,
    ANGULAR16 = 16,
    ANGULAR17 = 17,
    ANGULAR18 = 18,
    ANGULAR19 = 19,
    ANGULAR20 = 20,
    ANGULAR21 = 21,
    ANGULAR22 = 22,
    ANGULAR23 = 23,
    ANGULAR24 = 24,
    ANGULAR25 = 25,
    ANGULAR26 = 26,
    ANGULAR27 = 27,
    ANGULAR28 = 28,
    ANGULAR29 = 29,
    ANGULAR30 = 30,
    ANGULAR31 = 31,
    ANGULAR32 = 32,
    ANGULAR33 = 33,
    ANGULAR34 = 34,
    ANGULAR35 = 35,
    ANGULAR36 = 36,
    ANGULAR37 = 37,
    ANGULAR38 = 38,
    ANGULAR39 = 39,
    ANGULAR40 = 40,
    ANGULAR41 = 41,
    ANGULAR42 = 42,
    ANGULAR43 = 43,
    ANGULAR44 = 44,
    ANGULAR45 = 45,
    ANGULAR46 = 46,
    ANGULAR47 = 47,
    ANGULAR48 = 48,
    ANGULAR49 = 49,
    ANGULAR50 = 50,
    ANGULAR51 = 51,
    ANGULAR52 = 52,
    ANGULAR53 = 53,
    ANGULAR54 = 54,
    ANGULAR55 = 55,
    ANGULAR56 = 56,
    ANGULAR57 = 57,
    ANGULAR58 = 58,
    ANGULAR59 = 59,
    ANGULAR60 = 60,
    ANGULAR61 = 61,
    ANGULAR62 = 62,
    ANGULAR63 = 63,
    ANGULAR64 = 64,
    ANGULAR65 = 65,
    ANGULAR66 = 66,
    LT_CCLM = 81,
    L_CCLM = 82,
    T_CCLM = 83,
}

pub const INTRA_ANGLE_TABLE: [isize; 95] = [
    512, 341, 256, 171, 128, 102, 86, 73, 64, 57, 51, 45, 39, 35, 0, 0, 32, 29, 26, 23, 20, 18, 16,
    14, 12, 10, 8, 6, 4, 3, 2, 1, 0, -1, -2, -3, -4, -6, -8, -10, -12, -14, -16, -18, -20, -23,
    -26, -29, -32, -29, -26, -23, -20, -18, -16, -14, -12, -10, -8, -6, -4, -3, -2, -1, 0, 1, 2, 3,
    4, 6, 8, 10, 12, 14, 16, 18, 20, 23, 26, 29, 32, 35, 39, 45, 51, 57, 64, 73, 86, 102, 128, 171,
    256, 341, 512,
];

pub const F_C: [[isize; 4]; 32] = [
    [0, 64, 0, 0],
    [-1, 63, 2, 0],
    [-2, 62, 4, 0],
    [-2, 60, 7, -1],
    [-2, 58, 10, -2],
    [-3, 57, 12, -2],
    [-4, 56, 14, -2],
    [-4, 55, 15, -2],
    [-4, 54, 16, -2],
    [-5, 53, 18, -2],
    [-6, 52, 20, -2],
    [-6, 49, 24, -3],
    [-6, 46, 28, -4],
    [-5, 44, 29, -4],
    [-4, 42, 30, -4],
    [-4, 39, 33, -4],
    [-4, 36, 36, -4],
    [-4, 33, 39, -4],
    [-4, 30, 42, -4],
    [-4, 29, 44, -5],
    [-4, 28, 46, -6],
    [-3, 24, 49, -6],
    [-2, 20, 52, -6],
    [-2, 18, 53, -5],
    [-2, 16, 54, -4],
    [-2, 15, 55, -4],
    [-2, 14, 56, -4],
    [-2, 12, 57, -3],
    [-2, 10, 58, -2],
    [-1, 7, 60, -2],
    [0, 4, 62, -2],
    [0, 2, 63, -1],
];

pub const F_G: [[isize; 4]; 32] = [
    [16, 32, 16, 0],
    [16, 32, 16, 0],
    [15, 31, 17, 1],
    [15, 31, 17, 1],
    [14, 30, 18, 2],
    [14, 30, 18, 2],
    [13, 29, 19, 3],
    [13, 29, 19, 3],
    [12, 28, 20, 4],
    [12, 28, 20, 4],
    [11, 27, 21, 5],
    [11, 27, 21, 5],
    [10, 26, 22, 6],
    [10, 26, 22, 6],
    [9, 25, 23, 7],
    [9, 25, 23, 7],
    [8, 24, 24, 8],
    [8, 24, 24, 8],
    [7, 23, 25, 9],
    [7, 23, 25, 9],
    [6, 22, 26, 10],
    [6, 22, 26, 10],
    [5, 21, 27, 11],
    [5, 21, 27, 11],
    [4, 20, 28, 12],
    [4, 20, 28, 12],
    [3, 19, 29, 13],
    [3, 19, 29, 13],
    [2, 18, 30, 14],
    [2, 18, 30, 14],
    [1, 17, 31, 15],
    [1, 17, 31, 15],
];

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum InterPredMode {
    PRED_L0 = 0,
    PRED_L1 = 1,
    PRED_BI = 2,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[allow(dead_code)]
pub enum IntraSubpartitionsSplitType {
    ISP_NO_SPLIT = 0,
    ISP_HOR_SPLIT = 1,
    ISP_VER_SPLIT = 2,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum TreeType {
    SINGLE_TREE = 0,
    DUAL_TREE_LUMA = 1,
    DUAL_TREE_CHROMA = 2,
}

pub struct WindowOffset {
    pub left_offset: isize,
    pub right_offset: isize,
    pub top_offset: isize,
    pub bottom_offset: isize,
}

impl WindowOffset {
    pub fn new() -> WindowOffset {
        WindowOffset {
            left_offset: 0,
            right_offset: 0,
            top_offset: 0,
            bottom_offset: 0,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ApsParamsType {
    ALF_APS = 0,
    LMCS_APS = 1,
    SCALING_APS = 2,
}

#[derive(Clone, Debug)]
pub struct Vec2d<T> {
    pub data: Vec<T>,
    pub height: usize,
    pub width: usize,
    pub log2_stride: usize,
}

impl<T: Copy> Vec2d<T> {
    #[inline(always)]
    pub fn new(v: T, height: usize, width: usize) -> Vec2d<T> {
        let log2_stride = (width * 2 - 1).ilog2() as usize;
        Vec2d {
            data: vec![v; height << log2_stride],
            height,
            width,
            log2_stride,
        }
    }

    #[inline(always)]
    pub fn _fill(&mut self, v: T) {
        self.data.fill(v);
    }
}

impl<T> Index<usize> for Vec2d<T> {
    type Output = [T];
    #[inline(always)]
    fn index(&self, index: usize) -> &Self::Output {
        let offset = index << self.log2_stride;
        &self.data[offset..offset + self.width]
    }
}

impl<T> IndexMut<usize> for Vec2d<T> {
    #[inline(always)]
    fn index_mut(&mut self, index: usize) -> &mut [T] {
        let offset = index << self.log2_stride;
        &mut self.data[offset..offset + self.width]
    }
}

#[macro_export]
macro_rules! vec2d {
    ($elem:expr; $h:expr; $w:expr) => {
        Vec2d::new($elem, $h, $w)
    };
}

use core::arch::x86_64::*;

#[inline(always)]
pub fn _mm_cvtepi16_epu8(v: __m128i) -> i64 {
    unsafe {
        let shuffle: __m128i =
            _mm_setr_epi8(0, 2, 4, 6, 8, 10, 12, 14, -1, -1, -1, -1, -1, -1, -1, -1);
        let v = _mm_shuffle_epi8(v, shuffle);
        _mm_extract_epi64(v, 0)
    }
}

#[inline(always)]
pub fn _mm256_cvtepi16_epu8(v: __m256i) -> __m128i {
    unsafe {
        let shuffle: __m256i = _mm256_setr_epi8(
            0, 2, 4, 6, 8, 10, 12, 14, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
            -1, 0, 2, 4, 6, 8, 10, 12, 14,
        );
        let v = _mm256_shuffle_epi8(v, shuffle);
        let low = _mm256_castsi256_si128(v);
        let high = _mm256_extracti128_si256(v, 1);
        _mm_or_si128(low, high)
    }
}

#[inline(always)]
pub fn _mm256_unordered_cvt2epi16_epu8(v0: __m256i, v1: __m256i) -> __m256i {
    unsafe {
        let shuffle0: __m256i = _mm256_setr_epi8(
            0, 2, 4, 6, 8, 10, 12, 14, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
            -1, 0, 2, 4, 6, 8, 10, 12, 14,
        );
        let shuffle1: __m256i = _mm256_setr_epi8(
            -1, -1, -1, -1, -1, -1, -1, -1, 0, 2, 4, 6, 8, 10, 12, 14, 0, 2, 4, 6, 8, 10, 12, 14,
            -1, -1, -1, -1, -1, -1, -1, -1,
        );
        let v0 = _mm256_shuffle_epi8(v0, shuffle0);
        let v1 = _mm256_shuffle_epi8(v1, shuffle1);
        _mm256_or_si256(v0, v1)
    }
}

#[inline(always)]
pub fn sum_128_epi32(v: __m128i) -> i32 {
    unsafe {
        let h = _mm_unpackhi_epi64(v, v);
        let v = _mm_add_epi32(h, v);
        let h = _mm_shuffle_epi32(v, 0b10110001);
        let v = _mm_add_epi32(v, h);
        _mm_cvtsi128_si32(v)
    }
}

#[inline(always)]
pub fn sum_256_epi32(v: __m256i) -> i32 {
    unsafe {
        let v = _mm_add_epi32(_mm256_castsi256_si128(v), _mm256_extracti128_si256(v, 1));
        sum_128_epi32(v)
    }
}

#[inline(always)]
pub fn msum_8_i16_le_i9(v0: &[i16], v1: &[i16]) -> i32 {
    unsafe {
        let v0 = _mm_lddqu_si128(v0.as_ptr() as *const _);
        let v1 = _mm_lddqu_si128(v1.as_ptr() as *const _);
        let v = _mm_madd_epi16(v0, v1);
        sum_128_epi32(v)
    }
}

#[inline(always)]
pub fn msum_16x1_i16_le_i9(v0: &[i16], v1: &[i16]) -> i32 {
    unsafe {
        let h = {
            let h0 = _mm256_lddqu_si256(v0.as_ptr() as *const _);
            let h1 = _mm256_lddqu_si256(v1.as_ptr() as *const _);
            _mm256_madd_epi16(h0, h1)
        };
        sum_256_epi32(h)
    }
}

#[inline(always)]
pub fn msum_16x2_i16_le_i9(v0: &[i16], v1: &[i16]) -> i32 {
    unsafe {
        let mut h = {
            let h0 = _mm256_lddqu_si256(v0.as_ptr() as *const _);
            let h1 = _mm256_lddqu_si256(v1.as_ptr() as *const _);
            _mm256_madd_epi16(h0, h1)
        };
        let t = {
            let t0 = _mm256_lddqu_si256(v0[16..].as_ptr() as *const _);
            let t1 = _mm256_lddqu_si256(v1[16..].as_ptr() as *const _);
            _mm256_madd_epi16(t0, t1)
        };
        h = _mm256_add_epi32(h, t);
        sum_256_epi32(h)
    }
}

#[inline(always)]
pub fn msum_16x4_i16_le_i9(v0: &[i16], v1: &[i16]) -> i32 {
    unsafe {
        let mut h = {
            let h0 = _mm256_lddqu_si256(v0.as_ptr() as *const _);
            let h1 = _mm256_lddqu_si256(v1.as_ptr() as *const _);
            _mm256_madd_epi16(h0, h1)
        };
        let t = {
            let t0 = _mm256_lddqu_si256(v0[16..].as_ptr() as *const _);
            let t1 = _mm256_lddqu_si256(v1[16..].as_ptr() as *const _);
            _mm256_madd_epi16(t0, t1)
        };
        h = _mm256_add_epi32(h, t);
        let t = {
            let t0 = _mm256_lddqu_si256(v0[32..].as_ptr() as *const _);
            let t1 = _mm256_lddqu_si256(v1[32..].as_ptr() as *const _);
            _mm256_madd_epi16(t0, t1)
        };
        h = _mm256_add_epi32(h, t);
        let t = {
            let t0 = _mm256_lddqu_si256(v0[48..].as_ptr() as *const _);
            let t1 = _mm256_lddqu_si256(v1[48..].as_ptr() as *const _);
            _mm256_madd_epi16(t0, t1)
        };
        h = _mm256_add_epi32(h, t);
        sum_256_epi32(h)
    }
}

pub fn _msum_8xN_i16_i32(v0: &[i16], v1: &[i32]) -> i32 {
    unsafe {
        let mut h = {
            let h0 = _mm_lddqu_si128(v0.as_ptr() as *const _);
            let h0 = _mm256_cvtepi16_epi32(h0);
            let h1 = _mm256_lddqu_si256(v1.as_ptr() as *const _);
            _mm256_mullo_epi32(h0, h1)
        };
        for s in (8..v0.len()).step_by(8) {
            let t = {
                let t0 = _mm_lddqu_si128(v0[s..].as_ptr() as *const _);
                let t0 = _mm256_cvtepi16_epi32(t0);
                let t1 = _mm256_lddqu_si256(v1[s..].as_ptr() as *const _);
                _mm256_mullo_epi32(t0, t1)
            };
            h = _mm256_add_epi32(h, t);
        }
        sum_256_epi32(h)
    }
}

pub fn msum_8x1_i16_i32_le_i9_i17(v0: &[i16], v1: &[i32]) -> i32 {
    unsafe {
        let h = {
            let h0 = _mm_lddqu_si128(v0.as_ptr() as *const _);
            let h0 = _mm256_cvtepi16_epi32(h0);
            let h1 = _mm256_lddqu_si256(v1.as_ptr() as *const _);
            _mm256_mullo_epi32(h0, h1)
        };
        sum_256_epi32(h)
    }
}

pub fn msum_8x2_i16_i32_le_i9_i17(v0: &[i16], v1: &[i32]) -> i32 {
    unsafe {
        let mut h = {
            let h0 = _mm_lddqu_si128(v0.as_ptr() as *const _);
            let h0 = _mm256_cvtepi16_epi32(h0);
            let h1 = _mm256_lddqu_si256(v1.as_ptr() as *const _);
            _mm256_mullo_epi32(h0, h1)
        };
        let t = {
            let t0 = _mm_lddqu_si128(v0[8..].as_ptr() as *const _);
            let t0 = _mm256_cvtepi16_epi32(t0);
            let t1 = _mm256_lddqu_si256(v1[8..].as_ptr() as *const _);
            _mm256_mullo_epi32(t0, t1)
        };
        h = _mm256_add_epi32(h, t);
        sum_256_epi32(h)
    }
}

pub fn msum_8x4_i16_i32_le_i9_i17(v0: &[i16], v1: &[i32]) -> i32 {
    unsafe {
        let mut h = {
            let h0 = _mm_lddqu_si128(v0.as_ptr() as *const _);
            let h0 = _mm256_cvtepi16_epi32(h0);
            let h1 = _mm256_lddqu_si256(v1.as_ptr() as *const _);
            _mm256_mullo_epi32(h0, h1)
        };
        let t = {
            let t0 = _mm_lddqu_si128(v0[8..].as_ptr() as *const _);
            let t0 = _mm256_cvtepi16_epi32(t0);
            let t1 = _mm256_lddqu_si256(v1[8..].as_ptr() as *const _);
            _mm256_mullo_epi32(t0, t1)
        };
        h = _mm256_add_epi32(h, t);
        let t = {
            let t0 = _mm_lddqu_si128(v0[16..].as_ptr() as *const _);
            let t0 = _mm256_cvtepi16_epi32(t0);
            let t1 = _mm256_lddqu_si256(v1[16..].as_ptr() as *const _);
            _mm256_mullo_epi32(t0, t1)
        };
        h = _mm256_add_epi32(h, t);
        let t = {
            let t0 = _mm_lddqu_si128(v0[24..].as_ptr() as *const _);
            let t0 = _mm256_cvtepi16_epi32(t0);
            let t1 = _mm256_lddqu_si256(v1[24..].as_ptr() as *const _);
            _mm256_mullo_epi32(t0, t1)
        };
        h = _mm256_add_epi32(h, t);
        sum_256_epi32(h)
    }
}

pub fn msum_8x8_i16_i32_le_i9_i17(v0: &[i16], v1: &[i32]) -> i32 {
    unsafe {
        let mut h = {
            let h0 = _mm_lddqu_si128(v0.as_ptr() as *const _);
            let h0 = _mm256_cvtepi16_epi32(h0);
            let h1 = _mm256_lddqu_si256(v1.as_ptr() as *const _);
            _mm256_mullo_epi32(h0, h1)
        };
        let t = {
            let t0 = _mm_lddqu_si128(v0[8..].as_ptr() as *const _);
            let t0 = _mm256_cvtepi16_epi32(t0);
            let t1 = _mm256_lddqu_si256(v1[8..].as_ptr() as *const _);
            _mm256_mullo_epi32(t0, t1)
        };
        h = _mm256_add_epi32(h, t);
        let t = {
            let t0 = _mm_lddqu_si128(v0[16..].as_ptr() as *const _);
            let t0 = _mm256_cvtepi16_epi32(t0);
            let t1 = _mm256_lddqu_si256(v1[16..].as_ptr() as *const _);
            _mm256_mullo_epi32(t0, t1)
        };
        h = _mm256_add_epi32(h, t);
        let t = {
            let t0 = _mm_lddqu_si128(v0[24..].as_ptr() as *const _);
            let t0 = _mm256_cvtepi16_epi32(t0);
            let t1 = _mm256_lddqu_si256(v1[24..].as_ptr() as *const _);
            _mm256_mullo_epi32(t0, t1)
        };
        h = _mm256_add_epi32(h, t);
        let t = {
            let t0 = _mm_lddqu_si128(v0[32..].as_ptr() as *const _);
            let t0 = _mm256_cvtepi16_epi32(t0);
            let t1 = _mm256_lddqu_si256(v1[32..].as_ptr() as *const _);
            _mm256_mullo_epi32(t0, t1)
        };
        h = _mm256_add_epi32(h, t);
        let t = {
            let t0 = _mm_lddqu_si128(v0[40..].as_ptr() as *const _);
            let t0 = _mm256_cvtepi16_epi32(t0);
            let t1 = _mm256_lddqu_si256(v1[40..].as_ptr() as *const _);
            _mm256_mullo_epi32(t0, t1)
        };
        h = _mm256_add_epi32(h, t);
        let t = {
            let t0 = _mm_lddqu_si128(v0[48..].as_ptr() as *const _);
            let t0 = _mm256_cvtepi16_epi32(t0);
            let t1 = _mm256_lddqu_si256(v1[48..].as_ptr() as *const _);
            _mm256_mullo_epi32(t0, t1)
        };
        h = _mm256_add_epi32(h, t);
        let t = {
            let t0 = _mm_lddqu_si128(v0[56..].as_ptr() as *const _);
            let t0 = _mm256_cvtepi16_epi32(t0);
            let t1 = _mm256_lddqu_si256(v1[56..].as_ptr() as *const _);
            _mm256_mullo_epi32(t0, t1)
        };
        h = _mm256_add_epi32(h, t);
        sum_256_epi32(h)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::{prelude::StdRng, Rng, SeedableRng};
    #[test]
    #[cfg(target_feature = "avx2")]
    fn msum_8_i16_le_i9_works() {
        let mut rng: StdRng = SeedableRng::seed_from_u64(2);
        let mut v0: Vec<i16> = vec![0; 8];
        let mut v1: Vec<i16> = vec![0; 8];
        rng.fill(&mut v0[..]);
        rng.fill(&mut v1[..]);
        v0.iter_mut().for_each(|x| *x %= 0b111111111);
        v1.iter_mut().for_each(|x| *x %= 0b111111111);
        let dut = msum_8_i16_le_i9(&v0[..], &v1[..]);
        let gt = (0..8).map(|i| v0[i] as i32 * v1[i] as i32).sum();
        assert_eq!(dut, gt);
    }

    #[test]
    #[cfg(target_feature = "avx2")]
    fn msum_16x1_i16_le_i9_works() {
        let mut rng: StdRng = SeedableRng::seed_from_u64(2);
        let mut v0: Vec<i16> = vec![0; 16];
        let mut v1: Vec<i16> = vec![0; 16];
        rng.fill(&mut v0[..]);
        rng.fill(&mut v1[..]);
        v0.iter_mut().for_each(|x| *x %= 0b111111111);
        v1.iter_mut().for_each(|x| *x %= 0b111111111);
        let dut = msum_16x1_i16_le_i9(&v0[..], &v1[..]);
        let gt = (0..16).map(|i| v0[i] as i32 * v1[i] as i32).sum();
        assert_eq!(dut, gt);
    }

    #[test]
    #[cfg(target_feature = "avx2")]
    fn msum_16x2_i16_le_i9_works() {
        let mut rng: StdRng = SeedableRng::seed_from_u64(2);
        let mut v0: Vec<i16> = vec![0; 32];
        let mut v1: Vec<i16> = vec![0; 32];
        rng.fill(&mut v0[..]);
        rng.fill(&mut v1[..]);
        v0.iter_mut().for_each(|x| *x %= 0b111111111);
        v1.iter_mut().for_each(|x| *x %= 0b111111111);
        let dut = msum_16x2_i16_le_i9(&v0[..], &v1[..]);
        let gt = (0..32).map(|i| v0[i] as i32 * v1[i] as i32).sum();
        assert_eq!(dut, gt);
    }

    #[test]
    #[cfg(target_feature = "avx2")]
    fn msum_16x4_i16_le_i9_works() {
        let mut rng: StdRng = SeedableRng::seed_from_u64(2);
        let mut v0: Vec<i16> = vec![0; 64];
        let mut v1: Vec<i16> = vec![0; 64];
        rng.fill(&mut v0[..]);
        rng.fill(&mut v1[..]);
        v0.iter_mut().for_each(|x| *x %= 0b111111111);
        v1.iter_mut().for_each(|x| *x %= 0b111111111);
        let dut = msum_16x4_i16_le_i9(&v0[..], &v1[..]);
        let gt = (0..64).map(|i| v0[i] as i32 * v1[i] as i32).sum();
        assert_eq!(dut, gt);
    }

    #[test]
    #[cfg(target_feature = "avx2")]
    fn msum_8x1_i16_i32_le_i9_i17_works() {
        let mut rng: StdRng = SeedableRng::seed_from_u64(2);
        let mut v0: Vec<i16> = vec![0; 8];
        let mut v1: Vec<i32> = vec![0; 8];
        rng.fill(&mut v0[..]);
        rng.fill(&mut v1[..]);
        v0.iter_mut().for_each(|x| *x %= 0b111111111);
        v1.iter_mut().for_each(|x| *x %= 0b11111111111111111);
        let dut = msum_8x1_i16_i32_le_i9_i17(&v0[..], &v1[..]);
        let gt = (0..8).map(|i| v0[i] as i32 * v1[i]).sum();
        assert_eq!(dut, gt);
    }

    #[test]
    #[cfg(target_feature = "avx2")]
    fn msum_8x2_i16_i32_le_i9_i17_works() {
        let mut rng: StdRng = SeedableRng::seed_from_u64(2);
        let mut v0: Vec<i16> = vec![0; 16];
        let mut v1: Vec<i32> = vec![0; 16];
        rng.fill(&mut v0[..]);
        rng.fill(&mut v1[..]);
        v0.iter_mut().for_each(|x| *x %= 0b111111111);
        v1.iter_mut().for_each(|x| *x %= 0b11111111111111111);
        let dut = msum_8x2_i16_i32_le_i9_i17(&v0[..], &v1[..]);
        let gt = (0..16).map(|i| v0[i] as i32 * v1[i]).sum();
        assert_eq!(dut, gt);
    }

    #[test]
    #[cfg(target_feature = "avx2")]
    fn msum_8x4_i16_i32_le_i9_i17_works() {
        let mut rng: StdRng = SeedableRng::seed_from_u64(2);
        let mut v0: Vec<i16> = vec![0; 32];
        let mut v1: Vec<i32> = vec![0; 32];
        rng.fill(&mut v0[..]);
        rng.fill(&mut v1[..]);
        v0.iter_mut().for_each(|x| *x %= 0b111111111);
        v1.iter_mut().for_each(|x| *x %= 0b11111111111111111);
        let dut = msum_8x4_i16_i32_le_i9_i17(&v0[..], &v1[..]);
        let gt = (0..32).map(|i| v0[i] as i32 * v1[i]).sum();
        assert_eq!(dut, gt);
    }

    #[test]
    #[cfg(target_feature = "avx2")]
    fn msum_8x8_i16_i32_le_i9_i17_works() {
        let mut rng: StdRng = SeedableRng::seed_from_u64(2);
        let mut v0: Vec<i16> = vec![0; 64];
        let mut v1: Vec<i32> = vec![0; 64];
        rng.fill(&mut v0[..]);
        rng.fill(&mut v1[..]);
        v0.iter_mut().for_each(|x| *x %= 0b111111111);
        v1.iter_mut().for_each(|x| *x %= 0b11111111111111111);
        let dut = msum_8x8_i16_i32_le_i9_i17(&v0[..], &v1[..]);
        let gt = (0..64).map(|i| v0[i] as i32 * v1[i]).sum();
        assert_eq!(dut, gt);
    }
}

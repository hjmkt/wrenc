use super::common::*;

pub struct DpbParameter {
    pub max_tid: usize,
    pub max_dec_pic_buffering: usize,
    pub max_num_reorder_pics: usize,
    pub max_latency_increase: usize,
}

impl DpbParameter {
    pub fn new() -> DpbParameter {
        DpbParameter {
            max_tid: 1,
            max_dec_pic_buffering: 8,
            max_num_reorder_pics: 4,
            max_latency_increase: 1,
        }
    }
}

pub struct OlsDpbParameter {
    pub pic_width: usize,
    pub pic_height: usize,
    pub chroma_format: ChromaFormat,
    pub bitdepth: usize,
    pub params_idx: usize,
}

impl OlsDpbParameter {
    pub fn new(
        width: usize,
        height: usize,
        chroma_format: ChromaFormat,
        bitdepth: usize,
        params_idx: usize,
    ) -> OlsDpbParameter {
        OlsDpbParameter {
            pic_width: width,
            pic_height: height,
            chroma_format,
            bitdepth,
            params_idx,
        }
    }
}

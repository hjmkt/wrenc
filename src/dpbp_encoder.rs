use super::bins::*;
use super::bool_coder::*;
use super::dpb::*;
use super::encoder_context::*;
use debug_print::*;
use std::sync::{Arc, Mutex};

pub struct DpbpEncoder<'a> {
    coder: &'a mut BoolCoder,
    _encoder_context: Arc<Mutex<EncoderContext>>,
}

impl<'a> DpbpEncoder<'a> {
    pub fn new(
        encoder_context: &Arc<Mutex<EncoderContext>>,
        coder: &'a mut BoolCoder,
    ) -> DpbpEncoder<'a> {
        DpbpEncoder {
            coder,
            _encoder_context: encoder_context.clone(),
        }
    }

    pub fn encode(
        &mut self,
        bins: &mut Bins,
        dpb_parameters: &[DpbParameter],
        max_sublayers: usize,
        sublayer_info_flag: bool,
    ) {
        let l = if sublayer_info_flag {
            0
        } else {
            max_sublayers - 1
        };
        for dpbp in dpb_parameters.iter().take(max_sublayers).skip(l) {
            debug_eprint!("dpbp.max_dec_pic_buffering ");
            self.coder
                .encode_unsigned_exp_golomb(bins, dpbp.max_dec_pic_buffering as u64);
            debug_eprint!("dpbp.max_num_reorder_pics ");
            self.coder
                .encode_unsigned_exp_golomb(bins, dpbp.max_num_reorder_pics as u64);
            debug_eprint!("dpbp.max_latency_increase ");
            self.coder
                .encode_unsigned_exp_golomb(bins, dpbp.max_latency_increase as u64);
        }
    }
}

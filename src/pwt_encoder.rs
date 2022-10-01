use super::bins::*;
use super::bool_coder::*;
use super::common::*;
use super::encoder_context::*;
use super::picture_header::*;
use super::pps::*;
use super::pred_weight_table::*;
use super::sps::*;
use std::sync::{Arc, Mutex};

pub struct PredWeightTableEncoder<'a> {
    coder: &'a mut BoolCoder,
    encoder_context: Arc<Mutex<EncoderContext>>,
}

impl<'a> PredWeightTableEncoder<'a> {
    pub fn new(
        encoder_context: &Arc<Mutex<EncoderContext>>,
        coder: &'a mut BoolCoder,
    ) -> PredWeightTableEncoder<'a> {
        PredWeightTableEncoder {
            coder,
            encoder_context: encoder_context.clone(),
        }
    }

    pub fn encode(
        &mut self,
        bins: &mut Bins,
        pwt: &PredWeightTable,
        sps: &SequenceParameterSet,
        pps: &PictureParameterSet,
        ph: &PictureHeader,
    ) {
        let ectx = self.encoder_context.lock().unwrap();
        print!("pwd.luma_log2_weight_denom ");
        self.coder
            .encode_unsigned_exp_golomb(bins, pwt.luma_log2_weight_denom as u64);
        if sps.chroma_format != ChromaFormat::Monochrome {
            print!("pwd.delta_chroma_log2_weight_denom ");
            self.coder
                .encode_signed_exp_golomb(bins, pwt.delta_chroma_log2_weight_denom as i64);
        }
        if pps.partition_parameters.wp_info_in_ph_flag {
            print!("pwd.num_l0_weights ");
            self.coder
                .encode_unsigned_exp_golomb(bins, pwt.num_l0_weights as u64);
        }
        for i in 0..ectx.num_weights_l0 {
            print!("pwd.luma_weight_l0_flag ");
            bins.push_bin(pwt.luma_weight_l0_flag[i]);
        }
        if sps.chroma_format != ChromaFormat::Monochrome {
            for i in 0..ectx.num_weights_l0 {
                print!("pwd.chroma_weight_l0_flag ");
                bins.push_bin(pwt.chroma_weight_l0_flag[i]);
            }
        }
        for i in 0..ectx.num_weights_l0 {
            if pwt.luma_weight_l0_flag[i] {
                print!("pwd.delta_luma_weight_l0 ");
                self.coder
                    .encode_signed_exp_golomb(bins, pwt.delta_luma_weight_l0[i] as i64);
                print!("pwd.luma_offset_l0 ");
                self.coder
                    .encode_signed_exp_golomb(bins, pwt.luma_offset_l0[i] as i64);
            }
            if pwt.chroma_weight_l0_flag[i] {
                for j in 0..2 {
                    print!("pwd.delta_chroma_weight_l0 ");
                    self.coder
                        .encode_signed_exp_golomb(bins, pwt.delta_chroma_weight_l0[i][j] as i64);
                    print!("pwd.delta_chroma_offset_l0 ");
                    self.coder
                        .encode_signed_exp_golomb(bins, pwt.delta_chroma_offset_l0[i][j] as i64);
                }
            }
        }
        if pps.weighted_bipred_flag
            && pps.partition_parameters.wp_info_in_ph_flag
            && ph.ref_pic_lists[1].ref_pic_list_structs[ectx.rpls_idx[1]].num_ref_entries > 0
        {
            print!("pwd.num_l1_weights ");
            self.coder
                .encode_unsigned_exp_golomb(bins, pwt.num_l1_weights as u64);
        }
        for i in 0..ectx.num_weights_l1 {
            print!("pwd.luma_weight_l1_flag ");
            bins.push_bin(pwt.luma_weight_l1_flag[i]);
        }
        if sps.chroma_format != ChromaFormat::Monochrome {
            for i in 0..ectx.num_weights_l1 {
                print!("pwd.chroma_weight_l1_flag ");
                bins.push_bin(pwt.chroma_weight_l1_flag[i]);
            }
        }
        for i in 0..ectx.num_weights_l1 {
            if pwt.luma_weight_l1_flag[i] {
                print!("pwd.delta_luma_weight_l1 ");
                self.coder
                    .encode_signed_exp_golomb(bins, pwt.delta_luma_weight_l1[i] as i64);
                print!("pwd.luma_offset_l1 ");
                self.coder
                    .encode_signed_exp_golomb(bins, pwt.luma_offset_l1[i] as i64);
            }
            if pwt.chroma_weight_l1_flag[i] {
                for j in 0..2 {
                    print!("pwd.delta_chroma_weight_l1 ");
                    self.coder
                        .encode_signed_exp_golomb(bins, pwt.delta_chroma_weight_l1[i][j] as i64);
                    print!("pwd.delta_chroma_offset_l1 ");
                    self.coder
                        .encode_signed_exp_golomb(bins, pwt.delta_chroma_offset_l1[i][j] as i64);
                }
            }
        }
    }
}

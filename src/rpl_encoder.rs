use super::bins::*;
use super::bool_coder::*;
use super::encoder_context::*;
use super::picture_header::*;
use super::pps::*;
use super::reference_picture::*;
use super::sps::*;
use debug_print::*;
use std::sync::{Arc, Mutex};

pub struct RefPicListStructEncoder<'a> {
    coder: &'a mut BoolCoder,
    encoder_context: Arc<Mutex<EncoderContext>>,
}

impl<'a> RefPicListStructEncoder<'a> {
    pub fn new(
        encoder_context: &Arc<Mutex<EncoderContext>>,
        coder: &'a mut BoolCoder,
    ) -> RefPicListStructEncoder<'a> {
        RefPicListStructEncoder {
            coder,
            encoder_context: encoder_context.clone(),
        }
    }

    pub fn encode(
        &mut self,
        bins: &mut Bins,
        ref_pic_lists: &[RefPicList; 2],
        sps: &SequenceParameterSet,
        pps: &PictureParameterSet,
        ph: &PictureHeader,
    ) {
        for (i, rpl) in ref_pic_lists.iter().enumerate().take(2) {
            if sps.ref_pic_lists[i].num_ref_pic_list > 0
                && (i == 0 || (i == 1 && pps.rpl1_idx_present_flag))
            {
                debug_eprint!("rpl ref_pic_lists_rpl_sps_flag ");
                bins.push_bin(rpl.rpl_sps_flag);
            }
            if ph.ref_pic_lists[i].rpl_sps_flag {
                if sps.ref_pic_lists[i].num_ref_pic_list > 1
                    && (i == 0 || (i == 1 && pps.rpl1_idx_present_flag))
                {
                    let n = (sps.ref_pic_lists[i].num_ref_pic_list as f64).log2().ceil() as usize;
                    debug_eprint!("rpl ref_pic_lists_rpl_idx ");
                    bins.push_bins_with_size(rpl.rpl_idx as u64, n);
                }
            } else {
                self.encode_rpls(
                    bins,
                    &ph.ref_pic_lists[i].ref_pic_list_structs
                        [sps.ref_pic_lists[i].num_ref_pic_list],
                    sps.ref_pic_lists[i].num_ref_pic_list,
                    sps.ref_pic_lists[i].num_ref_pic_list,
                    sps,
                );
            }
            let ectx = self.encoder_context.lock().unwrap();
            for j in 0..ectx.num_ltrp_entries[i][ectx.rpls_idx[i]] {
                if ph.ref_pic_lists[i].ref_pic_list_structs[ectx.rpls_idx[i]].ltrp_in_header_flag {
                    let n = sps.log2_max_pic_order_cnt_lsb;
                    debug_eprint!("rpl ref_pic_lists_poc_lsb_lt ");
                    bins.push_bins_with_size(rpl.poc_lsb_lt[j] as u64, n);
                }

                debug_eprint!("rpl ref_pic_lists_delta_poc_msb_cycle_present_flag ");
                bins.push_bin(ph.ref_pic_lists[i].delta_poc_msb_cycle_present_flag[j]);
                if ph.ref_pic_lists[i].delta_poc_msb_cycle_present_flag[j] {
                    debug_eprint!("rpl ref_pic_lists_delta_poc_msb_cycle_lt ");
                    self.coder
                        .encode_unsigned_exp_golomb(bins, rpl.delta_poc_msb_cycle_lt[j] as u64);
                }
            }
        }
    }

    pub fn encode_rpls(
        &mut self,
        bins: &mut Bins,
        ref_pic_list_struct: &RefPicListStruct,
        rpls_idx: usize,
        num_ref_pic_list: usize,
        sps: &SequenceParameterSet,
    ) {
        debug_eprint!("rpls num_ref_entries ");
        self.coder
            .encode_unsigned_exp_golomb(bins, ref_pic_list_struct.num_ref_entries as u64);
        if sps.long_term_ref_pics_flag
            && rpls_idx < num_ref_pic_list
            && ref_pic_list_struct.num_ref_entries > 0
        {
            debug_eprint!("rpls ltrp_in_header_flag ");
            bins.push_bin(ref_pic_list_struct.ltrp_in_header_flag);
        }
        let mut j = 0;
        for i in 0..ref_pic_list_struct.num_ref_entries {
            if sps.inter_layer_prediction_enabled_flag {
                debug_eprint!("rpls inter_layer_ref_pic_flag ");
                bins.push_bin(ref_pic_list_struct.inter_layer_ref_pic_flag[i]);
            }
            if !ref_pic_list_struct.inter_layer_ref_pic_flag[i] {
                if sps.long_term_ref_pics_flag {
                    debug_eprint!("rpls st_ref_pic_flag ");
                    bins.push_bin(ref_pic_list_struct.st_ref_pic_flag[i]);
                }
                if ref_pic_list_struct.st_ref_pic_flag[i] {
                    debug_eprint!("rpls abs_delta_poc_st ");
                    self.coder.encode_unsigned_exp_golomb(
                        bins,
                        ref_pic_list_struct.abs_delta_poc_st[i] as u64,
                    );
                    // FIXME set to encoder context
                    let abs_delta_poc_st =
                        if (sps.weighted_pred_flag || sps.weighted_bipred_flag) && i != 0 {
                            ref_pic_list_struct.abs_delta_poc_st[i]
                        } else {
                            ref_pic_list_struct.abs_delta_poc_st[i] + 1
                        };
                    if abs_delta_poc_st > 0 {
                        debug_eprint!("rpls strp_entry_sign_flag ");
                        bins.push_bin(ref_pic_list_struct.strp_entry_sign_flag[i]);
                    }
                } else if !ref_pic_list_struct.ltrp_in_header_flag {
                    // FIXME?
                    let _n = sps.log2_max_pic_order_cnt_lsb;
                    debug_eprint!("rpls rpls_poc_lsb_lt ");
                    bins.push_bins_with_size(ref_pic_list_struct.rpls_poc_lsb_lt[i] as u64, 4);
                    j += 1;
                }
            } else {
                debug_eprint!("rpls ilrp_idx ");
                self.coder
                    .encode_unsigned_exp_golomb(bins, ref_pic_list_struct.ilrp_idx[j] as u64);
            }
        }
    }
}

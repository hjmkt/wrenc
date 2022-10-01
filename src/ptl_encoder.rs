use super::bins::*;
use super::bool_coder::*;
use super::encoder_context::*;
use super::gci_encoder::*;
use super::ptl::*;
use debug_print::*;
use std::sync::{Arc, Mutex};

pub struct PtlEncoder<'a> {
    coder: &'a mut BoolCoder,
    encoder_context: Arc<Mutex<EncoderContext>>,
}

impl<'a> PtlEncoder<'a> {
    pub fn new(
        encoder_context: &Arc<Mutex<EncoderContext>>,
        coder: &'a mut BoolCoder,
    ) -> PtlEncoder<'a> {
        PtlEncoder {
            coder,
            encoder_context: encoder_context.clone(),
        }
    }

    pub fn encode(
        &mut self,
        bins: &mut Bins,
        ptl: &ProfileTierLevel,
        _profile_tier_present_flag: bool,
        max_num_sublayers: usize,
    ) {
        if ptl.pt_present_flags {
            debug_eprint!("ptl.general_profile_idc ");
            bins.push_bins_with_size(ptl.general_profile_idc as u64, 7);
            debug_eprint!("ptl.general_tier_flag ");
            bins.push_bin(ptl.general_tier_flag);
        }
        debug_eprint!("ptl.general_level_idc ");
        bins.push_bins_with_size(ptl.general_level_idc as u64, 8);
        debug_eprint!("ptl.ptl_frame_only_constraint_flag ");
        bins.push_bin(ptl.ptl_frame_only_constraint_flag);
        debug_eprint!("ptl.ptl_multilayer_enabled_flag ");
        bins.push_bin(ptl.ptl_multilayer_enabled_flag);
        if ptl.pt_present_flags {
            let ectx = self.encoder_context.clone();
            let mut gci_encoder = GCIEncoder::new(&ectx, self.coder);
            gci_encoder.encode(bins, &ptl.general_constraints_info);
        }
        // TODO
        for i in (0..max_num_sublayers - 1).rev() {
            debug_eprint!("ptl.sublayer_level_idc_present ");
            bins.push_bin(ptl.sub_layer_level_idcs[i].is_some());
        }
        bins.byte_align();
        for i in (0..max_num_sublayers - 1).rev() {
            if let Some(idc) = ptl.sub_layer_level_idcs[i] {
                debug_eprint!("ptl.sublayer_level_idc ");
                bins.push_bins_with_size(idc as u64, 8);
            }
        }
        if ptl.pt_present_flags {
            debug_eprint!("ptl.ptl_num_sub_profiles ");
            bins.push_bins_with_size(ptl.ptl_num_sub_profiles as u64, 8);
            for i in 0..ptl.ptl_num_sub_profiles {
                debug_eprint!("ptl.general_sub_profile_idc ");
                bins.push_bins_with_size(ptl.general_sub_profile_idcs[i].unwrap() as u64, 32);
            }
        }
    }
}

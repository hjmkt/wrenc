use super::bins::*;
use super::bool_coder::*;
use super::encoder_context::*;
use super::gci::*;
use debug_print::*;
use std::sync::{Arc, Mutex};

pub struct GCIEncoder<'a> {
    _coder: &'a mut BoolCoder,
    _encoder_context: Arc<Mutex<EncoderContext>>,
}

impl<'a> GCIEncoder<'a> {
    pub fn new(
        encoder_context: &Arc<Mutex<EncoderContext>>,
        _coder: &'a mut BoolCoder,
    ) -> GCIEncoder<'a> {
        GCIEncoder {
            _coder,
            _encoder_context: encoder_context.clone(),
        }
    }

    pub fn encode(&mut self, bins: &mut Bins, gci: &Option<GeneralConstraintsInfo>) {
        debug_eprint!("gci.present ");
        bins.push_bin(gci.is_some());
        if let Some(gci) = gci {
            // general
            bins.push_bin(gci.intra_only_constraint_flag);
            bins.push_bin(gci.all_layers_independent_constraint_flag);
            bins.push_bin(gci.one_au_only_constraint_flag);
            // picture format
            bins.push_bins_with_size(gci.sixteen_minus_max_bitdepth_constraint_idc as u64, 4);
            bins.push_bins_with_size(gci.three_minus_max_chroma_format_constraint_idc as u64, 2);
            // NAL unit type related
            bins.push_bin(gci.no_mixed_nalu_types_in_pic_constraint_flag);
            bins.push_bin(gci.no_trail_constraint_flag);
            bins.push_bin(gci.no_stsa_constraint_flag);
            bins.push_bin(gci.no_rasl_constraint_flag);
            bins.push_bin(gci.no_radl_constraint_flag);
            bins.push_bin(gci.no_idr_constraint_flag);
            bins.push_bin(gci.no_cra_constraint_flag);
            bins.push_bin(gci.no_gdr_constraint_flag);
            bins.push_bin(gci.no_aps_constraint_flag);
            bins.push_bin(gci.no_idr_rpl_constraint_flag);
            // tile, slice, subpicture partitioning
            bins.push_bin(gci.one_tile_per_pic_constraint_flag);
            bins.push_bin(gci.pic_header_in_slice_header_constraint_flag);
            bins.push_bin(gci.one_slice_per_pic_constraint_flag);
            bins.push_bin(gci.no_rectangular_slice_constraint_flag);
            bins.push_bin(gci.one_slice_per_subpic_constraint_flag);
            bins.push_bin(gci.no_subpic_info_constraint_flag);
            // CTU and block partitioning
            bins.push_bins_with_size(gci.three_minus_max_log2_ctu_size_constraint_idc as u64, 2);
            bins.push_bin(gci.no_partition_constraints_override_constraint_flag);
            bins.push_bin(gci.no_mtt_constraint_flag);
            bins.push_bin(gci.no_qtbtt_dual_tree_intra_constraint_flag);
            // intra
            bins.push_bin(gci.no_palette_constraint_flag);
            bins.push_bin(gci.no_ibc_constraint_flag);
            bins.push_bin(gci.no_isp_constraint_flag);
            bins.push_bin(gci.no_mrl_constraint_flag);
            bins.push_bin(gci.no_mip_constraint_flag);
            bins.push_bin(gci.no_cclm_constraint_flag);
            // inter
            bins.push_bin(gci.no_ref_pic_resampling_constraint_flag);
            bins.push_bin(gci.no_res_change_in_clvs_constraint_flag);
            bins.push_bin(gci.no_weighted_prediction_constraint_flag);
            bins.push_bin(gci.no_ref_wraparound_constraint_flag);
            bins.push_bin(gci.no_temporal_mvp_constraint_flag);
            bins.push_bin(gci.no_sbtmvp_constraint_flag);
            bins.push_bin(gci.no_amvr_constraint_flag);
            bins.push_bin(gci.no_bdof_constraint_flag);
            bins.push_bin(gci.no_smvd_constraint_flag);
            bins.push_bin(gci.no_dmvr_constraint_flag);
            bins.push_bin(gci.no_mmvd_constraint_flag);
            bins.push_bin(gci.no_affine_motion_constraint_flag);
            bins.push_bin(gci.no_prof_constraint_flag);
            bins.push_bin(gci.no_bcw_constraint_flag);
            bins.push_bin(gci.no_ciip_constraint_flag);
            bins.push_bin(gci.no_gpm_constraint_flag);
            // transform, quantization, residual
            bins.push_bin(gci.no_luma_transform_size_64_constraint_flag);
            bins.push_bin(gci.no_transform_skip_constraint_flag);
            bins.push_bin(gci.no_bdpcm_constraint_flag);
            bins.push_bin(gci.no_mts_constraint_flag);
            bins.push_bin(gci.no_lfnst_constraint_flag);
            bins.push_bin(gci.no_joint_cbcr_constraint_flag);
            bins.push_bin(gci.no_sbt_constraint_flag);
            bins.push_bin(gci.no_act_constraint_flag);
            bins.push_bin(gci.no_explicit_scaling_list_constraint_flag);
            bins.push_bin(gci.no_dep_quant_constraint_flag);
            bins.push_bin(gci.no_sign_data_hiding_constraint_flag);
            bins.push_bin(gci.no_cu_qp_delta_constraint_flag);
            bins.push_bin(gci.no_chroma_qp_offset_constraint_flag);
            // loop filter
            bins.push_bin(gci.no_sao_constraint_flag);
            bins.push_bin(gci.no_alf_constraint_flag);
            bins.push_bin(gci.no_ccalf_constraint_flag);
            bins.push_bin(gci.no_lmcs_constraint_flag);
            bins.push_bin(gci.no_ladf_constraint_flag);
            bins.push_bin(gci.no_virtual_boundaries_constraint_flag);
            bins.push_bins_with_size(gci.num_reserved_bits as u64, 8);
            for _ in 0..gci.num_reserved_bits {
                let reserved_zero_bit = false;
                bins.push_bin(reserved_zero_bit);
            }
        }
        bins.byte_align();
    }
}

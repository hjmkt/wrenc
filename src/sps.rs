use super::common::*;
use super::dpb::*;
use super::partition::*;
use super::ptl::*;
use super::reference_picture::*;
use super::timing_hrd::*;
use super::virtual_boundary::*;

pub struct SpsSubpicInfo {
    pub num_subpics: usize,
    pub independent_subpics_flag: bool,
    pub subpic_same_size_flag: bool,
    pub subpic_ctu_top_left_xs: Vec<usize>,
    pub subpic_ctu_top_left_ys: Vec<usize>,
    pub subpic_widths: Vec<usize>,
    pub subpic_heights: Vec<usize>,
    pub subpic_treated_as_pic_flags: Vec<bool>,
    pub loop_filter_across_subpic_enabled_flags: Vec<bool>,
    pub subpic_id_len: usize,
    pub subpic_id_mapping_explicitly_signalled_flag: bool,
    pub subpic_id_mapping_present_flag: bool,
    pub subpic_id: Vec<usize>,
}

pub struct QpTable {
    pub qp_table_start: isize,
    pub num_points_in_qp_table: usize,
    pub delta_qp_in_val: Vec<isize>,
    pub delta_qp_diff_val: Vec<isize>,
}

// TODO optimize chroma qp table
impl QpTable {
    pub fn new(bit_depth: usize, num_points: usize, qp_start: isize) -> QpTable {
        // FIXME
        let _qp_bd_offset = 6 * (bit_depth - 8);
        // [-qp_bd_offset, ..., 26, ..., 63]
        //let chroma_qps = [
        //0, 1, 2, 3, 4, 5, 6, 7, 8, 9, //
        //10, 11, 12, 13, 14, 15, 16, 17, 18, 19, //
        //20, 21, 22, 23, 24, 25, 26, 27, 28, 29, //
        //30, 31, 32, 33, 34, 35, 36, 37, 38, 39, //
        //40, 41, 42, 43, 44, 45, 46, 47, 48, 49, //
        //50, 51, 52, 53, 54, 55, 56, 57, 58, 59, //
        //60, 61, 62, 63,
        //];
        let delta_qp_in_val = vec![1; (63 - qp_start) as usize];
        let delta_qp_diff_val = vec![1; (63 - qp_start) as usize];
        QpTable {
            qp_table_start: qp_start,
            num_points_in_qp_table: num_points,
            delta_qp_in_val,
            delta_qp_diff_val,
        }
    }
}

pub struct LadfParameters {
    pub num_ladf_intervals: usize,
    pub lowest_interval_qp_offset: isize,
    pub qp_offset: Vec<isize>,
    pub delta_threshold: Vec<usize>,
}

pub struct SequenceParameterSet {
    /// an identifier for the SPS for reference by other syntax elements.
    pub id: usize,
    /// when greater than 0, specifies the value of vps_video_parameter_set_id for the VPS referred to by the SPS.
    /// When sps_video_parameter_set_id is equal to 0, the following applies:
    ///   – The SPS does not refer to a VPS, and no VPS is referred to when decoding each CLVS referring to the SPS.
    ///   – The value of vps_max_layers_minus1 is inferred to be equal to 0.
    ///   – The CVS shall contain only one layer (i.e., all VCL NAL unit in the CVS shall have the same value of nuh_layer_id).
    ///   – The value of GeneralLayerIdx[ nuh_layer_id ] is set equal to 0.
    ///   – The value of vps_independent_layer_flag[ GeneralLayerIdx[ nuh_layer_id ] ] is inferred to be equal to 1.
    ///   – The value of TotalNumOlss is set equal to 1, the value of NumLayersInOls[ 0 ] is set equal to 1, and value of vps_layer_id[ 0 ] is inferred to be equal to the value of nuh_layer_id of all the VCL NAL units, and the value of LayerIdInOls[ 0 ][ 0 ] is set equal to vps_layer_id[ 0 ].
    pub video_parameter_set_id: usize,
    /// the maximum number of temporal sublayers that could be present in each CLVS referring to the SPS.
    /// If sps_video_parameter_set_id is greater than 0, the value of sps_max_sublayers_minus1 shall be in the range of 0 to vps_max_sublayers_minus1, inclusive.
    /// Otherwise (sps_video_parameter_set_id is equal to 0), the following applies:
    ///   – The value of sps_max_sublayers_minus1 shall be in the range of 0 to 6, inclusive.
    ///   – The value of vps_max_sublayers_minus1 is inferred to be equal to sps_max_sublayers: usize,
    ///   – The value of NumSubLayersInLayerInOLS[ 0 ][ 0 ] is inferred to be equal to sps_max_sublayers_minus1 + 1.
    ///   – The value of vps_ols_ptl_idx[ 0 ] is inferred to be equal to 0, and the value of vps_ptl_max_tid[ vps_ols_ptl_idx[ 0 ] ], i.e., vps_ptl_max_tid[ 0 ], is inferred to be equal to sps_max_sublayers_minus1.
    pub max_sublayers: usize,
    /// the chroma sampling relative to the luma sampling as specified in clause 6.2.
    pub chroma_format: ChromaFormat,
    pub log2_ctu_size: usize,
    pub ptl_dpb_hrd_params_present_flag: bool,
    pub profile_tier_level: Option<ProfileTierLevel>,
    /// equal to 1 specifies that GDR pictures are enabled and could be present in the CLVS. sps_gdr_enabled_flag equal to 0 specifies that GDR pictures are disabled and not present in the CLVS.
    pub gdr_enabled_flag: bool,
    pub ref_pic_resampling_enabled_flag: bool,
    pub res_change_in_clvs_allowed_flag: bool,
    pub pic_width_max_in_luma_samples: usize,
    pub pic_height_max_in_luma_samples: usize,
    pub conformance_window: Option<WindowOffset>,
    pub subpic_info: Option<SpsSubpicInfo>,
    pub bitdepth: usize,
    /// equal to 1 specifies that a specific synchronization process for context variables is invoked before decoding the CTU that includes the first CTB of a row of CTBs in each tile in each picture referring to the SPS, and a specific storage process for context variables is invoked after decoding the CTU that includes the first CTB of a row of CTBs in each tile in each picture referring to the SPS. sps_entropy_coding_sync_enabled_flag equal to 0 specifies that no specific synchronization process for context variables is required to be invoked before decoding the CTU that includes the first CTB of a row of CTBs in each tile in each picture referring to the SPS, and no specific storage process for context variables is required to be invoked after decoding the CTU that includes the first CTB of a row of CTBs in each tile in each picture referring to the SPS.
    pub entropy_coding_sync_enabled_flag: bool,
    /// equal to 1 specifies that signalling for entry point offsets for tiles or tile-specific CTU rows could be present in the slice headers of pictures referring to the SPS. sps_entry_point_offsets_present_flag equal to 0 specifies that signalling for entry point offsets for tiles or tile-specific CTU rows are not present in the slice headers of pictures referring to the SPS.
    pub entry_point_offsets_present_flag: bool,
    pub log2_max_pic_order_cnt_lsb: usize, // [4, 16]
    pub poc_msb_cycle_flag: bool,
    /// the length, in bits, of the ph_poc_msb_cycle_val syntax elements, when present in PH syntax structures referring to the SPS. The value of sps_poc_msb_cycle_len_minus1 shall be in the range of 0 to 32 − sps_log2_max_pic_order_cnt_lsb_minus4 − 5, inclusive.
    pub poc_msb_cycle_len: usize,
    /// the number of bytes of extra bits in the PH syntax structure for coded pictures referring to the SPS. The value of sps_num_extra_ph_bytes shall be equal to 0 in bitstreams conforming to this version of this Specification. Although the value of sps_num_extra_ph_bytes is required to be equal to 0 in this version of this Specification, decoders conforming to this version of this Specification shall allow the value of sps_num_extra_ph_bytes equal to 1 or 2 to appear in the syntax.
    pub num_extra_ph_bytes: usize,
    /// [ i ] equal to 1 specifies that the i-th extra bit is present in PH syntax structures referring to the SPS. sps_extra_ph_bit_present_flag[ i ] equal to 0 specifies that the i-th extra bit is not present in PH syntax structures referring to the SPS.
    pub extra_ph_bit_present_flags: Vec<bool>,
    /// the number of bytes of extra bits in the slice headers for coded pictures referring to the SPS. The value of sps_num_extra_sh_bytes shall be equal to 0 in bitstreams conforming to this version of this Specification. Although the value of sps_num_extra_sh_bytes is required to be equal to 0 in this version of this Specification, decoders conforming to this version of this Specification shall allow the value of sps_num_extra_sh_bytes equal to 1 or 2 to appear in the syntax.
    pub num_extra_sh_bytes: usize,
    /// [ i ] equal to 1 specifies that the i-th extra bit is present in the slice headers of pictures referring to the SPS. sps_extra_sh_bit_present_flag[ i ] equal to 0 specifies that the i-th extra bit is not present in the slice headers of pictures referring to the SPS.
    pub extra_sh_bit_present_flags: Vec<bool>,
    /// is used to control the presence of dpb_max_dec_pic_buffering_minus1[ i ], dpb_max_num_reorder_pics[ i ], and dpb_max_latency_increase_plus1[ i ] syntax elements in the dpb_parameters( ) syntax strucure in the SPS for i in range from 0 to sps_max_sublayers_minus1 − 1, inclusive, when sps_max_sublayers_minus1 is greater than 0. When not present, the value of sps_sublayer_dpb_params_flag is inferred to be equal to 0.
    pub sublayer_dpb_params_flag: bool,
    pub dpb_parameters: Vec<DpbParameter>,
    pub log2_min_luma_coding_block_size: usize,
    /// equal to 1 specifies the presence of ph_partition_constraints_ override_flag in PH syntax structures referring to the SPS. sps_partition_constraints_override_enabled_flag equal to 0 specifies the absence of ph_partition_constraints_override_flag in PH syntax structures referring to the SPS.
    pub partition_constraints_override_enabled_flag: bool,
    pub partition_constraints: PartitionConstraints,
    /// equal to 1 specifies that the maximum transform size in luma samples is equal to 64. sps_max_luma_transform_size_64_flag equal to 0 specifies that the maximum transform size in luma samples is equal to 32. When not present, the value of sps_max_luma_transform_size_64_flag is inferred to be equal to 0.
    pub max_luma_transform_size_64_flag: bool,
    /// equal to 1 specifies that transform_skip_flag could be present in the transform unit syntax. sps_transform_skip_enabled_flag equal to 0 specifies that transform_skip_flag is not present in the transform unit syntax.
    pub transform_skip_enabled_flag: bool,
    /// the maximum block size used for transform skip, and shall be in the range of 2 to 5, inclusive.
    pub log2_transform_skip_max_size: usize,
    /// equal to 1 specifies that intra_bdpcm_luma_flag and intra_bdpcm_chroma_flag could be present in the coding unit syntax for intra coding units. sps_bdpcm_enabled_flag equal to 0 specifies that intra_bdpcm_luma_flag and intra_bdpcm_chroma_flag are not present in the coding unit syntax for intra coding units. When not present, the value of sps_bdpcm_enabled_flag is inferred to be equal to 0.
    pub bdpcm_enabled_flag: bool,
    /// equal to 1 specifies that sps_explicit_mts_intra_enabled_flag and sps_explicit_mts_inter_enabled_flag are present in the SPS. sps_mts_enabled_flag equal to 0 specifies that sps_explicit_mts_intra_enabled_flag and sps_explicit_mts_inter_enabled_flag are not present in the SPS.
    pub mts_enabled_flag: bool,
    /// equal to 1 specifies that mts_idx could be present in the intra coding unit syntax of the CLVS. sps_explicit_mts_intra_enabled_flag equal to 0 specifies that mts_idx is not present in the intra coding unit syntax of the CLVS. When not present, the value of sps_explicit_mts_intra_enabled_flag is inferred to be equal to 0.
    pub explicit_mts_intra_enabled_flag: bool,
    /// equal to 1 specifies that mts_idx could be present in the inter coding unit syntax of the CLVS. sps_explicit_mts_inter_enabled_flag equal to 0 specifies that mts_idx is not present in the inter coding unit syntax of the CLVS. When not present, the value of sps_explicit_mts_inter_enabled_flag is inferred to be equal to 0.
    pub explicit_mts_inter_enabled_flag: bool,
    /// equal to 1 specifies that lfnst_idx could be present in intra coding unit syntax. sps_lfnst_enabled_flag equal to 0 specifies that lfnst_idx is not present in intra coding unit syntax.
    pub lfnst_enabled_flag: bool,
    /// equal to 1 specifies that the joint coding of chroma residuals is enabled for the CLVS. sps_joint_cbcr_enabled_flag equal to 0 specifies that the joint coding of chroma residuals is disabled for the CLVS. When not present, the value of sps_joint_cbcr_enabled_flag is inferred to be equal to 0.
    pub joint_cbcr_enabled_flag: bool,
    /// equal to 1 specifies that only one chroma QP mapping table is signalled and this table applies to Cb and Cr residuals and additionally to joint Cb-Cr residuals when sps_joint_cbcr_enabled_flag is equal to 1. sps_same_qp_table_for_chroma_flag equal to 0 specifies that chroma QP mapping tables, two for Cb and Cr, and one additional for joint Cb-Cr when sps_joint_cbcr_enabled_flag is equal to 1, are signalled in the SPS. When not present, the value of sps_same_qp_table_for_chroma_flag is inferred to be equal to 1.
    pub same_qp_table_for_chroma_flag: bool,
    pub num_qp_tables: usize,
    pub qp_tables: Vec<QpTable>,
    pub sao_enabled_flag: bool,
    pub alf_enabled_flag: bool,
    pub ccalf_enabled_flag: bool,
    pub lmcs_enabled_flag: bool,
    pub weighted_pred_flag: bool,
    pub weighted_bipred_flag: bool,
    pub long_term_ref_pics_flag: bool,
    /// equal to 1 specifies that inter-layer prediction is enabled for the CLVS and ILRPs might be used for inter prediction of one or more coded pictures in the CLVS. sps_inter_layer_prediction_enabled_flag equal to 0 specifies that inter-layer prediction is disabled for the CLVS and no ILRP is used for inter prediction of any coded picture in the CLVS. When sps_video_parameter_set_id is equal to 0, the value of sps_inter_layer_prediction_enabled_flag is inferred to be equal to 0. When vps_independent_layer_flag[ GeneralLayerIdx[ nuh_layer_id ] ] is equal to 1, the value of sps_inter_layer_prediction_enabled_flag shall be equal to 0.
    pub inter_layer_prediction_enabled_flag: bool,
    /// equal to 1 specifies that RPL syntax elements could be present in slice headers of slices with nal_unit_type equal to IDR_N_LP or IDR_W_RADL. sps_idr_rpl_present_flag equal to 0 specifies that RPL syntax elements are not present in slice headers of slices with nal_unit_type equal to IDR_N_LP or IDR_W_RADL.
    pub idr_rpl_present_flag: bool,
    pub rpl1_same_as_rpl0_flag: bool,
    pub ref_pic_lists: Vec<RefPicList>,
    /// equal to 1 specifies that horizontal wrap-around motion compensation is enabled for the CLVS. sps_ref_wraparound_enabled_flag equal to 0 specifies that horizontal wrap-around motion compensation is disabled for the CLVS.
    pub ref_wraparound_enabled_flag: bool,
    pub temporal_mvp_enabled_flag: bool,
    /// equal to 1 specifies that subblock-based temporal motion vector predictors are enabled and might be used in decoding of pictures with all slices having sh_slice_type not equal to I in the CLVS. sps_sbtmvp_enabled_flag equal to 0 specifies that subblock-based temporal motion vector predictors are disabled and not used in decoding of pictures in the CLVS. When sps_sbtmvp_enabled_flag is not present, it is inferred to be equal to 0.
    pub sbtmvp_enabled_flag: bool,
    /// equal to 1 specifies that adaptive motion vector difference resolution is enabled for the CVLS. amvr_enabled_flag equal to 0 specifies that adaptive motion vector difference resolution is disabled for the CLVS.
    pub amvr_enabled_flag: bool,
    /// equal to 1 specifies that the bi-directional optical flow inter prediction is enabled for the CLVS. sps_bdof_enabled_flag equal to 0 specifies that the bi-directional optical flow inter prediction is disabled for the CLVS.
    pub bdof_enabled_flag: bool,
    pub bdof_control_present_in_ph_flag: bool,
    /// equal to 1 specifies that symmetric motion vector difference is enabled for the CLVS. sps_smvd_enabled_flag equal to 0 specifies that symmetric motion vector difference is disabled for the CLVS.
    pub smvd_enabled_flag: bool,
    /// equal to 1 specifies that decoder motion vector refinement based inter bi-prediction is enabled for the CLVS. sps_dmvr_enabled_flag equal to 0 specifies that decoder motion vector refinement based inter bi-prediction is disabled for the CLVS.
    pub dmvr_enabled_flag: bool,
    pub dmvr_control_present_in_ph_flag: bool,
    /// equal to 1 specifies that merge mode with motion vector difference is enabled for the CLVS. sps_mmvd_enabled_flag equal to 0 specifies that merge mode with motion vector difference is disabled for the CLVS.
    pub mmvd_enabled_flag: bool,
    pub mmvd_fullpel_only_enabled_flag: bool,
    pub six_minus_max_num_merge_cand: usize,
    /// equal to 1 specifies that subblock transform for inter-predicted CUs is enabled for the CLVS. sps_sbt_enabled_flag equal to 0 specifies that subblock transform for inter-predicted CUs is disabled for the CLVS.
    pub sbt_enabled_flag: bool,
    pub affine_enabled_flag: bool,
    pub five_minus_max_num_subblock_merge_cand: usize,
    pub six_param_affine_enabled_flag: bool,
    pub affine_amvr_enabled_flag: bool,
    pub affine_prof_enabled_flag: bool,
    pub prof_control_present_in_ph_flag: bool,
    /// equal to 1 specifies that bi-prediction with CU weights is enabled for the CLVS and bcw_idx could be present in the coding unit syntax of the CLVS. sps_bcw_enabled_flag equal to 0 specifies that bi-prediction with CU weights is disabled for the CLVS and bcw_idx is not present in the coding unit syntax of the CLVS.
    pub bcw_enabled_flag: bool,
    pub ciip_enabled_flag: bool,
    /// equal to 1 specifies that the geometric partition based motion compensation is enabled for the CLVS and merge_gpm_partition_idx, merge_gpm_idx0, and merge_gpm_idx1 could be present in the coding unit syntax of the CLVS. sps_gpm_enabled_flag equal to 0 specifies that the geometric partition based motion compensation is disabled for the CLVS and merge_gpm_partition_idx, merge_gpm_idx0, and merge_gpm_idx1 are not present in the coding unit syntax of the CLVS. When not present, the value of sps_gpm_enabled_flag is inferred to be equal to 0.
    pub gpm_enabled_flag: bool,
    pub max_num_merge_cand_minus_max_num_gpm_cand: usize,
    pub log2_parallel_merge_level: usize,
    /// equal to 1 specifies that intra prediction with subpartitions is enabled for the CLVS. sps_isp_enabled_flag equal to 0 specifies that intra prediction with subpartitions is disabled for the CLVS.
    pub isp_enabled_flag: bool,
    /// equal to 1 specifies that intra prediction with multiple reference lines is enabled for the CLVS. sps_mrl_enabled_flag equal to 0 specifies that intra prediction with multiple reference lines is disabled for the CLVS.
    pub mrl_enabled_flag: bool,
    /// equal to 1 specifies that the matrix-based intra prediction is enabled for the CLVS. sps_mip_enabled_flag equal to 0 specifies that the matrix-based intra prediction is disabled for the CLVS.
    pub mip_enabled_flag: bool,
    /// equal to 1 specifies that the cross-component linear model intra prediction from luma component to chroma component is enabled for the CLVS. sps_cclm_enabled_flag equal to 0 specifies that the cross-component linear model intra prediction from luma component to chroma component is disabled for the CLVS. When sps_cclm_enabled_flag is not present, it is inferred to be equal to 0.
    pub cclm_enabled_flag: bool,
    pub chroma_horizontal_collocated_flag: bool,
    pub chroma_vertical_collocated_flag: bool,
    pub palette_enabled_flag: bool,
    /// equal to 1 specifies that the adaptive colour transform is enabled for the CLVS and the cu_act_enabled_flag could be present in the coding unit syntax of the CLVS. sps_act_enabled_flag equal to 0 speifies that the adaptive colour transform is disabled for the CLVS and cu_act_enabled_flag is not present in the coding unit syntax of the CLVS. When sps_act_enabled_flag is not present, it is inferred to be equal to 0.
    pub act_enabled_flag: bool,
    pub min_qp_prime_ts: usize,
    /// equal to 1 specifies that the IBC prediction mode is enabled for the CLVS. sps_ibc_enabled_flag equal to 0 specifies that the IBC prediction mode is disabled for the CLVS. When sps_ibc_enabled_flag is not present, it is inferred to be equal to 0.
    pub ibc_enabled_flag: bool,
    pub six_minus_max_num_ibc_merge_cand: usize,
    pub ladf_parameters: Option<LadfParameters>,
    pub explicit_scaling_list_enabled_flag: bool,
    pub scaling_matrix_for_lfnst_disabled_flag: bool,
    pub scaling_matrix_for_alternative_colour_space_disabled_flag: bool,
    pub scaling_matrix_designated_colour_space_flag: bool,
    pub dep_quant_enabled_flag: bool,
    pub sign_data_hiding_enabled_flag: bool,
    pub virtual_boundaries_enabled_flag: bool,
    pub virtual_boundary_parameters: VirtualBoundaryParameters,
    pub timing_hrd_params_present_flag: bool,
    pub general_timing_hrd_parameters: Option<GeneralTimingHrdParameters>,
    pub sublayer_cpb_params_present_flag: bool,
    pub ols_timing_hrd_parameters: Vec<OlsTimingHrdParameter>,
    pub field_seq_flag: bool,
    pub vui_parameters_present_flag: bool,
    pub vui_payload_size: usize,
    pub vui_payload: Vec<bool>,
    pub extension_data: Vec<bool>,
}

impl SequenceParameterSet {
    pub fn new(
        id: usize,
        video_parameter_set_id: usize,
        picture_width: usize,
        picture_height: usize,
        bit_depth: usize,
    ) -> SequenceParameterSet {
        SequenceParameterSet {
            id,
            video_parameter_set_id,
            max_sublayers: 1,
            chroma_format: ChromaFormat::YCbCr420,
            log2_ctu_size: 5,
            ptl_dpb_hrd_params_present_flag: true,
            profile_tier_level: Some(ProfileTierLevel::new(true)),
            gdr_enabled_flag: false,
            ref_pic_resampling_enabled_flag: false,
            res_change_in_clvs_allowed_flag: false,
            pic_width_max_in_luma_samples: picture_width,
            pic_height_max_in_luma_samples: picture_height,
            conformance_window: None,
            subpic_info: None,
            bitdepth: 8,
            entropy_coding_sync_enabled_flag: false,
            entry_point_offsets_present_flag: false,
            log2_max_pic_order_cnt_lsb: 4,
            poc_msb_cycle_flag: false,
            poc_msb_cycle_len: 0,
            num_extra_ph_bytes: 0,
            extra_ph_bit_present_flags: vec![],
            num_extra_sh_bytes: 0,
            extra_sh_bit_present_flags: vec![],
            sublayer_dpb_params_flag: false,
            dpb_parameters: vec![DpbParameter::new()],
            log2_min_luma_coding_block_size: 2,
            partition_constraints_override_enabled_flag: false,
            partition_constraints: PartitionConstraints::new(),
            max_luma_transform_size_64_flag: false,
            transform_skip_enabled_flag: true,
            log2_transform_skip_max_size: 5,
            bdpcm_enabled_flag: false,
            mts_enabled_flag: true,
            explicit_mts_intra_enabled_flag: true,
            explicit_mts_inter_enabled_flag: true,
            lfnst_enabled_flag: false,
            joint_cbcr_enabled_flag: false,
            same_qp_table_for_chroma_flag: true,
            num_qp_tables: 3,
            qp_tables: vec![
                QpTable::new(bit_depth, 63, 0),
                QpTable::new(bit_depth, 63, 0),
                QpTable::new(bit_depth, 63, 0),
            ],
            sao_enabled_flag: false,
            alf_enabled_flag: false,
            ccalf_enabled_flag: false,
            lmcs_enabled_flag: false,
            weighted_pred_flag: false,
            weighted_bipred_flag: false,
            long_term_ref_pics_flag: false,
            inter_layer_prediction_enabled_flag: false,
            idr_rpl_present_flag: false,
            rpl1_same_as_rpl0_flag: false,
            ref_pic_lists: vec![RefPicList::new(0), RefPicList::new(1)],
            ref_wraparound_enabled_flag: false,
            temporal_mvp_enabled_flag: false,
            sbtmvp_enabled_flag: false,
            amvr_enabled_flag: false,
            bdof_enabled_flag: false,
            bdof_control_present_in_ph_flag: false,
            smvd_enabled_flag: false,
            dmvr_enabled_flag: false,
            dmvr_control_present_in_ph_flag: false,
            mmvd_enabled_flag: false,
            mmvd_fullpel_only_enabled_flag: false,
            six_minus_max_num_merge_cand: 0,
            sbt_enabled_flag: false,
            affine_enabled_flag: false,
            five_minus_max_num_subblock_merge_cand: 0,
            six_param_affine_enabled_flag: false,
            affine_amvr_enabled_flag: false,
            affine_prof_enabled_flag: false,
            prof_control_present_in_ph_flag: false,
            bcw_enabled_flag: false,
            ciip_enabled_flag: false,
            gpm_enabled_flag: false,
            max_num_merge_cand_minus_max_num_gpm_cand: 0,
            log2_parallel_merge_level: 2,
            isp_enabled_flag: false,
            mrl_enabled_flag: false,
            mip_enabled_flag: false,
            cclm_enabled_flag: true,
            chroma_horizontal_collocated_flag: false,
            chroma_vertical_collocated_flag: false,
            palette_enabled_flag: false,
            act_enabled_flag: false,
            min_qp_prime_ts: 0,
            ibc_enabled_flag: false,
            six_minus_max_num_ibc_merge_cand: 0,
            ladf_parameters: None,
            explicit_scaling_list_enabled_flag: false,
            scaling_matrix_for_lfnst_disabled_flag: false,
            scaling_matrix_for_alternative_colour_space_disabled_flag: false,
            scaling_matrix_designated_colour_space_flag: false,
            dep_quant_enabled_flag: false,
            sign_data_hiding_enabled_flag: false,
            virtual_boundaries_enabled_flag: false,
            virtual_boundary_parameters: VirtualBoundaryParameters::new(),
            timing_hrd_params_present_flag: false,
            general_timing_hrd_parameters: None,
            sublayer_cpb_params_present_flag: false,
            ols_timing_hrd_parameters: vec![],
            field_seq_flag: false,
            vui_parameters_present_flag: false,
            vui_payload_size: 0,
            vui_payload: vec![],
            extension_data: vec![],
        }
    }
}

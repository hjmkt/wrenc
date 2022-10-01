pub struct GeneralConstraintsInfo {
    // general
    pub intra_only_constraint_flag: bool,
    pub all_layers_independent_constraint_flag: bool,
    pub one_au_only_constraint_flag: bool,
    // picture format
    pub sixteen_minus_max_bitdepth_constraint_idc: usize,
    pub three_minus_max_chroma_format_constraint_idc: usize,
    // NAL unit type related
    pub no_mixed_nalu_types_in_pic_constraint_flag: bool,
    pub no_trail_constraint_flag: bool,
    pub no_stsa_constraint_flag: bool,
    pub no_rasl_constraint_flag: bool,
    pub no_radl_constraint_flag: bool,
    pub no_idr_constraint_flag: bool,
    pub no_cra_constraint_flag: bool,
    pub no_gdr_constraint_flag: bool,
    pub no_aps_constraint_flag: bool,
    pub no_idr_rpl_constraint_flag: bool,
    // tile, slice, subpicture partitioning
    pub one_tile_per_pic_constraint_flag: bool,
    pub pic_header_in_slice_header_constraint_flag: bool,
    pub one_slice_per_pic_constraint_flag: bool,
    pub no_rectangular_slice_constraint_flag: bool,
    pub one_slice_per_subpic_constraint_flag: bool,
    pub no_subpic_info_constraint_flag: bool,
    // CTU and block partitioning
    pub three_minus_max_log2_ctu_size_constraint_idc: usize,
    pub no_partition_constraints_override_constraint_flag: bool,
    pub no_mtt_constraint_flag: bool,
    pub no_qtbtt_dual_tree_intra_constraint_flag: bool,
    // intra
    pub no_palette_constraint_flag: bool,
    pub no_ibc_constraint_flag: bool,
    pub no_isp_constraint_flag: bool,
    pub no_mrl_constraint_flag: bool,
    pub no_mip_constraint_flag: bool,
    pub no_cclm_constraint_flag: bool,
    // inter
    pub no_ref_pic_resampling_constraint_flag: bool,
    pub no_res_change_in_clvs_constraint_flag: bool,
    pub no_weighted_prediction_constraint_flag: bool,
    pub no_ref_wraparound_constraint_flag: bool,
    pub no_temporal_mvp_constraint_flag: bool,
    pub no_sbtmvp_constraint_flag: bool,
    pub no_amvr_constraint_flag: bool,
    pub no_bdof_constraint_flag: bool,
    pub no_smvd_constraint_flag: bool,
    pub no_dmvr_constraint_flag: bool,
    pub no_mmvd_constraint_flag: bool,
    pub no_affine_motion_constraint_flag: bool,
    pub no_prof_constraint_flag: bool,
    pub no_bcw_constraint_flag: bool,
    pub no_ciip_constraint_flag: bool,
    pub no_gpm_constraint_flag: bool,
    // transform, quantization, residual
    pub no_luma_transform_size_64_constraint_flag: bool,
    pub no_transform_skip_constraint_flag: bool,
    pub no_bdpcm_constraint_flag: bool,
    pub no_mts_constraint_flag: bool,
    pub no_lfnst_constraint_flag: bool,
    pub no_joint_cbcr_constraint_flag: bool,
    pub no_sbt_constraint_flag: bool,
    pub no_act_constraint_flag: bool,
    pub no_explicit_scaling_list_constraint_flag: bool,
    pub no_dep_quant_constraint_flag: bool,
    pub no_sign_data_hiding_constraint_flag: bool,
    pub no_cu_qp_delta_constraint_flag: bool,
    pub no_chroma_qp_offset_constraint_flag: bool,
    // loop filter
    pub no_sao_constraint_flag: bool,
    pub no_alf_constraint_flag: bool,
    pub no_ccalf_constraint_flag: bool,
    pub no_lmcs_constraint_flag: bool,
    pub no_ladf_constraint_flag: bool,
    pub no_virtual_boundaries_constraint_flag: bool,
    pub num_reserved_bits: usize,
}

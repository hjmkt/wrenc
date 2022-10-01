use super::common::*;
use super::sps::*;

pub struct PpsSlice {
    pub slice_width_in_tiles: usize,
    pub slice_height_in_tiles: usize,
    pub num_exp_slices_in_tile: usize,
    pub exp_slice_height_in_ctus: Vec<usize>,
    pub tile_idx_delta_val: isize,
}

pub struct PartitionParameters {
    pub loop_filter_across_tiles_enabled_flag: bool,
    pub rect_slice_flag: bool,
    pub single_slice_per_subpic_flag: bool,
    pub num_slices_in_pic: usize,
    pub tile_idx_delta_present_flag: bool,
    pub slices: Vec<PpsSlice>,
    pub loop_filter_across_slices_enabled_flag: bool,
    pub rpl_info_in_ph_flag: bool,
    pub sao_info_in_ph_flag: bool,
    pub alf_info_in_ph_flag: bool,
    pub wp_info_in_ph_flag: bool,
    pub qp_delta_info_in_ph_flag: bool,
}

impl PartitionParameters {
    pub fn new() -> PartitionParameters {
        PartitionParameters {
            loop_filter_across_tiles_enabled_flag: false,
            rect_slice_flag: false,
            single_slice_per_subpic_flag: false,
            num_slices_in_pic: 0,
            tile_idx_delta_present_flag: false,
            slices: vec![],
            loop_filter_across_slices_enabled_flag: false,
            rpl_info_in_ph_flag: false,
            sao_info_in_ph_flag: false,
            alf_info_in_ph_flag: false,
            wp_info_in_ph_flag: false,
            qp_delta_info_in_ph_flag: false,
        }
    }
}

pub struct PpsChromaToolOffsets {
    pub cb_qp_offset: isize,
    pub cr_qp_offset: isize,
    pub joint_cbcr_qp_offset_present_flag: bool,
    pub joint_cbcr_qp_offset_value: isize,
    pub slice_chroma_qp_offsets_present_flag: bool,
    pub cu_chroma_qp_offset_list_enabled_flag: bool,
    pub chroma_qp_offset_list_len: usize,
    pub cb_qp_offset_list: Vec<isize>,
    pub cr_qp_offset_list: Vec<isize>,
    pub joint_cbcr_qp_offset_list: Vec<isize>,
}

impl PpsChromaToolOffsets {
    pub fn new() -> PpsChromaToolOffsets {
        PpsChromaToolOffsets {
            cb_qp_offset: 0,
            cr_qp_offset: 0,
            joint_cbcr_qp_offset_present_flag: false,
            joint_cbcr_qp_offset_value: 0,
            slice_chroma_qp_offsets_present_flag: false,
            cu_chroma_qp_offset_list_enabled_flag: false,
            chroma_qp_offset_list_len: 0,
            cb_qp_offset_list: vec![],
            cr_qp_offset_list: vec![],
            joint_cbcr_qp_offset_list: vec![],
        }
    }
}

pub struct PpsDeblockingFilterControl {
    pub deblocking_filter_override_enabled_flag: bool,
    pub deblocking_filter_disabled_flag: bool,
    pub dbf_info_in_ph_flag: bool,
    pub luma_beta_offset: isize,
    pub luma_tc_offset: isize,
    pub cb_beta_offset: isize,
    pub cb_tc_offset: isize,
    pub cr_beta_offset: isize,
    pub cr_tc_offset: isize,
}

impl PpsDeblockingFilterControl {
    pub fn new() -> PpsDeblockingFilterControl {
        PpsDeblockingFilterControl {
            deblocking_filter_override_enabled_flag: false,
            deblocking_filter_disabled_flag: true,
            dbf_info_in_ph_flag: false,
            luma_beta_offset: 0,
            luma_tc_offset: 0,
            cb_beta_offset: 0,
            cb_tc_offset: 0,
            cr_beta_offset: 0,
            cr_tc_offset: 0,
        }
    }
}

pub struct PictureParameterSet {
    pub id: usize,
    pub seq_parameter_set_id: usize,
    /// equal to 1 specifies that each picture referring to the PPS has more than one VCL NAL unit and the VCL NAL units do not have the same value of nal_unit_type. pps_mixed_nalu_types_in_pic_flag equal to 0 specifies that each picture referring to the PPS has one or more VCL NAL units and the VCL NAL units of each picture refering to the PPS have the same value of nal_unit_type.
    pub mixed_nalu_types_in_pic_flag: bool,
    pub pic_width_in_luma_samples: usize,
    pub pic_height_in_luma_samples: usize,
    pub conformance_window: Option<WindowOffset>,
    pub scaling_window_explicit_signalling_flag: bool,
    pub scaling_window: WindowOffset,
    /// equal to 1 specifies that the ph_pic_output_flag syntax element could be present in PH syntax structures referring to the PPS. pps_output_flag_present_flag equal to 0 specifies that the ph_pic_output_flag syntax element is not present in PH syntax structures referring to the PPS.
    pub output_flag_present_flag: bool,
    /// equal to 1 specifies that no picture partitioning is applied to each picture referring to the PPS. pps_no_pic_partition_flag equal to 0 specifies that each picture referring to the PPS might be partitioned into more than one tile or slice.
    pub no_pic_partition_flag: bool,
    /// equal to 1 specifies that the subpicture ID mapping is signalled in the PPS. pps_subpic_id_mapping_present_flag equal to 0 specifies that the subpicture ID mapping is not signalled in the PPS. If sps_subpic_id_mapping_explicitly_signalled_flag is 0 or sps_subpic_id_mapping_present_flag is equal to 1, the value of pps_subpic_id_mapping_present_flag shall be equal to 0. Otherwise (sps_subpic_id_mapping_explicitly_signalled_flag is equal to 1 and sps_subpic_id_mapping_present_flag is equal to 0), the value of pps_subpic_id_mapping_present_flag shall be equal to 1.
    pub subpic_id_mapping_present_flag: bool,
    pub num_subpics: usize,
    pub subpic_id_len: usize,
    pub subpic_id: Vec<usize>,
    pub log2_ctu_size: usize,
    pub num_exp_tile_columns: usize,
    pub num_exp_tile_rows: usize,
    pub tile_column_widths: Vec<usize>,
    pub tile_column_heights: Vec<usize>,
    pub partition_parameters: PartitionParameters,
    pub cabac_init_present_flag: bool,
    /// when i is equal to 0, specifies the inferred value of the variable NumRefIdxActive[ 0 ] for P or B slices with sh_num_ref_idx_active_override_flag equal to 0, and, when i is equal to 1, specifies the inferred value of NumRefIdxActive[ 1 ] for B slices with sh_num_ref_idx_active_override_flag equal to 0. The value of pps_num_ref_idx_default_active_minus1[ i ] shall be in the range of 0 to 14, inclusive.
    pub num_ref_idx_default_active: [usize; 2],
    pub rpl1_idx_present_flag: bool,
    pub weighted_pred_flag: bool,
    pub weighted_bipred_flag: bool,
    pub ref_wraparound_enabled_flag: bool,
    pub pic_width_minus_wraparound_offset: usize,
    /// the initial value of SliceQp Y for each slice referring to the PPS. The initial value of SliceQp Y is modified at the picture level when a non-zero value of ph_qp_delta is decoded or at the slice level when a non-zero value of sh_qp_delta is decoded. The value of pps_init_qp_minus26 shall be in the range of −( 26 + QpBdOffset ) to +37, inclusive.
    pub init_qp: isize,
    pub cu_qp_delta_enabled_flag: bool,
    pub chroma_tool_offsets_present_flag: bool,
    pub chroma_tool_offsets: PpsChromaToolOffsets,
    pub deblocking_filter_control_present_flag: bool,
    pub deblocking_filter_control: PpsDeblockingFilterControl,
    pub picture_header_extension_present_flag: bool,
    pub slice_header_extension_present_flag: bool,
    pub extension_data: Vec<bool>,
}

impl PictureParameterSet {
    pub fn new(
        id: usize,
        sps: &SequenceParameterSet,
        fixed_qp: Option<isize>,
    ) -> PictureParameterSet {
        PictureParameterSet {
            id,
            seq_parameter_set_id: sps.id,
            mixed_nalu_types_in_pic_flag: false,
            pic_width_in_luma_samples: sps.pic_width_max_in_luma_samples,
            pic_height_in_luma_samples: sps.pic_height_max_in_luma_samples,
            conformance_window: None,
            scaling_window_explicit_signalling_flag: false,
            scaling_window: WindowOffset::new(),
            output_flag_present_flag: false,
            no_pic_partition_flag: true,
            subpic_id_mapping_present_flag: false,
            num_subpics: 0,
            subpic_id_len: 0,
            subpic_id: vec![],
            log2_ctu_size: 0,
            num_exp_tile_columns: 0,
            num_exp_tile_rows: 0,
            tile_column_widths: vec![],
            tile_column_heights: vec![],
            partition_parameters: PartitionParameters::new(),
            cabac_init_present_flag: false,
            num_ref_idx_default_active: [3, 3],
            rpl1_idx_present_flag: false,
            weighted_pred_flag: false,
            weighted_bipred_flag: false,
            ref_wraparound_enabled_flag: false,
            pic_width_minus_wraparound_offset: 0,
            init_qp: if let Some(qp) = fixed_qp {
                qp.max(26)
            } else {
                26
            },
            cu_qp_delta_enabled_flag: true,
            chroma_tool_offsets_present_flag: false,
            chroma_tool_offsets: PpsChromaToolOffsets::new(),
            deblocking_filter_control_present_flag: true,
            deblocking_filter_control: PpsDeblockingFilterControl::new(),
            picture_header_extension_present_flag: false,
            slice_header_extension_present_flag: false,
            extension_data: vec![],
        }
    }
}

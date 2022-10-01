use super::partition::*;
use super::pps::*;
use super::pred_weight_table::*;
use super::reference_picture::*;
use super::virtual_boundary::*;

pub struct AlfInfo {
    pub num_alf_aps_ids_luma: usize,
    pub aps_id_luma: Vec<usize>,
    pub cb_enabled_flag: bool,
    pub cr_enabled_flag: bool,
    pub aps_id_chroma: usize,
    pub cc_cb_enabled_flag: bool,
    pub cc_cb_aps_id: usize,
    pub cc_cr_enabled_flag: bool,
    pub cc_cr_aps_id: usize,
}

impl AlfInfo {
    pub fn new() -> AlfInfo {
        AlfInfo {
            num_alf_aps_ids_luma: 0,
            aps_id_luma: vec![],
            cb_enabled_flag: false,
            cr_enabled_flag: false,
            aps_id_chroma: 0,
            cc_cb_enabled_flag: false,
            cc_cb_aps_id: 0,
            cc_cr_enabled_flag: false,
            cc_cr_aps_id: 0,
        }
    }
}

pub struct PictureHeader {
    /// equal to 1 specifies that the current picture is a GDR or IRAP picture. ph_gdr_or_irap_pic_flag equal to 0 specifies that the current picture is not a GDR picture and might or might not be an IRAP picture.
    pub gdr_or_irap_pic_flag: bool,
    /// equal to 1 specifies that the current picture is never used as a reference picture. ph_non_ref_pic_flag equal to 0 specifies that the current picture might or might not be used as a reference picture.
    pub non_ref_pic_flag: bool,
    /// equal to 1 specifies that the current picture is a GDR picture. ph_gdr_pic_flag equal to 0 specifies that the current picture is not a GDR picture. When not present, the value of ph_gdr_pic_flag is inferred to be equal to 0. When sps_gdr_enabled_flag is equal to 0, the value of ph_gdr_pic_flag shall be equal to 0.
    pub gdr_pic_flag: bool,
    pub inter_slice_allowed_flag: bool,
    pub intra_slice_allowed_flag: bool,
    pub pic_parameter_set_id: usize,
    /// the picture order count modulo MaxPicOrderCntLsb for the current picture. The length of the ph_pic_order_cnt_lsb syntax element is sps_log2_max_pic_order_cnt_lsb_minus4 + 4 bits. The value of the ph_pic_order_cnt_lsb shall be in the range of 0 to MaxPicOrderCntLsb − 1, inclusive.
    pub pic_order_cnt_lsb: usize,
    pub recovery_poc_cnt: usize,
    pub extra_bit: Vec<bool>,
    pub poc_msb_cycle_val: usize,
    pub alf_enabled_flag: bool,
    pub alf_info: AlfInfo,
    pub lmcs_enabled_flag: bool,
    pub lmcs_aps_id: usize,
    pub chroma_residual_scale_flag: bool,
    pub explicit_scaling_list_enabled_flag: bool,
    pub scaling_list_aps_id: usize,
    pub virtual_boundary: VirtualBoundaryParameters,
    pub pic_output_flag: bool,
    pub ref_pic_lists: [RefPicList; 2],
    pub partition_constraints_override_flag: bool,
    pub partition_constraints: Option<PartitionConstraints>,
    pub cu_qp_delta_subdiv_inter_slice: usize,
    pub cu_chroma_qp_offset_subdiv_inter_slice: usize,
    pub cu_qp_delta_subdiv_intra_slice: usize,
    pub cu_chroma_qp_offset_subdiv_intra_slice: usize,
    pub temporal_mvp_enabled_flag: bool,
    pub collocated_from_l0_flag: bool,
    pub collocated_ref_idx: usize,
    pub mmvd_fullpel_only_flag: bool,
    pub mvd_l1_zero_flag: bool,
    pub bdof_disabled_flag: bool,
    pub dmvr_disabled_flag: bool,
    pub prof_disabled_flag: bool,
    pub pred_weight_table: Option<PredWeightTable>,
    pub qp_delta: isize,
    pub joint_cbcr_sign_flag: bool,
    pub sao_luma_enabled_flag: bool,
    pub sao_chroma_enabled_flag: bool,
    pub deblocking_params_present_flag: bool,
    pub deblocking_filter_disabled_flag: bool,
    pub luma_beta_offset: isize,
    pub luma_tc_offset: isize,
    pub cb_beta_offset: isize,
    pub cb_tc_offset: isize,
    pub cr_beta_offset: isize,
    pub cr_tc_offset: isize,
    pub extension_data_byte: Vec<usize>,
}

impl PictureHeader {
    pub fn new(pps: &PictureParameterSet, intra: bool, poc: usize) -> PictureHeader {
        PictureHeader {
            gdr_or_irap_pic_flag: true,
            non_ref_pic_flag: false,
            gdr_pic_flag: false,
            inter_slice_allowed_flag: !intra,
            intra_slice_allowed_flag: true,
            pic_parameter_set_id: pps.id,
            pic_order_cnt_lsb: poc & 0b1111, // FIXME
            recovery_poc_cnt: 0,
            extra_bit: vec![],
            poc_msb_cycle_val: 0,
            alf_enabled_flag: false,
            alf_info: AlfInfo::new(),
            lmcs_enabled_flag: false,
            lmcs_aps_id: 0,
            chroma_residual_scale_flag: false,
            explicit_scaling_list_enabled_flag: false,
            scaling_list_aps_id: 0,
            virtual_boundary: VirtualBoundaryParameters::new(),
            pic_output_flag: true,
            ref_pic_lists: [RefPicList::new(0), RefPicList::new(1)],
            partition_constraints_override_flag: false,
            partition_constraints: None,
            cu_qp_delta_subdiv_inter_slice: 0,
            cu_chroma_qp_offset_subdiv_inter_slice: 0,
            cu_qp_delta_subdiv_intra_slice: 0,
            cu_chroma_qp_offset_subdiv_intra_slice: 0,
            temporal_mvp_enabled_flag: false,
            collocated_from_l0_flag: false,
            collocated_ref_idx: 0,
            mmvd_fullpel_only_flag: false,
            mvd_l1_zero_flag: false,
            bdof_disabled_flag: true,
            dmvr_disabled_flag: true,
            prof_disabled_flag: true,
            pred_weight_table: None,
            qp_delta: 0,
            joint_cbcr_sign_flag: false,
            sao_luma_enabled_flag: false,
            sao_chroma_enabled_flag: false,
            deblocking_params_present_flag: false,
            deblocking_filter_disabled_flag: true,
            luma_beta_offset: 0,
            luma_tc_offset: 0,
            cb_beta_offset: 0,
            cb_tc_offset: 0,
            cr_beta_offset: 0,
            cr_tc_offset: 0,
            extension_data_byte: vec![],
        }
    }
}

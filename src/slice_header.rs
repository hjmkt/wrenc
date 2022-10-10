use super::aps::*;
use super::encoder_context::*;
use super::picture_header::*;
use super::pps::*;
use super::pred_weight_table::*;
use super::reference_picture::*;
use super::sps::*;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum SliceType {
    B = 0,
    P = 1,
    I = 2,
}

pub struct SliceHeader<'a> {
    pub sps: &'a SequenceParameterSet,
    pub pps: &'a PictureParameterSet,
    pub aps: [&'a AdaptationParameterSet; 3],
    pub ph: Option<&'a PictureHeader>,
    pub ph_in_sh: Option<PictureHeader>,
    pub subpic_id: usize,
    pub slice_address: usize,
    pub extra_bit: Vec<bool>,
    pub num_tiles_in_slice: usize,
    pub slice_type: SliceType,
    pub no_output_of_prior_pics_flag: bool,
    pub alf_enabled_flag: bool,
    pub alf_info: AlfInfo,
    pub lmcs_used_flag: bool,
    pub explicit_scaling_list_used_flag: bool,
    pub ref_pic_lists: [RefPicList; 2],
    pub num_ref_idx_active_override_flag: bool,
    pub num_ref_idx_active: Vec<usize>,
    pub cabac_init_flag: bool,
    pub collocated_from_l0_flag: bool,
    pub collocated_ref_idx: usize,
    pub pred_weight_table: Option<PredWeightTable>,
    pub qp_delta: isize,
    pub cb_qp_offset: isize,
    pub cr_qp_offset: isize,
    pub joint_cbcr_qp_offset: isize,
    pub cu_chroma_qp_offset_enabled_flag: bool,
    pub sao_luma_used_flag: bool,
    pub sao_chroma_used_flag: bool,
    pub deblocking_params_present_flag: bool,
    pub deblocking_filter_disabled_flag: bool,
    pub luma_beta_offset: isize,
    pub luma_tc_offset: isize,
    pub cb_beta_offset: isize,
    pub cb_tc_offset: isize,
    pub cr_beta_offset: isize,
    pub cr_tc_offset: isize,
    pub dep_quant_used_flag: bool,
    pub sign_data_hiding_used_flag: bool,
    pub ts_residual_coding_disabled_flag: bool,
    pub slice_header_extension_length: usize,
    pub slice_header_extension_data_byte: Vec<usize>,
    pub entry_offset_len: usize,
    pub entry_point_offset: Vec<usize>,
}

impl<'a, 'b: 'a> SliceHeader<'a> {
    pub fn new(
        sps: &'b SequenceParameterSet,
        pps: &'b PictureParameterSet,
        aps: [&'b AdaptationParameterSet; 3],
        ph: Option<&'b PictureHeader>,
        fixed_qp: Option<isize>,
        ectx: &EncoderContext,
    ) -> SliceHeader<'a> {
        SliceHeader {
            sps,
            pps,
            aps,
            ph,
            ph_in_sh: None,
            subpic_id: 0,
            slice_address: 0,
            extra_bit: vec![],
            num_tiles_in_slice: 1,
            slice_type: SliceType::I,
            no_output_of_prior_pics_flag: false,
            alf_enabled_flag: false,
            alf_info: AlfInfo::new(),
            lmcs_used_flag: false,
            explicit_scaling_list_used_flag: false,
            ref_pic_lists: [RefPicList::new(0), RefPicList::new(1)],
            num_ref_idx_active_override_flag: false,
            num_ref_idx_active: vec![],
            cabac_init_flag: false,
            collocated_from_l0_flag: false,
            collocated_ref_idx: 0,
            pred_weight_table: None,
            qp_delta: if let Some(qp) = fixed_qp {
                qp - ectx.slice_qp_y
            } else {
                0
            },
            //qp_delta: 0,
            cb_qp_offset: 0,
            cr_qp_offset: 0,
            joint_cbcr_qp_offset: 0,
            cu_chroma_qp_offset_enabled_flag: false,
            sao_luma_used_flag: false,
            sao_chroma_used_flag: false,
            deblocking_params_present_flag: false,
            deblocking_filter_disabled_flag: true,
            luma_beta_offset: 0,
            luma_tc_offset: 0,
            cb_beta_offset: 0,
            cb_tc_offset: 0,
            cr_beta_offset: 0,
            cr_tc_offset: 0,
            dep_quant_used_flag: true,
            sign_data_hiding_used_flag: false,
            ts_residual_coding_disabled_flag: false,
            slice_header_extension_length: 0,
            slice_header_extension_data_byte: vec![],
            entry_offset_len: 0,
            entry_point_offset: vec![],
        }
    }
}

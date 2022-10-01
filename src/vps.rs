use super::common::*;
use super::dpb::*;
use super::ptl::*;
use super::timing_hrd::*;

pub struct VpsLayer {
    /// the nuh_layer_id value of the i-th layer. For any two non-negative integer values of m and n, when m is less than n, the value of vps_layer_id[ m ] shall be less than vps_layer_id[ n ].
    pub id: usize,
    /// equal to 1 specifies that the layer with index i does not use inter-layer prediction. vps_independent_layer_flag[ i ] equal to 0 specifies that the layer with index i might use inter-layer prediction and the syntax elements vps_direct_ref_layer_flag[ i ][ j ] for j in the range of 0 to i − 1, inclusive, are present in the VPS. When not present, the value of vps_independent_layer_flag[ i ] is inferred to be equal to 1.
    pub is_independent_layer: bool,
    /// [ i ][ j ] equal to 0 specifies that the pictures of the j-th layer that are neither IRAP pictures nor GDR pictures with ph_recovery_poc_cnt equal to 0 are not used as ILRPs for decoding of pictures of the i-th layer. vps_max_tid_il_ref_pics_plus1[ i ][ j ] greater than 0 specifies that, for decoding pictures of the i-th layer, no picture from the j-th layer with TemporalId greater than vps_max_tid_il_ref_pics_plus1[ i ][ j ] − 1 is used as ILRP and no APS with nuh_layer_id equal to vps_layer_id[ j ] and TemporalId greater than vps_max_tid_il_ref_pics_plus1[ i ][ j ] − 1 is referenced. When not present, the value of vps_max_tid_il_ref_pics_plus1[ i ][ j ] is inferred to be equal to vps_max_sublayers_minus1 + 1.
    pub max_tid_il_ref_pics: Vec<usize>,
    pub max_tid_ref_present_flag: bool,
    /// [ i ][ j ] equal to 0 specifies that the layer with index j is not a direct reference layer for the layer with index i. vps_direct_ref_layer_flag [ i ][ j ] equal to 1 specifies that the layer with index j is a direct reference layer for the layer with index i. When vps_direct_ref_layer_flag[ i ][ j ] is not present for i and j in the range of 0 to vps_max_layers_minus1, inclusive, it is inferred to be equal to 0. When vps_independent_layer_flag[ i ] is equal to 0, there shall be at least one value of j in the range of 0 to i − 1, inclusive, such that the value of vps_direct_ref_layer_flag[ i ][ j ] is equal to 1.
    pub direct_ref_layer_flag: Vec<bool>,
}

impl VpsLayer {
    pub fn new(id: usize) -> VpsLayer {
        VpsLayer {
            id,
            is_independent_layer: true,
            max_tid_il_ref_pics: vec![0],
            max_tid_ref_present_flag: false,
            direct_ref_layer_flag: vec![false],
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)]
pub enum OlsMode {
    /// the total number of OLSs specified by the VPS is equal to vps_max_layers_minus1 + 1, the i-th OLS includes the layers with layer indices from 0 to i, inclusive, and for each OLS only the highest layer in the OLS is an output layer.{
    Highest = 0,
    /// the total number of OLSs specified by the VPS is equal to vps_max_layers_minus1 + 1, the i-th OLS includes the layers with layer indices from 0 to i, inclusive, and for each OLS all layers in the OLS are output layers.
    All = 1,
    /// the total number of OLSs specified by the VPS is explicitly signalled and for each OLS the output layers are explicitly signalled and other layers are the layers that are direct or indirect reference layers of the output layers of the OLS.
    Explicit = 2,
}

pub struct VideoParameterSet {
    /// an identifier for the VPS for reference by other syntax elements.
    pub id: usize,
    /// the number of layers specified by the VPS, which is the maximum allowed number of layers in each CVS referring to the VPS.
    pub max_layers: usize,
    /// the maximum number of temporal sublayers that may be present in a layer specified by the VPS. The value of vps_max_sublayers_minus1 shall be in the range of 0 to 6, inclusive.
    pub max_sublayers: usize,
    pub layers: Vec<VpsLayer>,
    /// equal to 1 specifies that each OLS specified by the VPS contains only one layer and each layer specified by the VPS is an OLS with the single included layer being the only output layer. vps_each_layer_is_an_ols_flag equal to 0 specifies that at least one OLS specified by the VPS contains more than one layer. If vps_max_layers_minus1 is equal to 0, the value of vps_each_layer_is_an_ols_flag is inferred to be equal to 1. Otherwise, when vps_all_independent_layers_flag is equal to 0, the value of vps_each_layer_is_an_ols_flag is inferred to be equal to 0.
    pub each_layer_is_an_ols: bool,
    pub ols_mode: OlsMode,
    /// the total number of OLSs specified by the VPS when vps_ols_mode_idc is equal to 2.
    pub num_output_layer_sets: usize,
    /// the TemporalId of the highest sublayer representation for which the level information is present in the i-th profile_tier_level( ) syntax structure in the VPS and the TemporalId of the highest sublayer representation that is present in the OLSs with OLS index olsIdx such that vps_ols_ptl_idx[ olsIdx ] is equal to i. The value of vps_ptl_max_tid[ i ] shall be in the range of 0 to vps_max_sublayers_minus1, inclusive. When vps_default_ptl_dpb_hrd_max_tid_flag is equal to 1, the value of vps_ptl_max_tid[ i ] is inferred to be equal to vps_max_sublayers_minus1.
    pub ptl_max_tids: Vec<usize>,
    /// the index, to the list of profile_tier_level( ) syntax structures in the VPS, of the profile_tier_level( ) syntax structure that applies to the i-th OLS. When present, the value of vps_ols_ptl_idx[ i ] shall be in the range of 0 to vps_num_ptls_minus1, inclusive.
    pub ols_ptl_idx: Vec<usize>,
    /// [ i ][ j ] equal to 1 specifies that the layer with nuh_layer_id equal to vps_layer_id[ j ] is an output layer of the i-th OLS when vps_ols_mode_idc is equal to 2. vps_ols_output_layer_flag[ i ][ j ] equal to 0 specifies that the layer with nuh_layer_id equal to vps_layer_id[ j ] is not an output layer of the i-th OLS when vps_ols_mode_idc is equal to 2.
    pub ols_output_layer_flags: Vec<Vec<bool>>,
    /// the number of profile_tier_level( ) syntax structures in the VPS. The value of vps_num_ptls_minus1 shall be less than TotalNumOlss. When not present, the value of vps_num_ptls_minus1 is inferred to be equal to 0.
    pub num_ptls: usize,
    /// equal to 1 specifies that the syntax elements vps_ptl_max_tid[ i ], vps_dpb_max_tid[ i ], and vps_hrd_max_tid[ i ] are not present and are inferred to be equal to the default value vps_max_sublayers_minus1. vps_default_ptl_dpb_hrd_max_tid_flag equal to 0 specifies that the syntax elements vps_ptl_max_tid[ i ], vps_dpb_max_tid[ i ], and vps_hrd_max_tid[ i ] are present. When not present, the value of vps_default_ptl_dpb_hrd_max_tid_flag is inferred to be equal to 1.
    pub default_ptl_dpb_hrd_max_tid_flag: bool,
    pub profile_tier_levels: Vec<ProfileTierLevel>,
    pub general_timing_hrd_parameters: Option<GeneralTimingHrdParameters>,
    pub sublayer_dpb_params_present_flag: bool,
    pub dpb_parameters: Vec<DpbParameter>,
    pub ols_dpb_parameters: Vec<OlsDpbParameter>,
    pub sublayer_cpb_params_present_flag: bool,
    pub num_ols_timing_hrd_params: usize,
    pub hrd_max_tids: Vec<usize>,
    pub ols_timing_hrd_parameters: Vec<OlsTimingHrdParameter>,
    pub ols_timing_hrd_idxs: Vec<usize>,
    pub extension_data: Vec<bool>,
}

impl VideoParameterSet {
    pub fn new(
        id: usize,
        width: usize,
        height: usize,
        bitdepth: usize,
        chroma_format: ChromaFormat,
    ) -> VideoParameterSet {
        VideoParameterSet {
            id,
            max_layers: 1,
            max_sublayers: 1,
            layers: vec![VpsLayer::new(9), VpsLayer::new(10)],
            each_layer_is_an_ols: false,
            ols_mode: OlsMode::All,
            num_output_layer_sets: 2,
            ptl_max_tids: vec![1, 2],
            ols_ptl_idx: vec![0, 1],
            ols_output_layer_flags: vec![vec![true, true], vec![true, true]],
            num_ptls: 1,
            default_ptl_dpb_hrd_max_tid_flag: true,
            profile_tier_levels: vec![ProfileTierLevel::new(true), ProfileTierLevel::new(false)],
            general_timing_hrd_parameters: None,
            sublayer_dpb_params_present_flag: false,
            dpb_parameters: vec![DpbParameter::new()],
            ols_dpb_parameters: vec![OlsDpbParameter::new(
                width,
                height,
                chroma_format,
                bitdepth,
                1,
            )],
            sublayer_cpb_params_present_flag: false,
            num_ols_timing_hrd_params: 0,
            hrd_max_tids: vec![],
            ols_timing_hrd_parameters: vec![],
            ols_timing_hrd_idxs: vec![],
            extension_data: vec![],
        }
    }

    pub fn validate(&self) {
        assert!(self.id > 0);
    }

    /// equal to 1 specifies that all layers specified by the VPS are independently coded without using inter-layer prediction. vps_all_independent_layers_flag equal to 0 specifies that one or more of the layers specified by the VPS might use inter-layer prediction. When not present, the value of vps_all_independent_layers_flag is inferred to be equal to 1.
    pub fn all_layers_are_independent(&self) -> bool {
        self.layers.iter().any(|layer| layer.is_independent_layer)
    }

    pub fn get_ols_mode_idc(&self) -> usize {
        self.ols_mode as usize
    }
}

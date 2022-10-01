use super::gci::*;

pub struct ProfileTierLevel {
    pub pt_present_flags: bool,
    pub ptl_max_tids: Vec<Option<usize>>,
    pub general_level_idc: usize,
    pub ptl_frame_only_constraint_flag: bool,
    pub ptl_multilayer_enabled_flag: bool,
    pub general_profile_idc: usize,
    pub general_tier_flag: bool,
    pub general_constraints_info: Option<GeneralConstraintsInfo>,
    pub ptl_num_sub_profiles: usize,
    pub general_sub_profile_idcs: Vec<Option<usize>>,
    pub sub_layer_level_idcs: Vec<Option<usize>>,
}

impl ProfileTierLevel {
    pub fn new(pt_present_flags: bool) -> ProfileTierLevel {
        ProfileTierLevel {
            pt_present_flags,
            ptl_max_tids: vec![],
            general_level_idc: 0,
            ptl_frame_only_constraint_flag: false,
            ptl_multilayer_enabled_flag: false,
            general_profile_idc: 0,
            general_tier_flag: false,
            general_constraints_info: None,
            ptl_num_sub_profiles: 0,
            general_sub_profile_idcs: vec![],
            sub_layer_level_idcs: vec![],
        }
    }
}

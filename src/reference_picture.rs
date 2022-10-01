pub struct RefPicListStruct {
    pub num_ref_entries: usize,
    pub ltrp_in_header_flag: bool,
    pub inter_layer_ref_pic_flag: Vec<bool>,
    pub st_ref_pic_flag: Vec<bool>,
    pub abs_delta_poc_st: Vec<usize>,
    pub strp_entry_sign_flag: Vec<bool>,
    pub rpls_poc_lsb_lt: Vec<usize>,
    pub ilrp_idx: Vec<usize>,
}

impl RefPicListStruct {
    pub fn new(lx: usize) -> RefPicListStruct {
        RefPicListStruct {
            num_ref_entries: 3,
            ltrp_in_header_flag: false,
            inter_layer_ref_pic_flag: vec![false; 3],
            st_ref_pic_flag: vec![true; 3],
            abs_delta_poc_st: vec![0, 2, 3],
            strp_entry_sign_flag: vec![lx == 0; 3],
            rpls_poc_lsb_lt: vec![0; 3],
            ilrp_idx: vec![0; 3],
        }
    }
}

pub struct RefPicList {
    pub num_ref_pic_list: usize,
    pub rpl_sps_flag: bool,
    pub rpl_idx: usize,
    pub ref_pic_list_structs: Vec<RefPicListStruct>,
    pub poc_lsb_lt: Vec<usize>,
    pub delta_poc_msb_cycle_present_flag: Vec<bool>,
    pub delta_poc_msb_cycle_lt: Vec<usize>,
}

impl RefPicList {
    pub fn new(lx: usize) -> RefPicList {
        RefPicList {
            num_ref_pic_list: 1,
            rpl_sps_flag: true,
            rpl_idx: 0,
            ref_pic_list_structs: vec![RefPicListStruct::new(lx)],
            poc_lsb_lt: vec![],
            delta_poc_msb_cycle_present_flag: vec![false],
            delta_poc_msb_cycle_lt: vec![0],
        }
    }
}

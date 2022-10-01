pub struct PartitionConstraints {
    pub log2_diff_min_qt_min_cb_intra_slice_luma: usize,
    pub max_mtt_hierarchy_depth_intra_slice_luma: usize,
    pub log2_diff_max_bt_min_qt_intra_slice_luma: usize,
    pub log2_diff_max_tt_min_qt_intra_slice_luma: usize,
    pub log2_diff_min_qt_min_cb_intra_slice_chroma: usize,
    pub max_mtt_hierarchy_depth_intra_slice_chroma: usize,
    pub log2_diff_max_bt_min_qt_intra_slice_chroma: usize,
    pub log2_diff_max_tt_min_qt_intra_slice_chroma: usize,
    pub qtbtt_dual_tree_intra_flag: bool,
    pub cu_qp_delta_subdiv_intra_slice: usize,
    pub cu_qp_delta_subdiv_inter_slice: usize,
    pub cu_chroma_qp_offset_subdiv_intra_slice: usize,
    pub cu_chroma_qp_offset_subdiv_inter_slice: usize,
    pub log2_diff_min_qt_min_cb_inter_slice: usize,
    pub max_mtt_hierarchy_depth_inter_slice: usize,
    pub log2_diff_max_bt_min_qt_inter_slice: usize,
    pub log2_diff_max_tt_min_qt_inter_slice: usize,
}

impl PartitionConstraints {
    pub fn new() -> PartitionConstraints {
        PartitionConstraints {
            log2_diff_min_qt_min_cb_intra_slice_luma: 0,
            max_mtt_hierarchy_depth_intra_slice_luma: 0,
            log2_diff_max_bt_min_qt_intra_slice_luma: 0,
            log2_diff_max_tt_min_qt_intra_slice_luma: 0,
            log2_diff_min_qt_min_cb_intra_slice_chroma: 0,
            max_mtt_hierarchy_depth_intra_slice_chroma: 0,
            log2_diff_max_bt_min_qt_intra_slice_chroma: 0,
            log2_diff_max_tt_min_qt_intra_slice_chroma: 0,
            qtbtt_dual_tree_intra_flag: false,
            cu_qp_delta_subdiv_intra_slice: 0,
            cu_qp_delta_subdiv_inter_slice: 0,
            cu_chroma_qp_offset_subdiv_intra_slice: 0,
            cu_chroma_qp_offset_subdiv_inter_slice: 0,
            log2_diff_min_qt_min_cb_inter_slice: 0,
            max_mtt_hierarchy_depth_inter_slice: 0,
            log2_diff_max_bt_min_qt_inter_slice: 0,
            log2_diff_max_tt_min_qt_inter_slice: 0,
        }
    }
}

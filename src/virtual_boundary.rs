pub struct VirtualBoundaryParameters {
    pub virtual_boundaries_present_flag: bool,
    pub num_ver_virtual_boundaries: usize,
    pub virtual_boundary_pos_xs: Vec<usize>,
    pub num_hor_virtual_boundaries: usize,
    pub virtual_boundary_pos_ys: Vec<usize>,
}

impl VirtualBoundaryParameters {
    pub fn new() -> VirtualBoundaryParameters {
        VirtualBoundaryParameters {
            virtual_boundaries_present_flag: false,
            num_ver_virtual_boundaries: 0,
            virtual_boundary_pos_xs: vec![],
            num_hor_virtual_boundaries: 0,
            virtual_boundary_pos_ys: vec![],
        }
    }
}

use super::picture::*;

pub trait SubpictureSplitter {
    fn get_subpicture_slice_index_groups(self, picture: &Picture) -> Vec<Vec<usize>>;
}

pub struct UnitSubpictureSplitter {}

impl SubpictureSplitter for UnitSubpictureSplitter {
    fn get_subpicture_slice_index_groups(self, picture: &Picture) -> Vec<Vec<usize>> {
        let slices = picture.slices.lock().unwrap();
        let slice_indices = (0..slices.len()).collect();
        vec![slice_indices]
    }
}

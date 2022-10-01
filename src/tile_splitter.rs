use super::picture::*;

pub trait TileSplitter {
    fn get_ctu_cols_and_rows(self, picture: &Picture) -> (Vec<usize>, Vec<usize>);
}

pub struct UnitTileSplitter {}

impl TileSplitter for UnitTileSplitter {
    fn get_ctu_cols_and_rows(self, _picture: &Picture) -> (Vec<usize>, Vec<usize>) {
        (vec![0], vec![0])
    }
}

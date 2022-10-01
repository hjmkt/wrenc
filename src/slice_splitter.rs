use super::picture::*;
use super::slice::*;

pub trait SliceSplitter {
    fn get_slice_types(self, picture: &Picture) -> Vec<SliceStruct>;
}

pub struct UnitSliceSplitter {}

impl SliceSplitter for UnitSliceSplitter {
    fn get_slice_types(self, picture: &Picture) -> Vec<SliceStruct> {
        let tiles = picture.tiles.lock().unwrap();
        vec![SliceStruct::Rectangle {
            tile_col: 0,
            tile_row: 0,
            num_tile_cols: tiles[0].len(),
            num_tile_rows: tiles.len(),
        }]
    }
}

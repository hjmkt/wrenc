use super::common::*;
use super::nal::*;
use super::tile::*;
use std::sync::{Arc, Mutex};

// FIXME better name
#[allow(dead_code)]
pub enum SliceStruct {
    Raster {
        tile_col: usize,
        tile_row: usize,
        num_tiles: usize,
    },
    Rectangle {
        tile_col: usize,
        tile_row: usize,
        num_tile_cols: usize,
        num_tile_rows: usize,
    },
    RectangleInTile {
        tile_col: usize,
        tile_row: usize,
        ctu_col_offset: usize,
        num_ctu_cols: usize,
    },
}

pub struct Slice {
    pub slice_struct: SliceStruct,
    pub nal_unit_type: NALUnitType,
    pub tiles: Arc<Mutex<Vec<Arc<Mutex<Tile>>>>>,
}

impl Slice {
    pub fn new(
        slice_struct: SliceStruct,
        nal_unit_type: NALUnitType,
        picture_tiles: ArcMutex<Vec<Vec<ArcMutex<Tile>>>>,
        picture_num_tile_cols: usize,
    ) -> Slice {
        let mut tiles = vec![];
        let picture_tiles = picture_tiles.lock().unwrap();
        match slice_struct {
            SliceStruct::Raster {
                tile_col,
                tile_row,
                num_tiles,
            } => {
                for i in 0..num_tiles {
                    let col = (tile_col + i) % picture_num_tile_cols;
                    let row = tile_row + (tile_col + i) / picture_num_tile_cols;
                    let tile = picture_tiles[row][col].clone();
                    tiles.push(tile);
                }
            }
            SliceStruct::Rectangle {
                tile_col,
                tile_row,
                num_tile_cols,
                num_tile_rows,
            } => {
                for row in tile_row..tile_row + num_tile_rows {
                    for col in tile_col..tile_col + num_tile_cols {
                        let tile = picture_tiles[row][col].clone();
                        tiles.push(tile);
                    }
                }
            }
            SliceStruct::RectangleInTile {
                tile_col,
                tile_row,
                ctu_col_offset: _,
                num_ctu_cols: _,
            } => {
                let tile = picture_tiles[tile_row][tile_col].clone();
                tiles.push(tile);
            }
        }
        Slice {
            slice_struct,
            nal_unit_type,
            tiles: Arc::new(Mutex::new(tiles)),
        }
    }
}

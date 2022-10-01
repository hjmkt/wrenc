use super::common::*;
use super::ctu::*;
use super::nal::*;
use super::slice::*;
use super::subpicture::*;
use super::tile::*;
use debug_print::*;
use std::sync::{Arc, Mutex};

#[allow(non_camel_case_types)]
#[allow(clippy::upper_case_acronyms)]
#[allow(dead_code)]
pub enum PictureType {
    IRAP_IDR, // Intra Random Access Point Instantaneous Decoding Refresh
    IRAP_CRA, // Intra Random Access Point Clean Random Access
    GDR,      // Gradual Decoding Refresh
    RADL,     // Random Access Decodable Leading
    RASL,     // Random Access Skipped Leading
    Trailing, // Trailing
    STLA,     // Stepwise Temporal Layer Access
    Intra,
    Predictive,
    BiPredictive,
}

pub struct Picture {
    pub picture_order_count: usize,
    pub picture_type: PictureType,
    pub width: usize,
    pub height: usize,
    pub chroma_format: ChromaFormat,
    pub pixels: Vec<Vec2d<u8>>,
    pub tiles: ArcMutex<Vec<Vec<ArcMutex<Tile>>>>,
    pub subpictures: Vec<Subpicture>,
    pub log2_ctu_size: usize,
    pub num_ctu_cols: usize,
    pub num_ctu_rows: usize,
    pub ctus: ArcMutex<Vec<Vec<ArcMutex<CodingTreeUnit>>>>,
    pub slices: ArcMutex<Vec<ArcMutex<Slice>>>,
    pub fixed_qp: Option<usize>,
}

impl Picture {
    pub fn new(width: usize, height: usize, fixed_qp: Option<usize>) -> Picture {
        #[cfg(debug_assertions)]
        {
            assert!(width > 0, "width must be greater than 0.");
            assert!(height > 0, "height must be greater than 0.");
        }

        Picture {
            picture_order_count: 0,
            picture_type: PictureType::IRAP_IDR,
            width,
            height,
            chroma_format: ChromaFormat::YCbCr420,
            pixels: vec![
                vec2d![0; height; width],
                vec2d![0; height/2; width/2],
                vec2d![0; height/2; width/2],
            ],
            tiles: Arc::new(Mutex::new(vec![])),
            subpictures: vec![],
            log2_ctu_size: 5, // 5 or 6 or 7
            num_ctu_cols: (width + (1 << 5) - 1) / (1 << 5),
            num_ctu_rows: (height + (1 << 5) - 1) / (1 << 5),
            ctus: Arc::new(Mutex::new(vec![])),
            slices: Arc::new(Mutex::new(vec![])),
            fixed_qp,
        }
    }

    pub fn init_ctus(
        &mut self,
        log2_ctu_size: usize,
        //pixels: &Vec<Vec2d<u8>>
    ) {
        let ctu_size = 1 << log2_ctu_size;
        #[cfg(debug_assertions)]
        {
            let available_ctu_sizes = vec![32, 64, 128];
            assert!(
                available_ctu_sizes.contains(&ctu_size),
                "Invalid ctu_size {} (must be one of {:?})",
                ctu_size,
                available_ctu_sizes,
            );
        }

        self.log2_ctu_size = log2_ctu_size;
        self.num_ctu_cols = (self.width + ctu_size - 1) / ctu_size;
        self.num_ctu_rows = (self.height + ctu_size - 1) / ctu_size;
        let mut ctus = vec![];
        for row in 0..self.num_ctu_rows {
            let mut row_ctus = vec![];
            for col in 0..self.num_ctu_cols {
                row_ctus.push(CodingTreeUnit::new_arc_mutex(
                    col * ctu_size,
                    row * ctu_size,
                    log2_ctu_size,
                    log2_ctu_size,
                    None,
                    self.fixed_qp,
                ));
            }
            ctus.push(row_ctus);
        }
        self.ctus = Arc::new(Mutex::new(ctus));
    }

    pub fn init_tiles(&mut self, ctu_cols: Vec<usize>, ctu_rows: Vec<usize>) {
        debug_eprintln!("init tile");
        #[cfg(debug_assertions)]
        {
            assert!(!ctu_cols.is_empty(), "ctu_cols must not be empty.");
            assert!(!ctu_rows.is_empty(), "ctu_rows must not be empty.");
            for (ctu_col_start, ctu_col_end) in
                (0..(ctu_cols.len() - 1)).map(|i| (ctu_cols[i], ctu_cols[i + 1]))
            {
                assert!(
                    ctu_col_start < ctu_col_end,
                    "Non-ascending consecutive ctu_cols detected: ..., {}, {}, ...",
                    ctu_col_start,
                    ctu_col_end
                );
            }
            for (ctu_row_start, ctu_row_end) in
                (0..(ctu_rows.len() - 1)).map(|i| (ctu_rows[i], ctu_rows[i + 1]))
            {
                assert!(
                    ctu_row_start < ctu_row_end,
                    "Non-ascending consecutive ctu_rows detected: ..., {}, {}, ...",
                    ctu_row_start,
                    ctu_row_end
                );
            }
            assert!(
                ctu_cols.last().unwrap() < &self.num_ctu_cols,
                "ctu_col {} is out of bound (num_ctu_cols={})",
                ctu_cols.last().unwrap(),
                self.num_ctu_cols,
            );
            assert!(
                ctu_rows.last().unwrap() < &self.num_ctu_rows,
                "ctu_row {} is out of bound (num_ctu_rows={})",
                ctu_rows.last().unwrap(),
                self.num_ctu_rows,
            );
        }

        let mut ctu_cols = ctu_cols;
        let mut ctu_rows = ctu_rows;
        ctu_cols.push(self.num_ctu_cols);
        ctu_rows.push(self.num_ctu_rows);
        let mut tiles = vec![];
        for (ctu_col_start, ctu_col_end) in
            (0..(ctu_cols.len() - 1)).map(|i| (ctu_cols[i], ctu_cols[i + 1]))
        {
            let mut row_tiles = vec![];
            for (ctu_row_start, ctu_row_end) in
                (0..(ctu_rows.len() - 1)).map(|i| (ctu_rows[i], ctu_rows[i + 1]))
            {
                let tile = Tile::new_arc_mutex(
                    ctu_col_start,
                    ctu_row_start,
                    ctu_col_end - ctu_col_start,
                    ctu_row_end - ctu_row_start,
                    self.log2_ctu_size,
                    self.ctus.clone(),
                );
                let ctus = self.ctus.lock().unwrap();
                for row_ctus in ctus.iter() {
                    for ctu in row_ctus.iter() {
                        let ctu = &mut ctu.lock().unwrap();
                        ctu.set_tile(tile.clone());
                    }
                }
                {
                    let tile = &mut tile.lock().unwrap();
                    for c_idx in 0..3 {
                        let ctu_size = 1 << tile.log2_ctu_size;
                        if c_idx == 0 {
                            let tile_pixels = &mut tile.original_pixels.borrow_mut()[c_idx];
                            let pixels = &self.pixels[c_idx];
                            for y in ctu_row_start * ctu_size..ctu_row_end * ctu_size {
                                let tile_pixels = &mut tile_pixels[y - ctu_row_start * ctu_size];
                                let pixels = &pixels[y][ctu_col_start * ctu_size..];
                                let end = (ctu_col_end - ctu_col_start) * ctu_size;
                                tile_pixels[..end].copy_from_slice(&pixels[..end]);
                            }
                        } else {
                            // check chroma format
                            let tile_pixels = &mut tile.original_pixels.borrow_mut()[c_idx];
                            let pixels = &self.pixels[c_idx];
                            for y in ctu_row_start * ctu_size / 2..ctu_row_end * ctu_size / 2 {
                                let tile_pixels =
                                    &mut tile_pixels[y - ctu_row_start * ctu_size / 2];
                                let pixels = &pixels[y][ctu_col_start * ctu_size / 2..]
                                    [ctu_col_start * ctu_size / 2..];
                                let end = (ctu_col_end - ctu_col_start) * ctu_size / 2;
                                tile_pixels[..end].copy_from_slice(&pixels[..end]);
                            }
                        }
                    }
                }
                row_tiles.push(tile);
            }
            tiles.push(row_tiles);
        }
        self.tiles = Arc::new(Mutex::new(tiles));
    }

    pub fn init_slices(&mut self, slice_structs: Vec<SliceStruct>, nal_unit_type: NALUnitType) {
        #[cfg(debug_assertions)]
        {
            assert!(!slice_structs.is_empty(), "slice_types must not be empty.");
        }

        let mut slices = vec![];
        for slice_struct in slice_structs {
            let slice = Slice::new(
                slice_struct,
                nal_unit_type,
                self.tiles.clone(),
                self.num_ctu_cols,
            );
            slices.push(Arc::new(Mutex::new(slice)));
        }
        self.slices = Arc::new(Mutex::new(slices));
    }

    pub fn init_subpictures(&mut self, slice_index_groups: Vec<Vec<usize>>) {
        let picture_slices = self.slices.clone();
        let picture_slices = picture_slices.lock().unwrap();
        let mut subpictures = vec![];
        for slice_indices in slice_index_groups {
            let mut slices = vec![];
            for slice_index in slice_indices {
                let slice = picture_slices[slice_index].clone();
                slices.push(slice);
            }
            let subpicture = Subpicture::new(slices);
            subpictures.push(subpicture);
        }
        self.subpictures = subpictures;
    }

    pub fn get_reconst_pixels(&self) -> Vec<Vec<u8>> {
        let mut reconst_pixels = vec![];
        for c_idx in 0..self.pixels.len() {
            let (width, height) = if c_idx == 0 {
                (self.width, self.height)
            } else {
                match self.chroma_format {
                    ChromaFormat::YCbCr420 => (self.width / 2, self.height / 2),
                    ChromaFormat::YCbCr422 => (self.width / 2, self.height),
                    ChromaFormat::YCbCr444 => (self.width, self.height),
                    _ => panic!(),
                }
            };
            let mut pixels = vec![0; width * height];
            let tiles = self.tiles.lock().unwrap();
            for tile_rows in tiles.iter() {
                for tile in tile_rows {
                    let tile = tile.lock().unwrap();
                    let reconst_pixels = &tile.reconst_pixels.borrow()[c_idx];
                    let ty = tile.ctu_row << tile.log2_ctu_size;
                    let tx = tile.ctu_col << tile.log2_ctu_size;
                    for dy in 0..reconst_pixels.height {
                        let reconst_pixels = &reconst_pixels[dy];
                        let pixels = &mut pixels[(ty + dy) * width + tx..];
                        pixels[..reconst_pixels.len()].copy_from_slice(reconst_pixels);
                    }
                }
            }
            reconst_pixels.push(pixels);
        }
        reconst_pixels
    }
}

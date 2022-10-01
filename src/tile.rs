use super::common::*;
use super::ctu::*;
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::{Arc, Mutex};

pub struct Tile {
    pub ctu_col: usize,
    pub ctu_row: usize,
    pub num_ctu_cols: usize,
    pub num_ctu_rows: usize,
    pub log2_ctu_size: usize,
    pub ctus: ArcMutex<Vec<Vec<ArcMutex<CodingTreeUnit>>>>,
    pub original_pixels: Rc<RefCell<Vec<Vec2d<u8>>>>,
    pub pred_pixels: Rc<RefCell<Vec<Vec2d<u8>>>>,
    pub residual_pixels: Rc<RefCell<Vec<Vec2d<i16>>>>,
    pub reconst_pixels: Rc<RefCell<Vec<Vec2d<u8>>>>,
}

impl Tile {
    pub fn new_arc_mutex(
        ctu_col: usize,
        ctu_row: usize,
        num_ctu_cols: usize,
        num_ctu_rows: usize,
        log2_ctu_size: usize,
        picture_ctus: ArcMutex<Vec<Vec<ArcMutex<CodingTreeUnit>>>>,
    ) -> ArcMutex<Tile> {
        let mut ctus = vec![];
        let picture_ctus = &picture_ctus.lock().unwrap();
        for row in ctu_row..ctu_row + num_ctu_rows {
            let mut row_ctus = vec![];
            for col in ctu_col..ctu_col + num_ctu_cols {
                row_ctus.push(picture_ctus[row][col].clone());
            }
            ctus.push(row_ctus);
        }
        Arc::new(Mutex::new(Tile {
            ctu_col,
            ctu_row,
            num_ctu_cols,
            num_ctu_rows,
            log2_ctu_size,
            ctus: Arc::new(Mutex::new(ctus)),
            original_pixels: Rc::new(RefCell::new(vec![
                vec2d![0; num_ctu_rows * (1 << log2_ctu_size); num_ctu_cols * (1 << log2_ctu_size)],
                vec2d![0; num_ctu_rows * (1 << log2_ctu_size) / 2; num_ctu_cols * (1 << log2_ctu_size) / 2],
                vec2d![0; num_ctu_rows * (1 << log2_ctu_size) / 2; num_ctu_cols * (1 << log2_ctu_size) / 2],
            ])),
            pred_pixels: Rc::new(RefCell::new(vec![
                vec2d![0; num_ctu_rows * (1 << log2_ctu_size); num_ctu_cols * (1 << log2_ctu_size)],
                vec2d![0; num_ctu_rows * (1 << log2_ctu_size) / 2; num_ctu_cols * (1 << log2_ctu_size) / 2],
                vec2d![0; num_ctu_rows * (1 << log2_ctu_size) / 2; num_ctu_cols * (1 << log2_ctu_size) / 2],
            ])),
            residual_pixels: Rc::new(RefCell::new(vec![
                vec2d![0; num_ctu_rows * (1 << log2_ctu_size); num_ctu_cols * (1 << log2_ctu_size)],
                vec2d![0; num_ctu_rows * (1 << log2_ctu_size) / 2; num_ctu_cols * (1 << log2_ctu_size) / 2],
                vec2d![0; num_ctu_rows * (1 << log2_ctu_size) / 2; num_ctu_cols * (1 << log2_ctu_size) / 2],
            ])),
            reconst_pixels: Rc::new(RefCell::new(vec![
                vec2d![0; num_ctu_rows * (1 << log2_ctu_size); num_ctu_cols * (1 << log2_ctu_size)],
                vec2d![0; num_ctu_rows * (1 << log2_ctu_size) / 2; num_ctu_cols * (1 << log2_ctu_size) / 2],
                vec2d![0; num_ctu_rows * (1 << log2_ctu_size) / 2; num_ctu_cols * (1 << log2_ctu_size) / 2],
            ])),
        }))
    }

    pub fn get_cu(&self, x: isize, y: isize) -> Option<ArcMutex<CodingUnit>> {
        if x < 0
            || y < 0
            || x as usize >= (self.ctu_col + self.num_ctu_cols) << self.log2_ctu_size
            || y as usize >= (self.ctu_row + self.num_ctu_rows) << self.log2_ctu_size
        {
            return None;
        }
        let (x, y) = (x as usize, y as usize);
        let row = y >> self.log2_ctu_size;
        let col = x >> self.log2_ctu_size;
        let ctus = self.ctus.lock().unwrap();
        let ctu = &ctus[row][col];
        let ctu = ctu.lock().unwrap();
        ctu.get_cu(x, y)
    }
}

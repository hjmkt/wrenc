#![feature(int_log)]
#![allow(clippy::comparison_chain)]
#![allow(clippy::too_many_arguments)]
extern crate num;
#[macro_use]
extern crate num_derive;
mod aps;
mod aps_encoder;
mod binary_reader;
mod binary_writer;
mod cabac_contexts;
#[macro_use]
mod common;
mod bins;
mod block_splitter;
mod bool_coder;
mod ctu;
mod ctu_encoder;
mod dpb;
mod dpbp_encoder;
mod encoder_context;
mod gci;
mod gci_encoder;
mod hrd_encoder;
mod intra_predictor;
mod nal;
mod partition;
mod ph_encoder;
mod picture;
mod picture_header;
mod pps;
mod pps_encoder;
mod pred_weight_table;
mod ptl;
mod ptl_encoder;
mod pwt_encoder;
mod quantizer;
mod reference_picture;
mod rpl_encoder;
mod slice;
mod slice_encoder;
mod slice_header;
mod slice_splitter;
mod sps;
mod sps_encoder;
mod subpicture;
mod subpicture_splitter;
mod tile;
mod tile_splitter;
mod timing_hrd;
mod transformer;
mod virtual_boundary;
mod vps;
mod vps_encoder;
use aps::*;
//use aps_encoder::*;
use binary_reader::BinaryReader;
use binary_writer::BinaryWriter;
use bins::*;
use bool_coder::*;
use clap::Parser;
use colored::*;
use common::*;
use debug_print::*;
use encoder_context::*;
use nal::*;
use ph_encoder::*;
use picture::Picture;
use picture_header::*;
use pps::*;
use pps_encoder::*;
use slice_encoder::*;
use slice_header::*;
use slice_splitter::*;
use sps::*;
use sps_encoder::*;
use std::io::{self, Write};
use std::process;
use std::sync::{Arc, Mutex};
use subpicture_splitter::*;
use tile_splitter::*;
use vps::*;
use vps_encoder::*;

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// Path to input raw video
    #[clap(short, long)]
    input: String,
    /// Path to output bitstream
    #[clap(short, long)]
    output: String,
    /// Path to reconstructed frames
    #[clap(short, long)]
    reconst: Option<String>,
    /// Input video resolution (WIDTHxHEIGHT)
    #[clap(long)]
    input_size: String,
    /// Output video resolution (WIDTHxHEIGHT)
    #[clap(long)]
    output_size: String,
    /// Number of pictures to encode
    #[clap(long)]
    num_pictures: usize,
    /// Fixed quantization parameter for entire video stream
    #[clap(long)]
    qp: Option<usize>,
    /// Max split depth of coding trees to search
    #[clap(long, default_value_t = 3)]
    max_split_depth: usize,
    /// Extra parameters (PARAM1=VAL1[,PARAM2=VAL2,...])
    #[clap(long)]
    extra_params: Option<String>,
}

fn main() {
    let args = Args::parse();

    let mut ectx = EncoderContext::new();

    // initialize binary reader
    let stdin = io::stdin();
    let mut reader = if args.input == *"-" {
        BinaryReader::standard(&stdin)
    } else {
        match BinaryReader::file(args.input) {
            Ok(f) => f,
            Err(e) => {
                eprintln!("{}: failed to open input file: {}", "error".red(), e);
                process::exit(0);
            }
        }
    };

    // initialize binary writer
    let stdout = io::stdout();
    let mut writer = if args.output == *"-" {
        BinaryWriter::standard(&stdout)
    } else {
        match BinaryWriter::file(args.output) {
            Ok(f) => f,
            Err(e) => {
                eprintln!("{}: failed to open output file: {}", "error".red(), e);
                process::exit(0);
            }
        }
    };

    // initialize reconstructed picture writer
    let mut reconst_writer = if let Some(reconst_path) = args.reconst {
        match BinaryWriter::file(reconst_path) {
            Ok(f) => Some(f),
            Err(e) => {
                eprintln!("{}: failed to open reconst file: {}", "error".red(), e);
                process::exit(0);
            }
        }
    } else {
        None
    };

    let input_size = args
        .input_size
        .split('x')
        .map(|x| x.parse::<usize>())
        .collect::<Vec<Result<usize, std::num::ParseIntError>>>();
    if let [Ok(width), Ok(height)] = input_size[..] {
        ectx.input_picture_width = width;
        ectx.input_picture_height = height;
    } else {
        eprintln!("{}: Invalid input-size: {}", "error".red(), args.input_size);
        process::exit(0);
    }

    let output_size = args
        .output_size
        .split('x')
        .map(|x| x.parse::<usize>())
        .collect::<Vec<Result<usize, std::num::ParseIntError>>>();
    if let [Ok(width), Ok(height)] = output_size[..] {
        ectx.output_picture_width = width;
        ectx.output_picture_height = height;
    } else {
        eprintln!(
            "{}: Invalid output-size: {}",
            "error".red(),
            args.output_size
        );
        process::exit(0);
    }

    let fixed_qp = args.qp;
    ectx.fixed_qp = fixed_qp;
    if let Some(qp) = fixed_qp {
        ectx.slice_qp_y = qp as isize;
        ectx.qp_y = qp;
    }

    ectx.max_split_depth = args.max_split_depth;

    if let Some(extra_params) = args.extra_params {
        for param in extra_params.split(',') {
            let param = param.split('=').collect::<Vec<&str>>();
            if param.len() != 2 {
                eprintln!("{}: Invalid extra-params: {}", "error".red(), extra_params);
                process::exit(0);
            }
            if let [key, val] = &param[..2] {
                ectx.extra_params.insert(key.to_string(), val.to_string());
            } else {
                eprintln!("{}: Invalid extra-params: {}", "error".red(), extra_params);
                process::exit(0);
            }
        }
    }

    let (output_width, output_height) = (ectx.output_picture_width, ectx.output_picture_height);

    let mut coder = BoolCoder::new();
    let ectx = Arc::new(Mutex::new(ectx));

    let vps = VideoParameterSet::new(8, output_width, output_height, 8, ChromaFormat::YCbCr420);
    vps.validate();
    {
        {
            let ectx = &mut ectx.lock().unwrap();
            ectx.update_from_vps(&vps);
        }
        let mut vps_encoder = VpsEncoder::new(&ectx, &mut coder);
        let vps_bits = vps_encoder.encode(&vps);
        debug_eprintln!("vps bits {}", vps_bits.len());
        write_byte_stream_nal_unit_bits(1, NALUnitType::VPS_NUT, 0, &vps_bits, &mut writer);
    }
    debug_eprintln!("vps end");

    let sps = SequenceParameterSet::new(1, 8, output_width, output_height, 8);
    {
        {
            let ectx = &mut ectx.lock().unwrap();
            ectx.update_from_sps(&sps);
        }
        let mut sps_encoder = SpsEncoder::new(&ectx, &mut coder);
        let sps_bits = sps_encoder.encode(&sps);
        debug_eprintln!("sps bits {}", sps_bits.len());
        write_byte_stream_nal_unit_bits(9, NALUnitType::SPS_NUT, 0, &sps_bits, &mut writer);
    }
    debug_eprintln!("sps end");

    let pps = PictureParameterSet::new(1, &sps, fixed_qp.map(|x| x as isize));
    {
        {
            let ectx = &mut ectx.lock().unwrap();
            ectx.update_from_sps_and_pps(&sps, &pps);
        }
        let mut pps_encoder = PpsEncoder::new(&ectx, &mut coder);
        let pps_bits = pps_encoder.encode(&pps);
        write_byte_stream_nal_unit_bits(9, NALUnitType::PPS_NUT, 0, &pps_bits, &mut writer);
    }
    debug_eprintln!("pps end");

    let alf_aps = AdaptationParameterSet::new_alf(1);
    let lmcs_aps = AdaptationParameterSet::new_lmcs(2);
    let sl_aps = AdaptationParameterSet::new_sl(3);
    {
        //let mut aps_encoder = ApsEncoder::new(&ectx, &mut coder);
        //let alf_aps_bits = aps_encoder.encode(&alf_aps);
        //write_byte_stream_nal_unit_bits(
        //9,
        //NALUnitType::PREFIX_APS_NUT,
        //0,
        //&alf_aps_bits,
        //&mut writer,
        //);
        //let lmcs_aps_bits = aps_encoder.encode(&lmcs_aps);
        //write_byte_stream_nal_unit_bits(
        //1,
        //NALUnitType::PREFIX_APS_NUT,
        //1,
        //&lmcs_aps_bits,
        //&mut writer,
        //);
        //let sl_aps_bits = aps_encoder.encode(&sl_aps);
        //write_byte_stream_nal_unit_bits(
        //1,
        //NALUnitType::PREFIX_APS_NUT,
        //1,
        //&sl_aps_bits,
        //&mut writer,
        //);
    }
    debug_eprintln!("aps end");

    for picture_index in 0..args.num_pictures {
        //eprintln!("picture #{}", picture_index);
        let intra = true;
        let ph = PictureHeader::new(&pps, intra, picture_index);
        let nuh_layer_id = 9;
        {
            {
                let ectx = &mut ectx.lock().unwrap();
                ectx.update_from_ph(&ph, &pps);
            }
            let mut ph_encoder = PhEncoder::new(&ectx, &mut coder);
            let mut ph_bins = Bins::new();
            ph_encoder.encode(&mut ph_bins, &ph, &sps, &pps);
            let ph_bins = ph_bins.into_iter().collect();
            let nuh_temporal_id = 0;
            write_byte_stream_nal_unit_bits(
                nuh_layer_id,
                NALUnitType::PH_NUT,
                nuh_temporal_id,
                &ph_bins,
                &mut writer,
            );
        }

        let mut picture = Picture::new(output_width, output_height, fixed_qp);
        {
            let mut luma = vec![0; output_height * output_width];
            if let Err(e) = reader.read_to_vec(&mut luma) {
                eprintln!("{e}");
                process::exit(0);
            }
            for y in 0..output_height {
                for x in 0..output_width {
                    picture.pixels[0][y][x] = luma[y * output_width + x];
                }
            }
            let mut cb = vec![0; (output_height / 2) * (output_width / 2)];
            if let Err(e) = reader.read_to_vec(&mut cb) {
                eprintln!("{e}");
                process::exit(0);
            }
            for y in 0..output_height / 2 {
                for x in 0..output_width / 2 {
                    picture.pixels[1][y][x] = cb[y * (output_width / 2) + x];
                }
            }
            let mut cr = vec![0; (output_height / 2) * (output_width / 2)];
            if let Err(e) = reader.read_to_vec(&mut cr) {
                eprintln!("{e}");
                process::exit(0);
            }
            for y in 0..output_height / 2 {
                for x in 0..output_width / 2 {
                    picture.pixels[2][y][x] = cr[y * (output_width / 2) + x];
                }
            }
        }

        let default_log2_ctu_size = 5;
        picture.init_ctus(default_log2_ctu_size);
        let (ctu_cols, ctu_rows) = UnitTileSplitter {}.get_ctu_cols_and_rows(&picture);
        debug_eprintln!("pre init tiles");
        picture.init_tiles(ctu_cols, ctu_rows);
        let slice_types = UnitSliceSplitter {}.get_slice_types(&picture);
        picture.init_slices(slice_types, NALUnitType::IDR_W_RADL);
        let slice_index_groups =
            UnitSubpictureSplitter {}.get_subpicture_slice_index_groups(&picture);
        picture.init_subpictures(slice_index_groups);

        let slices = picture.slices.lock().unwrap();
        for slice in slices.iter() {
            let sh = {
                let ectx = &ectx.lock().unwrap();
                SliceHeader::new(
                    &sps,
                    &pps,
                    [&alf_aps, &lmcs_aps, &sl_aps],
                    Some(&ph),
                    fixed_qp.map(|x| x as isize),
                    ectx,
                )
            };
            {
                let ectx = &mut ectx.lock().unwrap();
                ectx.update_from_sh(&sh, &pps);
            }
            let mut slice_encoder = SliceEncoder::new(&ectx, &mut coder);
            let slice = slice.lock().unwrap();
            let slice_bins = slice_encoder.encode(&slice, &sh);
            write_byte_stream_nal_unit_bins(
                nuh_layer_id,
                NALUnitType::IDR_W_RADL,
                0,
                &slice_bins,
                &mut writer,
            );
        }
        if let Some(ref mut reconst_writer) = reconst_writer {
            let reconst_pixels = picture.get_reconst_pixels();
            for component_pixels in &reconst_pixels {
                if let Err(e) = reconst_writer.write(&component_pixels[..]) {
                    panic!("{e}");
                }
            }
            if let Err(e) = reconst_writer.flush() {
                panic!("{e}");
            }
        }
    }
}

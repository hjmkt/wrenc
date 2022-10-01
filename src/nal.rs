use super::binary_writer::*;
use super::bins::*;
use super::ptl::*;
use debug_print::*;
use std::io::Write;

#[allow(non_camel_case_types)]
#[allow(dead_code)]
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum NALUnitType {
    TRAIL_NUT = 0,       // Coded slice of a trailing picture or subpicture
    STSA_NUT = 1,        // Coded slice of an STSA picture or subpicture
    RADL_NUT = 2,        // Coded slice of a RADL picture or subpicture
    RASL_NUT = 3,        // Coded slice of a RASL picture or subpicture
    RSV_VCL_4 = 4,       // Reserved non IRAP VCL NAL unit types
    RSV_VCL_5 = 5,       // Reserved non IRAP VCL NAL unit types
    RSV_VCL_6 = 6,       // Reserved non IRAP VCL NAL unit types
    IDR_W_RADL = 7,      // Coded slice of an IDR picture or subpicture
    IDR_N_LP = 8,        // Coded slice of an IDR picture or subpicture
    CRA_NUT = 9,         // Coded slice of a CRA picture or subpicture
    GDR_NUT = 10,        // Coded slice of a GDR picture or subpicture
    RSV_IRAP_11 = 11,    // Reserved IRAP VCL NAL unit type
    OPI_NUT = 12,        // Operating point information
    DCI_NUT = 13,        // Decoding capability information
    VPS_NUT = 14,        // Video parameter set
    SPS_NUT = 15,        // Sequence parameter set
    PPS_NUT = 16,        // Picture parameter set
    PREFIX_APS_NUT = 17, // Adaptation parameter set
    SUFFIX_APS_NUT = 18, // Adaptation parameter set
    PH_NUT = 19,         // Picture header
    AUD_NUT = 20,        // AU delimiter
    EOS_NUT = 21,        // End of sequence
    EOB_NUT = 22,        // End of bitstream
    PREFIX_SEI_NUT = 23, // Supplemental enhancement information
    SUFFIX_SEI_NUT = 24, // Supplemental enhancement information
    FD_NUT = 25,         // Filter data
    RSV_NVCL_26 = 26,    // Reserved non VCL NAL unit types
    RSV_NVCL_27 = 27,    // Reserved non VCL NAL unit types
    UNSPEC_28 = 28,      // Unspecified non VCL NAL unit types
    UNSPEC_29 = 29,      // Unspecified non VCL NAL unit types
    UNSPEC_30 = 30,      // Unspecified non VCL NAL unit types
    UNSPEC_31 = 31,      // Unspecified non VCL NAL unit types
}

#[allow(dead_code)]
#[allow(clippy::upper_case_acronyms)]
pub enum RBSP {
    //SPS(SequenceParameterSet),
    //PPS(PictureParameterSet),
    //PH(PictureHeader),
    //IDR_N_LP(Picture), // "I"<-PPBPP
    //TRAIL(Picture),    // I<-"PPBPP"
    EOB(EndOfBitstream),
}

#[allow(dead_code)]
#[allow(clippy::upper_case_acronyms)]
pub struct NAL {
    nuh_layer_id: usize,
    nal_unit_type: NALUnitType,
    nuh_temporal_id: usize,
    rbsp: RBSP,
}

#[allow(dead_code)]
pub struct DecodingCapabilityInformation {
    profile_tier_levels: Vec<ProfileTierLevel>,
    dci_extension_flags: Vec<bool>,
}

#[allow(dead_code)]
pub struct OperatingPointInformation {
    opi_ols_idx: Option<usize>,
    opi_htid: Option<usize>,
    opi_extension_data: Vec<bool>,
}

#[allow(dead_code)]
pub struct SeiMessage {
    payload_type_byte: usize,
    payload_size_byte: usize,
    // TODO
}

#[allow(dead_code)]
#[allow(clippy::upper_case_acronyms)]
pub struct SEI {
    sei_messages: Vec<SeiMessage>,
}

#[allow(dead_code)]
pub struct AccessUnitDelimiter {
    aud_irap_or_gdr_flag: bool,
    aud_pic_type: usize,
}

#[allow(dead_code)]
pub struct EndOfSeq {}

pub struct EndOfBitstream {}

#[allow(dead_code)]
pub struct FillerData {
    fd_ff_bytes: usize,
}

pub fn write_nal_unit_bits(
    nuh_layer_id: usize,
    nal_unit_type: NALUnitType,
    nuh_temporal_id: usize,
    bits: &Vec<bool>,
    writer: &mut BinaryWriter,
) {
    //assert_ne!(nuh_temporal_id, 0);
    // nal_unit_header
    let forbidden_zero_bit = false;
    writer.write_bit(forbidden_zero_bit);
    let nuh_reserved_zero_bit = false;
    writer.write_bit(nuh_reserved_zero_bit);
    let nuh_layer_id_bits = vec![
        (nuh_layer_id >> 5) & 1 > 0,
        (nuh_layer_id >> 4) & 1 > 0,
        (nuh_layer_id >> 3) & 1 > 0,
        (nuh_layer_id >> 2) & 1 > 0,
        (nuh_layer_id >> 1) & 1 > 0,
        nuh_layer_id & 1 > 0,
    ];
    writer.write_bits(&nuh_layer_id_bits);
    let nal_unit_type = nal_unit_type as usize;
    let nal_unit_type_bits = vec![
        (nal_unit_type >> 4) & 1 > 0,
        (nal_unit_type >> 3) & 1 > 0,
        (nal_unit_type >> 2) & 1 > 0,
        (nal_unit_type >> 1) & 1 > 0,
        nal_unit_type & 1 > 0,
    ];
    writer.write_bits(&nal_unit_type_bits);
    let nuh_temporal_id_plus1_bits = vec![
        ((nuh_temporal_id + 1) >> 2) & 1 > 0,
        ((nuh_temporal_id + 1) >> 1) & 1 > 0,
        (nuh_temporal_id + 1) & 1 > 0,
    ];
    writer.write_bits(&nuh_temporal_id_plus1_bits);

    let mut bytes = vec![];
    assert_eq!(bits.len() % 8, 0);
    for i in (0..bits.len()).step_by(8) {
        let byte = ((bits[i] as u8) << 7)
            | ((bits[i + 1] as u8) << 6)
            | ((bits[i + 2] as u8) << 5)
            | ((bits[i + 3] as u8) << 4)
            | ((bits[i + 4] as u8) << 3)
            | ((bits[i + 5] as u8) << 2)
            | ((bits[i + 6] as u8) << 1)
            | bits[i + 7] as u8;
        bytes.push(byte);
    }

    let emulation_prevention_three_byte: [u8; 1] = [3];
    let mut idx = 0;
    while idx + 3 < bytes.len() {
        if bytes[idx] == 0 && bytes[idx + 1] == 0 && bytes[idx + 2] <= 3 {
            if let Err(e) = writer.write(&bytes[idx..idx + 2]) {
                panic!("{e}");
            }
            idx += 2;
            if let Err(e) = writer.write(&emulation_prevention_three_byte) {
                panic!("{e}");
            }
        } else {
            if let Err(e) = writer.write(&bytes[idx..idx + 1]) {
                panic!("{e}");
            }
            idx += 1;
        }
    }
    debug_eprintln!("nal");
    while idx < bytes.len() {
        if let Err(e) = writer.write(&bytes[idx..idx + 1]) {
            panic!("{e}");
        }
        idx += 1;
    }
}

pub fn write_byte_stream_nal_unit_bits(
    nuh_layer_id: usize,
    nal_unit_type: NALUnitType,
    nuh_temporal_id: usize,
    bits: &Vec<bool>,
    writer: &mut BinaryWriter,
) {
    // no leading_zero_8bits
    let header_bytes: [u8; 3] = [0, 0, 0];
    if let Err(e) = writer.write(&header_bytes) {
        panic!("{e}");
    }
    let start_code_prefix_one_3bytes: [u8; 3] = [0, 0, 1];
    if let Err(e) = writer.write(&start_code_prefix_one_3bytes) {
        panic!("{e}");
    }
    write_nal_unit_bits(nuh_layer_id, nal_unit_type, nuh_temporal_id, bits, writer);
    //let trailing_zero_8bits = [0];
    //writer.write(&trailing_zero_8bits);
    if let Err(e) = writer.flush() {
        panic!("{e}");
    }
}

pub fn write_byte_stream_nal_unit_bins(
    nuh_layer_id: usize,
    nal_unit_type: NALUnitType,
    nuh_temporal_id: usize,
    bins: &Bins,
    writer: &mut BinaryWriter,
) {
    // no leading_zero_8bits
    let header_bytes: [u8; 3] = [0, 0, 0];
    if let Err(e) = writer.write(&header_bytes) {
        panic!("{e}");
    }
    let start_code_prefix_one_3bytes: [u8; 3] = [0, 0, 1];
    if let Err(e) = writer.write(&start_code_prefix_one_3bytes) {
        panic!("{e}");
    }
    write_nal_unit_bins(nuh_layer_id, nal_unit_type, nuh_temporal_id, bins, writer);
    //let trailing_zero_8bits = [0];
    //writer.write(&trailing_zero_8bits);
    if let Err(e) = writer.flush() {
        panic!("{e}");
    }
}

pub fn write_nal_unit_bins(
    nuh_layer_id: usize,
    nal_unit_type: NALUnitType,
    nuh_temporal_id: usize,
    bins: &Bins,
    writer: &mut BinaryWriter,
) {
    //assert_ne!(nuh_temporal_id, 0);
    // nal_unit_header
    let forbidden_zero_bit = false;
    writer.write_bit(forbidden_zero_bit);
    let nuh_reserved_zero_bit = false;
    writer.write_bit(nuh_reserved_zero_bit);
    let nuh_layer_id_bits = vec![
        (nuh_layer_id >> 5) & 1 > 0,
        (nuh_layer_id >> 4) & 1 > 0,
        (nuh_layer_id >> 3) & 1 > 0,
        (nuh_layer_id >> 2) & 1 > 0,
        (nuh_layer_id >> 1) & 1 > 0,
        nuh_layer_id & 1 > 0,
    ];
    writer.write_bits(&nuh_layer_id_bits);
    let nal_unit_type = nal_unit_type as usize;
    let nal_unit_type_bits = vec![
        (nal_unit_type >> 4) & 1 > 0,
        (nal_unit_type >> 3) & 1 > 0,
        (nal_unit_type >> 2) & 1 > 0,
        (nal_unit_type >> 1) & 1 > 0,
        nal_unit_type & 1 > 0,
    ];
    writer.write_bits(&nal_unit_type_bits);
    let nuh_temporal_id_plus1_bits = vec![
        ((nuh_temporal_id + 1) >> 2) & 1 > 0,
        ((nuh_temporal_id + 1) >> 1) & 1 > 0,
        (nuh_temporal_id + 1) & 1 > 0,
    ];
    writer.write_bits(&nuh_temporal_id_plus1_bits);

    let bytes: Vec<u8> = bins.bytes().collect();

    let emulation_prevention_three_byte: [u8; 1] = [3];
    let mut idx = 0;
    while idx + 3 < bytes.len() {
        if bytes[idx] == 0 && bytes[idx + 1] == 0 && bytes[idx + 2] <= 3 {
            if let Err(e) = writer.write(&bytes[idx..idx + 2]) {
                panic!("{e}");
            }
            idx += 2;
            if let Err(e) = writer.write(&emulation_prevention_three_byte) {
                panic!("{e}");
            }
        } else {
            if let Err(e) = writer.write(&bytes[idx..idx + 1]) {
                panic!("{e}");
            }
            idx += 1;
        }
    }
    debug_eprintln!("nal");
    while idx < bytes.len() {
        if let Err(e) = writer.write(&bytes[idx..idx + 1]) {
            panic!("{e}");
        }
        idx += 1;
    }
}

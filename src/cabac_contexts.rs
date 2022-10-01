#![allow(non_upper_case_globals)]

use lazy_static::lazy_static;

#[allow(clippy::upper_case_acronyms)]
pub enum BinProcess {
    FL(usize),        // c_max
    TR(usize, usize), // c_max, c_rice_param
    TB(usize),        // c_max
    EG(usize),        // EGk
    ETC,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[allow(dead_code)]
pub enum CabacContext {
    AlfCtbFlag = 0,
    AlfUseApsFlag = 1,
    AlfCtbCcCbIdc = 2,
    AlfCtbCcCrIdc = 3,
    AlfLumaFixedFilterIdx = 4,
    AlfLumaPrevFilterIdx = 5,
    AlfCtbFilterAltIdx = 6,
    AlfSaoMergeLeftFlag = 7,
    AlfSaoMergeUpFlag = 8,
    AlfSaoTypeIdxLuma = 9,
    AlfSaoTypeIdxChroma = 10,
    SaoOffsetAbs = 11,
    SaoOffsetSignFlag = 12,
    SaoBandPosition = 13,
    SaoEoClassLuma = 14,
    SaoEoClassChroma = 15,
    SplitCuFlag = 16,
    SplitQtFlag = 17,
    MttSplitCuVerticalFlag = 18,
    MttSplitCuBinaryFlag = 19,
    NonInterFlag = 20,
    CuSkipFlag = 21,
    PredModeIbcFlag = 22,
    PredModeFlag = 23,
    PredModePltFlag = 24,
    CuActEnabledFlag = 25,
    IntraBdpcmLumaFlag = 26,
    IntraBdpcmLumaDirFlag = 27,
    IntraMipFlag = 28,
    IntraMipTransposedFlag = 29,
    IntraMipMode = 30,
    IntraLumaRefIdx = 31,
    IntraSubpartitionsModeFlag = 32,
    IntraSubpartitionsSplitFlag = 33,
    IntraLumaMpmFlag = 34,
    IntraLumaNotPlanarFlag = 35,
    IntraLumaMpmIdx = 36,
    IntraLumaMpmRemainder = 37,
    IntraBdpcmChromaFlag = 38,
    IntraBdpcmChromaDirFlag = 39,
    CclmModeFlag = 40,
    CclmModeIdx = 41,
    IntraChromaPredMode = 42,
    PalettePredictorRun = 43,
    NumSignalledPaletteEntries = 44,
    NewPaletteEntries = 45,
    PaletteEscapeValPresentFlag = 46,
    PaletteIdxIdc = 47,
    PaletteEscapeVal = 48,
    GeneralMergeFlag = 49,
    InterPredIdc = 50,
    InterAffineFlag = 51,
    CuAffineTypeFlag = 52,
    SymMvdFlag = 53,
    RefIdxL0 = 54,
    RefIdxL1 = 55,
    MvpL0Flag = 56,
    MvpL1Flag = 57,
    AmvrFlag = 58,
    AmvrPrecisionIdx = 59,
    BcwIdx = 60,
    CuCodedFlag = 61,
    CuSbtFlag = 62,
    CuSbtQuadFlag = 63,
    CuSbtHorizontalFlag = 64,
    CuSbtPosFlag = 65,
    LfnstIdx = 66,
    MtsIdx = 67,
    CopyAbovePaletteIndicesFlag = 68,
    PaletteTransposeFlag = 69,
    RunCopyFlag = 70,
    RegularMergeFlag = 71,
    MmvdMergeFlag = 72,
    MmvdCandFlag = 73,
    MmvdDistanceIdx = 74,
    MmvdDirectionIdx = 75,
    CiipFlag = 76,
    MergeSubblockFlag = 77,
    MergeSubblockIdx = 78,
    MergeIdx = 79,
    MergeGpmPartitionIdx = 80,
    MergeGpmIdx0 = 81,
    MergeGpmIdx1 = 82,
    AbsMvdGreater0Flag = 83,
    AbsMvdGreater1Flag = 84,
    AbsMvd = 85,
    MvdSignFlag = 86,
    TuYCodedFlag = 87,
    TuCbCodedFlag = 88,
    TuCrCodedFlag = 89,
    CuQpDeltaAbs = 90,
    CuQpDeltaSignFlag = 91,
    CuChromaQpOffsetFlag = 92,
    CuChromaQpOffsetIdx = 93,
    TransformSkipFlag = 94,
    TuJointCbcrResidualFlag = 95,
    LastSigCoeffXPrefix = 96,
    LastSigCoeffYPrefix = 97,
    LastSigCoeffXSuffix = 98,
    LastSigCoeffYSuffix = 99,
    SbCodedFlag = 100,
    SigCoeffFlag = 101,
    ParLevelFlag = 102,
    AbsLevelGtxFlag = 103,
    AbsRemainder = 104,
    DecAbsLevel = 105,
    CoeffSignFlag = 106,

    EndOfSliceOneBit = 107,
    EndOfTileOneBit = 108,
    EndOfSubsetOneBit = 109,
}

pub const ctx_to_bin_process: [BinProcess; 110] = [
    BinProcess::FL(1),    // AlfCtbFlag = 0,
    BinProcess::FL(1),    //AlfUseApsFlag = 1,
    BinProcess::ETC,      //AlfCtbCcCbIdc = 2,
    BinProcess::ETC,      //AlfCtbCcCrIdc = 3,
    BinProcess::TB(15),   //AlfLumaFixedFilterIdx = 4,
    BinProcess::ETC,      //AlfLumaPrevFilterIdx = 5,
    BinProcess::ETC,      //AlfCtbFilterAltIdx = 6,
    BinProcess::FL(1),    //AlfSaoMergeLeftFlag = 7,
    BinProcess::FL(1),    //AlfSaoMergeUpFlag = 8,
    BinProcess::TR(2, 0), //AlfSaoTypeIdxLuma = 9,
    BinProcess::TR(2, 0), //AlfSaoTypeIdxChroma = 10,
    BinProcess::ETC,      //SaoOffsetAbs = 11,
    BinProcess::FL(1),    //SaoOffsetSignFlag = 12,
    BinProcess::FL(31),   //SaoBandPosition = 13,
    BinProcess::FL(3),    //SaoEoClassLuma = 14,
    BinProcess::FL(3),    //SaoEoClassChroma = 15,
    BinProcess::FL(1),    //SplitCuFlag = 16,
    BinProcess::FL(1),    //SplitQtFlag = 17,
    BinProcess::FL(1),    //MttSplitCuVerticalFlag = 18,
    BinProcess::FL(1),    //MttSplitCuBinaryFlag = 19,
    BinProcess::FL(1),    //NonInterFlag = 20,
    BinProcess::FL(1),    //CuSkipFlag = 21,
    BinProcess::FL(1),    //PredModeIbcFlag = 22,
    BinProcess::FL(1),    //PredModeFlag = 23,
    BinProcess::FL(1),    //PredModePltFlag = 24,
    BinProcess::FL(1),    //CuActEnabledFlag = 25,
    BinProcess::FL(1),    //IntraBdpcmLumaFlag = 26,
    BinProcess::FL(1),    //IntraBdpcmLumaDirFlag = 27,
    BinProcess::FL(1),    //IntraMipFlag = 28,
    BinProcess::FL(1),    //IntraMipTransposedFlag = 29,
    BinProcess::ETC,      //IntraMipMode = 30,
    BinProcess::TR(2, 0), //IntraLumaRefIdx = 31,
    BinProcess::FL(1),    //IntraSubpartitionsModeFlag = 32,
    BinProcess::FL(1),    //IntraSubpartitionsSplitFlag = 33,
    BinProcess::FL(1),    //IntraLumaMpmFlag = 34,
    BinProcess::FL(1),    //IntraLumaNotPlanarFlag = 35,
    BinProcess::TR(4, 0), //IntraLumaMpmIdx = 36,
    BinProcess::TB(60),   //IntraLumaMpmRemainder = 37,
    BinProcess::FL(1),    //IntraBdpcmChromaFlag = 38,
    BinProcess::FL(1),    //IntraBdpcmChromaDirFlag = 39,
    BinProcess::FL(1),    //CclmModeFlag = 40,
    BinProcess::TR(2, 0), //CclmModeIdx = 41,
    BinProcess::ETC,      //IntraChromaPredMode = 42,
    BinProcess::EG(0),    //PalettePredictorRun = 43,
    BinProcess::EG(0),    //NumSignalledPaletteEntries = 44,
    BinProcess::ETC,      //NewPaletteEntries = 45,
    BinProcess::FL(1),    //PaletteEscapeValPresentFlag = 46,
    BinProcess::ETC,      //PaletteIdxIdc = 47,
    BinProcess::EG(5),    //PaletteEscapeVal = 48,
    BinProcess::FL(1),    //GeneralMergeFlag = 49,
    BinProcess::ETC,      //InterPredIdc = 50,
    BinProcess::FL(1),    //InterAffineFlag = 51,
    BinProcess::FL(1),    //CuAffineTypeFlag = 52,
    BinProcess::FL(1),    //SymMvdFlag = 53,
    BinProcess::ETC,      //RefIdxL0 = 54,
    BinProcess::ETC,      //RefIdxL1 = 55,
    BinProcess::FL(1),    //MvpL0Flag = 56,
    BinProcess::FL(1),    //MvpL1Flag = 57,
    BinProcess::FL(1),    //AmvrFlag = 58,
    BinProcess::ETC,      //AmvrPrecisionIdx = 59,
    BinProcess::ETC,      //BcwIdx = 60,
    BinProcess::FL(1),    //CuCodedFlag = 61,
    BinProcess::FL(1),    //CuSbtFlag = 62,
    BinProcess::FL(1),    //CuSbtQuadFlag = 63,
    BinProcess::FL(1),    //CuSbtHorizontalFlag = 64,
    BinProcess::FL(1),    //CuSbtPosFlag = 65,
    BinProcess::TR(2, 0), //LfnstIdx = 66,
    BinProcess::TR(4, 0), //MtsIdx = 67,
    BinProcess::FL(1),    //CopyAbovePaletteIndicesFlag = 68,
    BinProcess::FL(1),    //PaletteTransposeFlag = 69,
    BinProcess::FL(1),    //RunCopyFlag = 70,
    BinProcess::FL(1),    //RegularMergeFlag = 71,
    BinProcess::FL(1),    //MmvdMergeFlag = 72,
    BinProcess::FL(1),    //MmvdCandFlag = 73,
    BinProcess::TR(7, 0), //MmvdDistanceIdx = 74,
    BinProcess::FL(3),    //MmvdDirectionIdx = 75,
    BinProcess::FL(1),    //CiipFlag = 76,
    BinProcess::FL(1),    //MergeSubblockFlag = 77,
    BinProcess::ETC,      //MergeSubblockIdx = 78,
    BinProcess::ETC,      //MergeIdx = 79,
    BinProcess::FL(63),   //MergeGpmPartitionIdx = 80,
    BinProcess::ETC,      //MergeGpmIdx0 = 81,
    BinProcess::ETC,      //MergeGpmIdx1 = 82,
    BinProcess::FL(1),    //AbsMvdGreater0Flag = 83,
    BinProcess::FL(1),    //AbsMvdGreater1Flag = 84,
    BinProcess::ETC,      //AbsMvd = 85,
    BinProcess::FL(1),    //MvdSignFlag = 86,
    BinProcess::FL(1),    //TuYCodedFlag = 87,
    BinProcess::FL(1),    //TuCbCodedFlag = 88,
    BinProcess::FL(1),    //TuCrCodedFlag = 89,
    BinProcess::ETC,      //CuQpDeltaAbs = 90,
    BinProcess::FL(1),    //CuQpDeltaSignFlag = 91,
    BinProcess::FL(1),    //CuChromaQpOffsetFlag = 92,
    BinProcess::ETC,      //CuChromaQpOffsetIdx = 93,
    BinProcess::FL(1),    //TransformSkipFlag = 94,
    BinProcess::FL(1),    //TuJointCbcrResidualFlag = 95,
    BinProcess::ETC,      //LastSigCoeffXPrefix = 96,
    BinProcess::ETC,      //LastSigCoeffYPrefix = 97,
    BinProcess::ETC,      //LastSigCoeffXSuffix = 98,
    BinProcess::ETC,      //LastSigCoeffYSuffix = 99,
    BinProcess::FL(1),    //SbCodedFlag = 100,
    BinProcess::FL(1),    //SigCoeffFlag = 101,
    BinProcess::FL(1),    //ParLevelFlag = 102,
    BinProcess::FL(1),    //AbsLevelGtxFlag = 103,
    BinProcess::ETC,      //AbsRemainder = 104,
    BinProcess::ETC,      //DecAbsLevel = 105,
    BinProcess::FL(1),    //CoeffSignFlag = 106,
    BinProcess::FL(1),    //EndOfSliceOneBit = 107,
    BinProcess::FL(1),    //EndOfTileOneBit = 108,
    BinProcess::FL(1),    //EndOfSubsetOneBit = 109,
];

lazy_static! {
// [[init_value[I[], P[], B[]], shift_idx[I[], P[], B[]]]]
pub static ref ctx_table: Vec<Vec<Vec<Vec<usize>>>> = vec![
    // alf_ctb_flag 0
    vec![
        vec![
            vec![62, 39, 39, 54, 39, 39, 31, 39, 39],
            vec![13, 23, 46, 4, 61, 54, 19, 46, 54],
            vec![33, 52, 46, 25, 61, 54, 25, 61, 54],
        ],
        vec![
            vec![0, 0, 0, 4, 0, 0, 1, 0, 0],
            vec![0, 0, 0, 4, 0, 0, 1, 0, 0],
            vec![0, 0, 0, 4, 0, 0, 1, 0, 0],
        ],
    ],
    // alf_use_aps 1
    vec![
        vec![vec![46], vec![46], vec![46]],
        vec![vec![0], vec![0], vec![0]],
    ],
    // alf_ctb_cc_cb_idc 2
    vec![
        vec![vec![18, 30, 31], vec![18, 21, 31], vec![25, 35, 38]],
        vec![vec![4, 1, 4], vec![4, 1, 4], vec![4, 1, 4]],
    ],
    // alf_ctb_cc_cr_idc 3
    vec![
        vec![vec![18, 30, 31], vec![18, 21, 31], vec![25, 28, 38]],
        vec![vec![4, 1, 4], vec![4, 1, 4], vec![4, 1, 4]],
    ],
    // alf_luma_fixed_filter_idx 4
    vec![],
    // alf_luma_prev_filter_idx 5
    vec![],
    // alf_ctb_filter_alt_idx 6
    vec![
        vec![vec![11, 11], vec![20, 12], vec![11, 26]],
        vec![vec![0, 0], vec![0, 0], vec![0, 0]],
    ],
    // alf_sao_merge_left_flag 7
    vec![
        vec![vec![60], vec![60], vec![2]],
        vec![vec![0], vec![0], vec![0]],
    ],
    // alf_sao_merge_up_flag 8
    vec![
        vec![vec![60], vec![60], vec![2]],
        vec![vec![0], vec![0], vec![0]],
    ],
    // alf_sao_type_idx_luma 9
    vec![
        vec![vec![13], vec![5], vec![2]],
        vec![vec![4], vec![4], vec![4]],
    ],
    // alf_sao_type_idx_chroma 10
    vec![
        vec![vec![13], vec![5], vec![2]],
        vec![vec![4], vec![4], vec![4]],
    ],
    // sao_offset_abs 11
    vec![],
    // sao_offset_sign_flag 12
    vec![],
    // sao_band_position 13
    vec![],
    // sao_eo_class_luma 14
    vec![],
    // sao_eo_class_chroma 15
    vec![],
    // split_cu_flag 16
    vec![
        vec![
            vec![19, 28, 38, 27, 29, 38, 20, 30, 31],
            vec![11, 35, 53, 12, 6, 30, 13, 15, 31],
            vec![18, 27, 15, 18, 28, 45, 26, 7, 23],
        ],
        vec![
            vec![12, 13, 8, 8, 13, 12, 5, 9, 9],
            vec![12, 13, 8, 8, 13, 12, 5, 9, 9],
            vec![12, 13, 8, 8, 13, 12, 5, 9, 9],
        ],
    ],
    // split_qt_flag 17
    vec![
        vec![
            vec![27, 6, 15, 25, 19, 37],
            vec![20, 14, 23, 18, 19, 6],
            vec![26, 36, 38, 18, 34, 21],
        ],
        vec![
            vec![0, 8, 8, 12, 12, 8],
            vec![0, 8, 8, 12, 12, 8],
            vec![0, 8, 8, 12, 12, 8],
        ],
    ],
    // mtt_split_cu_vertical_flag 18
    vec![
        vec![
            vec![43, 42, 29, 27, 44],
            vec![43, 35, 37, 34, 52],
            vec![43, 42, 37, 42, 44],
        ],
        vec![
            vec![9, 8, 9, 8, 5],
            vec![9, 8, 9, 8, 5],
            vec![9, 8, 9, 8, 5],
        ],
    ],
    // mtt_split_cu_binary_flag 19
    vec![
        vec![
            vec![36, 45, 36, 45],
            vec![43, 37, 21, 22],
            vec![28, 29, 28, 29],
        ],
        vec![
            vec![12, 13, 12, 13],
            vec![12, 13, 12, 13],
            vec![12, 13, 12, 13],
        ],
    ],
    // non_inter_flag (P/B only) 20
    vec![
        vec![vec![0, 0], vec![25, 12], vec![25, 20]],
        vec![vec![0, 0], vec![1, 0], vec![1, 0]],
    ],
    // cu_skip_flag 21
    vec![
        vec![vec![0, 26, 28], vec![57, 59, 45], vec![57, 60, 46]],
        vec![vec![5, 4, 8], vec![5, 4, 8], vec![5, 4, 8]],
    ],
    // pred_mode_ibc_flag 22
    vec![
        vec![vec![17, 42, 36], vec![0, 57, 44], vec![0, 43, 45]],
        vec![vec![1, 5, 8], vec![1, 5, 8], vec![1, 5, 8]],
    ],
    // pred_mode_flag (P/B only) 23
    vec![
        vec![vec![0, 0], vec![40, 35], vec![40, 35]],
        vec![vec![0, 0], vec![5, 1], vec![5, 1]],
    ],
    // pred_mode_plt_flag 24
    vec![
        vec![vec![25], vec![0], vec![17]],
        vec![vec![1], vec![1], vec![1]],
    ],
    // cu_act_enabled_flag 25
    vec![
        vec![vec![52], vec![46], vec![46]],
        vec![vec![1], vec![1], vec![1]],
    ],
    // intra_bdpcm_luma_flag 26
    vec![
        vec![vec![19], vec![40], vec![19]],
        vec![vec![1], vec![1], vec![1]],
    ],
    // intra_bdpcm_luma_dir_flag 27
    vec![
        vec![vec![35], vec![36], vec![21]],
        vec![vec![4], vec![4], vec![4]],
    ],
    // intra_mip_flag 28
    vec![
        vec![
            vec![33, 49, 50, 25],
            vec![41, 57, 58, 26],
            vec![56, 57, 50, 26],
        ],
        vec![vec![9, 10, 9, 6], vec![9, 10, 9, 6], vec![9, 10, 9, 6]],
    ],
    // intra_mip_transpose_flag 29
    vec![],
    // intra_mip_mode 30
    vec![],
    // intra_luma_ref_idx 31
    vec![
        vec![vec![25, 60], vec![25, 58], vec![25, 59]],
        vec![vec![5, 8], vec![5, 8], vec![5, 8]],
    ],
    // intra_subpartitions_mode_flag 32
    vec![
        vec![vec![33], vec![33], vec![33]],
        vec![vec![9], vec![9], vec![9]],
    ],
    // intra_subpartitions_splilt_flag 33
    vec![
        vec![vec![43], vec![36], vec![43]],
        vec![vec![2], vec![2], vec![2]],
    ],
    // intra_luma_mpm_flag 34
    vec![
        vec![vec![45], vec![36], vec![44]],
        vec![vec![6], vec![6], vec![6]],
    ],
    // intra_luma_not_planar_flag 35
    vec![
        vec![vec![13, 28], vec![12, 20], vec![13, 6]],
        vec![vec![1, 5], vec![1, 5], vec![1, 5]],
    ],
    // intra_luma_mpm_idx 36
    vec![],
    // intra_luma_mpm_remainder 37
    vec![],
    // intra_bdpcm_chroma_flag 38
    vec![
        vec![vec![1], vec![0], vec![0]],
        vec![vec![1], vec![1], vec![1]],
    ],
    // intra_bdpcm_chroma_dir_flag 39
    vec![
        vec![vec![27], vec![13], vec![28]],
        vec![vec![0], vec![0], vec![0]],
    ],
    // cclm_mode_flag 40
    vec![
        vec![vec![59], vec![34], vec![26]],
        vec![vec![4], vec![4], vec![4]],
    ],
    // cclm_mode_idx 41
    vec![
        vec![vec![27], vec![27], vec![27]],
        vec![vec![9], vec![9], vec![9]],
    ],
    // intra_chroma_pred_mode 42
    vec![
        vec![vec![34], vec![25], vec![25]],
        vec![vec![5], vec![5], vec![5]],
    ],
    // palette_predictor_run 43
    vec![],
    // num_signalled_palette_entries 44
    vec![],
    // new_palette_entries 45
    vec![],
    // palette_escape_val_present_flag 46
    vec![],
    // palette_idx_idc 47
    vec![],
    // palette_escape_val 48
    vec![],
    // general_merge_flag 49
    vec![
        vec![vec![26], vec![21], vec![6]],
        vec![vec![4], vec![4], vec![4]],
    ],
    // inter_pred_idc (P/B only) 50
    vec![
        vec![
            vec![0, 0, 0, 0, 0, 0],
            vec![7, 6, 5, 12, 4, 40],
            vec![14, 13, 5, 4, 3, 40],
        ],
        vec![
            vec![0, 0, 0, 0, 0, 0],
            vec![0, 0, 1, 4, 4, 0],
            vec![0, 0, 1, 4, 4, 0],
        ],
    ],
    // inter_affine_flag (P/B only) 51
    vec![
        vec![vec![0, 0, 0], vec![12, 13, 14], vec![19, 13, 6]],
        vec![vec![0, 0, 0], vec![4, 0, 0], vec![4, 0, 0]],
    ],
    // cu_affine_type_flag (P/B only) 52
    vec![
        vec![vec![0], vec![35], vec![35]],
        vec![vec![0], vec![4], vec![4]],
    ],
    // sym_mvd_flag (P/B only) 53
    vec![
        vec![vec![0], vec![28], vec![28]],
        vec![vec![0], vec![5], vec![5]],
    ],
    // ref_idx_l0 54
    vec![
        vec![vec![0, 0], vec![20, 35], vec![5, 35]],
        vec![vec![0, 0], vec![0, 4], vec![0, 4]],
    ],
    // ref_idx_l1 55
    vec![
        vec![vec![0, 0], vec![20, 35], vec![5, 35]],
        vec![vec![0, 0], vec![0, 4], vec![0, 4]],
    ],
    // mvp_l0_flag 56
    vec![
        vec![vec![42], vec![34], vec![34]],
        vec![vec![12], vec![12], vec![12]],
    ],
    // mvp_l1_flag 57
    vec![
        vec![vec![42], vec![34], vec![34]],
        vec![vec![12], vec![12], vec![12]],
    ],
    // amvr_flag (P/B only) 58
    vec![
        vec![vec![0, 0], vec![59, 58], vec![59, 50]],
        vec![vec![0, 0], vec![0, 0], vec![0, 0]],
    ],
    // amvr_precision_idx 59
    vec![
        vec![vec![35, 34, 35], vec![60, 48, 60], vec![38, 26, 60]],
        vec![vec![4, 5, 0], vec![4, 5, 0], vec![4, 5, 0]],
    ],
    // bcw_idx (P/B only) 60
    vec![
        vec![vec![0], vec![4], vec![5]],
        vec![vec![0], vec![1], vec![1]],
    ],
    // cu_coded_flag 61
    vec![
        vec![vec![6], vec![5], vec![12]],
        vec![vec![4], vec![4], vec![4]],
    ],
    //vec![
        //vec![vec![0, 0], vec![56, 57], vec![41, 57]],
        //vec![vec![0, 0], vec![1, 5], vec![1, 5]],
    //],
    // cu_sbt_flag (P/B only) 62
    vec![
        vec![vec![0, 0], vec![56, 57], vec![41, 57]],
        vec![vec![0, 0], vec![1, 5], vec![1, 5]],
    ],
    // cu_sbt_quad_flag (P/B only) 63
    vec![
        vec![vec![0], vec![42], vec![42]],
        vec![vec![0], vec![10], vec![10]],
    ],
    // cu_sbt_horizontal_flag (P/B only) 64
    vec![
        vec![vec![0, 0, 0], vec![20, 43, 12], vec![35, 51, 27]],
        vec![vec![0, 0, 0], vec![8, 4, 1], vec![8, 4, 1]],
    ],
    // cu_sbt_pos_flag (P/B only) 65
    vec![
        vec![vec![0], vec![28], vec![28]],
        vec![vec![0], vec![13], vec![13]],
    ],
    // lfnst_idx 66
    vec![
        vec![vec![28, 52, 42], vec![37, 45, 27], vec![52, 37, 27]],
        vec![vec![9, 9, 10], vec![9, 9, 10], vec![9, 9, 10]],
    ],
    // mts_idx 67
    vec![
        vec![vec![29, 0, 28, 0], vec![45, 40, 27, 0], vec![45, 25, 27, 0]],
        vec![vec![8, 0, 9, 0], vec![8, 0, 9, 0], vec![8, 0, 9, 0]],
    ],
    // copy_above_palette_indices_flag 68
    vec![
        vec![vec![42], vec![59], vec![50]],
        vec![vec![9], vec![9], vec![9]],
    ],
    // palette_transpose_flag 69
    vec![
        vec![vec![42], vec![42], vec![35]],
        vec![vec![5], vec![5], vec![5]],
    ],
    // run_copy_flag 70
    vec![
        vec![
            vec![50, 37, 45, 30, 46, 45, 38, 46],
            vec![51, 30, 30, 38, 23, 38, 53, 46],
            vec![58, 45, 45, 30, 38, 45, 38, 46],
        ],
        vec![
            vec![9, 6, 9, 10, 5, 0, 9, 5],
            vec![9, 6, 9, 10, 5, 0, 9, 5],
            vec![9, 6, 9, 10, 5, 0, 9, 5],
        ],
    ],
    // regular_merge_flag (P/B only) 71
    vec![
        vec![vec![0, 0], vec![38, 7], vec![46, 15]],
        vec![vec![0, 0], vec![5, 5], vec![5, 5]],
    ],
    // mmvd_merge_flag (P/B only) 72
    vec![
        vec![vec![0], vec![26], vec![25]],
        vec![vec![0], vec![4], vec![4]],
    ],
    // mmvd_cand_flag (P/B only) 73
    vec![
        vec![vec![0], vec![43], vec![43]],
        vec![vec![0], vec![10], vec![10]],
    ],
    // mmvd_distance_idx (P/B only) 74
    vec![
        vec![vec![0], vec![60], vec![59]],
        vec![vec![0], vec![0], vec![0]],
    ],
    // mmvd_direction_idx 75
    vec![],
    // ciip_flag (P/B only) 76
    vec![
        vec![vec![0], vec![57], vec![57]],
        vec![vec![0], vec![1], vec![1]],
    ],
    // merge_subblock_flag (P/B only) 77
    vec![
        vec![vec![0, 0, 0], vec![48, 57, 44], vec![25, 58, 45]],
        vec![vec![0, 0, 0], vec![4, 4, 4], vec![4, 4, 4]],
    ],
    // merge_subblock_idx (P/B only) 78
    vec![
        vec![vec![0], vec![5], vec![4]],
        vec![vec![0], vec![0], vec![0]],
    ],
    // merge_idx 79
    vec![
        vec![vec![34], vec![20], vec![18]],
        vec![vec![4], vec![4], vec![4]],
    ],
    // merge_gpm_partition_idx 80
    vec![],
    // merge_gpm_idx0 81
    vec![
        vec![vec![34], vec![20], vec![18]],
        vec![vec![4], vec![4], vec![4]],
    ],
    // merge_gpm_idx1 82
    vec![
        vec![vec![34], vec![20], vec![18]],
        vec![vec![4], vec![4], vec![4]],
    ],
    // abs_mvd_greater0_flag 83
    vec![
        vec![vec![14], vec![44], vec![51]],
        vec![vec![9], vec![9], vec![9]],
    ],
    // abs_mvd_greater1_flag 84
    vec![
        vec![vec![45], vec![43], vec![36]],
        vec![vec![5], vec![5], vec![5]],
    ],
    // abs_mvd_minus2 85
    vec![],
    // mvd_sign_flag 86
    vec![],
    // tu_y_coded_flag 87
    vec![
        vec![vec![15, 12, 5, 7], vec![23, 5, 20, 7], vec![15, 6, 5, 14]],
        vec![vec![5, 1, 8, 9], vec![5, 1, 8, 9], vec![5, 1, 8, 9]],
    ],
    // tu_cb_coded_flag 88
    vec![
        vec![vec![12, 21], vec![25, 28], vec![25, 37]],
        vec![vec![5, 0], vec![5, 0], vec![5, 0]],
    ],
    // tu_cr_coded_flag 89
    vec![
        vec![vec![33, 28, 36], vec![25, 29, 45], vec![9, 36, 45]],
        vec![vec![2, 1, 0], vec![2, 1, 0], vec![2, 1, 0]],
    ],
    // cu_qp_delta_abs 90
    vec![
        vec![vec![35, 35], vec![35, 35], vec![35, 35]],
        vec![vec![8, 8], vec![8, 8], vec![8, 8]],
    ],
    // cu_qp_delta_sign_flag 91
    vec![],
    // cu_chroma_qp_offset_flag 92
    vec![
        vec![vec![35], vec![35], vec![35]],
        vec![vec![8], vec![8], vec![8]],
    ],
    // cu_chroma_qp_offset_idx 93
    vec![
        vec![vec![35], vec![35], vec![35]],
        vec![vec![8], vec![8], vec![8]],
    ],
    // transform_skip_flag 94
    vec![
        vec![vec![25, 9], vec![25, 9], vec![25, 17]],
        vec![vec![1, 1], vec![1, 1], vec![1, 1]],
    ],
    // tu_joint_cbcr_residual_flag 95
    vec![
        vec![vec![12, 21, 35], vec![27, 36, 45], vec![42, 43, 52]],
        vec![vec![1, 1, 0], vec![1, 1, 0], vec![1, 1, 0]],
    ],
    // last_sig_coeff_x_prefix 96
    vec![
        vec![
            vec![
                13, 5, 4, 21, 14, 4, 6, 14, 21, 11, 14, 7, 14, 5, 11, 21, 30, 22, 13, 42, 12, 4, 3,
            ],
            vec![
                6, 13, 12, 6, 6, 12, 14, 14, 13, 12, 29, 7, 6, 13, 36, 28, 14, 13, 5, 26, 12, 4, 18,
            ],
            vec![
                6, 6, 12, 14, 6, 4, 14, 7, 6, 4, 29, 7, 6, 6, 12, 28, 7, 13, 13, 35, 19, 5, 4,
            ],
        ],
        vec![
            vec![
                8, 5, 4, 5, 4, 4, 5, 4, 1, 0, 4, 1, 0, 0, 0, 0, 1, 0, 0, 0, 5, 4, 4,
            ],
            vec![
                8, 5, 4, 5, 4, 4, 5, 4, 1, 0, 4, 1, 0, 0, 0, 0, 1, 0, 0, 0, 5, 4, 4,
            ],
            vec![
                8, 5, 4, 5, 4, 4, 5, 4, 1, 0, 4, 1, 0, 0, 0, 0, 1, 0, 0, 0, 5, 4, 4,
            ],
        ],
    ],
    // last_sig_coeff_y_prefix 97
    vec![
        vec![
            vec![
                13, 5, 4, 6, 13, 11, 14, 6, 5, 3, 14, 22, 6, 4, 3, 6, 22, 29, 20, 34, 12, 4, 3,
            ],
            vec![
                5, 5, 12, 6, 6, 4, 6, 14, 5, 12, 14, 7, 13, 5, 13, 21, 14, 20, 12, 34, 11, 4, 18,
            ],
            vec![
                5, 5, 20, 13, 13, 19, 21, 6, 12, 12, 14, 14, 5, 4, 12, 13, 7, 13, 12, 41, 11, 5, 27,
            ],
        ],
        vec![
            vec![
                8, 5, 8, 5, 5, 4, 5, 5, 4, 0, 5, 4, 1, 0, 0, 1, 4, 0, 0, 0, 6, 5, 5,
            ],
            vec![
                8, 5, 8, 5, 5, 4, 5, 5, 4, 0, 5, 4, 1, 0, 0, 1, 4, 0, 0, 0, 6, 5, 5,
            ],
            vec![
                8, 5, 8, 5, 5, 4, 5, 5, 4, 0, 5, 4, 1, 0, 0, 1, 4, 0, 0, 0, 6, 5, 5,
            ],
        ],
    ],
    // last_sig_coeff_x_suffix 98
    vec![],
    // last_sig_coeff_y_suffix 99
    vec![],
    // sb_coded_flag 100
    vec![
        vec![
            vec![18, 31, 25, 15, 18, 20, 38],
            vec![25, 30, 25, 45, 18, 12, 29],
            vec![25, 45, 25, 14, 18, 35, 45],
        ],
        vec![
            vec![8, 5, 5, 8, 5, 8, 8],
            vec![8, 5, 5, 8, 5, 8, 8],
            vec![8, 5, 5, 8, 5, 8, 8],
        ],
    ],
    // sig_coeff_flag 101
    vec![
        vec![
            vec![
                25, 19, 28, 14, 25, 20, 29, 30, 19, 37, 30, 38, 11, 38, 46, 54, 27, 39, 39, 39, 44,
                39, 39, 39, 18, 39, 39, 39, 27, 39, 39, 39, 0, 39, 39, 39, 25, 27, 28, 37, 34, 53,
                53, 46, 19, 46, 38, 39, 52, 39, 39, 39, 11, 39, 39, 39, 19, 39, 39, 39, 25, 28, 38,
            ],
            vec![
                17, 41, 42, 29, 25, 49, 43, 37, 33, 58, 51, 30, 19, 38, 38, 46, 34, 54, 54, 39, 6,
                39, 39, 39, 19, 39, 54, 39, 19, 39, 39, 39, 56, 39, 39, 39, 17, 34, 35, 21, 41, 59,
                60, 38, 35, 45, 53, 54, 44, 39, 39, 39, 34, 38, 62, 39, 26, 39, 39, 39, 40, 35, 44,
            ],
            vec![
                17, 41, 49, 36, 1, 49, 50, 37, 48, 51, 58, 45, 26, 45, 53, 46, 49, 54, 61, 39, 35,
                39, 39, 39, 19, 54, 39, 39, 50, 39, 39, 39, 0, 39, 39, 39, 9, 49, 50, 36, 48, 59,
                59, 38, 34, 45, 38, 31, 58, 39, 39, 39, 34, 38, 54, 39, 41, 39, 39, 39, 25, 50, 37,
            ],
        ],
        vec![
            vec![
                12, 9, 9, 10, 9, 9, 9, 10, 8, 8, 8, 10, 9, 13, 8, 8, 8, 8, 8, 5, 8, 0, 0, 0, 8, 8,
                8, 8, 8, 0, 4, 4, 0, 0, 0, 0, 12, 12, 9, 13, 4, 5, 8, 9, 8, 12, 12, 8, 4, 0, 0, 0,
                8, 8, 8, 8, 4, 0, 0, 0, 13, 13, 8,
            ],
            vec![
                12, 9, 9, 10, 9, 9, 9, 10, 8, 8, 8, 10, 9, 13, 8, 8, 8, 8, 8, 5, 8, 0, 0, 0, 8, 8,
                8, 8, 8, 0, 4, 4, 0, 0, 0, 0, 12, 12, 9, 13, 4, 5, 8, 9, 8, 12, 12, 8, 4, 0, 0, 0,
                8, 8, 8, 8, 4, 0, 0, 0, 13, 13, 8,
            ],
            vec![
                12, 9, 9, 10, 9, 9, 9, 10, 8, 8, 8, 10, 9, 13, 8, 8, 8, 8, 8, 5, 8, 0, 0, 0, 8, 8,
                8, 8, 8, 0, 4, 4, 0, 0, 0, 0, 12, 12, 9, 13, 4, 5, 8, 9, 8, 12, 12, 8, 4, 0, 0, 0,
                8, 8, 8, 8, 4, 0, 0, 0, 13, 13, 8,
            ],
        ],
    ],
    // par_level_flag 102
    vec![
        vec![
            vec![
                33, 25, 18, 26, 34, 27, 25, 26, 19, 42, 35, 33, 19, 27, 35, 35, 34, 42, 20, 43, 20,
                33, 25, 26, 42, 19, 27, 26, 50, 35, 20, 43, 11,
            ],
            vec![
                18, 17, 33, 18, 26, 42, 25, 33, 26, 42, 27, 25, 34, 42, 42, 35, 26, 27, 42, 20, 20,
                25, 25, 26, 11, 19, 27, 33, 42, 35, 35, 43, 3,
            ],
            vec![
                33, 40, 25, 41, 26, 42, 25, 33, 26, 34, 27, 25, 41, 42, 42, 35, 33, 27, 35, 42, 43,
                33, 25, 26, 34, 19, 27, 33, 42, 43, 35, 43, 11,
            ],
        ],
        vec![
            vec![
                8, 9, 12, 13, 13, 13, 10, 13, 13, 13, 13, 13, 13, 13, 13, 13, 10, 13, 13, 13, 13,
                8, 12, 12, 12, 13, 13, 13, 13, 13, 13, 13, 6,
            ],
            vec![
                8, 9, 12, 13, 13, 13, 10, 13, 13, 13, 13, 13, 13, 13, 13, 13, 10, 13, 13, 13, 13,
                8, 12, 12, 12, 13, 13, 13, 13, 13, 13, 13, 6,
            ],
            vec![
                8, 9, 12, 13, 13, 13, 10, 13, 13, 13, 13, 13, 13, 13, 13, 13, 10, 13, 13, 13, 13,
                8, 12, 12, 12, 13, 13, 13, 13, 13, 13, 13, 6,
            ],
        ],
    ],
    // abs_level_gtx_flag 103
    vec![
        vec![
            vec![
                25, 25, 11, 27, 20, 21, 33, 12, 28, 21, 22, 34, 28, 29, 29, 30, 36, 29, 45, 30, 23,
                40, 33, 27, 28, 21, 37, 36, 37, 45, 38, 46, 25, 1, 40, 25, 33, 11, 17, 25, 25, 18,
                4, 17, 33, 26, 19, 13, 33, 19, 20, 28, 22, 40, 9, 25, 18, 26, 35, 25, 26, 35, 28,
                37, 11, 5, 5, 14, 10, 3, 3, 3,
            ],
            vec![
                0, 17, 26, 19, 35, 21, 25, 34, 20, 28, 29, 33, 27, 28, 29, 22, 34, 28, 44, 37, 38,
                0, 25, 19, 20, 13, 14, 57, 44, 30, 30, 23, 17, 0, 1, 17, 25, 18, 0, 9, 25, 33, 34,
                9, 25, 18, 26, 20, 25, 18, 19, 27, 29, 17, 9, 25, 10, 18, 4, 17, 33, 19, 20, 29,
                18, 11, 4, 28, 2, 10, 3, 3,
            ],
            vec![
                0, 0, 33, 34, 35, 21, 25, 34, 35, 28, 29, 40, 42, 43, 29, 30, 49, 36, 37, 45, 38,
                0, 40, 34, 43, 36, 37, 57, 52, 45, 38, 46, 25, 0, 0, 17, 25, 26, 0, 9, 25, 33, 19,
                0, 25, 33, 26, 20, 25, 33, 27, 35, 22, 25, 1, 25, 33, 26, 12, 25, 33, 27, 28, 37,
                19, 11, 4, 6, 3, 4, 4, 5,
            ],
        ],
        vec![
            vec![
                9, 5, 10, 13, 13, 10, 9, 10, 13, 13, 13, 9, 10, 10, 10, 13, 8, 9, 10, 10, 13, 8, 8,
                9, 12, 12, 10, 5, 9, 9, 9, 13, 1, 5, 9, 9, 9, 6, 5, 9, 10, 10, 9, 9, 9, 9, 9, 9, 6,
                8, 9, 9, 10, 1, 5, 8, 8, 9, 6, 6, 9, 8, 8, 9, 4, 2, 1, 6, 1, 1, 1, 1,
            ],
            vec![
                9, 5, 10, 13, 13, 10, 9, 10, 13, 13, 13, 9, 10, 10, 10, 13, 8, 9, 10, 10, 13, 8, 8,
                9, 12, 12, 10, 5, 9, 9, 9, 13, 1, 5, 9, 9, 9, 6, 5, 9, 10, 10, 9, 9, 9, 9, 9, 9, 6,
                8, 9, 9, 10, 1, 5, 8, 8, 9, 6, 6, 9, 8, 8, 9, 4, 2, 1, 6, 1, 1, 1, 1,
            ],
            vec![
                9, 5, 10, 13, 13, 10, 9, 10, 13, 13, 13, 9, 10, 10, 10, 13, 8, 9, 10, 10, 13, 8, 8,
                9, 12, 12, 10, 5, 9, 9, 9, 13, 1, 5, 9, 9, 9, 6, 5, 9, 10, 10, 9, 9, 9, 9, 9, 9, 6,
                8, 9, 9, 10, 1, 5, 8, 8, 9, 6, 6, 9, 8, 8, 9, 4, 2, 1, 6, 1, 1, 1, 1,
            ],
        ],
    ],
    // abs_remainder 104
    vec![],
    // dec_abs_level 105
    vec![],
    // coeff_sign_flag 106
    vec![
        vec![
            vec![12, 17, 46, 28, 25, 46],
            vec![5, 10, 53, 43, 25, 46],
            vec![35, 25, 46, 28, 33, 38],
        ],
        vec![
            vec![1, 4, 4, 5, 8, 8],
            vec![1, 4, 4, 5, 8, 8],
            vec![1, 4, 4, 5, 8, 8],
        ],
    ],
];
}

pub const c_rice_params: [usize; 32] = [
    0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 1, 1, 1, 1, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 3, 3, 3, 3,
];

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum CtxInc {
    Bypass,
    NA,
    Terminate,
    Number(usize),
    Invalid,
}

pub const CTX_INC_TABLE: [[CtxInc; 6]; 110] = [
    // alf_ctb_flag
    [
        CtxInc::Invalid,
        CtxInc::NA,
        CtxInc::NA,
        CtxInc::NA,
        CtxInc::NA,
        CtxInc::NA,
    ],
    // alf_use_aps_flag
    [
        CtxInc::Number(0),
        CtxInc::NA,
        CtxInc::NA,
        CtxInc::NA,
        CtxInc::NA,
        CtxInc::NA,
    ],
    // alf_ctb_cc_cb_idc
    [
        CtxInc::Invalid,
        CtxInc::Bypass,
        CtxInc::Bypass,
        CtxInc::Bypass,
        CtxInc::Bypass,
        CtxInc::Bypass,
    ],
    // alf_ctb_cc_cr_idc
    [
        CtxInc::Invalid,
        CtxInc::Bypass,
        CtxInc::Bypass,
        CtxInc::Bypass,
        CtxInc::Bypass,
        CtxInc::Bypass,
    ],
    // alf_luma_fixed_filter_idx
    [CtxInc::Bypass; 6],
    // alf_luma_prev_filter_idx
    [CtxInc::Bypass; 6],
    // alf_ctb_filter_alt_idx
    [CtxInc::Invalid; 6],
    // sao_merge_left_flag
    [
        CtxInc::Number(0),
        CtxInc::NA,
        CtxInc::NA,
        CtxInc::NA,
        CtxInc::NA,
        CtxInc::NA,
    ],
    // sao_merge_up_flag
    [
        CtxInc::Number(0),
        CtxInc::NA,
        CtxInc::NA,
        CtxInc::NA,
        CtxInc::NA,
        CtxInc::NA,
    ],
    // sao_type_idx_luma
    [
        CtxInc::Number(0),
        CtxInc::Bypass,
        CtxInc::NA,
        CtxInc::NA,
        CtxInc::NA,
        CtxInc::NA,
    ],
    // sao_type_idx_chroma
    [
        CtxInc::Number(0),
        CtxInc::Bypass,
        CtxInc::NA,
        CtxInc::NA,
        CtxInc::NA,
        CtxInc::NA,
    ],
    // sao_offset_abs
    [
        CtxInc::Bypass,
        CtxInc::Bypass,
        CtxInc::Bypass,
        CtxInc::Bypass,
        CtxInc::Bypass,
        CtxInc::NA,
    ],
    // sao_offset_sign_flag
    [
        CtxInc::Bypass,
        CtxInc::NA,
        CtxInc::NA,
        CtxInc::NA,
        CtxInc::NA,
        CtxInc::NA,
    ],
    // sao_band_position
    [
        CtxInc::Bypass,
        CtxInc::Bypass,
        CtxInc::Bypass,
        CtxInc::Bypass,
        CtxInc::Bypass,
        CtxInc::Bypass,
    ],
    // sao_eo_class_luma
    [
        CtxInc::NA,
        CtxInc::NA,
        CtxInc::Bypass,
        CtxInc::Bypass,
        CtxInc::Bypass,
        CtxInc::Bypass,
    ],
    // sao_eo_class_chroma
    [
        CtxInc::NA,
        CtxInc::NA,
        CtxInc::Bypass,
        CtxInc::Bypass,
        CtxInc::Bypass,
        CtxInc::Bypass,
    ],
    // split_cu_flag
    [
        CtxInc::Invalid,
        CtxInc::NA,
        CtxInc::NA,
        CtxInc::NA,
        CtxInc::NA,
        CtxInc::NA,
    ],
    // split_qt_flag
    [
        CtxInc::Invalid,
        CtxInc::NA,
        CtxInc::NA,
        CtxInc::NA,
        CtxInc::NA,
        CtxInc::NA,
    ],
    // mtt_split_cu_vertical_flag
    [
        CtxInc::Invalid,
        CtxInc::NA,
        CtxInc::NA,
        CtxInc::NA,
        CtxInc::NA,
        CtxInc::NA,
    ],
    // mtt_split_cu_binary_flag
    [
        CtxInc::Invalid,
        CtxInc::NA,
        CtxInc::NA,
        CtxInc::NA,
        CtxInc::NA,
        CtxInc::NA,
    ],
    // non_inter_flag
    [
        CtxInc::Invalid,
        CtxInc::NA,
        CtxInc::NA,
        CtxInc::NA,
        CtxInc::NA,
        CtxInc::NA,
    ],
    // cu_skip_flag
    [
        CtxInc::Invalid,
        CtxInc::NA,
        CtxInc::NA,
        CtxInc::NA,
        CtxInc::NA,
        CtxInc::NA,
    ],
    // pred_mode_ibc_flag
    [
        CtxInc::Invalid,
        CtxInc::NA,
        CtxInc::NA,
        CtxInc::NA,
        CtxInc::NA,
        CtxInc::NA,
    ],
    // pred_mode_flag
    [
        CtxInc::Invalid,
        CtxInc::NA,
        CtxInc::NA,
        CtxInc::NA,
        CtxInc::NA,
        CtxInc::NA,
    ],
    // pred_mode_plt_flag
    [
        CtxInc::Number(0),
        CtxInc::NA,
        CtxInc::NA,
        CtxInc::NA,
        CtxInc::NA,
        CtxInc::NA,
    ],
    // cu_act_enabled_flag
    [
        CtxInc::Number(0),
        CtxInc::NA,
        CtxInc::NA,
        CtxInc::NA,
        CtxInc::NA,
        CtxInc::NA,
    ],
    // intra_bdpcm_luma_flag
    [
        CtxInc::Number(0),
        CtxInc::NA,
        CtxInc::NA,
        CtxInc::NA,
        CtxInc::NA,
        CtxInc::NA,
    ],
    // intra_bdpcm_luma_dir_flag
    [
        CtxInc::Number(0),
        CtxInc::NA,
        CtxInc::NA,
        CtxInc::NA,
        CtxInc::NA,
        CtxInc::NA,
    ],
    // intra_mip_flag
    [
        CtxInc::Invalid,
        CtxInc::NA,
        CtxInc::NA,
        CtxInc::NA,
        CtxInc::NA,
        CtxInc::NA,
    ],
    // intra_mip_transposed_flag
    [
        CtxInc::Bypass,
        CtxInc::NA,
        CtxInc::NA,
        CtxInc::NA,
        CtxInc::NA,
        CtxInc::NA,
    ],
    // intra_mip_mode
    [
        CtxInc::Bypass,
        CtxInc::Bypass,
        CtxInc::Bypass,
        CtxInc::Bypass,
        CtxInc::Bypass,
        CtxInc::NA,
    ],
    // intra_luma_ref_idx
    [
        CtxInc::Number(0),
        CtxInc::Number(1),
        CtxInc::NA,
        CtxInc::NA,
        CtxInc::NA,
        CtxInc::NA,
    ],
    // intra_subpartitions_mode_flag
    [
        CtxInc::Number(0),
        CtxInc::NA,
        CtxInc::NA,
        CtxInc::NA,
        CtxInc::NA,
        CtxInc::NA,
    ],
    // intra_subpartitions_split_flag
    [
        CtxInc::Number(0),
        CtxInc::NA,
        CtxInc::NA,
        CtxInc::NA,
        CtxInc::NA,
        CtxInc::NA,
    ],
    // intra_luma_mpm_flag
    [
        CtxInc::Number(0),
        CtxInc::NA,
        CtxInc::NA,
        CtxInc::NA,
        CtxInc::NA,
        CtxInc::NA,
    ],
    // intra_luma_not_planar_flag
    [
        CtxInc::Invalid,
        CtxInc::NA,
        CtxInc::NA,
        CtxInc::NA,
        CtxInc::NA,
        CtxInc::NA,
    ],
    // intra_luma_mpm_idx
    [
        CtxInc::Bypass,
        CtxInc::Bypass,
        CtxInc::Bypass,
        CtxInc::Bypass,
        CtxInc::NA,
        CtxInc::NA,
    ],
    // intra_luma_mpm_remainder
    [
        CtxInc::Bypass,
        CtxInc::Bypass,
        CtxInc::Bypass,
        CtxInc::Bypass,
        CtxInc::Bypass,
        CtxInc::Bypass,
    ],
    // intra_bdpcm_chroma_flag
    [
        CtxInc::Number(0),
        CtxInc::NA,
        CtxInc::NA,
        CtxInc::NA,
        CtxInc::NA,
        CtxInc::NA,
    ],
    // intra_bdpcm_chroma_dir_flag
    [
        CtxInc::Number(0),
        CtxInc::NA,
        CtxInc::NA,
        CtxInc::NA,
        CtxInc::NA,
        CtxInc::NA,
    ],
    // cclm_mode_flag
    [
        CtxInc::Number(0),
        CtxInc::NA,
        CtxInc::NA,
        CtxInc::NA,
        CtxInc::NA,
        CtxInc::NA,
    ],
    // cclm_mode_idx
    [
        CtxInc::Number(0),
        CtxInc::Bypass,
        CtxInc::NA,
        CtxInc::NA,
        CtxInc::NA,
        CtxInc::NA,
    ],
    // intra_chroma_pred_mode
    [
        CtxInc::Number(0),
        CtxInc::Bypass,
        CtxInc::Bypass,
        CtxInc::NA,
        CtxInc::NA,
        CtxInc::NA,
    ],
    // palette_predictor_run
    [
        CtxInc::Bypass,
        CtxInc::Bypass,
        CtxInc::Bypass,
        CtxInc::Bypass,
        CtxInc::Bypass,
        CtxInc::Bypass,
    ],
    // num_signalled_palette_entries
    [
        CtxInc::Bypass,
        CtxInc::Bypass,
        CtxInc::Bypass,
        CtxInc::Bypass,
        CtxInc::Bypass,
        CtxInc::Bypass,
    ],
    // new_palette_entries
    [
        CtxInc::Bypass,
        CtxInc::Bypass,
        CtxInc::Bypass,
        CtxInc::Bypass,
        CtxInc::Bypass,
        CtxInc::Bypass,
    ],
    // palette_escape_val_present_flag
    [
        CtxInc::Bypass,
        CtxInc::NA,
        CtxInc::NA,
        CtxInc::NA,
        CtxInc::NA,
        CtxInc::NA,
    ],
    // palette_idx_idc
    [
        CtxInc::Bypass,
        CtxInc::Bypass,
        CtxInc::Bypass,
        CtxInc::Bypass,
        CtxInc::Bypass,
        CtxInc::Bypass,
    ],
    // palette_escape_val
    [
        CtxInc::Bypass,
        CtxInc::Bypass,
        CtxInc::Bypass,
        CtxInc::Bypass,
        CtxInc::Bypass,
        CtxInc::Bypass,
    ],
    // general_merge_flag
    [
        CtxInc::Number(0),
        CtxInc::Bypass,
        CtxInc::Bypass,
        CtxInc::Bypass,
        CtxInc::Bypass,
        CtxInc::Bypass,
    ],
    // inter_pred_idc
    [
        CtxInc::Invalid,
        CtxInc::Number(5),
        CtxInc::NA,
        CtxInc::NA,
        CtxInc::NA,
        CtxInc::NA,
    ],
    // inter_affine_flag
    [CtxInc::Invalid; 6],
    // cu_affine_type_flag
    [
        CtxInc::Number(0),
        CtxInc::NA,
        CtxInc::NA,
        CtxInc::NA,
        CtxInc::NA,
        CtxInc::NA,
    ],
    // sym_mvd_flag
    [
        CtxInc::Number(0),
        CtxInc::NA,
        CtxInc::NA,
        CtxInc::NA,
        CtxInc::NA,
        CtxInc::NA,
    ],
    // ref_idx_l0
    [
        CtxInc::Number(0),
        CtxInc::Number(1),
        CtxInc::Bypass,
        CtxInc::Bypass,
        CtxInc::Bypass,
        CtxInc::Bypass,
    ],
    // ref_idx_l1
    [
        CtxInc::Number(0),
        CtxInc::Number(1),
        CtxInc::Bypass,
        CtxInc::Bypass,
        CtxInc::Bypass,
        CtxInc::Bypass,
    ],
    // mvp_l0_flag
    [CtxInc::Invalid; 6],
    // mvp_l1_flag
    [CtxInc::Invalid; 6],
    // amvr_flag
    [CtxInc::Invalid; 6],
    // amvr_precision_idx
    [
        CtxInc::Invalid,
        CtxInc::Number(1),
        CtxInc::NA,
        CtxInc::NA,
        CtxInc::NA,
        CtxInc::NA,
    ],
    // bcw_idx (ectx.no_backward_pred_flag==false)
    [
        CtxInc::Number(0),
        CtxInc::Bypass,
        CtxInc::NA,
        CtxInc::NA,
        CtxInc::NA,
        CtxInc::NA,
    ],
    // cu_coded_flag
    [
        CtxInc::Number(0),
        CtxInc::NA,
        CtxInc::NA,
        CtxInc::NA,
        CtxInc::NA,
        CtxInc::NA,
    ],
    // cu_sbt_flag
    [
        CtxInc::Invalid,
        CtxInc::NA,
        CtxInc::NA,
        CtxInc::NA,
        CtxInc::NA,
        CtxInc::NA,
    ],
    // cu_sbt_quad_flag
    [
        CtxInc::Number(0),
        CtxInc::NA,
        CtxInc::NA,
        CtxInc::NA,
        CtxInc::NA,
        CtxInc::NA,
    ],
    // cu_sbt_horizontal_flag
    [
        CtxInc::Invalid,
        CtxInc::NA,
        CtxInc::NA,
        CtxInc::NA,
        CtxInc::NA,
        CtxInc::NA,
    ],
    // cu_sbt_pos_flag
    [
        CtxInc::Number(0),
        CtxInc::NA,
        CtxInc::NA,
        CtxInc::NA,
        CtxInc::NA,
        CtxInc::NA,
    ],
    // lfnst_idx
    [
        CtxInc::Invalid,
        CtxInc::Number(2),
        CtxInc::NA,
        CtxInc::NA,
        CtxInc::NA,
        CtxInc::NA,
    ],
    // mts_idx
    [
        CtxInc::Number(0),
        CtxInc::Number(1),
        CtxInc::Number(2),
        CtxInc::Number(3),
        CtxInc::NA,
        CtxInc::NA,
    ],
    // copy_above_palette_indices_flag
    [
        CtxInc::Number(0),
        CtxInc::NA,
        CtxInc::NA,
        CtxInc::NA,
        CtxInc::NA,
        CtxInc::NA,
    ],
    // palette_transpose_flag
    [
        CtxInc::Number(0),
        CtxInc::NA,
        CtxInc::NA,
        CtxInc::NA,
        CtxInc::NA,
        CtxInc::NA,
    ],
    // run_copy_flag
    [
        CtxInc::Invalid,
        CtxInc::NA,
        CtxInc::NA,
        CtxInc::NA,
        CtxInc::NA,
        CtxInc::NA,
    ],
    // regular_merge_flag
    [
        CtxInc::Invalid,
        CtxInc::Bypass,
        CtxInc::Bypass,
        CtxInc::Bypass,
        CtxInc::Bypass,
        CtxInc::Bypass,
    ],
    // mmvd_merge_flag
    [
        CtxInc::Number(0),
        CtxInc::NA,
        CtxInc::NA,
        CtxInc::NA,
        CtxInc::NA,
        CtxInc::NA,
    ],
    // mmvd_cand_flag
    [
        CtxInc::Number(0),
        CtxInc::NA,
        CtxInc::NA,
        CtxInc::NA,
        CtxInc::NA,
        CtxInc::NA,
    ],
    // mmvd_distance_idx
    [
        CtxInc::Number(0),
        CtxInc::Bypass,
        CtxInc::Bypass,
        CtxInc::Bypass,
        CtxInc::Bypass,
        CtxInc::Bypass,
    ],
    // mmvd_direction_idx
    [
        CtxInc::Bypass,
        CtxInc::Bypass,
        CtxInc::NA,
        CtxInc::NA,
        CtxInc::NA,
        CtxInc::NA,
    ],
    // ciip_flag
    [
        CtxInc::Number(0),
        CtxInc::NA,
        CtxInc::NA,
        CtxInc::NA,
        CtxInc::NA,
        CtxInc::NA,
    ],
    // merge_subblock_flag
    [
        CtxInc::Invalid,
        CtxInc::NA,
        CtxInc::NA,
        CtxInc::NA,
        CtxInc::NA,
        CtxInc::NA,
    ],
    // merge_subblock_idx
    [
        CtxInc::Number(0),
        CtxInc::Bypass,
        CtxInc::Bypass,
        CtxInc::Bypass,
        CtxInc::Bypass,
        CtxInc::NA,
    ],
    // merge_idx
    [
        CtxInc::Number(0),
        CtxInc::Bypass,
        CtxInc::Bypass,
        CtxInc::Bypass,
        CtxInc::Bypass,
        CtxInc::NA,
    ],
    // merge_gpm_partition_idx
    [
        CtxInc::Bypass,
        CtxInc::Bypass,
        CtxInc::Bypass,
        CtxInc::Bypass,
        CtxInc::Bypass,
        CtxInc::Bypass,
    ],
    // merge_gpm_idx0
    [
        CtxInc::Number(0),
        CtxInc::Bypass,
        CtxInc::Bypass,
        CtxInc::Bypass,
        CtxInc::Bypass,
        CtxInc::NA,
    ],
    // merge_gpm_idx1
    [
        CtxInc::Number(0),
        CtxInc::Bypass,
        CtxInc::Bypass,
        CtxInc::Bypass,
        CtxInc::NA,
        CtxInc::NA,
    ],
    // abs_mvd_greater0_flag
    [
        CtxInc::Number(0),
        CtxInc::NA,
        CtxInc::NA,
        CtxInc::NA,
        CtxInc::NA,
        CtxInc::NA,
    ],
    // abs_mvd_greater1_flag
    [
        CtxInc::Number(0),
        CtxInc::NA,
        CtxInc::NA,
        CtxInc::NA,
        CtxInc::NA,
        CtxInc::NA,
    ],
    // abs_mvd_minus2
    [
        CtxInc::Bypass,
        CtxInc::Bypass,
        CtxInc::Bypass,
        CtxInc::Bypass,
        CtxInc::Bypass,
        CtxInc::Bypass,
    ],
    // mvd_sign_flag
    [
        CtxInc::Bypass,
        CtxInc::NA,
        CtxInc::NA,
        CtxInc::NA,
        CtxInc::NA,
        CtxInc::NA,
    ],
    // tu_y_coded_flag
    [
        CtxInc::Invalid,
        CtxInc::NA,
        CtxInc::NA,
        CtxInc::NA,
        CtxInc::NA,
        CtxInc::NA,
    ],
    // tu_cb_coded_flag
    [
        CtxInc::Invalid,
        CtxInc::NA,
        CtxInc::NA,
        CtxInc::NA,
        CtxInc::NA,
        CtxInc::NA,
    ],
    // tu_cr_coded_flag
    [
        CtxInc::Invalid,
        CtxInc::NA,
        CtxInc::NA,
        CtxInc::NA,
        CtxInc::NA,
        CtxInc::NA,
    ],
    // cu_qp_delta_abs
    [
        CtxInc::Number(0),
        CtxInc::Number(1),
        CtxInc::Number(1),
        CtxInc::Number(1),
        CtxInc::Number(1),
        CtxInc::Bypass,
    ],
    // cu_qp_delta_sign_flag
    [
        CtxInc::Bypass,
        CtxInc::NA,
        CtxInc::NA,
        CtxInc::NA,
        CtxInc::NA,
        CtxInc::NA,
    ],
    // cu_chroma_qp_offset_flag
    [
        CtxInc::Number(0),
        CtxInc::NA,
        CtxInc::NA,
        CtxInc::NA,
        CtxInc::NA,
        CtxInc::NA,
    ],
    // cu_chroma_qp_offset_idx
    [
        CtxInc::Number(0),
        CtxInc::Number(0),
        CtxInc::Number(0),
        CtxInc::Number(0),
        CtxInc::Number(0),
        CtxInc::NA,
    ],
    // transform_skip_flag
    [
        CtxInc::Invalid,
        CtxInc::NA,
        CtxInc::NA,
        CtxInc::NA,
        CtxInc::NA,
        CtxInc::NA,
    ],
    // tu_joint_cbcr_residual_flag
    [
        CtxInc::Invalid,
        CtxInc::NA,
        CtxInc::NA,
        CtxInc::NA,
        CtxInc::NA,
        CtxInc::NA,
    ],
    // last_sig_coeff_x_prefix
    [
        CtxInc::Invalid,
        CtxInc::Invalid,
        CtxInc::Invalid,
        CtxInc::Invalid,
        CtxInc::Invalid,
        CtxInc::Invalid,
    ],
    // last_sig_coeff_y_prefix
    [
        CtxInc::Invalid,
        CtxInc::Invalid,
        CtxInc::Invalid,
        CtxInc::Invalid,
        CtxInc::Invalid,
        CtxInc::Invalid,
    ],
    // last_sig_coeff_x_suffix
    [
        CtxInc::Bypass,
        CtxInc::Bypass,
        CtxInc::Bypass,
        CtxInc::Bypass,
        CtxInc::Bypass,
        CtxInc::Bypass,
    ],
    // last_sig_coeff_y_suffix
    [
        CtxInc::Bypass,
        CtxInc::Bypass,
        CtxInc::Bypass,
        CtxInc::Bypass,
        CtxInc::Bypass,
        CtxInc::Bypass,
    ],
    // sb_coded_flag
    [
        CtxInc::Invalid,
        CtxInc::NA,
        CtxInc::NA,
        CtxInc::NA,
        CtxInc::NA,
        CtxInc::NA,
    ],
    // sig_coeff_flag
    [
        CtxInc::Invalid,
        CtxInc::NA,
        CtxInc::NA,
        CtxInc::NA,
        CtxInc::NA,
        CtxInc::NA,
    ],
    // par_level_flag
    [
        CtxInc::Invalid,
        CtxInc::NA,
        CtxInc::NA,
        CtxInc::NA,
        CtxInc::NA,
        CtxInc::NA,
    ],
    // abs_level_gtx_flag
    [
        CtxInc::Invalid,
        CtxInc::NA,
        CtxInc::NA,
        CtxInc::NA,
        CtxInc::NA,
        CtxInc::NA,
    ],
    // abs_remainder
    [
        CtxInc::Bypass,
        CtxInc::Bypass,
        CtxInc::Bypass,
        CtxInc::Bypass,
        CtxInc::Bypass,
        CtxInc::Bypass,
    ],
    // dec_abs_level
    [
        CtxInc::Bypass,
        CtxInc::Bypass,
        CtxInc::Bypass,
        CtxInc::Bypass,
        CtxInc::Bypass,
        CtxInc::Bypass,
    ],
    // coeff_sign_flag
    [
        CtxInc::Invalid,
        CtxInc::NA,
        CtxInc::NA,
        CtxInc::NA,
        CtxInc::NA,
        CtxInc::NA,
    ],
    // end_of_slice_one_bit
    [
        CtxInc::Terminate,
        CtxInc::NA,
        CtxInc::NA,
        CtxInc::NA,
        CtxInc::NA,
        CtxInc::NA,
    ],
    // end_of_tile_one_bit
    [
        CtxInc::Terminate,
        CtxInc::NA,
        CtxInc::NA,
        CtxInc::NA,
        CtxInc::NA,
        CtxInc::NA,
    ],
    // end_of_subset_one_bit
    [
        CtxInc::Terminate,
        CtxInc::NA,
        CtxInc::NA,
        CtxInc::NA,
        CtxInc::NA,
        CtxInc::NA,
    ],
];

import calculate_bd_rate_against_x265 as bd_rate_x265
import optuna


def objective(trial):

    # Trial 132 finished with value: 0.8964687594870602 and parameters: {'quant_lv_pow': 0.5004010166085378, 'quant_qp_div_trellis': 5.218413785332902, 'quant_qp_div': 4.049512651290126, 'quant_lambda_mul': 1.2602364115635767, 'quant_lambda_mul_trellis': 1.2709404305806742, 'quant_lambda_offset': 4, 'quant_lambda_offset_trellis': 11, 'lv_offset_dq': 0.13731084642527322, 'lv_offset_dq_trellis': 0.15150746310196822, 'qp_div_dq': 3.9707361412344673, 'qp_div_dq_trellis': 4.40436638858524, 'lambda_mul_dq': 1.3439287209386943, 'lambda_mul_dq_trellis': 1.1282581658680004, 'lv_pow_dq': 0.5850246891437862, 'lv_pow_dq_trellis': 0.48592678233563835, 'cclm_pow': 0.45876507677563716, 'mpm_idx_pow': 0.4027128383607289, 'mpm_remainder_pow': 0.34385093742772876, 'non_planar_offset_dq': 2.600296541775863, 'non_planar_offset_dq_trellis': 2.2153597706635244, 'mpm_idx_offset_dq': 1.506942649768825, 'mpm_idx_offset_dq_trellis': 1.3660220587918186, 'mpm_remainder_mult_dq': 0.45641026081899505, 'mpm_remainder_mult_dq_trellis': 0.5007182023755636, 'mpm_remainder_offset_dq': 2.3529479122647867, 'mpm_remainder_offset_dq_trellis': 2.2973303527769664, 'planar_offset_dq': 1.0008109773984588, 'planar_offset_dq_trellis': 0.8946932717342444, 'header_bits_dq': 0.9821256417438512, 'header_bits_dq_trellis': 1.177287259225102, 'chroma_header_bits_dq': 1.1223906676063702, 'chroma_header_bits_dq_trellis': 1.4263605985223209}. Best is trial 132 with value: 0.8964687594870602.

    quant_lv_pow = trial.suggest_float("quant_lv_pow", 0.496, 0.502)
    quant_qp_div_trellis = trial.suggest_float("quant_qp_div_trellis", 5.2, 5.3)
    quant_qp_div = trial.suggest_float("quant_qp_div", 4.0, 4.15)
    quant_lambda_mul = trial.suggest_float("quant_lambda_mul", 1.26, 1.3)
    quant_lambda_mul_trellis = trial.suggest_float("quant_lambda_mul_trellis", 1.27, 1.31)
    quant_lambda_offset = trial.suggest_int("quant_lambda_offset", 3, 5)
    quant_lambda_offset_trellis = trial.suggest_int("quant_lambda_offset_trellis", 9, 11)

    lv_offset_dq = trial.suggest_float("lv_offset_dq", 0.13, 0.15)
    lv_offset_dq_trellis = trial.suggest_float("lv_offset_dq_trellis", 0.15, 0.18)
    qp_div_dq = trial.suggest_float("qp_div_dq", 3.9, 4.0)
    qp_div_dq_trellis = trial.suggest_float("qp_div_dq_trellis", 4.38, 4.41)
    lambda_mul_dq = trial.suggest_float("lambda_mul_dq", 1.26, 1.9)
    lambda_mul_dq_trellis = trial.suggest_float("lambda_mul_dq_trellis", 1.12, 1.15)
    lv_pow_dq = trial.suggest_float("lv_pow_dq", 0.575, 0.59)
    lv_pow_dq_trellis = trial.suggest_float("lv_pow_dq_trellis", 0.47, 0.49)
    cclm_pow = trial.suggest_float("cclm_pow", 0.458, 0.465)
    mpm_idx_pow = trial.suggest_float("mpm_idx_pow", 0.402, 0.41)
    mpm_remainder_pow = trial.suggest_float("mpm_remainder_pow", 0.34, 0.36)

    non_planar_offset_dq = trial.suggest_float("non_planar_offset_dq", 2.5, 2.65)
    non_planar_offset_dq_trellis = trial.suggest_float("non_planar_offset_dq_trellis", 2.1, 2.28)
    mpm_idx_offset_dq = trial.suggest_float("mpm_idx_offset_dq", 1.48, 1.55)
    mpm_idx_offset_dq_trellis = trial.suggest_float("mpm_idx_offset_dq_trellis", 1.3, 1.4)
    mpm_remainder_mult_dq = trial.suggest_float("mpm_remainder_mult_dq", 0.41, 0.46)
    mpm_remainder_mult_dq_trellis = trial.suggest_float("mpm_remainder_mult_dq_trellis", 0.48, 0.51)
    mpm_remainder_offset_dq = trial.suggest_float("mpm_remainder_offset_dq", 2.3, 2.4)
    mpm_remainder_offset_dq_trellis = trial.suggest_float("mpm_remainder_offset_dq_trellis", 2.28, 2.37)
    planar_offset_dq = trial.suggest_float("planar_offset_dq", 0.95, 1.05)
    planar_offset_dq_trellis = trial.suggest_float("planar_offset_dq_trellis", 0.78, 0.9)
    header_bits_dq = trial.suggest_float("header_bits_dq", 0.94, 1.0)
    header_bits_dq_trellis = trial.suggest_float("header_bits_dq_trellis", 1.16, 1.22)
    chroma_header_bits_dq = trial.suggest_float("chroma_header_bits_dq", 1.12, 1.16)
    chroma_header_bits_dq_trellis = trial.suggest_float("chroma_header_bits_dq_trellis", 1.35, 1.45)

    ex_params = f"quant_lv_pow={quant_lv_pow},quant_qp_div_trellis={quant_qp_div_trellis},quant_qp_div={quant_qp_div},quant_lambda_mul={quant_lambda_mul},quant_lambda_mul_trellis={quant_lambda_mul_trellis},quant_lambda_offset={quant_lambda_offset},quant_lambda_offset_trellis={quant_lambda_offset_trellis},lv_offset_dq={lv_offset_dq},lv_offset_dq_trellis={lv_offset_dq_trellis},qp_div_dq={qp_div_dq},qp_div_dq_trellis={qp_div_dq_trellis},lambda_mul_dq={lambda_mul_dq},lambda_mul_dq_trellis={lambda_mul_dq_trellis},lv_pow_dq={lv_pow_dq},lv_pow_dq_trellis={lv_pow_dq_trellis},cclm_pow={cclm_pow},mpm_idx_pow={mpm_idx_pow},mpm_remainder_pow={mpm_remainder_pow},non_planar_offset_dq={non_planar_offset_dq},non_planar_offset_dq_trellis={non_planar_offset_dq_trellis},mpm_idx_offset_dq={mpm_idx_offset_dq},mpm_idx_offset_dq_trellis={mpm_idx_offset_dq_trellis},mpm_remainder_mult_dq={mpm_remainder_mult_dq},mpm_remainder_mult_dq_trellis={mpm_remainder_mult_dq_trellis},mpm_remainder_offset_dq={mpm_remainder_offset_dq},mpm_remainder_offset_dq_trellis={mpm_remainder_offset_dq_trellis},planar_offset_dq={planar_offset_dq},planar_offset_dq_trellis={planar_offset_dq_trellis},header_bits_dq={header_bits_dq},header_bits_dq_trellis={header_bits_dq_trellis},chroma_header_bits_dq={chroma_header_bits_dq},chroma_header_bits_dq_trellis={chroma_header_bits_dq_trellis}"
    # ex_params = f"quant_lv_pow={quant_lv_pow},quant_qp_div_trellis={quant_qp_div_trellis},quant_qp_div={quant_qp_div},quant_lambda_mul={quant_lambda_mul},quant_lambda_mul_trellis={quant_lambda_mul_trellis},quant_lambda_offset={quant_lambda_offset},quant_lambda_offset_trellis={quant_lambda_offset_trellis},lv_offset_dq={lv_offset_dq},lv_offset_dq_trellis={lv_offset_dq_trellis},qp_div_dq={qp_div_dq},qp_div_dq_trellis={qp_div_dq_trellis},lambda_mul_dq={lambda_mul_dq},lambda_mul_dq_trellis={lambda_mul_dq_trellis},lv_pow_dq={lv_pow_dq},lv_pow_dq_trellis={lv_pow_dq_trellis},cclm_pow={cclm_pow},mpm_idx_pow={mpm_idx_pow},mpm_remainder_pow={mpm_remainder_pow}"

    loss = bd_rate_x265.calculate_bd_rate(ex_params)

    return loss


study = optuna.create_study(
    study_name="optimize-bd-psnr",
    direction="minimize",
    storage="sqlite:///optuna_study.db",
    load_if_exists=True,
)
study.optimize(objective, n_trials=1000)
best_params = study.best_params
print(best_params)

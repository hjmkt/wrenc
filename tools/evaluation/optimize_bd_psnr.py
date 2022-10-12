import calculate_bd_rate_against_x264 as bd_rate_x264
import optuna


def objective(trial):

    # Trial 46 finished with value: 0.9116499193258718 and parameters: {'p0': 1.5874337195318615, 'p1': 1.8801394404513865, 'p2': 0.8606240750566909, 'p3': 1.3092519975511046, 'l0': 2.435058463545105, 'l1': 1.4817103857222098, 'l2': 0.4398482488002428, 'l3': 2.285208421597874, 'l4': 0.9626864242757724, 'l5': 1.1528848660715467, 'l6': 1.2475348149202359, 'l7': 4.977222955478007, 'l8': 2.122240051467992, 'l9': 1.9392960754708515, 'l10': 0.8670199511883868, 'l11': 0.5406538737080708, 'q0': 5.47901333556984, 'q1': 1.1206878265138847, 'q2': 2.5315974947405664}. Best is trial 46 with value: 0.9116499193258718.

    p0 = trial.suggest_float("p0", 1.5, 1.6)
    p1 = trial.suggest_float("p1", 1.83, 1.92)
    p2 = trial.suggest_float("p2", 0.85, 0.9)
    p3 = trial.suggest_float("p3", 1.28, 1.35)
    l0 = trial.suggest_float("l0", 2.38, 2.45)
    l1 = trial.suggest_float("l1", 1.45, 1.53)
    l2 = trial.suggest_float("l2", 0.39, 0.45)
    l3 = trial.suggest_float("l3", 2.25, 2.33)
    l4 = trial.suggest_float("l4", 0.9, 0.98)
    l5 = trial.suggest_float("l5", 1.1, 1.2)
    l6 = trial.suggest_float("l6", 1.2, 1.3)
    l7 = trial.suggest_float("l7", 4.9, 5.0)
    l8 = trial.suggest_float("l8", 2.1, 2.25)
    l9 = trial.suggest_float("l9", 1.85, 1.98)
    l10 = trial.suggest_float("l10", 0.8, 0.87)
    l11 = trial.suggest_float("l11", 0.48, 0.55)
    q0 = trial.suggest_float("q0", 5.4, 5.6)
    q1 = trial.suggest_float("q1", 1.12, 1.2)
    q2 = trial.suggest_float("q2", 2.52, 2.62)

    ex_params = f"p0={p0},p1={p1},p2={p2},p3={p3},l0={l0},l1={l1},l2={l2},l3={l3},l4={l4},l5={l5},l6={l6},l7={l7},l8={l8},l9={l9},l10={l10},l11={l11},q0={q0},q1={q1},q2={q2}"

    loss = bd_rate_x264.calculate_bd_rate(ex_params)

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

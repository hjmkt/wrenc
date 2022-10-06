import calculate_bd_rate_against_x264 as bd_rate_x264
import optuna

# bd_rate_x264.calculate_bd_rate("b=1.13")


def objective(trial):
    p0 = trial.suggest_float("p0", 2.61, 2.63)
    p1 = trial.suggest_float("p1", 1.935, 1.955)
    p2 = trial.suggest_float("p2", 0.55, 0.56)
    p3 = trial.suggest_float("p3", 1.145, 1.155)
    p4 = trial.suggest_float("p4", 0.665, 0.675)
    p5 = trial.suggest_float("p5", 0.922, 0.927)
    p6 = trial.suggest_float("p6", 7.69, 7.73)
    # ex_params = f"p0={p0},p1={p1},p2={p2},p3={p3},p4={p4},p5={p5},p6={p6}"

    l0 = trial.suggest_float("l0", 2.49, 2.51)
    l1 = trial.suggest_float("l1", 1.38, 1.395)
    l2 = trial.suggest_float("l2", 0.658, 0.67)
    l3 = trial.suggest_float("l3", 2.64, 2.67)
    l4 = trial.suggest_float("l4", 0.585, 0.595)
    l5 = trial.suggest_float("l5", 1.755, 1.765)
    l6 = trial.suggest_float("l6", 0.695, 0.705)
    l7 = trial.suggest_float("l7", 0.918, 0.925)
    l8 = trial.suggest_float("l8", 7.76, 7.82)
    l9 = trial.suggest_float("l9", 1.935, 1.945)
    l10 = trial.suggest_float("l10", -0.14, -0.08)
    # ex_params = f"l0={l0},l1={l1},l2={l2},l3={l3},l4={l4},l5={l5},l6={l6},l7={l7},l8={l8},l9={l9},l10={l10}"
    ex_params = f"p0={p0},p1={p1},p2={p2},p3={p3},p4={p4},p5={p5},p6={p6},l0={l0},l1={l1},l2={l2},l3={l3},l4={l4},l5={l5},l6={l6},l7={l7},l8={l8},l9={l9},l10={l10}"
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



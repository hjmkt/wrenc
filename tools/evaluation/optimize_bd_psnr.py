import calculate_bd_rate_against_x264 as bd_rate_x264
import optuna

# bd_rate_x264.calculate_bd_rate("b=1.13")


def objective(trial):
    p0 = trial.suggest_float("p0", 2.663, 2.673)
    p1 = trial.suggest_float("p1", 1.918, 1.926)
    p2 = trial.suggest_float("p2", 0.556, 0.564)
    p3 = trial.suggest_float("p3", 1.176, 1.184)
    p4 = trial.suggest_float("p4", 0.743, 0.751)
    p5 = trial.suggest_float("p5", 1.025, 1.034)
    p6 = trial.suggest_float("p6", 7.73, 7.81)
    # ex_params = f"p0={p0},p1={p1},p2={p2},p3={p3},p4={p4},p5={p5},p6={p6}"

    l0 = trial.suggest_float("l0", 2.48, 2.51)
    l1 = trial.suggest_float("l1", 1.316, 1.326)
    l2 = trial.suggest_float("l2", 0.668, 0.678)
    l3 = trial.suggest_float("l3", 2.68, 2.71)
    l4 = trial.suggest_float("l4", 0.592, 0.60)
    l5 = trial.suggest_float("l5", 1.758, 1.766)
    l6 = trial.suggest_float("l6", 0.668, 0.676)
    l7 = trial.suggest_float("l7", 1.15, 1.163)
    l8 = trial.suggest_float("l8", 7.88, 7.94)
    l9 = trial.suggest_float("l9", 1.935, 1.952)
    l10 = trial.suggest_float("l10", 0.93, 1.02)
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



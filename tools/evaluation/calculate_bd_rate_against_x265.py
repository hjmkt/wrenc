import json
import subprocess
import os
import statistics
import shutil
from pathlib import Path
from scipy import interpolate
from concurrent.futures import ProcessPoolExecutor


def encode(params):
    (
        target,
        input,
        output_dir,
        qp,
        frames,
        encode_command,
        metric_command,
        output_name,
        env,
    ) = params
    output = f"{output_dir}/{output_name}"
    env["qp"] = str(qp)
    env["output"] = output

    file_bytes = (
        subprocess.check_output(
            f"./{encode_command}".split(),
            env=env,
        )
        .strip()
        .decode("utf-8")
    )

    env["original"] = input
    env["compressed"] = output
    env["target"] = target
    result = (
        subprocess.check_output(
            f"./{metric_command}".split(),
            env=env,
        )
        .strip()
        .decode("utf-8")
    )
    result = json.loads(result)
    psnr = 0
    for frame_result in result:
        psnr += frame_result["Avg"]
    psnr /= frames
    if psnr == float("inf"):
        psnr = 100
    return (int(file_bytes), psnr)


def calculate_bd_rate(ex_params):

    with open("videos.json") as f:
        videos = json.load(f)
    with open("metrics.json") as f:
        metrics = json.load(f)
    with open("presets.json") as f:
        presets = json.load(f)

    wrenc_preset = presets["wrenc_fixed_qp"]
    x265_preset = presets["x265_intra_fixed_qp"]
    qps = [20, 23, 26, 29, 32, 35, 38, 41]
    pid = os.getpid()

    video_results = {}
    for video_name, video in videos.items():
        video_results[video_name] = {
            "wrenc": {"bytes": [], "psnr": []},
            "x265": {"bytes": [], "psnr": []},
        }

    wrenc_bd_psnrs = []
    for video_name, video in videos.items():
        input = f"../../assets/{video_name}"
        frames = video["frames"]
        width = video["width"]
        height = video["height"]
        frame_rate = video["frame_rate"]
        output_dir = f"videos_{pid}"
        Path(output_dir).mkdir(parents=True, exist_ok=True)
        wrenc_encode_command = wrenc_preset["command"]
        x265_encode_command = x265_preset["command"]
        metric_command = metrics["PSNR"]["command"]
        env = os.environ.copy()
        env["input"] = input
        env["frames"] = str(frames)
        env["width"] = str(width)
        env["height"] = str(height)
        env["frame_rate"] = str(frame_rate)
        env["extra_params"] = f"--extra-params {ex_params}"
        env["preset"] = "placebo"
        # env["max_split_depth"] = "--max-split-depth 2"

        with ProcessPoolExecutor(8) as executor:
            results = executor.map(
                encode,
                map(
                    lambda qp: (
                        "wrenc",
                        input,
                        output_dir,
                        qp,
                        frames,
                        wrenc_encode_command,
                        metric_command,
                        f"wrenc_{qp}.vvc",
                        env.copy(),
                    ),
                    qps,
                ),
            )
            for (file_bytes, psnr) in results:
                video_results[video_name]["wrenc"]["bytes"].append(file_bytes)
                video_results[video_name]["wrenc"]["psnr"].append(psnr)

            results = executor.map(
                encode,
                map(
                    lambda qp: (
                        "x265",
                        input,
                        output_dir,
                        qp,
                        frames,
                        x265_encode_command,
                        metric_command,
                        f"x265_{qp}.mp4",
                        env.copy(),
                    ),
                    list(map(lambda qp: qp + 3, qps)),
                ),
            )
            for (file_bytes, psnr) in results:
                video_results[video_name]["x265"]["bytes"].append(file_bytes)
                video_results[video_name]["x265"]["psnr"].append(psnr)

        wrenc_samples = list(
            zip(
                video_results[video_name]["wrenc"]["psnr"], video_results[video_name]["wrenc"]["bytes"]
            )
        )
        wrenc_samples.sort(key=lambda x: x[0])

        x265_samples = list(
            zip(
                video_results[video_name]["x265"]["psnr"],
                video_results[video_name]["x265"]["bytes"],
            )
        )
        x265_samples.sort(key=lambda x: x[0])

        min_psnr = max(wrenc_samples[0][0], x265_samples[0][0])
        max_psnr = min(wrenc_samples[-1][0], x265_samples[-1][0])
        d = max_psnr - min_psnr
        n = 100
        points = list(
            map(
                lambda i: min_psnr + (i + 1) * d / (n + 2 - 1),
                range(n),
            )
        )

        xs = list(map(lambda s: s[0], wrenc_samples))
        ys = list(
            map(
                lambda s: s[1],
                wrenc_samples,
            )
        )
        f = interpolate.interp1d(xs, ys, kind="cubic")
        wrenc_rates = []
        for p in points:
            wrenc_rates.append(f(p))

        xs = list(map(lambda s: s[0], x265_samples))
        ys = list(
            map(
                lambda s: s[1],
                x265_samples,
            )
        )
        f = interpolate.interp1d(xs, ys, kind="cubic")
        x265_rates = []
        for p in points:
            x265_rates.append(f(p))

        s = 0
        for wrenc_rate, x265_rate in zip(wrenc_rates, x265_rates):
            s += wrenc_rate / x265_rate
        wrenc_bd_psnr = s / n
        wrenc_bd_psnrs.append(wrenc_bd_psnr)

    print(wrenc_bd_psnrs)
    output_dir = f"videos_{pid}"
    shutil.rmtree(output_dir)
    return statistics.mean(wrenc_bd_psnrs)

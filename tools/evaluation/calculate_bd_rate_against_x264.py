import json
import subprocess
import os
import numpy as np
import statistics
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

    rwc_preset = presets["rwc_fixed_qp"]
    x264_preset = presets["x264_intra_fixed_qp"]
    qps = [20, 23, 26, 29, 32, 35, 38, 41]
    pid = os.getpid()

    video_results = {}
    for video_name, video in videos.items():
        video_results[video_name] = {
            "rwc": {"bytes": [], "psnr": []},
            "x264": {"bytes": [], "psnr": []},
        }

    rwc_bd_psnrs = []
    for video_name, video in videos.items():
        input = f"../../assets/{video_name}"
        frames = video["frames"]
        width = video["width"]
        height = video["height"]
        frame_rate = video["frame_rate"]
        output_dir = f"videos_{pid}"
        Path(output_dir).mkdir(parents=True, exist_ok=True)
        rwc_encode_command = rwc_preset["command"]
        x264_encode_command = x264_preset["command"]
        metric_command = metrics["PSNR"]["command"]
        env = os.environ.copy()
        env["input"] = input
        env["frames"] = str(frames)
        env["width"] = str(width)
        env["height"] = str(height)
        env["frame_rate"] = str(frame_rate)
        env["extra_params"] = f"--extra-params {ex_params}"
        env["preset"] = "superfast"

        with ProcessPoolExecutor(8) as executor:
            results = executor.map(
                encode,
                map(
                    lambda qp: (
                        "rwc",
                        input,
                        output_dir,
                        qp,
                        frames,
                        rwc_encode_command,
                        metric_command,
                        f"rwc_{qp}.vvc",
                        env.copy(),
                    ),
                    qps,
                ),
            )
            for (file_bytes, psnr) in results:
                video_results[video_name]["rwc"]["bytes"].append(file_bytes)
                video_results[video_name]["rwc"]["psnr"].append(psnr)

            results = executor.map(
                encode,
                map(
                    lambda qp: (
                        "x264",
                        input,
                        output_dir,
                        qp,
                        frames,
                        x264_encode_command,
                        metric_command,
                        f"x264_{qp}.mp4",
                        env.copy(),
                    ),
                    list(map(lambda qp: qp + 6, qps)),
                ),
            )
            for (file_bytes, psnr) in results:
                video_results[video_name]["x264"]["bytes"].append(file_bytes)
                video_results[video_name]["x264"]["psnr"].append(psnr)

        rwc_samples = list(
            zip(
                video_results[video_name]["rwc"]["psnr"], video_results[video_name]["rwc"]["bytes"]
            )
        )
        rwc_samples.sort(key=lambda x: x[0])

        x264_samples = list(
            zip(
                video_results[video_name]["x264"]["psnr"],
                video_results[video_name]["x264"]["bytes"],
            )
        )
        x264_samples.sort(key=lambda x: x[0])

        min_psnr = max(rwc_samples[0][0], x264_samples[0][0])
        max_psnr = min(rwc_samples[-1][0], x264_samples[-1][0])
        d = max_psnr - min_psnr
        n = 100
        points = list(
            map(
                lambda i: min_psnr + (i + 1) * d / (n + 2 - 1),
                range(n),
            )
        )

        xs = list(map(lambda s: s[0], rwc_samples))
        ys = list(
            map(
                lambda s: s[1],
                rwc_samples,
            )
        )
        f = interpolate.interp1d(xs, ys, kind="cubic")
        rwc_rates = []
        for p in points:
            rwc_rates.append(f(p))

        xs = list(map(lambda s: s[0], x264_samples))
        ys = list(
            map(
                lambda s: s[1],
                x264_samples,
            )
        )
        f = interpolate.interp1d(xs, ys, kind="cubic")
        x264_rates = []
        for p in points:
            x264_rates.append(f(p))

        s = 0
        for rwc_rate, x264_rate in zip(rwc_rates, x264_rates):
            s += rwc_rate / x264_rate
        rwc_bd_psnr = s / n
        rwc_bd_psnrs.append(rwc_bd_psnr)

    print(rwc_bd_psnrs)
    return statistics.mean(rwc_bd_psnrs)

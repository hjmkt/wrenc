import json
import pprint
from copy import copy
import subprocess
import os
import time
import argparse
from pathlib import Path
from datetime import datetime, timedelta, timezone
from scipy import interpolate
from concurrent.futures import ProcessPoolExecutor
from statistics import mean

commit_id = subprocess.check_output("git rev-parse --short HEAD".split()).strip().decode("utf-8")
commit_message = (
    subprocess.check_output("git log -1 --pretty='%s'".split()).strip().decode("utf-8")
)
JST = timezone(timedelta(hours=+9), "JST")
date = datetime.now(JST)
date = date.strftime("%Y-%m-%d %H:%M:%S %z")

with open("config.json") as f:
    config = json.load(f)

with open("videos.json") as f:
    videos = json.load(f)

with open("metrics.json") as f:
    metrics = json.load(f)

with open("presets.json") as f:
    presets = json.load(f)


def encode(params):
    video_name, preset, video, prefix, env, param_set, base = params
    description = (
        f"{preset['base']}={base}" if len(prefix) == 0 else f"{prefix},{preset['base']}={base}"
    )
    target = f"{preset['target']}#{commit_id}" if preset["target"] == "wrenc" else preset["target"]
    title = f"{video_name.split('.')[0]}[{target}@{description}]"
    input = f"../../assets/{video_name}"
    frames = video["frames"]
    width = video["width"]
    height = video["height"]
    frame_rate = video["frame_rate"]
    output_dir = "videos"
    Path(output_dir).mkdir(parents=True, exist_ok=True)
    output = (
        f"{output_dir}/{title}.vvc" if preset["target"] == "wrenc" else f"{output_dir}/{title}.mp4"
    )
    encode_command = preset["command"]
    env["input"] = input
    env["frames"] = str(frames)
    env["width"] = str(width)
    env["height"] = str(height)
    env["frame_rate"] = str(frame_rate)
    env["qp"] = str(base)
    env["output"] = output
    if preset["target"] == "wrenc":
        if "max_split_depth" in param_set.keys():
            env["max_split_depth"] = f"--max-split-depth {str(param_set['max_split_depth'])}"
        if "extra_params" in param_set.keys():
            env["extra_params"] = f"--extra-params {param_set['extra_params']}"
    elif preset["target"] == "x265":
        env["preset"] = str(param_set["preset"])
    start_time = time.time()
    file_bytes = (
        subprocess.check_output(f"./{encode_command}".split(), env=env).strip().decode("utf-8")
    )
    end_time = time.time()
    duration = end_time - start_time

    metric_result = {
        "title": title,
        preset["base"]: base,
        "bytes": int(file_bytes),
        "duration": duration,
        "metrics": {},
    }

    for metric_name, metric in metrics.items():
        metric_command = metric["command"]
        env["original"] = input
        env["compressed"] = output
        env["target"] = preset["target"]
        result = (
            subprocess.check_output(f"./{metric_command}".split(), env=env).strip().decode("utf-8")
        )
        result = json.loads(result)
        if metric["type"] == "per_frame":
            metric_attrs = metric["attr"]
            metric_summary = {}
            for attr in metric_attrs:
                metric_summary[attr] = 0
            for frame_result in result:
                for attr in metric_attrs:
                    metric_summary[attr] += frame_result[attr]
            for attr in metric_attrs:
                metric_summary[attr] /= frames
                if metric_summary[attr] == float("inf"):
                    metric_summary[attr] = 100
            metric_result["metrics"][metric_name] = {
                "summary": metric_summary,
                "per_frame": result,
            }
        else:
            metric_result["metrics"][metric_name] = {
                "summary": metric_summary,
            }
    print(f"{title} completed")
    return metric_result


def evaluate(threads):

    summary = {
        "date": date,
        "commit_id": commit_id,
        "commit_message": commit_message,
        "results": [],
    }

    for preset_name in config["default_presets"]:
        preset = presets[preset_name]
        param_sets = [{}]
        for param_name in preset["parameters"].keys():
            if param_name == preset["base"]:
                continue
            if len(preset["parameters"][param_name]) == 0:
                continue
            new_param_sets = []
            for param_set in param_sets:
                for param in preset["parameters"][param_name]:
                    new_param_set = copy(param_set)
                    new_param_set[param_name] = param
                    new_param_sets.append(new_param_set)
            param_sets = new_param_sets
        preset_result = {"preset": preset_name, "results": []}
        for param_set in param_sets:
            prefix = ""
            for param_name, param_val in param_set.items():
                prefix += f",{param_name}={param_val}"
            prefix = prefix[1:]
            tag = f"{preset_name}#{commit_id}" if preset["target"] == "wrenc" else preset_name
            if len(prefix) > 1:
                tag += f"@{prefix}"
            param_result = {"parameters": param_set, "tag": tag, "results": []}
            for video_name, video in videos.items():
                video_result = {"video": video_name, "results": []}
                with ProcessPoolExecutor(threads) as executor:
                    metric_results = executor.map(
                        encode,
                        map(
                            lambda base: (
                                video_name,
                                preset,
                                video,
                                prefix,
                                os.environ.copy(),
                                param_set,
                                base,
                            ),
                            preset["parameters"][preset["base"]],
                        ),
                    )
                    for metric_result in metric_results:
                        video_result["results"].append(metric_result)
                param_result["results"].append(video_result)
            preset_result["results"].append(param_result)
        summary["results"].append(preset_result)

    bd_rates = {}
    for video_name in videos.keys():
        bd_rates[video_name] = {}

    for preset_result in summary["results"]:
        for parameter_result in preset_result["results"]:
            tag = parameter_result["tag"]
            for video_result in parameter_result["results"]:
                video = video_result["video"]
                samples = []
                for result in video_result["results"]:
                    samples.append((result["metrics"]["PSNR"]["summary"]["Avg"], result["bytes"]))
                samples.sort(key=lambda x: x[0])
                bd_rates[video][tag] = samples

    bd_psnr = {}
    for video_name in videos.keys():
        bd_psnr[video_name] = {}
    for video in bd_rates.keys():
        min_psnr = 0
        max_psnr = 1000
        for tag, samples in bd_rates[video].items():
            min_psnr = max(min_psnr, samples[0][0])
            max_psnr = min(max_psnr, samples[-1][0])
        d = max_psnr - min_psnr
        n = 100
        points = list(map(lambda i: min_psnr + (i + 1) * d / (n + 2 - 1), range(n)))
        base_bd_psnrs = []
        for tag, samples in bd_rates[video].items():
            print(tag, config["base_preset"])
            if tag == config["base_preset"]:
                xs = list(map(lambda s: s[0], samples))
                ys = list(map(lambda s: s[1], samples))
                f = interpolate.interp1d(xs, ys, kind="cubic")
                for p in points:
                    base_bd_psnrs.append(f(p))

        for tag, samples in bd_rates[video].items():
            xs = list(map(lambda s: s[0], samples))
            ys = list(map(lambda s: s[1], samples))
            f = interpolate.interp1d(xs, ys, kind="cubic")
            s = 0
            for i, p in enumerate(points):
                s += f(p) / base_bd_psnrs[i]
                if i%10==0 and f(p)!=base_bd_psnrs[i]:
                    print("rate", f(p) / base_bd_psnrs[i])
            s /= n
            bd_psnr[video][tag] = s
    bd_psnr_summary = {}
    for video_result in bd_psnr.values():
        for tag, psnr in video_result.items():
            if tag not in bd_psnr_summary.keys():
                bd_psnr_summary[tag] = [psnr]
            else:
                bd_psnr_summary[tag].append(psnr)
    for tag, psnrs in bd_psnr_summary.items():
        bd_psnr_summary[tag] = mean(psnrs)
    bd_psnr["summary"] = bd_psnr_summary
    pprint.pprint(bd_psnr)

    with open("summary.json", "w") as f:
        json.dump(summary, f, indent=4, separators=(",", ": "))


if __name__ == "__main__":
    parser = argparse.ArgumentParser()
    parser.add_argument("--threads", type=int, default=1)
    args = parser.parse_args()
    evaluate(threads=args.threads)

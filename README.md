# rwc ![workflow](https://github.com/hjmkt/rwc/actions/workflows/test.yml/badge.svg)

rwc is an experimental H.266/VVC encoder implemented in Rust.

The latest evaluation dashboard can be found [here](https://hjmkt.github.io/rwc).

# Requirements

- Rust 1.66.0-nightly or later
- Python 3.10 for running evaluation scripts
- Node.js & npm & yarn for building an evaluation dashboard

# Supported features

## Codec-specific features

|        Feature         |             Availability              |        Remarks         |
| :--------------------: | :-----------------------------------: | :--------------------: |
|        CTU size        |                 32x32                 |           -            |
|        CT size         |     32x32 or 16x16 or 8x8 or 4x4      |           -            |
|          Tile          |          1 tile per picture           |           -            |
|         Slice          |          1 slice per picture          |           -            |
|      Sub-picture       |                  No                   |           -            |
|       Slice type       |                I only                 |           -            |
|     Chroma format      |             YCbCr420 only             |           -            |
|      Color depth       |              8-bit only               |           -            |
|    Intra prediction    |  PLANAR or DC or ANGULARX or CCLMX    |           -            |
|     Transform skip     |                 Yes\*                 | Not elaborately tested |
|     Transform size     | 64x64 or 32x32 or 16x16 or 8x8 or 4x4 |           -            |
|         LFNST          |                  No                   |           -            |
|      Loop filter       |                  No                   |           -            |
| Dependent quantization |                  Yes                   |           -            |

## Encoder-specific features

|         Feature         |                Availability                |   Remarks   |
| :---------------------: | :----------------------------------------: | :---------: |
|      Input format       |                Raw YUV only                |      -      |
|     Input protocol      |             File or Unix Pipe              |      -      |
|    Input resolution     | Width and height should be multiples of 32 | To be fixed |
|     Output protocol     |                 File only                  |      -      |
|   CT partition search   |       Exhaustive search by RD costs        |      -      |
| Intra prediction search |          Step search by RD costs           |      -      |
|      Rate control       |               Fixed QP only                |      -      |
|          SIMD           |        Utilize AVX2 when supported         |      -      |

# Build

```bash
# building rwc for debug
cargo build

# building rwc for development
cargo build --profile dev

# building rwc for release
cargo build --release

# building an evaluation dashboard
cd tools/dashboard
yarn
node_modules/.bin/webpack
```

# Usage

## Run rwc

```bash
# running rwc for a file input
cargo run --release --bin rwc -- -i /path/to/video.yuv --input-size {WIDTH}x{HEIGHT} --num-pictures NUM_OF_FRAMES -o /path/to/output.vvc --output-size {WIDTH}x{HEIGHT} [--qp QP] [--max-split-depth MAX_SPLIT_DEPTH] [--reconst /path/to/reconstructed.yuv] [--extra-params KEY1=VAL1[,KEY2=VAL2,...]]

# running rwc for a pipe input
ffmpeg -i /path/to/input.mp4 -f rawvideo -pix_fmt yuv420p -s {WIDTH}x{HEIGHT} - | cargo run --release --bin rwc -- -i - --input-size {WIDTH}x{HEIGHT} --num-pictures NUM_OF_FRAMES -o /path/to/output.vvc --output-size {WIDTH}x{HEIGHT} [--qp QP] [--max-split-depth MAX_SPLIT_DEPTH] [--reconst /path/to/reconstructed.yuv] [--extra-params KEY1=VAL1[,KEY2=VAL2,...]]
```

## Evaluation

The following command will run rwc on test videos with some presets of parameters specified in tools/evaluation/config.json.
The result will be generated as summary.json.

```
cd tools/evaluation

# for pipenv users
pipenv shell
pipenv install

# otherwise
pip install -r requirements.txt

python evaluate_mp.py --threads NUM_OF_THREADS
```

The following command will run the evaluation, update the dashboard and export it as docs/index.html.

```
scripts/update_dashboard.sh
```

# Test

```
# unit tests
cargo test

# integration tests
VTM_ROOT=/path/to/vtm/bin scripts/integration_test.sh

# check coding style
cargo fmt --all --check --verbose
cargo clippy --all-targets --all-features
```

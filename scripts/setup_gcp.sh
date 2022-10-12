#!/bin/bash

curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
source $HOME/.cargo/env
sudo add-apt-repository -y ppa:deadsnakes/ppa
sudo apt update
sudo apt install -y build-essential ffmpeg python3.10 cmake jq
sudo ln -s /usr/bin/python3.10 /usr/bin/python
curl -sS https://bootstrap.pypa.io/get-pip.py | python
echo "export PATH=$HOME/.local/bin:$PATH" >> ~/.bashrc
source ~/.bashrc
rustup install nightly
rustup default nightly
cargo build --release
git clone https://vcgit.hhi.fraunhofer.de/jvet/VVCSoftware_VTM.git
pushd VVCSoftware_VTM
mkdir build
cd build
cmake .. -DCMAKE_BUILD_TYPE=Release
make -j
pip install pipenv
pushd tools/evaluation
pipenv install --system
popd

#!/bin/bash

cd "$(dirname "${BASH_SOURCE[0]}")"
./delete_swap_models.sh
cd ..

mkdir -p web_ui/static/models
cp -r data/models/*.yaml web_ui/static/models

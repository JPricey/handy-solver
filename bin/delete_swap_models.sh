#!/bin/bash

cd "$(dirname "${BASH_SOURCE[0]}")"
cd ..
cd data/models

touch swap.deleteme
rm swap.*

#!/bin/bash
# run this in a directory where we'll do stuff
cd $(dirname ${BASH_SOURCE[0]})
echo "This will mess around with the contents of $PWD
Now's your last chance change to terminate this program.

Ensure ec.rs has STORE_INPUTS set to true and STORE_FILENAME_PREFIX = \"input_contextual\".
Note that this is not deterministic.
Press Enter to continue"
read

EC_STORAGE=$PWD
EC=/home/lucasem/repo/context/ec
EC_ONE=/home/lucasem/repo/context/ec_one
EC_CURRICULUM=/home/lucasem/repo/context/curriculum/ec_reiter
ALL=/home/lucasem/repo/context/curriculum/all.json
CONTEXT=/home/lucasem/repo/context/target/debug/context
PRODUCE_DATA=$PWD/produce_data.sh
PLOTTER=$PWD/plot_tsv.py

cp "$EC_CURRICULUM"/*.json ./
ls | grep '^course.*\.json$' | perl -pe 's/^(.*_(\d+)\.json)$/mv $1 input_$2.json/' | sh
cp "$ALL" ./
ALL=$(basename $ALL)

export EC_CURRICULUM
export EC_STORAGE
export EC
$CONTEXT
$PRODUCE_DATA primitive.tsv \
              output_primitive \
              $EC_ONE \
              input_01.json \
              input_02.json \
              input_03.json
$PRODUCE_DATA specialized_per_phase.tsv \
              output_specialized_per_phase \
              $EC \
              input_01.json \
              input_02.json \
              input_03.json
$PRODUCE_DATA specialized_full_domain.tsv \
              output_specialized_full_domain \
              $EC \
              $ALL
$PRODUCE_DATA contextual.tsv \
              output_contextual \
              $EC \
              input_contextual_6.json \
              input_contextual_7.json \
              input_contextual_8.json

for plot in speed likelihood
do python $PLOTTER $plot \
                   primitive.tsv \
                   specialized_per_phase.tsv \
                   specialized_full_domain.tsv \
                   contextual.tsv
done

#!/bin/bash
# requires jq (https://stedolan.github.io/jq)

##########
## HELP ##
##########
if test "$1" = -h || test "$#" -lt 4
then echo "usage: $0 OUT OUTPREFIX EC INPUT...
  OUT is the tsv destination
  OUTPREFIX is prepended to each INPUT when saving ec's results
  EC is the ec binary
  each INPUT is an input for EC.
  " ; exit
fi


################
## PARSE ARGS ##
################
out="$1"
outprefix="$2"
outprefix+="_"
ec="$3"
shift ; shift ; shift

inputs=()
while test $# -gt 0
do
  inputs+=("$1")
  shift
done


########################
## PRODUCE EC OUTPUTS ##
########################
outputs=()
out_times=()
for input in ${inputs[@]}
do
  output="$outprefix$input"
  echo "producing $output"
  /bin/time -f'%U' $ec $input > $output 2>tmp
  t=$(cat tmp)
  outputs+=("$output")
  out_times+=("$t")
done


#################
## PRODUCE TSV ##
#################
if test -e $out
then rm $out
fi

for ((i = 0; i < ${#outputs[@]}; i++))
do
  file="${outputs[$i]}"
  t="${out_times[$i]}"
  cat $file | jq -rc '.programs | .[] | if .result == null then empty else . end | "\(.task)\t\(.result.time)\t\(.result.log_probability)\t'"$t"'\t\(.result.expr)"' >> $out
done


echo "done."

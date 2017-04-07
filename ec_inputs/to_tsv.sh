#!/bin/sh
if test "$1" = -h
then echo "usage: $0 OUT INPUT...
  where an INPUT corresponds to an ec output json" ; exit
fi

rm $1
for file in ${@:2}
do cat $file | jq -rc '.programs | .[] | if .result == null then empty else . end | "\(.task)\t\(.result.time)\t\(.result.log_probability)"' >> $1
done

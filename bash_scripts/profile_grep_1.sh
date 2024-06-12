#!/bin/bash

echo "file,size,custom_grep_avg_time_us,standard_grep_avg_time_us" > results/timings.csv

for file in test_files/*.txt; do
    size=$(wc -c < "$file" | xargs)
    echo "Processing $file with size $size"

    pattern="$(printf 'b?%.0s' $(seq 1 $size))$(printf 'a%.0s' $(seq 1 $size))"

    total_custom_duration_us=0
    total_standard_duration_us=0

    # run each grep 10 times
    for i in {1..10}; do
        # timing custom grep
        custom_duration=$(TIMEFORMAT='%3R'; time (./bin/grep "$pattern" "$file" > /dev/null 2>&1) 2>&1)
         
        custom_duration_us=$(echo "$custom_duration * 1000" | bc)
        total_custom_duration_us=$(echo "$total_custom_duration_us + $custom_duration_us" | bc)

        # timing standard grep
        standard_duration=$(TIMEFORMAT='%3R'; time (grep -onE "$pattern" "$file" > /dev/null 2>&1) 2>&1)
        standard_duration_us=$(echo "$standard_duration * 1000" | bc)
        total_standard_duration_us=$(echo "$total_standard_duration_us + $standard_duration_us" | bc)
    done

    # calculate average times
    avg_custom_duration_us=$(echo "$total_custom_duration_us / 10" | bc)
    avg_standard_duration_us=$(echo "$total_standard_duration_us / 10" | bc)

    # store the results
    echo "$file,$size,$avg_custom_duration_us,$avg_standard_duration_us" >> results/timings.csv
done

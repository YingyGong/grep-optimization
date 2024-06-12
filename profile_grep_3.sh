#!/bin/bash

echo "file,size,improved__grep_avg_time_us,original_grep_avg_time_us" > results/timings8.csv

for file in test_files/*.txt; do
    # Calculate the file size to generate the regex pattern
    size=$(wc -c < "$file" | xargs)
    echo "Processing $file with size $size"
    tenth_size=$((size / 10))

    # Generate the regex pattern based on the file size

    # Initialize total duration variables
    total_improved_duration_us=0
    total_original_duration_us=0

    # Run each grep 3 times
    for i in {1..3}; do
        # Timing updated custom grep
        pattern="$(printf 'b?%.0s' $(seq 1 $tenth_size))$(printf 'a%.0s' $(seq 1 $tenth_size))"

        improved_duration=$(TIMEFORMAT='%3R'; time (./bin/grep "$pattern" "$file" > /dev/null 2>&1) 2>&1)
         
        improved_duration_us=$(echo "$improved_duration * 1000" | bc)
        total_improved_duration_us=$(echo "$total_improved_duration_us + $improved_duration_us" | bc)

        # Timing original grep
        original_duration=$(TIMEFORMAT='%3R'; time (./bin/grep1 "$pattern" "$file" > /dev/null 2>&1) 2>&1)
        original_duration_us=$(echo "$original_duration * 1000" | bc)
        total_original_duration_us=$(echo "$total_original_duration_us + $original_duration_us" | bc)
    done

    # Calculate average times
    avg_improved_duration_us=$(echo "$total_improved_duration_us / 10" | bc)
    avg_original_duration_us=$(echo "$total_original_duration_us / 10" | bc)

    # Store the results
    echo "$file,$tenth_size,$avg_improved_duration_us,$avg_original_duration_us" >> results/timings8.csv
done

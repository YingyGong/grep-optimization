#!/bin/bash

echo "file,size,no_prefixgrep_avg_time_us,with_prefix_grep_avg_time_us" > results/timings.csv

for file in test_files/*.txt; do
    # Calculate the file size to generate the regex pattern
    size=$(wc -c < "$file" | xargs)
    echo "Processing $file with size $size"
    tenth_size=$((size / 10))

    # Generate the regex pattern based on the file size

    # Initialize total duration variables
    total_no_prefixduration_us=0
    total_with_prefix_duration_us=0

    # Run each grep 3 times
    for i in {1..3}; do
        # Timing your custom grep
        pattern="$(printf 'b?%.0s' $(seq 1 $tenth_size))$(printf 'a%.0s' $(seq 1 $tenth_size))"

        no_prefixduration=$(TIMEFORMAT='%3R'; time (./bin/grep "$pattern" "$file" > /dev/null 2>&1) 2>&1)
         
        no_prefixduration_us=$(echo "$no_prefixduration * 1000" | bc)
        total_no_prefixduration_us=$(echo "$total_no_prefixduration_us + $no_prefixduration_us" | bc)

        # Timing with_prefix grep
        pattern="$(printf 'a?%.0s' $(seq 1 $tenth_size))$(printf 'a%.0s' $(seq 1 $tenth_size))"

        with_prefix_duration=$(TIMEFORMAT='%3R'; time (grep -onE "$pattern" "$file" > /dev/null 2>&1) 2>&1)
        with_prefix_duration_us=$(echo "$with_prefix_duration * 1000" | bc)
        total_with_prefix_duration_us=$(echo "$total_with_prefix_duration_us + $with_prefix_duration_us" | bc)
    done

    # Calculate average times
    avg_no_prefixduration_us=$(echo "$total_no_prefixduration_us / 10" | bc)
    avg_with_prefix_duration_us=$(echo "$total_with_prefix_duration_us / 10" | bc)

    # Store the results
    echo "$file,$tenth_size,$avg_no_prefixduration_us,$avg_with_prefix_duration_us" >> results/timings7.csv
done

# Set the file path
file="test_files/file_10.txt"

# Define the regex pattern for exactly 300 'a?' followed by 300 'a'

size=$(wc -c < "$file" | xargs)
size=$((size / 10))

pattern="$(printf 'b?%.0s' $(seq 1 $size))$(printf 'a%.0s' $(seq 1 $size))"

# print out the pattern
echo "Pattern: $pattern"

# Execute grep with timing
# time grep -onE "$pattern" "$file" 
# Execute custom grep with timing
# time ./bin/grep "$pattern" "$file"
echo "Custom grep for no prefix"
time (./bin/grep "$pattern" "$file" )
./bin/grep "$pattern" "$file"

pattern="$(printf 'a?%.0s' $(seq 1 $size))$(printf 'a%.0s' $(seq 1 $size))"

echo "Custom grep for with prefix"
time (./bin/grep0 "$pattern" "$file" )


time grep -onE "$pattern" "$file" 

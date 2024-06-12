# Set the file path
file="test_files/file_30.txt"

# Define the regex pattern for exactly 300 'a?' followed by 300 'a'

size=$(wc -c < "$file" | xargs)

pattern="$(printf 'a?%.0s' $(seq 1 $size))$(printf 'a%.0s' $(seq 1 $size))"

# print out the pattern
echo "Pattern: $pattern"

# Execute grep with timing
# time grep -onE "$pattern" "$file" 
# Execute custom grep with timing
# time ./bin/grep "$pattern" "$file"
time (./bin/grep "$pattern" "$file" )
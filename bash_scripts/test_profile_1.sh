file="test_files/file_30.txt"


size=$(wc -c < "$file" | xargs)

pattern="$(printf 'a?%.0s' $(seq 1 $size))$(printf 'a%.0s' $(seq 1 $size))"

echo "Pattern: $pattern"


time (./bin/grep "$pattern" "$file" )
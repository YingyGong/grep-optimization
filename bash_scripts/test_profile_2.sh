file="test_files/file_10.txt"


size=$(wc -c < "$file" | xargs)
size=$((size / 10))

pattern="$(printf 'b?%.0s' $(seq 1 $size))$(printf 'a%.0s' $(seq 1 $size))"

echo "Pattern: $pattern"

echo "Custom grep for no prefix"
time (./bin/grep "$pattern" "$file" )
./bin/grep "$pattern" "$file"

pattern="$(printf 'a?%.0s' $(seq 1 $size))$(printf 'a%.0s' $(seq 1 $size))"

echo "Custom grep for with prefix"
time (./bin/grep0 "$pattern" "$file" )


time grep -onE "$pattern" "$file" 

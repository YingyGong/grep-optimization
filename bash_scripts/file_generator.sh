rm -rf test_files
mkdir test_files
for i in $(seq 10 10 200); do
    yes a | head -n $((i * 10)) | tr -d '\n' > "test_files/file_${i}.txt"
done
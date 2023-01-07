# delete all albums from cache

for file in cache/meta/*
do
      if jq -e "has(\"Album\")" $file > /dev/null; then
            echo "$file" && rm $file
      fi
done
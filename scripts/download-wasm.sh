#!/usr/bin/env bash

URLS=$(cat seed.json | jq '.data.getInterfaceVersion.packageVersions.edges[].node.distribution.downloadUrl')
mkdir -p wasm/tmp && cd wasm/tmp
for URL in ${URLS}
do
    ARCHIVE=$(basename $URL)
    echo "Downloading $ARCHIVE"
    wget $(echo $URL | xargs)
done

for ARCHIVE in $(ls *.tar.gz)
do
    tar xzf $ARCHIVE
done

find . -name "*.wasm" -exec mv {} .. \;
cd ..
echo "Downloaded modules..."
rm -rf tmp
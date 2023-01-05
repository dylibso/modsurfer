# Modsurfer CLI Docker

## Building the image

```sh
# from the root of the repo:
docker build -f action/Dockerfile -t modsurfer:latest .
```

## Running the image

```sh
docker run -v `pwd`/test:/cli/test modsurfer:latest validate -p test/spidermonkey.wasm -c test/mod.yaml
```

# Space Check

A simple HTTP service that monitors free space on a given file system. Designed to be used as an external webhook for [autobrr](https://autobrr.com/) when driving a remote qBittorrent instance.

## Overview

This project provides a basic HTTP server that returns statistics about the file system, including free space, available space, total space, and buffer size. The server also checks if the requested amount of space is available, taking into account a 1GB buffer size.

## Features

* Returns file system statistics (free space, available space, total space, buffer size)
* Checks if requested amount of space is available (with 1GB buffer size)
* Supports GET and POST requests

## Usage

To use this project, simply run the server and send a GET or POST request to the specified URL.

```
$ cargo run /tmp
Monitoring: "/tmp"
2024-11-30 17:12:51.567945 UTC - GET / - 134.6us - 200
2024-11-30 17:12:52.618908 UTC - GET / - 100.9us - 200
2024-11-30 17:13:00.999227 UTC - POST / - 137.8us - 400
2024-11-30 17:13:09.735084 UTC - POST / - 2.2ms - 200
```

Or relying on the docker image:

```
$ docker run --rm -p 8080:8080 -v /tmp:/monitored_fs:ro ghcr.io/agrenott/spacecheck:latest
Monitoring: "/monitored_fs"
2024-11-30 17:12:51.567945 UTC - GET / - 134.6us - 200
```

### GET Request

Send a GET request to `http://localhost:8080` to retrieve file system statistics.
```
$ curl http://127.0.0.1:8080
{"path":"/tmp","free":11758252032,"available":9329881088,"total":51599257600,"buffer_size":1073741824}
```

### POST Request

Send a POST request to `http://localhost:8080` with a JSON body containing the requested amount of space. The server will return a 200 status code if the requested space is available, or a 400 status code if it is not.
```
$ curl -v http://127.0.0.1:8080 -H'Content-Type: application/json' -d'{"requested":1000}'
...
< HTTP/1.1 200 OK
...
{"path":"/tmp","free":11758252032,"available":9329881088,"total":51599257600,"buffer_size":1073741824}

$ curl -v http://127.0.0.1:8080 -H'Content-Type: application/json' -d'{"requested":1000}'
...
< HTTP/1.1 400 Bad Request
...
{"path":"/tmp","free":11758252032,"available":9329881088,"total":51599257600,"buffer_size":9329881088}
```

## Docker

The `Dockerfile` in this directory can be used to build a Docker image with the
service.
The image is Alpine based to be as small as possible.

### Build

```
$ docker build -t spacecheck .
```

### Run

```
$ docker run --rm -p 8080:8080 -v /tmp:/monitored_fs:ro spacecheck
Monitoring: "/monitored_fs"
2024-11-30 17:12:51.567945 UTC - GET / - 134.6us - 200
```

## Dependencies

This project depends on the following crates:

* `clap` for command-line argument parsing
* `fs2` for file system statistics
* `rouille` for HTTP server functionality
* `serde` for JSON serialization and deserialization

## License

This project is licensed under the MIT License.
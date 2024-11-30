// Very simple HTTP service monitoring free space on a given file system.
// The GET version simply returns statistics about the file system.
// The POST version checks if the requested amount of space is available, taking into account
// a 1GB buffer size.

use clap::Parser;
use fs2::{statvfs, FsStats};
use rouille::{try_or_400, Request, Response};
use serde::{Deserialize, Serialize};
use std::io;

// 1GB buffer
const BUFFER_SIZE: u64 = 1024 * 1024 * 1024;

fn get_stats(path: &str) -> FsStats {
    let stats =
        statvfs(path).expect(std::format!("Can't get VFS stats for {}", path.to_string()).as_str());
    return stats;
}

#[derive(Deserialize)]
struct Query {
    requested: u64,
}

#[derive(Serialize, Deserialize)]
struct Result {
    path: String,
    free: u64,
    available: u64,
    total: u64,
    buffer_size: u64,
}

#[derive(Parser)]
struct Cli {
    /// The file system to monitor
    monitored_fs: String,
}

fn handle_request(request: &Request, monitored_fs: &str) -> Response {
    // Output is common for GET and POST requests
    let stats = get_stats(&monitored_fs);
    let result = Result {
        path: monitored_fs.to_string(),
        free: stats.free_space(),
        available: stats.available_space(),
        total: stats.total_space(),
        buffer_size: BUFFER_SIZE,
    };

    let mut status_code = 200;
    if request.method() == "POST" {
        // Actually check space
        let query: Query = try_or_400!(rouille::input::json_input(request));
        if query.requested > stats.available_space() - BUFFER_SIZE {
            status_code = 400;
        }
    }
    return Response::json(&result).with_status_code(status_code);
}

fn main() {
    let args = Cli::parse();
    let monitored_fs = args.monitored_fs;

    println!("Monitoring: {:?}", monitored_fs);

    // Very basic server, we don't even car about the URL
    rouille::start_server("0.0.0.0:8080", move |request| {
        rouille::log(request, io::stdout(), || {
            handle_request(&request, &monitored_fs)
        })
    });
}

#[cfg(test)]
mod tests {
    // Note this useful idiom: importing names from outer (for mod tests) scope.
    use super::*;

    const MONITORED_FS: &str = "/tmp";

    #[test]
    fn test_handle_request_get() {
        let request = Request::fake_http("GET", "/", vec![], vec![]);
        let response = handle_request(&request, MONITORED_FS);
        assert_eq!(response.status_code, 200);
        let (mut res_data, _) = response.data.into_reader_and_size();
        let mut buffer = String::new();
        res_data.read_to_string(&mut buffer).unwrap();
        let result: Result = serde_json::from_str(&buffer).unwrap();
        assert_eq!(result.path, MONITORED_FS);
        assert!(result.available <= result.free);
    }

    #[test]
    fn test_handle_request_post_ok() {
        // This test requires at least BUFFER_SIZE+1 of free space on /tmp...
        let request = Request::fake_http(
            "POST",
            "/",
            vec![("Content-Type".to_owned(), "application/json".to_owned())],
            "{ \"requested\": 1 }".as_bytes().to_vec(),
        );
        let response = handle_request(&request, MONITORED_FS);
        assert_eq!(response.status_code, 200);
        let (mut res_data, _) = response.data.into_reader_and_size();
        let mut buffer = String::new();
        res_data.read_to_string(&mut buffer).unwrap();
        let result: Result = serde_json::from_str(&buffer).unwrap();
        assert_eq!(result.path, MONITORED_FS);
        assert!(result.available <= result.free);
    }

    #[test]
    fn test_handle_request_post_ko() {
        // Not enough free space
        let stats = get_stats(MONITORED_FS);
        let request = Request::fake_http(
            "POST",
            "/",
            vec![("Content-Type".to_owned(), "application/json".to_owned())],
            format!("{{ \"requested\": {} }}", stats.available_space() + 5)
                .as_bytes()
                .to_vec(),
        );
        let response = handle_request(&request, MONITORED_FS);
        assert_eq!(response.status_code, 400);
        let (mut res_data, _) = response.data.into_reader_and_size();
        let mut buffer = String::new();
        res_data.read_to_string(&mut buffer).unwrap();
        let result: Result = serde_json::from_str(&buffer).unwrap();
        assert_eq!(result.path, MONITORED_FS);
        assert!(result.available <= result.free);
    }
}

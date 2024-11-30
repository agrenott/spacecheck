// Very simple HTTP service monitoring free space on a given file system.
// The GET version simply returns statistics about the file system.
// The POST version checks if the requested amount of space is available, taking into account
// a 1GB buffer size.

use fs2::{statvfs, FsStats};
use rouille::try_or_400;

use clap::Parser;
use rouille::Response;
use serde::{Deserialize, Serialize};
use std::io;

// 1GB buffer
const BUFFER_SIZE: u64 = 1024 * 1024 * 1024;

fn get_stats(path: &str) -> FsStats {
    let stats = statvfs(path).expect("failed");
    return stats;
}

#[derive(Deserialize)]
struct Query {
    requested: u64,
}

#[derive(Serialize)]
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

fn main() {
    let args = Cli::parse();
    let monitored_fs = args.monitored_fs;

    println!("Monitoring: {:?}", monitored_fs);

    // Very basic server, we don't even car about the URL
    rouille::start_server("0.0.0.0:8080", move |request| {
        rouille::log(request, io::stdout(), || {
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
        })
    });
}

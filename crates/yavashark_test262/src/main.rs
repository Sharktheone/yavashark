use std::path::PathBuf;
use yavashark_env::print::PrettyPrint;
#[cfg(feature = "parser_test")]
use yavashark_test262::parsers::test_file;
#[cfg(not(feature = "parser_test"))]
use yavashark_test262::run::run_file;

#[cfg(feature = "pprof")]
use pprof::ProfilerGuard;
#[cfg(feature = "pprof")]
use pprof::protos::Message;
#[cfg(feature = "pprof")]
use std::time::{SystemTime, UNIX_EPOCH};
use std::fs;
#[cfg(feature = "pprof")]
use std::fs::File;
#[cfg(feature = "pprof")]
use std::io::Write;
#[cfg(feature = "pprof")]
use flate2::{write::GzEncoder, Compression};

const TEST262_ROOT: &str = "../../test262";

fn main() {
    let mut args = std::env::args();

    args.next();

    let mut enable_prof = false;
    let mut next = args.next().expect("please provide a test path or flags followed by a test path");
    if next == "--prof" || next == "--profile" {
        enable_prof = true;
        next = args.next().expect("please provide a test path after --prof");
    }

    let f = next;

    let path = if f.starts_with("test/") {
        PathBuf::from(TEST262_ROOT).join(f)
    } else {
        PathBuf::from(f)
    };

    if enable_prof {
        let _ = fs::create_dir_all("profiles");
    }

    #[cfg(not(feature = "parser_test"))]
    {
        #[cfg(feature = "pprof")]
        if enable_prof {
            let guard = ProfilerGuard::new(1000000).expect("failed to start profiler");
            let res = run_file(path.clone());

            if let Ok(report) = guard.report().build() {
                if let Ok(profile) = report.pprof() {
                    let mut buf = Vec::new();
                    if profile.encode(&mut buf).is_ok() {
                        let ts = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
                        let test_name = path.file_stem().and_then(|s| s.to_str()).unwrap_or("profile");
                        let pid = std::process::id();
                        let out = format!("profiles/{}-{}-{}.pb.gz", test_name, pid, ts);
                        if let Ok(file) = File::create(&out) {
                            let mut encoder = GzEncoder::new(file, Compression::default());
                            encoder.write_all(&buf).expect("failed to write profile");
                            encoder.finish().expect("failed to finish profile");
                            eprintln!("wrote profile to {} rep {}", out, buf.len());
                        }
                    }
                }
            }

            match res {
                Err(e) => println!("FAIL:\n {}", e.pretty_print()),
                Ok(v) => println!("PASS:\n {v}"),
            }
            return;
        }

        #[cfg(not(feature = "pprof"))]
        if enable_prof {
            eprintln!("Profiling requested but not enabled at compile time. Rebuild with --features pprof.");
        }

        match run_file(path) {
            Err(e) => println!("FAIL:\n {}", e.pretty_print()),
            Ok(v) => println!("PASS:\n {v}"),
        }
    }

    #[cfg(feature = "parser_test")]
    {
        #[cfg(feature = "pprof")]
        if enable_prof {
            let guard = ProfilerGuard::new(100).expect("failed to start profiler");
            let res = test_file(path.clone());

            if let Ok(report) = guard.report().build() {
                if let Ok(profile) = report.pprof() {
                    let mut buf = Vec::new();
                    if profile.encode(&mut buf).is_ok() {
                        let ts = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
                        let test_name = path.file_stem().and_then(|s| s.to_str()).unwrap_or("profile");
                        let pid = std::process::id();
                        let out = format!("profiles/{}-{}-{}.pb.gz", test_name, pid, ts);
                        if let Ok(file) = File::create(&out) {
                            let mut encoder = GzEncoder::new(file, Compression::default());
                            let _ = encoder.write_all(&buf);
                            let _ = encoder.finish();
                            eprintln!("wrote profile to {}", out);
                        }
                    }
                }
            }

            match res {
                Err(e) => println!("FAIL:\n {}", e.pretty_print()),
                Ok(v) => println!("PASS:\n {v}"),
            }
            return;
        }

        // Fallback: no profiling
        #[cfg(not(feature = "pprof"))]
        if enable_prof {
            eprintln!("Profiling requested but not enabled at compile time. Rebuild with --features pprof.");
        }

        match test_file(path) {
            Err(e) => println!("FAIL:\n {}", e.pretty_print()),
            Ok(v) => println!("PASS:\n {v}"),
        }
    }
}

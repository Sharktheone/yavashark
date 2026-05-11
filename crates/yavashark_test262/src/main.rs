use std::path::PathBuf;
#[cfg(feature = "parser_test")]
use yavashark_test262::parsers::test_file;
#[cfg(not(feature = "parser_test"))]
use yavashark_test262::run::run_file;

#[cfg(feature = "pprof")]
use flate2::{Compression, write::GzEncoder};
#[cfg(feature = "pprof")]
use pprof::ProfilerGuard;
#[cfg(feature = "pprof")]
use pprof::protos::Message;
use std::fs;
#[cfg(feature = "pprof")]
use std::fs::File;
#[cfg(feature = "pprof")]
use std::io::Write;

const TEST262_ROOT: &str = "../../test262";

const ROOT_DIRS: &[&str] = &[
    "harness/",
    "language/",
    "built-ins/",
    "annexB/",
    "intl402/",
    "staging/",
];

fn main() {
    #[cfg(feature = "timings")]
    let now = std::time::Instant::now();
    run();
    #[cfg(feature = "timings")]
    let total = now.elapsed();

    #[cfg(feature = "timings")]
    #[allow(static_mut_refs)]
    unsafe {
        eprintln!("PARSE: {:?}", yavashark_test262::PARSE_DURATION.as_nanos());
        eprintln!("SETUP: {:?}", yavashark_test262::SETUP_DURATION.as_nanos());
        eprintln!("REALM: {:?}", yavashark_test262::REALM_DURATION.as_nanos());
        eprintln!("TOTAL: {:?}", total.as_nanos());
    }
}

fn run() {
    let mut args = std::env::args();

    args.next();

    let mut enable_prof = false;
    let mut profile_out = None;
    let mut next = args
        .next()
        .expect("please provide a test path or flags followed by a test path");
    if next == "--profile-out" {
        profile_out = Some(
            args.next()
                .expect("please provide an output path after --profile-out"),
        );
        enable_prof = true;
        next = args
            .next()
            .expect("please provide a test path after --profile-out PATH");
    } else if next == "--prof" || next == "--profile" {
        enable_prof = true;
        next = args
            .next()
            .expect("please provide a test path after --prof");
    }

    let f = next;

    let path = if f.starts_with("test/") {
        PathBuf::from(TEST262_ROOT).join(f)
    } else if ROOT_DIRS.iter().any(|d| f.starts_with(d)) {
        PathBuf::from(TEST262_ROOT).join("test").join(f)
    } else {
        PathBuf::from(f)
    };

    if enable_prof {
        let _ = fs::create_dir_all("profiles");
    }

    #[allow(unused_variables)]
    let profile_out_path = profile_out
        .map(PathBuf::from)
        .or_else(|| enable_prof.then(|| PathBuf::from("profiles/test262.pb.gz")));

    #[cfg(not(feature = "parser_test"))]
    {
        #[cfg(feature = "pprof")]
        if enable_prof {
            let guard = ProfilerGuard::new(1_000_000).expect("failed to start profiler");
            let res = run_file(
                path.clone(),
                #[cfg(feature = "profiler")]
                profile_out_path.as_deref(),
            );

            write_native_profile(profile_out_path.as_deref(), &path, guard);

            match res {
                Err(e) => println!("FAIL:\n {}", e),
                Ok(v) => println!("PASS:\n {v}"),
            }
            return;
        }

        #[cfg(all(not(feature = "pprof"), feature = "profiler"))]
        if enable_prof {
            let res = run_file(path.clone(), profile_out_path.as_deref());

            match res {
                Err(e) => println!("FAIL:\n {}", e),
                Ok(v) => println!("PASS:\n {v}"),
            }
            return;
        }

        #[cfg(all(not(feature = "pprof"), not(feature = "profiler")))]
        if enable_prof {
            eprintln!(
                "Profiling requested but not enabled at compile time. Rebuild with --features pprof or profiler."
            );
        }

        match run_file(
            path,
            #[cfg(feature = "profiler")]
            None,
        ) {
            Err(e) => println!("FAIL:\n {}", e),
            Ok(v) => println!("PASS:\n {v}"),
        }
    }

    #[cfg(feature = "parser_test")]
    {
        #[cfg(feature = "pprof")]
        if enable_prof {
            let guard = ProfilerGuard::new(100).expect("failed to start profiler");
            let res = test_file(path.clone());

            write_native_profile(profile_out_path.as_deref(), &path, guard);

            match res {
                Err(e) => println!("FAIL:\n {}", e),
                Ok(v) => println!("PASS:\n {v}"),
            }
            return;
        }

        #[cfg(not(feature = "pprof"))]
        if enable_prof {
            eprintln!(
                "Profiling requested but not enabled at compile time. Rebuild with --features pprof."
            );
        }

        match test_file(path) {
            Err(e) => println!("FAIL:\n {}", e),
            Ok(v) => println!("PASS:\n {v}"),
        }
    }
}

#[cfg(feature = "pprof")]
fn write_native_profile(
    profile_out: Option<&std::path::Path>,
    path: &std::path::Path,
    guard: ProfilerGuard<'_>,
) {
    let Ok(report) = guard.report().build() else {
        return;
    };
    let Ok(profile) = report.pprof() else {
        return;
    };

    let mut buf = Vec::new();
    if profile.encode(&mut buf).is_err() {
        return;
    }

    let out = profile_out
        .map(std::path::Path::to_path_buf)
        .unwrap_or_else(|| {
            let test_name = path
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("profile");
            PathBuf::from(format!("profiles/{test_name}.native.pb.gz"))
        });

    if let Some(parent) = out.parent() {
        let _ = fs::create_dir_all(parent);
    }

    let Ok(file) = File::create(&out) else {
        return;
    };
    let mut encoder = GzEncoder::new(file, Compression::default());
    if encoder.write_all(&buf).is_err() {
        return;
    }
    if encoder.finish().is_err() {
        return;
    }

    eprintln!("wrote native profile to {}", out.display());
}

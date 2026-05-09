use flate2::Compression;
use flate2::write::GzEncoder;
use pprof::protos::Message;
use pprof::protos::{Function, Line, Location, Profile as PProfProfile, Sample, ValueType};
use std::collections::HashMap;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use indexmap::IndexSet;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct FrameId(u64);

#[derive(Debug, Clone)]
struct Frame {
    id: FrameId,
    parent: Option<FrameId>,
    fn_name: String,
    start: Instant,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct SampleKey {
    stack: Vec<String>,
}

#[derive(Debug, Clone, Copy, Default)]
struct SampleValue {
    count: u64,
    nanos: u64,
}

#[derive(Debug, Clone)]
pub struct Profile {
    start_time: SystemTime,
    root_start: Instant,
    next_frame_id: FrameId,
    roots: Vec<FrameId>,
    active_frames: HashMap<FrameId, Frame>, //TODO: this could probably also just be a stack. Not fully sure about async though
    finished_samples: HashMap<SampleKey, SampleValue>,
}

impl Default for Profile {
    fn default() -> Self {
        Self {
            start_time: SystemTime::now(),
            root_start: Instant::now(),
            next_frame_id: FrameId(1),
            roots: Vec::new(),
            active_frames: HashMap::new(),
            finished_samples: HashMap::new(),
        }
    }
}

impl Profile {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_frame(&mut self, fn_name: String, start: Instant) -> FrameId {
        let id = self.next_frame_id;
        self.next_frame_id.0 += 1;
        let parent = self.roots.last().copied();

        self.active_frames.insert(
            id,
            Frame {
                id,
                parent,
                fn_name,
                start,
            },
        );
        self.roots.push(id);
        id
    }

    pub fn end_frame(&mut self, frame_id: FrameId, end: Instant) {
        let Some(frame) = self.active_frames.remove(&frame_id) else {
            return;
        };

        if frame.id != frame_id {
            return;
        }

        if self.roots.last().copied() == Some(frame_id) {
            self.roots.pop();
        } else if let Some(index) = self.roots.iter().rposition(|id| *id == frame_id) {
            self.roots.remove(index);
        }

        let elapsed = frame.start.duration_since(end);
        let nanos = elapsed.as_nanos() as u64;
        let stack = self.stack_for_frame(&frame);
        let sample = self.finished_samples.entry(SampleKey { stack }).or_default();
        sample.count += 1;
        sample.nanos += nanos;
    }

    #[must_use]
    pub fn duration(&self) -> Duration {
        self.root_start.elapsed()
    }

    fn stack_for_frame(&self, frame: &Frame) -> Vec<String> {
        let mut stack = vec![frame.fn_name.clone()];
        let mut parent = frame.parent;

        while let Some(parent_id) = parent {
            let Some(parent_frame) = self.active_frames.get(&parent_id) else {
                break;
            };

            stack.push(parent_frame.fn_name.clone());
            parent = parent_frame.parent;
        }

        stack
    }
}

pub trait ProfileWriter: Send {
    fn write_profile(&mut self, profile: &Profile) -> io::Result<Vec<u8>>;
}

#[derive(Debug, Clone)]
pub enum ProfileWriterKind {
    Pprof,
}

impl ProfileWriterKind {
    pub fn from_path(path: &Path) -> io::Result<Self> {
        match path
            .extension()
            .and_then(|ext| ext.to_str())
            .unwrap_or_default()
        {
            "pb" | "gz" => Ok(Self::Pprof),
            other => Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                format!("unsupported profile output format: {other}"),
            )),
        }
    }
}

pub struct PprofWriter;

impl ProfileWriter for PprofWriter {
    fn write_profile(&mut self, profile: &Profile) -> io::Result<Vec<u8>> {
        let encoded = Self::build_pprof(profile)?.encode_to_vec();
        let mut gz = GzEncoder::new(Vec::new(), Compression::default());
        gz.write_all(&encoded)?;
        gz.finish()
    }
}

impl PprofWriter {
    fn build_pprof(profile: &Profile) -> io::Result<PProfProfile> {
        let mut strings = StringTable::default();
        let mut locations = Vec::new();
        let mut functions = Vec::new();
        let mut samples = Vec::new();
        let mut function_ids = HashMap::new();
        let mut location_ids = HashMap::new();

        for (sample_key, sample_value) in &profile.finished_samples {
            let mut location_id = Vec::with_capacity(sample_key.stack.len());

            for fn_name in &sample_key.stack {
                let function_id = *function_ids.entry(fn_name.clone()).or_insert_with(|| {
                    let id = (functions.len() + 1) as u64;
                    let name = strings.intern(fn_name);

                    functions.push(Function {
                        id,
                        name,
                        system_name: name,
                        filename: 0,
                        start_line: 0,
                    });

                    id
                });

                let location = *location_ids.entry(function_id).or_insert_with(|| {
                    let id = (locations.len() + 1) as u64;

                    locations.push(Location {
                        id,
                        mapping_id: 0,
                        address: 0,
                        line: vec![Line {
                            function_id,
                            line: 0,
                        }],
                        is_folded: false,
                    });

                    id
                });

                location_id.push(location);
            }

            samples.push(Sample {
                location_id,
                value: vec![sample_value.count as i64, sample_value.nanos as i64],
                label: Vec::new(),
            });
        }

        let time_nanos = profile
            .start_time
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos() as i64;

        let sample_count_type = strings.intern("samples");
        let sample_count_unit = strings.intern("count");
        let wall_type = strings.intern("wall");
        let nanos_unit = strings.intern("nanoseconds");

        let period_type = ValueType {
            ty: wall_type,
            unit: nanos_unit,
        };
        let string_table = strings.finish();

        Ok(PProfProfile {
            sample_type: vec![
                ValueType {
                    ty: sample_count_type,
                    unit: sample_count_unit,
                },
                ValueType {
                    ty: wall_type,
                    unit: nanos_unit,
                },
            ],
            sample: samples,
            mapping: Vec::new(),
            location: locations,
            function: functions,
            string_table,
            drop_frames: 0,
            keep_frames: 0,
            time_nanos,
            duration_nanos: profile.duration().as_nanos() as i64,
            period_type: Some(period_type),
            period: 1,
            comment: Vec::new(),
            default_sample_type: wall_type,
        })
    }
}

pub struct FileProfileWriter {
    path: PathBuf,
    inner: Box<dyn ProfileWriter>,
}

impl FileProfileWriter {
    pub fn new(path: impl Into<PathBuf>, inner: Box<dyn ProfileWriter>) -> Self {
        Self {
            path: path.into(),
            inner,
        }
    }

    pub fn from_path(path: impl Into<PathBuf>) -> io::Result<Self> {
        let path = path.into();
        let kind = ProfileWriterKind::from_path(&path)?;
        let inner: Box<dyn ProfileWriter> = match kind {
            ProfileWriterKind::Pprof => Box::new(PprofWriter),
        };

        Ok(Self::new(path, inner))
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn write_to_path(&mut self, profile: &Profile) -> io::Result<PathBuf> {
        if let Some(parent) = self.path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let bytes = self.inner.write_profile(profile)?;
        std::fs::write(&self.path, bytes)?;
        Ok(self.path.clone())
    }
}




#[derive(Default)]
struct StringTable {
    strings: IndexSet<String>,
}

impl StringTable {
    fn intern(&mut self, value: &str) -> i64 {
        self.strings.insert_full(value.to_string()).0 as i64
    }

    fn finish(self) -> Vec<String> {
        let mut result = Vec::with_capacity(self.strings.len() + 1);
        result.push(String::new());
        result.extend(self.strings);
        result
    }
}

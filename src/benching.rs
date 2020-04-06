use std::fmt::{self, Display};
use std::fs::File;
use std::io;
use std::io::{BufWriter, Write};
use std::time::{Duration, Instant};

use rayon::prelude::*;
use termion::{color, style};

#[derive(Debug, Clone)]
pub struct BenchVec {
    pub inner: Vec<Duration>,
}

/// A struct that stores a vector of Durations for benchmarks
/// and allows some statistical operations on it
impl BenchVec {
    /// Creates a new empty BenchVec
    pub fn new() -> Self {
        Self { inner: Vec::new() }
    }

    /// Creates a BenchVec from an existing vector of Durations
    pub fn from_vec(vec: &Vec<Duration>) -> Self {
        Self { inner: vec.clone() }
    }

    /// Adds an element to the BenchVec
    pub fn push(&mut self, item: Duration) -> &mut Self {
        self.inner.push(item);

        self
    }

    /// Appends a different BenchVec to this one
    pub fn append(&mut self, other: Self) -> &mut Self {
        self.inner.append(&mut other.inner.clone());

        self
    }

    /// Returns the length of stored elements
    pub fn len(&self) -> usize {
        self.inner.len()
    }

    /// Returns the sum of all stored elements
    pub fn sum(&self) -> Duration {
        self.inner.par_iter().sum::<Duration>()
    }

    /// Returns the average of all durations
    pub fn average(&self) -> Duration {
        self.sum() / self.inner.len() as u32
    }

    /// Returns the standard deviation of all durations
    pub fn standard_deviation(&self) -> f64 {
        (self.sum().as_nanos() as f64 / (self.len() as f64 - 1f64)).sqrt()
    }

    /// Compares two benchmarks by calculating the average
    pub fn compare(&self, other: Self) -> DurationDifference {
        let avg1 = self.average();
        let avg2 = other.average();
        if avg1 > avg2 {
            DurationDifference {
                inner: avg1 - avg2,
                positive: true,
            }
        } else {
            DurationDifference {
                inner: avg2 - avg1,
                positive: false,
            }
        }
    }
}

impl Display for BenchVec {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let avg_duration = self.average();
        let standard_deviation = self.standard_deviation();
        write!(
            f,
            "{:?} (±{:.2}ns ~ {:.2}%)",
            avg_duration,
            standard_deviation,
            (standard_deviation / avg_duration.as_nanos() as f64) * 100f64
        )
    }
}

#[derive(Debug, Clone)]
pub struct DurationDifference {
    pub inner: Duration,
    pub positive: bool,
}

impl DurationDifference {
    pub fn new(left: &BenchVec, right: &BenchVec) -> Self {
        let left_avg = left.average();
        let right_avg = right.average();
        if left_avg > right_avg {
            Self {
                inner: left_avg - right_avg,
                positive: true,
            }
        } else {
            Self {
                inner: right_avg - left_avg,
                positive: false,
            }
        }
    }
}

impl Display for DurationDifference {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}{:?}",
            if self.positive { "+" } else { "-" },
            self.inner
        )
    }
}

pub struct Bencher {
    measurements: Vec<BenchVec>,
    iterations: usize,
    max_auto_iterations: usize,
    bench_duration: Duration,
    writer: Option<BufWriter<File>>,
}

impl Bencher {
    pub fn new() -> Self {
        Self {
            bench_duration: Self::calculate_bench_duration(),
            measurements: Vec::new(),
            iterations: 100,
            max_auto_iterations: 10000,
            writer: None,
        }
    }

    /// Calculates the time it takes to measure a benchmark
    fn calculate_bench_duration() -> Duration {
        let mut durations = BenchVec::new();
        for _ in 0..1000 {
            let start = Instant::now();
            durations.push(start.elapsed());
        }

        durations.average()
    }

    /// Sets the number of iterations a benchmark will be run
    /// If set to 0 it iterates until the standard deviation is below 1%
    pub fn set_iterations(&mut self, iterations: usize) -> &mut Self {
        self.iterations = iterations;

        self
    }

    pub fn set_max_iterations(&mut self, iterations: usize) -> &mut Self {
        self.max_auto_iterations = iterations;

        self
    }

    /// Benchmarks a closure a configured number of times.
    /// The result will be printed to the console with the given name.
    pub fn bench<T, F: FnMut() -> T>(&mut self, name: &str, mut func: F) -> &mut Self {
        let mut durations = BenchVec::new();
        println!(
            "\n{}{}{}{}",
            color::Fg(color::LightBlue),
            style::Bold,
            name,
            style::Reset
        );
        if self.iterations == 0 {
            let mut count = 0;
            while count < self.max_auto_iterations {
                let start = Instant::now();
                func();
                let duration = start.elapsed();
                if duration > self.bench_duration {
                    durations.push(duration - self.bench_duration);
                } else {
                    durations.push(duration);
                }
                if (durations.standard_deviation() / durations.average().as_nanos() as f64) < 0.01
                    && count > 1
                {
                    break;
                }
                count += 1;
            }
            println!("{}After {} iterations{}", style::Faint, count, style::Reset);
        } else {
            for _ in 0..self.iterations {
                let start = Instant::now();
                func();
                let duration = start.elapsed();
                if duration > self.bench_duration {
                    durations.push(duration - self.bench_duration);
                } else {
                    durations.push(duration);
                }
            }
        }
        println!("Result: {}", durations);
        if let Some(writer) = &mut self.writer {
            let _ = writer.write(
                format!(
                    "{}\t{:?}\t{:.2}ns\n",
                    name,
                    durations.average(),
                    durations.standard_deviation()
                )
                .as_bytes(),
            );
        }
        self.measurements.push(durations);

        self
    }

    /// Compares the last two benchmarks
    /// If the number of benchmarks is below 2 it doesn't do anything
    pub fn compare(&mut self) -> &mut Self {
        if self.measurements.len() > 1 {
            let left = self.measurements.get(self.measurements.len() - 1).unwrap();
            let right = self.measurements.get(self.measurements.len() - 2).unwrap();
            let diff = DurationDifference::new(left, right);
            println!("Difference: {}", diff);
        }

        self
    }

    /// Prints the settings of the Bencher
    pub fn print_settings(&mut self) -> &mut Self {
        println!(
            "\n{}{}Benchmarking Settings{}",
            color::Fg(color::Green),
            style::Underline,
            style::Reset
        );
        println!("Benchmarking accuracy delay:\t {:?}", self.bench_duration);
        println!(
            "Number of iterations:\t {}",
            if self.iterations > 0 {
                self.iterations.to_string()
            } else {
                "auto".to_string()
            }
        );
        if self.iterations == 0 {
            println!("Maximum number of iterations: {}", self.max_auto_iterations)
        }

        self
    }

    /// Adds a file to write the output to
    pub fn write_output_to(&mut self, writer: BufWriter<File>) -> &mut Self {
        self.writer = Some(writer);

        self
    }

    pub fn flush(&mut self) -> io::Result<()> {
        if let Some(writer) = &mut self.writer {
            writer.flush()
        } else {
            Ok(())
        }
    }
}

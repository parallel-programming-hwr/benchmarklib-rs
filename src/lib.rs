pub mod benching;

#[cfg(test)]
mod tests {
    use super::benching::Bencher;
    use crate::benching::BENCH_FILE_HEAD;
    use std::fs::{read_to_string, remove_file, File};
    use std::io::BufWriter;

    #[test]
    fn it_works() {
        let mut bencher = Bencher::new();
        let mut executed = false;
        bencher.bench("lol", || executed = true);
        assert!(executed)
    }

    #[test]
    fn it_benches_specifically() {
        let mut bencher = Bencher::new();
        let mut count = 0;
        bencher.set_iterations(243);
        bencher.bench("lol", || count += 1);
        assert_eq!(count, 243);
    }

    #[test]
    fn it_benches_automatically() {
        let mut bencher = Bencher::new();
        let mut count = 0;
        bencher.set_iterations(0).set_max_iterations(1000);
        bencher.bench("lol", || count += 1);
        assert!(count > 1);
    }

    #[test]
    fn it_reports_differences() {
        let mut bencher = Bencher::new();
        bencher.bench("lol", || 3 * 4);
        bencher.bench("lol2", || 35 * 4);
        bencher.compare();
    }

    #[test]
    fn it_prints_settings() {
        let mut bencher = Bencher::new();
        bencher.print_settings();
        bencher.set_iterations(0);
        bencher.print_settings();
    }

    #[test]
    fn it_writes_into_files() {
        let mut bencher = Bencher::new();
        let file = File::create("test.tsv").unwrap();
        bencher
            .write_output_to(BufWriter::new(file))
            .bench("closure", || {})
            .flush()
            .unwrap();
        let contents = read_to_string("test.tsv").unwrap();
        assert!(contents.contains(BENCH_FILE_HEAD));
        assert!(contents.len() > BENCH_FILE_HEAD.len());
        remove_file("test.tsv").unwrap();
    }
}

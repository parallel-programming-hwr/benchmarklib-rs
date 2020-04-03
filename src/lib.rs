pub mod benching;

#[cfg(test)]
mod tests {
    use super::benching::Bencher;

    #[test]
    fn bencher_works() {
        let mut bencher = Bencher::new();
        let mut executed = false;
        bencher.bench("lol", || executed = true);
        assert!(executed)
    }

    #[test]
    fn bench_iterations() {
        let mut bencher = Bencher::new();
        let mut count = 0;
        bencher.set_iterations(243);
        bencher.bench("lol", || count += 1);
        assert_eq!(count, 243);
    }

    #[test]
    fn bench_auto() {
        let mut bencher = Bencher::new();
        let mut count = 0;
        bencher.set_iterations(0).set_max_iterations(1000);
        bencher.bench("lol", || count += 1);
        assert!(count > 1);
    }

    #[test]
    fn bench_difference() {
        let mut bencher = Bencher::new();
        bencher.bench("lol", || 3*4);
        bencher.bench("lol2", || 35*4);
        bencher.compare();
    }
}

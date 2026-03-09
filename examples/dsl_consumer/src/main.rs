fn main() {
    let state = dsl_consumer::run_demo();
    println!("home_points={} log_count={}", state.home_points, state.log_count);
}

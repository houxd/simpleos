pub trait CpuDriver {
    fn cpu_reset(&mut self) -> !;
}

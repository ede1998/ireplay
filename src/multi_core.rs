use embassy_executor::Spawner;
use esp_hal::{
    cpu_control::{CpuControl, Error, Stack},
    peripheral::Peripheral,
    peripherals::CPU_CTRL,
};
use esp_hal_embassy::Executor;
use picoserve::make_static;

pub fn start_2nd_core<F>(cpu: impl Peripheral<P = CPU_CTRL>, f: F) -> Result<(), Error>
where
    F: FnOnce(Spawner) + Send,
{
    let mut cpu_control = CpuControl::new(cpu);
    let stack = make_static!(Stack<8192>, Stack::new());

    let guard = cpu_control.start_app_core(stack, move || {
        let executor = make_static!(Executor, Executor::new());
        executor.run(f);
    })?;

    core::mem::forget(guard);

    Ok(())
}

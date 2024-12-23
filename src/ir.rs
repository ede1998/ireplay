use alloc::{boxed::Box, vec};
use embassy_time::{Duration, Ticker};
use esp_hal::{
    gpio::{Input, Output},
    peripheral::Peripheral,
};

pub struct Ir {
    receiver: Input<'static>,
    sender: Output<'static>,
    ticker: Ticker,
}

impl core::fmt::Debug for Ir {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        f.debug_struct("Ir").finish()
    }
}

impl Ir {
    pub fn new(
        receiver: impl Peripheral<P = impl esp_hal::gpio::InputPin> + 'static,
        sender: impl Peripheral<P = impl esp_hal::gpio::OutputPin> + 'static,
    ) -> Self {
        let receiver = Input::new(receiver, esp_hal::gpio::Pull::Up);
        let sender = Output::new(sender, esp_hal::gpio::Level::Low);

        // Min. pulse duration = 500us -> double it to protect against aliasing
        const TICK_RATE: Duration = Duration::from_micros(250);
        let ticker = Ticker::every(TICK_RATE);

        Self {
            receiver,
            sender,
            ticker,
        }
    }

    pub async fn record(&mut self) -> Box<[u8]> {
        let mut signal = vec![0u8; 1000];
        self.receiver.wait_for_falling_edge().await;
        self.ticker.reset();
        for v in &mut signal {
            let value = if self.receiver.is_high() { 1 } else { 0 };
            *v = value;
            self.ticker.next().await;
        }

        assert_eq!(signal.last(), Some(&1));
        // let last_sample = signal.iter().rposition(|&v| v == 0).unwrap_or_default();
        // let last_sample = (last + 10).min(signal.len());
        const SIGNAL_DURATION: Duration = Duration::from_millis(60);
        let last_sample = (SIGNAL_DURATION.as_micros() / 250).try_into().unwrap();
        signal.truncate(last_sample);
        signal.into()
    }

    pub async fn replay(&mut self, signal: &[u8]) {
        self.ticker.reset();

        for &value in signal.iter() {
            if value == 0 {
                self.sender.set_low();
            } else {
                self.sender.set_high();
            }
            self.ticker.next().await;
        }

        self.sender.set_low();
    }
}

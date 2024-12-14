use embassy_executor::Spawner;
use embassy_net::{Stack, StackResources};
use embassy_time::{Duration, Timer};
use esp_hal::rng::Rng;
use esp_println::println;
use esp_wifi::{
    wifi::{
        ClientConfiguration, Configuration, WifiController, WifiDevice, WifiEvent, WifiStaDevice,
        WifiState,
    },
    EspWifiController,
};
use picoserve::make_static;
use rand::RngCore;

use crate::WEB_TASK_POOL_SIZE;

const SSID: &str = env!("SSID");
const PASSWORD: &str = env!("PASSWORD");

pub struct WifiBuilder<W, T, R> {
    pub wifi: W,
    pub radio_clock: R,
    pub timer: T,
    pub rng: Rng,
}

impl<W, T, R> WifiBuilder<W, T, R> {
    pub async fn connect(mut self, spawner: &Spawner) -> Stack<'static>
    where
        W: esp_hal::peripheral::Peripheral<P = esp_hal::peripherals::WIFI> + 'static,
        T: esp_hal::peripheral::Peripheral<P: esp_wifi::EspWifiTimerSource> + 'static,
        R: esp_hal::peripheral::Peripheral<P = esp_hal::peripherals::RADIO_CLK> + 'static,
    {
        let init = &*make_static!(
            EspWifiController<'static>,
            esp_wifi::init(self.timer, self.rng, self.radio_clock).unwrap(),
        );

        let (wifi_interface, controller) =
            esp_wifi::wifi::new_with_mode(init, self.wifi, WifiStaDevice).unwrap();

        let (stack, runner) = embassy_net::new(
            wifi_interface,
            embassy_net::Config::dhcpv4(Default::default()),
            make_static!(
                StackResources<{ WEB_TASK_POOL_SIZE + 1 }>,
                StackResources::new()
            ),
            self.rng.next_u64(),
        );

        spawner.must_spawn(connection(controller));
        spawner.must_spawn(net_task(runner));

        loop {
            if stack.is_link_up() {
                break;
            }
            Timer::after(Duration::from_millis(500)).await;
        }

        println!("Waiting to get IP address...");
        loop {
            if let Some(config) = stack.config_v4() {
                println!("Got IP: {}", config.address);
                break stack;
            }
            Timer::after(Duration::from_millis(500)).await;
        }
    }
}

#[embassy_executor::task]
async fn connection(mut controller: WifiController<'static>) {
    println!("start connection task");
    println!("Device capabilities: {:?}", controller.capabilities());
    loop {
        if let WifiState::StaConnected = esp_wifi::wifi::wifi_state() {
            // wait until we're no longer connected
            controller.wait_for_event(WifiEvent::StaDisconnected).await;
            Timer::after(Duration::from_millis(5000)).await
        }
        if !matches!(controller.is_started(), Ok(true)) {
            let client_config = Configuration::Client(ClientConfiguration {
                ssid: SSID.try_into().unwrap(),
                password: PASSWORD.try_into().unwrap(),
                ..Default::default()
            });
            controller.set_configuration(&client_config).unwrap();
            println!("Starting wifi");
            controller.start_async().await.unwrap();
            println!("Wifi started!");
        }
        println!("About to connect...");

        match controller.connect_async().await {
            Ok(_) => println!("Wifi connected!"),
            Err(e) => {
                println!("Failed to connect to wifi: {e:?}");
                Timer::after(Duration::from_millis(5000)).await
            }
        }
    }
}

#[embassy_executor::task]
async fn net_task(mut runner: embassy_net::Runner<'static, WifiDevice<'static, WifiStaDevice>>) {
    runner.run().await
}

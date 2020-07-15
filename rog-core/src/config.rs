use crate::virt_device::ConsumerKeys;
use rog_client::aura_modes::AuraModes;
use rog_fan_curve::Curve;
use serde_derive::{Deserialize, Serialize};
use std::fs::{File, OpenOptions};
use std::io::{Read, Write};

pub static CONFIG_PATH: &str = "/etc/rogcore.conf";

#[derive(Default, Deserialize, Serialize)]
pub struct Config {
    pub rog_key: ConsumerKeys,
    pub fan_mode: u8,
    pub bat_charge_limit: u8,
    pub brightness: u8,
    pub current_mode: u8,
    pub builtin_modes: Vec<AuraModes>,
    pub mode_performance: FanModeSettings,
}

impl Config {
    /// `load` will attempt to read the config, but if it is not found it
    /// will create a new default config and write that out.
    pub fn load(mut self, supported_led_modes: &[u8]) -> Self {
        let mut file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(&CONFIG_PATH)
            .expect("config file error");
        let mut buf = String::new();
        if let Ok(l) = file.read_to_string(&mut buf) {
            if l == 0 {
                // create a default config here
                let mut c = Config::default();
                c.bat_charge_limit = 100;
                c.current_mode = 0;

                for n in supported_led_modes {
                    c.builtin_modes.push(AuraModes::from(*n))
                }

                // Should be okay to unwrap this as is since it is a Default
                let json = serde_json::to_string_pretty(&c).unwrap();
                file.write_all(json.as_bytes())
                    .unwrap_or_else(|_| panic!("Could not deserialise {}", CONFIG_PATH));
                self = c;
            } else {
                self = serde_json::from_str(&buf)
                    .unwrap_or_else(|_| panic!("Could not deserialise {}", CONFIG_PATH));
            }
        }
        self
    }

    pub fn read(&mut self) {
        let mut file = OpenOptions::new()
            .read(true)
            .open(&CONFIG_PATH)
            .expect("config file error");
        let mut buf = String::new();
        if let Ok(l) = file.read_to_string(&mut buf) {
            if l == 0 {
                panic!("Missing {}", CONFIG_PATH);
            } else {
                let x: Config = serde_json::from_str(&buf)
                    .unwrap_or_else(|_| panic!("Could not deserialise {}", CONFIG_PATH));
                *self = x;
            }
        }
    }

    pub fn write(&self) {
        let mut file = File::create(CONFIG_PATH).expect("Couldn't overwrite config");
        let json = serde_json::to_string_pretty(self).expect("Parse config to JSON failed");
        file.write_all(json.as_bytes())
            .expect("Saving config failed");
    }

    pub fn set_mode_data(&mut self, mode: AuraModes) {
        let byte: u8 = (&mode).into();
        for (index, n) in self.builtin_modes.iter().enumerate() {
            if byte == u8::from(n) {
                // Consume it, OMNOMNOMNOM
                self.builtin_modes[index] = mode;
                break;
            }
        }
    }

    pub fn get_led_mode_data(&self, num: u8) -> Option<&AuraModes> {
        for mode in &self.builtin_modes {
            if u8::from(mode) == num {
                return Some(mode);
            }
        }
        None
    }
}

#[derive(Default, Deserialize, Serialize)]
pub struct FanModeSettings {
    pub normal: IntelPState,
    pub boost: IntelPState,
    pub silent: IntelPState,
}

#[derive(Deserialize, Serialize)]
pub struct IntelPState {
    pub min_percentage: u8,
    pub max_percentage: u8,
    pub no_turbo: bool,
    pub fan_curve: Option<Curve>,
}

impl Default for IntelPState {
    fn default() -> Self {
        IntelPState {
            min_percentage: 0,
            max_percentage: 100,
            no_turbo: false,
            fan_curve: None,
        }
    }
}

use vgm_parser::vgmfile::VgmFile;

const SAMPLE_RATE: usize = 44100;
const PSG_CLOCK_RATE: usize = 3580000;

fn sleep_for_n_samples(n: usize) {
    std::thread::sleep(std::time::Duration::from_secs_f64(
        (n as f64) / (SAMPLE_RATE as f64),
    ));
}

#[derive(Copy, Clone, Debug, Default)]
struct Sn76489an {
    tone_1_frequency: u16,
    tone_1_attenuation: u8,
    tone_2_frequency: u16,
    tone_2_attenuation: u8,
    tone_3_frequency: u16,
    tone_3_attenuation: u8,
    noise_type: u8,
    noise_attenuation: u8,
    latched_channel: LatchedChannel,
}

#[derive(Copy, Clone, Debug)]
enum LatchedChannel {
    Tone1,
    Tone2,
    Tone3,
}

impl Default for LatchedChannel {
    fn default() -> Self {
        Self::Tone1
    }
}

impl Sn76489an {
    fn tone_1_frequency(self, data: u16) -> Self {
        Sn76489an {
            tone_1_frequency: data,
            ..self
        }
    }

    fn tone_1_attenuation(self, data: u8) -> Self {
        Sn76489an {
            tone_1_attenuation: data,
            ..self
        }
    }

    fn tone_2_frequency(self, data: u16) -> Self {
        Sn76489an {
            tone_2_frequency: data,
            ..self
        }
    }

    fn tone_2_attenuation(self, data: u8) -> Self {
        Sn76489an {
            tone_2_attenuation: data,
            ..self
        }
    }

    fn tone_3_frequency(self, data: u16) -> Self {
        Sn76489an {
            tone_3_frequency: data,
            ..self
        }
    }

    fn tone_3_attenuation(self, data: u8) -> Self {
        Sn76489an {
            tone_3_attenuation: data,
            ..self
        }
    }

    fn noise_type(self, data: u8) -> Self {
        Sn76489an {
            noise_type: data,
            ..self
        }
    }

    fn noise_attenuation(self, data: u8) -> Self {
        Sn76489an {
            noise_attenuation: data,
            ..self
        }
    }

    fn latched_channel(self, channel: LatchedChannel) -> Self {
        Sn76489an {
            latched_channel: channel,
            ..self
        }
    }

    fn update(self, cmd: u8) -> Self {
        

        if cmd >> 7 == 1 {
            let operand = cmd & 0x0F;

            match (cmd & 0x70) >> 4 {
                0b000 => self.tone_1_frequency((self.tone_1_frequency & 0x3F0) | (operand as u16)).latched_channel(LatchedChannel::Tone1),
                0b001 => self.tone_1_attenuation(operand),
                0b010 => self.tone_2_frequency((self.tone_2_frequency & 0x3F0) | (operand as u16)).latched_channel(LatchedChannel::Tone2),
                0b011 => self.tone_2_attenuation(operand),
                0b100 => self.tone_3_frequency((self.tone_3_frequency & 0x3F0) | (operand as u16)).latched_channel(LatchedChannel::Tone3),
                0b101 => self.tone_3_attenuation(operand),
                0b110 => self.noise_type(operand),
                0b111 => self.noise_attenuation(operand),
                _ => unreachable!(),
            }
        } else {
            let operand = ((cmd & 0x3F) as u16) << 4;

            match self.latched_channel {
                LatchedChannel::Tone1 => self.tone_1_frequency(
                    operand | (self.tone_1_frequency & 0xF),
                ),
                LatchedChannel::Tone2 => self.tone_2_frequency(
                    operand | (self.tone_2_frequency & 0xF),
                ),
                LatchedChannel::Tone3 => self.tone_3_frequency(
                    operand | (self.tone_3_frequency & 0xF),
                ),
            }
        }
    }
}

fn get_note(freq: u16) -> Option<isize> {
    match freq {
        0 => None,
        _ => Some(
            (12.0 * (((PSG_CLOCK_RATE as f64) / (32.0 * (freq as f64))) / 440.0).log2()).round()
                as isize,
        ),
    }
}

fn main() {
    let vgm_file = VgmFile::from_path_gz("tests/14 - Stage 2 - Dancing Bunny Girls.vgz");

    let mut psg = Sn76489an::default();
    let mut last_psg = Sn76489an::default();

    let mut ch1_note: Option<isize> = None;
    let mut ch2_note: Option<isize> = None;
    let mut ch3_note: Option<isize> = None;

    for cmd in vgm_file.commands {
        type C = vgm_parser::command::Command;

        match cmd {
            C::PSGWrite { value } => {
                psg = psg.update(value);

                //println!("{:?}", psg);
            }
            C::YM2612Port0Write { register: _, value: _ } => {
                //println!("Write Port 0 Register {register}: {value}")
            }
            C::YM2612Port1Write { register: _, value: _ } => {
                //println!("Write Port 1 Register {register}: {value}")
            }
            C::WaitNSamples { n } => sleep_for_n_samples(n as usize),
            C::Wait735Samples => sleep_for_n_samples(735),
            C::Wait882Samples => sleep_for_n_samples(882),
            C::WaitNSamplesPlus1 { n } => sleep_for_n_samples(n as usize + 1),
            _ => (),
        }

        fn check_channel_changed(last_freq: u16, freq: u16, last_attenuation: u8, attenuation: u8) -> Option<Option<isize>> {
            // Is the PSG on?
            if attenuation != 0xF {
                // Did we just turn our note on or change frequency?
                if last_attenuation == 0xF
                    || last_freq != freq
                {
                    return Some(get_note(freq))
                }
            } else if last_attenuation != 0xF {
                return Some(None)
            }

            None
        }

        if matches!(cmd, C::WaitNSamples {n: _} | C::Wait735Samples | C::Wait882Samples | C::WaitNSamplesPlus1 {n: _}) {
            let last_ch1_note = ch1_note;
            ch1_note = match check_channel_changed(last_psg.tone_1_frequency, psg.tone_1_frequency, psg.tone_1_attenuation, psg.tone_1_attenuation) {
                    Some(n) => n,
                    None => ch1_note
            };

            if last_ch1_note != ch1_note {
                match ch1_note {
                    Some(n) => println!("Ch 1: note on {n}"),
                    None => println!("Ch 1: note off"),
                }
            }

            let last_ch2_note = ch2_note;
            ch2_note = match check_channel_changed(last_psg.tone_2_frequency, psg.tone_2_frequency, psg.tone_2_attenuation, psg.tone_2_attenuation) {
                    Some(n) => n,
                    None => ch2_note
            };

            if last_ch2_note != ch2_note {
                match ch2_note {
                    Some(n) => println!("Ch 2: note on {n}"),
                    None => println!("Ch 2: note off"),
                }
            }

            let last_ch3_note = ch3_note;
            ch3_note = match check_channel_changed(last_psg.tone_3_frequency, psg.tone_3_frequency, psg.tone_3_attenuation, psg.tone_3_attenuation) {
                    Some(n) => n,
                    None => ch3_note
            };

            if last_ch3_note != ch3_note {
                match ch3_note {
                    Some(n) => println!("Ch 3: note on {n}"),
                    None => println!("Ch 3: note off"),
                }
            }

            last_psg = psg;
        }
    }
}

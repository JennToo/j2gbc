use std::ops::DerefMut;
use std::thread;

use cpal;
use hound;
use rb;
use rb::{RbConsumer, RbProducer, RB};

use emu::audio::AudioSink;

pub struct CpalSink {
    prod: rb::Producer<f32>,
    rate: u64,

    samples: Vec<f32>,
}

impl CpalSink {
    pub fn new() -> Result<CpalSink, String> {
        let event_loop = cpal::EventLoop::new();
        let device_o = cpal::default_output_device();
        if device_o.is_none() {
            return Err("No default output device".into());
        }
        let device = device_o.unwrap();
        let format = device.default_output_format().map_err(|e| e.to_string())?;
        let stream_id = event_loop
            .build_output_stream(&device, &format)
            .map_err(|e| e.to_string())?;
        event_loop.play_stream(stream_id);

        let buffer = rb::SpscRb::new(format.sample_rate.0 as usize * format.channels as usize);
        let prod = buffer.producer();
        let consumer = buffer.consumer();

        thread::spawn(move || {
            event_loop.run(move |_, data| match data {
                cpal::StreamData::Output {
                    buffer: cpal::UnknownTypeOutputBuffer::F32(mut buffer),
                } => match consumer.read(buffer.deref_mut()) {
                    Ok(read) => {
                        let shortfall = buffer.deref_mut().len() - read;
                        if shortfall > 0 {
                            warn!(target: "events", "Audio shortfall {}", shortfall);
                        }
                    }
                    Err(shortfall) => {
                        warn!(target: "events", "Audio {}", shortfall);
                    }
                },

                _ => (),
            });
        });

        Ok(CpalSink {
            prod,
            rate: format.sample_rate.0 as u64,
            samples: Vec::new(),
        })
    }
}

impl AudioSink for CpalSink {
    fn emit_sample(&mut self, sample: (f32, f32)) {
        match self.prod.write(&[sample.0, sample.1]) {
            Ok(_) => {}
            Err(_) => {
                warn!(target: "events", "Audio buffer overflow");
            }
        }

        // self.samples.push(sample.0);
        // self.samples.push(sample.1);
    }

    fn sample_rate(&self) -> u64 {
        self.rate
    }
}

impl Drop for CpalSink {
    fn drop(&mut self) {
        let spec = hound::WavSpec {
            channels: 2,
            sample_rate: self.rate as u32,
            bits_per_sample: 32,
            sample_format: hound::SampleFormat::Float,
        };
        let mut writer = hound::WavWriter::create("audio.wav", spec).unwrap();
        for s in self.samples.iter() {
            writer.write_sample(*s).unwrap();
        }
    }
}

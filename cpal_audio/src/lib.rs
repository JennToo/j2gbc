use std::ops::DerefMut;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc, Mutex,
};
use std::thread;

use cpal::{
    traits::{DeviceTrait, EventLoopTrait, HostTrait},
    StreamData, UnknownTypeOutputBuffer,
};
use j2ds::{ElasticPopResult, ElasticRingBuffer};
use log::info;

use j2gbc::AudioSink;

pub struct CpalSink {
    queue: Arc<Mutex<ElasticRingBuffer<(f32, f32)>>>,
    local_queue: Vec<(f32, f32)>,
    rate: u64,

    samples: Vec<f32>,
    channel_buffers: [Vec<f32>; 4],

    capture_config: Arc<CaptureConfig>,
}

impl CpalSink {
    pub fn new() -> Result<CpalSink, String> {
        let host = cpal::default_host();
        let event_loop = host.event_loop();
        let device_o = host.default_output_device();
        if device_o.is_none() {
            return Err("No default output device".into());
        }
        let device = device_o.unwrap();
        let format = device.default_output_format().map_err(|e| e.to_string())?;
        let stream_id = event_loop
            .build_output_stream(&device, &format)
            .map_err(|e| e.to_string())?;
        event_loop.play_stream(stream_id).unwrap();

        let queue = Arc::new(Mutex::new(ElasticRingBuffer::new(
            format.sample_rate.0 as usize / 4,
            (0., 0.),
            format.sample_rate.0 as usize / 8,
        )));
        let q2 = queue.clone();

        thread::spawn(move || {
            feed_cpal_events(&event_loop, q2);
        });

        Ok(CpalSink {
            queue,
            local_queue: Vec::with_capacity(10),
            rate: u64::from(format.sample_rate.0),
            samples: Vec::new(),
            channel_buffers: [Vec::new(), Vec::new(), Vec::new(), Vec::new()],
            capture_config: Arc::new(CaptureConfig::default()),
        })
    }

    pub fn get_capture_config(&self) -> Arc<CaptureConfig> {
        self.capture_config.clone()
    }
}

impl AudioSink for CpalSink {
    fn emit_sample(&mut self, sample: (f32, f32)) {
        self.local_queue.push(sample);
        if self.local_queue.len() >= 10 {
            self.queue
                .lock()
                .unwrap()
                .push_back_slice(self.local_queue.as_slice());
            self.local_queue.clear();
        }

        if self.capture_config.mixed.load(Ordering::Relaxed) {
            self.samples.push(sample.0);
            self.samples.push(sample.1);
        }
    }

    fn sample_rate(&self) -> u64 {
        self.rate
    }

    fn emit_raw_chans(&mut self, chans: [f32; 4]) {
        for ((source, dest), config) in chans
            .iter()
            .zip(self.channel_buffers.iter_mut())
            .zip(self.capture_config.channels.iter())
        {
            if config.load(Ordering::Relaxed) {
                dest.push(*source);
            }
        }
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

        if !self.samples.is_empty() {
            let mut writer = hound::WavWriter::create("target/audio.wav", spec).unwrap();
            for s in &self.samples {
                writer.write_sample(*s).unwrap();
            }
        }

        for i in 0..4 {
            if !self.channel_buffers[i].is_empty() {
                let mut writer =
                    hound::WavWriter::create(format!("target/chan{}.wav", i), spec).unwrap();
                for s in self.channel_buffers[i].iter() {
                    writer.write_sample(*s).unwrap();
                    writer.write_sample(*s).unwrap();
                }
            }
        }
    }
}

fn feed_cpal_events<E: EventLoopTrait>(
    event_loop: &E,
    queue: Arc<Mutex<ElasticRingBuffer<(f32, f32)>>>,
) {
    let mut temp_buffer = Vec::new();
    event_loop.run(move |_, data| {
        if let Ok(StreamData::Output {
            buffer: UnknownTypeOutputBuffer::F32(mut buffer),
        }) = data
        {
            temp_buffer.resize(buffer.deref_mut().len() / 2, (0., 0.));
            let r = queue
                .lock()
                .unwrap()
                .pop_front_slice(temp_buffer.as_mut_slice());

            if r != ElasticPopResult::Exact && r != ElasticPopResult::Empty {
                info!(target: "events", "Pop front result {:?}", r);
            }

            for (i, value) in temp_buffer.iter().enumerate() {
                buffer.deref_mut()[2 * i] = value.0;
                buffer.deref_mut()[2 * i + 1] = value.1;
            }
        }
    });
}

pub struct CaptureConfig {
    pub mixed: AtomicBool,
    pub channels: [AtomicBool; 4],
}

impl Default for CaptureConfig {
    fn default() -> CaptureConfig {
        CaptureConfig {
            mixed: false.into(),
            channels: [false.into(), false.into(), false.into(), false.into()],
        }
    }
}

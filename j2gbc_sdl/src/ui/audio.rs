use std::ops::DerefMut;
use std::sync::{Arc, Mutex};
use std::thread;

use cpal;
use hound;
use j2ds::{ElasticPopResult, ElasticRingBuffer};

use j2gbc::audio::AudioSink;

pub struct CpalSink {
    queue: Arc<Mutex<ElasticRingBuffer<(f32, f32)>>>,
    local_queue: Vec<(f32, f32)>,
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
        })
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
        for s in &self.samples {
            writer.write_sample(*s).unwrap();
        }
    }
}

fn feed_cpal_events(
    event_loop: &cpal::EventLoop,
    queue: Arc<Mutex<ElasticRingBuffer<(f32, f32)>>>,
) {
    let mut temp_buffer = Vec::new();
    event_loop.run(move |_, data| match data {
        cpal::StreamData::Output {
            buffer: cpal::UnknownTypeOutputBuffer::F32(mut buffer),
        } => {
            temp_buffer.resize(buffer.deref_mut().len() / 2, (0., 0.));
            let r = queue
                .lock()
                .unwrap()
                .pop_front_slice(temp_buffer.as_mut_slice());

            if r != ElasticPopResult::Exact {
                info!(target: "events", "Pop front result {:?}", r);
            }

            for i in 0..temp_buffer.len() {
                buffer.deref_mut()[2 * i] = temp_buffer[i].0;
                buffer.deref_mut()[2 * i + 1] = temp_buffer[i].1;
            }
        }

        _ => (),
    });
}

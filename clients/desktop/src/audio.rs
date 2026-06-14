use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use ringbuf::{
    HeapProd, HeapRb,
    traits::{Consumer, Split},
};

pub type AudioProducer = HeapProd<f32>;

pub struct AudioOutput {
    pub stream: cpal::Stream,
    pub producer: AudioProducer,
    pub output_sample_rate: u32,
}

pub fn start() -> Option<AudioOutput> {
    let host = cpal::default_host();
    let device = host.default_output_device()?;

    let default_config = device
        .default_output_config()
        .map_err(|e| tracing::error!("failed to query default output config: {e}"))
        .ok()?;
    let config = default_config.config();
    let output_sample_rate = config.sample_rate;

    let capacity = (output_sample_rate as usize / 60) * 2 * 4;
    let (producer, mut consumer) = HeapRb::<f32>::new(capacity).split();

    let stream = device
        .build_output_stream(
            config,
            move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
                for sample in data.iter_mut() {
                    *sample = consumer.try_pop().unwrap_or(0.0);
                }
            },
            |err| tracing::error!("audio stream error: {err}"),
            None,
        )
        .map_err(|e| tracing::error!("failed to build audio stream: {e}"))
        .ok()?;

    stream
        .play()
        .map_err(|e| tracing::error!("failed to start audio stream: {e}"))
        .ok()?;

    Some(AudioOutput {
        stream,
        producer,
        output_sample_rate,
    })
}

pub struct Resampler {
    step: f64,
    next_output_position: f64,
    previous_frame: (f32, f32),
}

impl Resampler {
    pub fn new(input_rate: u32, output_rate: u32) -> Self {
        Resampler {
            step: input_rate as f64 / output_rate as f64,
            next_output_position: 0.0,
            previous_frame: (0.0, 0.0),
        }
    }

    pub fn push_frame(&mut self, frame: (f32, f32), mut emit: impl FnMut(f32, f32)) {
        let (previous_left, previous_right) = self.previous_frame;
        let (current_left, current_right) = frame;

        while self.next_output_position < 1.0 {
            let fraction = self.next_output_position as f32;
            let left = previous_left + (current_left - previous_left) * fraction;
            let right = previous_right + (current_right - previous_right) * fraction;
            emit(left, right);
            self.next_output_position += self.step;
        }
        self.next_output_position -= 1.0;
        self.previous_frame = frame;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn upsampling_emits_roughly_the_output_rate() {
        let input_rate = 32768;
        let output_rate = 48000;
        let mut resampler = Resampler::new(input_rate, output_rate);

        let mut emitted = 0;
        for _ in 0..input_rate {
            resampler.push_frame((0.5, -0.5), |_, _| emitted += 1);
        }

        // One second of input frames should produce ~one second of output frames.
        let difference = (emitted as i64 - output_rate as i64).abs();
        assert!(difference <= 1, "emitted {emitted}, expected ~{output_rate}");
    }

    #[test]
    fn constant_input_yields_constant_output() {
        let mut resampler = Resampler::new(32768, 48000);
        // Prime previous_frame so interpolation has no edge to ramp from.
        resampler.push_frame((1.0, -1.0), |_, _| {});

        let mut all_constant = true;
        for _ in 0..1000 {
            resampler.push_frame((1.0, -1.0), |left, right| {
                if (left - 1.0).abs() > f32::EPSILON || (right + 1.0).abs() > f32::EPSILON {
                    all_constant = false;
                }
            });
        }
        assert!(all_constant, "constant input must produce constant output");
    }
}

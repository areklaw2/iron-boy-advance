use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use ringbuf::{
    HeapProd, HeapRb,
    traits::{Consumer, Split},
};

//TODO: add resampler
const SAMPLE_RATE: u32 = 32768;

pub type AudioProducer = HeapProd<f32>;

pub fn start() -> Option<(cpal::Stream, AudioProducer)> {
    let host = cpal::default_host();
    let device = host.default_output_device()?;

    let config = cpal::StreamConfig {
        channels: 2,
        sample_rate: SAMPLE_RATE,
        buffer_size: cpal::BufferSize::Default,
    };

    let capacity = (SAMPLE_RATE as usize / 60) * 2 * 4;
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

    Some((stream, producer))
}

use jack::jack_sys::jack_default_audio_sample_t;

type Sample = jack_default_audio_sample_t;

/// A struct that smooths points used in e.g. an audio bar
#[derive(Debug)]
pub(crate) struct Smoother {
	decay_factor: f32,
	last_value: Sample,
}

impl Smoother {
	/// When we get a peak, send it to the smoother as such
	fn peak(&mut self, peak: Sample) {
		if self.last_value < peak {
			self.last_value = peak;
		}
	}

	/// The next value from the smoother
	fn next(&mut self) -> Sample {
		let ret_val = self.last_value;
		self.last_value *= self.decay_factor;
		ret_val
	}
}

impl Default for Smoother {
	fn default() -> Self {
	    Self { decay_factor: 0.9999, last_value: 0.0 }
	}
}

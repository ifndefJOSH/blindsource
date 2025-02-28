use std::fmt::format;

use jack::{jack_sys::jack_default_audio_sample_t, AudioIn, AudioOut, Port};
use nalgebra::{SVector, SMatrix};
use ringbuf::HeapRb;

type sample = jack_default_audio_sample_t;

#[derive(Clone, PartialEq)]
enum Density {
	/// Supergaussian density of function $g = -2 tanh(y_t)$
	Supergaussian,
	/// Subgaussian density of function $g = -y_t^3$ (elementwise)
	Subgaussian,
	/// Subgaussian density of funciton $g = tanh(y_t) - y_t$
	SubgaussianHyperbolicTangent,
}

impl Density {
	/// Maps the samples to a specific density function.
	fn generate_density(&self) -> Box<dyn Fn(sample) -> sample> {
		match self {
		   Self::Supergaussian => Box::new(|y: sample| -2.0 * y.tanh()),
		   Self::Subgaussian => Box::new(|y: sample| -y.powi(3)),
		   Self::SubgaussianHyperbolicTangent => Box::new(|y: sample| y.tanh() - y),
		}
	}
}

struct Separator<const C: usize, const BufSize: usize> {
	density: Density,
	ident: SMatrix<sample, C, C>,
	zeros: SMatrix<sample, C, C>,
	covariance: SMatrix<sample, C, C>, // B_k in the matlab code
	expectation: sample, // mu
	audio_buffer: HeapRb<Box<[sample;BufSize]>>,
	training_iterations: u16,
	// Input and output ports
	input_ports: Vec<Port<AudioIn>>,
	output_ports: Vec<Port<AudioOut>>,
}

impl<const C: usize, const BufSize: usize> Separator<C, BufSize> {
	fn new(
		jack_client: &mut jack::Client,
		dens: Density,
		mu: sample,
		iters: u16,
		ring_buffer_size: usize
	) -> Self {
		Self {
			density: dens,
			ident: SMatrix::identity(),
			zeros: SMatrix::zeros(),
			covariance: SMatrix::identity(),
			expectation: mu,
			audio_buffer: HeapRb::<Box<[sample;BufSize]>>::new(ring_buffer_size),
			training_iterations: iters,
			// Register the input ports with the client
			input_ports: (0..C)
				.map(|i|
					jack_client.register_port(&format!("input_{}", i), jack::AudioIn::default()).unwrap()
				).collect::<Vec<_>>(),
			// Register the output ports with the client
			output_ports: (0..C)
				.map(|i|
					jack_client.register_port(&format!("input_{}", i), jack::AudioOut::default()).unwrap()
				).collect::<Vec<_>>(),
		}
	}

	fn train(&mut self, ps: &jack::ProcessScope) -> jack::Control {
		// Get the
		// Continue
		jack::Control::Continue
	}
}

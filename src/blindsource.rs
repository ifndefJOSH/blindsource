use std::fmt::format;

use jack::{jack_sys::jack_default_audio_sample_t, AudioIn, AudioOut, Port};
use nalgebra::{SVector, SMatrix};
use ringbuf::{traits::{Consumer, RingBuffer}, HeapRb};
use itertools::*;

type sample = jack_default_audio_sample_t;

#[derive(Clone, PartialEq)]
pub(crate) enum Density {
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

pub(crate) trait SeparatorTrait: Send {
	fn train(&mut self, ps: &jack::ProcessScope) -> jack::Control;
}

pub(crate) struct Separator<const C: usize> {
	density: Density,
	ident: SMatrix<sample, C, C>,
	zeros: SMatrix<sample, C, C>,
	covariance: SMatrix<sample, C, C>, // B_k in the matlab code
	mu: sample, // mu
	audio_buffer: HeapRb<Vec<SVector<sample, C>>>,
	training_iterations: u16,
	// Input and output ports
	input_ports: Vec<Port<AudioIn>>,
	output_ports: Vec<Port<AudioOut>>,
}

impl<const C: usize> Separator<C> {
	/// Creates a new separator that automatically connects to a JACK client
	pub(crate) fn new(
		jack_client: &mut jack::Client,
		dens: Density,
		mu_val: sample,
		iters: u16,
		ring_bufsize: usize,
	) -> Self {
		Self {
			density: dens,
			ident: SMatrix::identity(),
			zeros: SMatrix::zeros(),
			covariance: SMatrix::identity(),
			mu: mu_val,
			audio_buffer: HeapRb::<Vec<SVector<sample, C>>>::new(ring_bufsize),
			training_iterations: iters,
			// Register the input ports with the client
			input_ports: (0..C)
				.map(|i|
					jack_client.register_port(
						&format!("input_{}", i),
						jack::AudioIn::default()
					).unwrap()
				).collect::<Vec<_>>(),
			// Register the output ports with the client
			output_ports: (0..C)
				.map(|i|
					jack_client.register_port(
						&format!("input_{}", i),
						jack::AudioOut::default()
					).unwrap()
				).collect::<Vec<_>>(),
		}
	}
}

impl<const C: usize> SeparatorTrait for Separator<C> {

	/// Actually train on a single frame. Or, more acurately, re-train on the entire ring buffer
	/// every time we get a frame. The more aggressively we train the better information we get.
	fn train(&mut self, ps: &jack::ProcessScope) -> jack::Control {
		let training_lambda = self.density.generate_density();
		// Get the current input and put them into the ringbuffer
		let slices = self.input_ports.iter()
			.map(|port| port.as_slice(ps))
			.collect::<Vec<_>>();
		// let mut heap_element = Box::new([0.0 as sample; C]);
		let heap_element = (0..slices[0].len())
			.map(|i| {
				SVector::<sample, C>::from_vec(slices.iter()
					.map(|slice| slice[i])
					.collect::<Vec<_>>())
			})
			.collect::<Vec<_>>();
		// assert!(heap_element.len() == BufSize);
		self.audio_buffer.push_overwrite(heap_element.clone());
		// Do the training
		for _ in 0..self.training_iterations {
			for frame in self.audio_buffer.iter_mut() {
				for channeled_samples in frame.iter_mut() {
					let g = channeled_samples.map(&training_lambda);
					let update_factor = self.ident + g * channeled_samples.transpose();
					self.covariance += self.mu * update_factor * self.covariance;
				}
			}
		}
		// Write the output buffer
		let mut out_slices = self.output_ports.iter_mut()
			.map(|port| port.as_mut_slice(ps))
			.collect::<Vec<_>>(); // Again, we need the entire
		for (i, col) in heap_element.into_iter().enumerate() {
			for (j, sampl) in col.iter().enumerate() {
				out_slices[j][i] = *sampl;
			}
		}

		// Continue to next frame
		jack::Control::Continue
	}
}

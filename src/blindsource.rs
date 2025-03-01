
use jack::{jack_sys::jack_default_audio_sample_t, AudioIn, AudioOut, Port};
use nalgebra::{SVector, SMatrix};
use ringbuf::{traits::{Consumer, RingBuffer}, HeapRb};

type Sample = jack_default_audio_sample_t;

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
	fn generate_density(&self) -> Box<dyn Fn(Sample) -> Sample> {
		match self {
		   Self::Supergaussian => Box::new(|y: Sample| -2.0 * y.tanh()),
		   Self::Subgaussian => Box::new(|y: Sample| -y.powi(3)),
		   Self::SubgaussianHyperbolicTangent => Box::new(|y: Sample| y.tanh() - y),
		}
	}
}

pub(crate) trait SeparatorTrait: Send {
	/// Trains based on the current frame of audio
	fn train(&mut self, ps: &jack::ProcessScope) -> jack::Control;
	fn set_enabled(&mut self, enabled: bool);
	fn is_enabled(&self) -> bool;
	fn set_density(&mut self, density: Density);
	fn get_density(&self) -> Density;
	fn set_training_iters(&mut self, iters: u16);
	fn get_training_iters(&self) -> u16;
}

pub(crate) struct Separator<const C: usize> {
	enabled: bool,
	density: Density,
	ident: SMatrix<Sample, C, C>,
	zeros: SMatrix<Sample, C, C>,
	covariance: SMatrix<Sample, C, C>, // B_k in the matlab code
	mu: Sample, // mu
	audio_buffer: HeapRb<Vec<SVector<Sample, C>>>,
	training_iterations: u16,
	// Input and output ports
	input_ports: Vec<Port<AudioIn>>,
	output_ports: Vec<Port<AudioOut>>,
	// Input and output peaks
	input_peaks: Box<[Sample; C]>,
	output_peaks: Box<[Sample; C]>,
}

impl<const C: usize> Separator<C> {
	/// Creates a new separator that automatically connects to a JACK client
	pub(crate) fn new(
		jack_client: &mut jack::Client,
		dens: Density,
		mu_val: Sample,
		iters: u16,
		ring_bufsize: usize,
	) -> Self {
		Self {
			enabled: true,
			density: dens,
			ident: SMatrix::identity(),
			zeros: SMatrix::zeros(),
			covariance: SMatrix::identity(),
			mu: mu_val,
			audio_buffer: HeapRb::<Vec<SVector<Sample, C>>>::new(ring_bufsize),
			training_iterations: iters,
			// Register the input ports with the client
			input_ports: (0..C)
				.map(|i|
					jack_client.register_port(
						&format!("input{}", i),
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
			input_peaks: Box::new([0.0; C]),
			output_peaks: Box::new([0.0; C]),
		}
	}
}

impl<const C: usize> SeparatorTrait for Separator<C> {

	/// Actually train on a single frame. Or, more acurately, re-train on the entire ring buffer
	/// every time we get a frame. The more aggressively we train the better information we get.
	fn train(&mut self, ps: &jack::ProcessScope) -> jack::Control {
		if !self.enabled {
			return jack::Control::Continue;
		}
		let training_lambda = self.density.generate_density();
		// Get the current input and put them into the ringbuffer
		let slices = self.input_ports.iter()
			.map(|port| port.as_slice(ps))
			.collect::<Vec<_>>();
		// for (i, max_sample) in slices.iter()
		// 	.map(|slice| slice.iter().fold(0.0, |mx, &val|
		// 			if val > mx { val } else { mx }))
		// 	.enumerate() {
		// 	self.input_peaks[i] = max_sample;
		// }
		// let mut heap_element = Box::new([0.0 as sample; C]);
		let heap_element = (0..slices[0].len())
			.map(|i| {
				SVector::<Sample, C>::from_vec(slices.iter()
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
			// self.output_peaks[i] = 0.0;
			for (j, sampl) in col.iter().enumerate() {
				// if *sampl > self.output_peaks[j] {
				// 	self.output_peaks[j] = *sampl;
				// }
				out_slices[j][i] = *sampl;
			}
		}

		// Continue to next frame
		jack::Control::Continue
	}

	fn set_enabled(&mut self, enabled: bool) {
	    self.enabled = enabled;
	}

	fn is_enabled(&self) -> bool {
		self.enabled
	}

	fn set_density(&mut self, density: Density) {
	    self.density = density
	}

	fn get_density(&self) -> Density {
		self.density.clone()
	}

	fn set_training_iters(&mut self, iters: u16) {
	    self.training_iterations = iters;
	}

	fn get_training_iters(&self) -> u16 {
	    self.training_iterations
	}
}

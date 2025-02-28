use std::fmt::{write, Display};
use nalgebra::{SVector, SMatrix};
use ringbuf::{traits::*, HeapRb};

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
	fn generate_density(&self) -> Box<dyn Fn(f64) -> f64> {
		match self {
		   Self::Supergaussian => Box::new(|y: f64| -2.0 * y.tanh()),
		   Self::Subgaussian => Box::new(|y: f64| -y.powi(3)),
		   Self::SubgaussianHyperbolicTangent => Box::new(|y: f64| y.tanh() - y),
		}
	}
}

struct Separator<const C: usize, const BufSize: usize> {
	density: Density,
	ident: SMatrix<f64, C, C>,
	zeros: SMatrix<f64, C, C>,
	covariance: SMatrix<f64, C, C>, // B_k in the matlab code
	mu: f64,
	audio_buffer: HeapRb<Box<[f64;BufSize]>>,
	training_iterations: u16,
}

impl<const C: usize, const BufSize: usize> Separator<C, BufSize> {
	fn new(dens: Density, mu: f64, iters: u16, ring_buffer_size: usize) -> Self {
		Self {
			density: dens,
			ident: SMatrix::identity(),
			zeros: SMatrix::zeros(),
			covariance: SMatrix::identity(),
			mu: mu,
			audio_buffer: HeapRb::<Box<[f64;BufSize]>>::new(ring_buffer_size),
			training_iterations: iters,
		}
	}
}

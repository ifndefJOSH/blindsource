use std::fmt::{write, Display};

#[derive(Display)]
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
	fn generate_density(&self, it: impl IntoIterator<f64>) -> impl IntoIterator<f64> {
		match self {
		   Self::Supergaussian => it.into_iter().map(|y: f64| -2.0 * y.tanh()),
		   Self::Subgaussian => it.into_iter().map(|y: f64| -y.powi(3)),
		   Self::SubgaussianHyperbolicTangent => it.into_iter().map(|y: f64| y.tanh() - y),
		}
	}
}

struct Separator {
	density: Density,
	training_terations: u16,

}

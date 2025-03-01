#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use clap::Parser;
use jack::jack_sys::jack_default_audio_sample_t;
use core::panic;
use std::{process::exit, sync::{Arc, Mutex}};

mod blindsource;

/// A blind source separator of any number of channels
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
	/// Number of channels
	#[arg(short, long, default_value_t = 3)]
	channels: usize,
	/// Training iterations
	#[arg(short, long, default_value_t = 5)]
	training_iters: u16,
	/// Agressiveness factor (mu)
	#[arg(short, long, default_value_t = 0.01)]
	aggressiveness: jack_default_audio_sample_t, // just keep this the same type as the sample
	/// Ring buffer size
	#[arg(short, long, default_value_t = 16)]
	ring_buffer_size: usize,
}

fn main() {
	let args = Args::parse();
	if let Ok((mut client, _status)) = jack::Client::new("SonicSplit", jack::ClientOptions::default()) {
		let shared_separator: Arc<Mutex<Box<dyn blindsource::SeparatorTrait>>> = Arc::new(
			Mutex::new(
				match args.channels {
					1 => Box::new(blindsource::Separator::<1>::new(
						&mut client,
						blindsource::Density::Supergaussian,
						args.aggressiveness,
						args.training_iters,
						args.ring_buffer_size
					)),
					2 => Box::new(blindsource::Separator::<2>::new(
						&mut client,
						blindsource::Density::Supergaussian,
						args.aggressiveness,
						args.training_iters,
						args.ring_buffer_size
					)),
					3 => Box::new(blindsource::Separator::<3>::new(
						&mut client,
						blindsource::Density::Supergaussian,
						args.aggressiveness,
						args.training_iters,
						args.ring_buffer_size
					)),
					4 => Box::new(blindsource::Separator::<4>::new(
						&mut client,
						blindsource::Density::Supergaussian,
						args.aggressiveness,
						args.training_iters,
						args.ring_buffer_size
					)),
					5 => Box::new(blindsource::Separator::<5>::new(
						&mut client,
						blindsource::Density::Supergaussian,
						args.aggressiveness,
						args.training_iters,
						args.ring_buffer_size
					)),
					6 => Box::new(blindsource::Separator::<5>::new(
						&mut client,
						blindsource::Density::Supergaussian,
						args.aggressiveness,
						args.training_iters,
						args.ring_buffer_size
					)),
					_ => panic!("Fuck"),
				}
			)
		);
	}
}

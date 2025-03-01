#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use clap::Parser;
use jack::jack_sys::jack_default_audio_sample_t;
use ui::{create_and_run_ui};
use core::panic;
use std::{io, sync::{Arc, Mutex}};

mod blindsource;
mod smoother;
mod ui;

/// A blind source separator of any number of channels
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
	/// Number of channels
	#[arg(short, long, default_value_t = 3)]
	channels: usize,
	/// Training iterations
	#[arg(short, long, default_value_t = 1)]
	training_iters: u16,
	/// Agressiveness factor (mu)
	#[arg(short, long, default_value_t = 0.01)]
	aggressiveness: jack_default_audio_sample_t, // just keep this the same type as the sample
	/// Ring buffer size
	#[arg(short, long, default_value_t = 1)]
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
						blindsource::Density::Subgaussian,
						args.aggressiveness,
						args.training_iters,
						args.ring_buffer_size
					)),
					2 => Box::new(blindsource::Separator::<2>::new(
						&mut client,
						blindsource::Density::Subgaussian,
						args.aggressiveness,
						args.training_iters,
						args.ring_buffer_size
					)),
					3 => Box::new(blindsource::Separator::<3>::new(
						&mut client,
						blindsource::Density::Subgaussian,
						args.aggressiveness,
						args.training_iters,
						args.ring_buffer_size
					)),
					4 => Box::new(blindsource::Separator::<4>::new(
						&mut client,
						blindsource::Density::Subgaussian,
						args.aggressiveness,
						args.training_iters,
						args.ring_buffer_size
					)),
					5 => Box::new(blindsource::Separator::<5>::new(
						&mut client,
						blindsource::Density::Subgaussian,
						args.aggressiveness,
						args.training_iters,
						args.ring_buffer_size
					)),
					6 => Box::new(blindsource::Separator::<6>::new(
						&mut client,
						blindsource::Density::Subgaussian,
						args.aggressiveness,
						args.training_iters,
						args.ring_buffer_size
					)),
					_ => panic!("Fuck"),
				}
			)
		);
		// The callback handled on each frame of audio
		let pc = register_jack_callback(shared_separator.clone());
		let process = jack::contrib::ClosureProcessHandler::new(pc);
		// Activate the client
		let active_client = client.activate_async((), process).unwrap();

		create_and_run_ui(&shared_separator);

		if let Err(err) = active_client.deactivate() {
			eprintln!("JACK exited with error: {err}");
		};
	}
}

fn register_jack_callback(demixer: Arc<Mutex<Box<dyn blindsource::SeparatorTrait>>>)
	-> impl FnMut(&jack::Client, &jack::ProcessScope) -> jack::Control  {
	let process_callback = {
		let demixer = Arc::clone(&demixer);
		move |_: &jack::Client, ps: &jack::ProcessScope| -> jack::Control {
			match demixer.lock() {
			    Ok(mut owned_demixer) => owned_demixer.train(ps),
				Err(err) => {
					eprintln!("Missed frame! {}", err);
					jack::Control::Continue
				},
			}
		}
	};
	process_callback
}

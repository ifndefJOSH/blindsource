use slint::ModelRc;

use crate::blindsource::{SeparatorTrait, Density};
use std::sync::{Arc, Mutex};

slint::slint!{
	import { ComboBox, SpinBox } from "std-widgets.slint";

	export component Monitor inherits Window {
		in property<float> magnitude: 0.9;

		Rectangle {
			background: #5d7;
			width: clamp(parent.width * magnitude, 0.0, parent.width);
		}
	}

	export component SoundPanel inherits Window {
		min-width: 150px;

		in property<[float]> magnitudes;
		in property<int> num_channels: 6;

		VerticalLayout {
			spacing: 5px;
			min-width: 150px;
			for index in num_channels: Monitor {
				magnitude: magnitudes[index];
			}
		}
	}

	export component SonicSplitWindow inherits Window {
		padding: 20px;
		background: #101519; //000;

		callback iterations-changed(int);
		callback density-changed(string);

		in property<[float]> magnitudes_in;
		in property<[float]> magnitudes_out;
		in property<int> num_channels: 6;
		in property<int> training_iterations;
		in property<string> density;

		VerticalLayout {
			// hackey vertical spacing
			Text{
				text: " ";
				font-size: 10pt;
			}
			Text {
				text: "SonicSplit";
				horizontal-alignment: center;
				font-size: 40pt;
				font-family: "Hack";
				padding-top: 20px;
				color: #fff;
			}

			HorizontalLayout {
				spacing: 5px;
				padding: 20px;
				input_panel := SoundPanel {
					magnitudes: magnitudes_in;
					num_channels: root.num_channels;
				}
				Rectangle {
					logo := Image {
						source: @image-url("resources/icon.svg");
						// colorize: #FFF;
						width: 40%;
					}
					ta := TouchArea {}
				}
				output_panel := SoundPanel {
					magnitudes: magnitudes_out;
					num_channels: root.num_channels;
				}
			}

			HorizontalLayout {
				padding: 20px;
				spacing: 5px;
				Text{
					text: "Density";
					vertical-alignment: center;
				}
				density_box := ComboBox {
					// id: densityBox;
					// width: 100px;
					model: ["Supergaussian", "Subgaussian", "Subgaussian (Hyperbolic Tangent)"];
					current_value: "Supergaussian";
					selected(value) => {
						root.density-changed(value);
					}
				}
				Text {
					text: "Training Iterations";
					vertical-alignment: center;
				}
				iterations_box := SpinBox {
					value: 10;
					edited(value) => {
						root.iterations-changed(value);
					}
				}
			}
			HorizontalLayout {
				Text {
					text: "Made by ifndefJOSH/kernelpanic and CodeTriangle/trongle";
					vertical-alignment: center;
					horizontal-alignment: center;
					font-size: 8px;
				}
				padding: 5px;
			}
		}
	}
}

pub fn create_and_run_ui(demixer: &Arc<Mutex<Box<dyn SeparatorTrait>>>) {
	let win = SonicSplitWindow::new().unwrap();
	{
		let demixer = Arc::clone(&demixer);
		win.on_density_changed(move |name| {
			match demixer.lock() {
				Ok(mut owned_demixer) => {
					match name.as_str() {
						"Supergaussian" => owned_demixer.set_density(Density::Supergaussian),
						"Subgaussian" => owned_demixer.set_density(Density::Subgaussian),
						"Subgaussian (Hyperbolic Tangent)" => owned_demixer.set_density(Density::SubgaussianHyperbolicTangent),
						_ => {
							eprintln!("Invalid combobox index");
						}
					};
				},
				Err(err) => {
					eprintln!("Cannot update density! {}", err);
				},
			}
		});
	};

	// win.density_box().set_on_selected(combobox_callback);
	{
		let demixer = Arc::clone(&demixer);
		win.on_iterations_changed(
			move |itrs: i32| {
				match demixer.lock() {
					Ok(mut owned_demixer) => {
						owned_demixer.set_training_iters(itrs as u16);
					},
					Err(err) => {
						eprintln!("Cannot update density! {}", err);
					},
				}
			}
		);
	};

	if let Ok(dem) = demixer.lock() {
		win.set_num_channels(dem.get_num_channels());
		let demo_slice: ModelRc<f32> = (0..dem.get_num_channels())
			.map(|x| x as f32 * 0.1 + 0.1)
			.collect::<Vec<_>>()
			.as_slice()
			.into();
		win.set_magnitudes_in(demo_slice.clone());
		win.set_magnitudes_out(demo_slice);
		win.set_training_iterations(dem.get_training_iters() as i32);
	}

	win.run().unwrap();
}

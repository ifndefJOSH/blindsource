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
		VerticalLayout {
			spacing: 5px;

			in property<float> amount: 10;
			for index in amount: Monitor {}
		}
	}

	export component SonicSplitWindow inherits Window {
		padding: 20px;
		background: #000;
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
			}

			HorizontalLayout {
				spacing: 5px;
				padding: 20px;
				input_panel := SoundPanel {}
				Rectangle {
					logo := Image {
						source: @image-url("resources/icon.svg");
						// colorize: #FFF;
						width: 40%;
					}
					ta := TouchArea {}
				}
				output_panel := SoundPanel {}
			}

			HorizontalLayout {
				padding: 20px;
				spacing: 5px;
				Text{
					text: "Density";
					vertical-alignment: center;
				}
				ComboBox {
					// id: densityBox;
					// width: 100px;
					model: ["Supergaussian", "Subgaussian", "Subgaussian (Hyperbolic Tangent"];
					current_value: "Supergaussian";
				}
				Text {
					text: "Training Iterations";
					vertical-alignment: center;
				}
				SpinBox {
					value: 10;
				}
			}
		}
	}
}

pub fn create_and_run_ui(demixer: &Arc<Mutex<Box<dyn SeparatorTrait>>>) {
	let win = SonicSplitWindow::new().unwrap();
	let combobox_callback = {
		let demixer = Arc::clone(&demixer);
		move |index| {
			match demixer.lock() {
				Ok(mut owned_demixer) => {
					match index {
						0 => owned_demixer.set_density(Density::Supergaussian),
						1 => owned_demixer.set_density(Density::Subgaussian),
						2 => owned_demixer.set_density(Density::SubgaussianHyperbolicTangent),
						_ => {
							eprintln!("Invalid combobox index");
						}
					};
				},
				Err(err) => {
					eprintln!("Cannot update density! {}", err);
				},
			}
		}
	};
	// win.density_box().set_on_selected(combobox_callback);
	win.run().unwrap();
}

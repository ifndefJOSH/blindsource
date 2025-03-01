slint::slint!{
	export component Monitor inherits Window {
		in property<float> magnitude;

		Rectangle {
			background: #5d7;
			width: clamp(parent.width * magnitude, 0.0, parent.width);
		}
	}

	export component SoundPanel inherits Window {
		VerticalLayout {
			spacing: 5px;

			in property<float> amount;
			for index in amount: Monitor {}
		}
	}

	export component SonicSplitWindow inherits Window {
		VerticalLayout {
			Text {
				text: "SonicSplit";
			}

			HorizontalLayout {
				input_panel := SoundPanel {}
				Rectangle {
					logo := Image {
						source: @image-url("resources/full_logo.svg");
					}
					ta := TouchArea {}
				}
				output_panel := SoundPanel {}
			}
		}
	}
}

pub fn create_and_run_ui() {
	let win = SonicSplitWindow::new().unwrap();

	win.run().unwrap();
}

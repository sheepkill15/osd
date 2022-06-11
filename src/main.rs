mod overlay;

use std::env;

use gtk::prelude::*;
use overlay::Overlay;

fn main() {
    let args: Vec<String> = env::args().collect();
    let mut value: Option<String> = None;
    let mut icon: Option<String> = None;
    let mut text: Option<String> = None;
    let mut state = 0;
    if args.len() < 2 {
        print_help();
        return;
    }
    for i in 1..args.len() {
        let arg = args[i].to_string();
        match arg.trim() {
            "-v" => {
                state = 1;
            }
            "-i" => {
                state = 2;
            }
            "-t" => {
                state = 3;
            }
            &_ => {
                match state {
                    1 => {
                        value = Some(arg);
                    }
                    2 => {
                        icon = Some(arg);
                    }
                    3 => {
                        text = Some(arg);
                    }
                    _ => {
                        println!("Unknown argument: {}", arg.to_string());
                        return;
                    }
                }
                state = 0;
            }
        }
    }

    let application = gtk::Application::new(Some("com.github.sheepkill15.osd"), Default::default());
    application.connect_activate(move |app| {
        build_ui(app, &value, &text, &icon);
    });
    application.run_with_args(&[""]);
}

fn build_ui(application: &gtk::Application, val: &Option<String>, text: &Option<String>, icon: &Option<String>) {
    let val_conv = if let Some(value) = val {Some(value.to_string())} else {None}; 
    let text_conv = if let Some(value) = text {Some(value.to_string())} else {None}; 
    let icon_conv = if let Some(value) = icon {Some(value.to_string())} else {None}; 
    let _overlay = Overlay::new(
        application,
        val_conv,
        text_conv,
        icon_conv,
    );
}

fn print_help() {
    print!(r#"Correct usage:
    -v [value]      Specify percentage value
    -t [value]      Specify text to display
    -i [value]      Specify icon file or name
    "#);
    println!();
}
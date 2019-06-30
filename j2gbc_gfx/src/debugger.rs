use enclose::enclose;
use gtk::prelude::*;

use crate::SystemRef;

const DEBUGGER_UI: &str = include_str!("../../assets/ui/debugger.glade");

#[derive(Clone)]
struct Context {
    system: SystemRef,
    pause_button: gtk::ToolButton,
    resume_button: gtk::ToolButton,
    step_button: gtk::ToolButton,
}

pub fn load_debugger(system: &SystemRef) -> gtk::Window {
    let builder = gtk::Builder::new_from_string(DEBUGGER_UI);
    let window: gtk::Window = builder.get_object("debugger_window").unwrap();
    let context = Context::from_builder(system.clone(), builder);
    context.running();

    context
        .pause_button
        .connect_clicked(enclose!((context) move |_| {
            context.system.borrow_mut().debugger().pause();
            context.halted();
        }));
    context
        .resume_button
        .connect_clicked(enclose!((context) move |_| {
            context.system.borrow_mut().debugger().resume();
            context.running();
        }));

    window.show_all();
    window
}

impl Context {
    pub fn from_builder(system: SystemRef, builder: gtk::Builder) -> Context {
        Context {
            system,
            pause_button: builder.get_object("pause_button").unwrap(),
            resume_button: builder.get_object("resume_button").unwrap(),
            step_button: builder.get_object("step_button").unwrap(),
        }
    }

    pub fn running(&self) {
        self.resume_button.set_sensitive(false);
        self.step_button.set_sensitive(false);

        self.pause_button.set_sensitive(true);
    }

    pub fn halted(&self) {
        self.resume_button.set_sensitive(true);
        self.step_button.set_sensitive(true);

        self.pause_button.set_sensitive(false);
    }
}

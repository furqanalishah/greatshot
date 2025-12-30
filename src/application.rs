use gettextrs::gettext;
use adw::prelude::*;
use adw::subclass::prelude::*;
use gtk::{gio, glib};
use crate::config::VERSION;
use crate::window::GreatshotWindow;

mod imp {
    use super::*;

    #[derive(Debug, Default)]
    pub struct GreatshotApplication {}

    #[glib::object_subclass]
    impl ObjectSubclass for GreatshotApplication {
        const NAME: &'static str = "GreatshotApplication";
        type Type = super::GreatshotApplication;
        type ParentType = adw::Application;
    }

    impl ObjectImpl for GreatshotApplication {
        fn constructed(&self) {
            self.parent_constructed();
            let obj = self.obj();
            obj.setup_gactions();
            obj.set_accels_for_action("app.quit", &["<control>q"]);
        }
    }

    impl ApplicationImpl for GreatshotApplication {
        fn activate(&self) {
            let application = self.obj();
            let window = application.active_window().unwrap_or_else(|| {
                let window = GreatshotWindow::new(application.upcast_ref::<adw::Application>());
                window.upcast()
            });
            window.present();
        }
    }

    impl GtkApplicationImpl for GreatshotApplication {}
    impl AdwApplicationImpl for GreatshotApplication {}
}

glib::wrapper! {
    pub struct GreatshotApplication(ObjectSubclass<imp::GreatshotApplication>)
        @extends gio::Application, gtk::Application, adw::Application,
        @implements gio::ActionGroup, gio::ActionMap;
}

impl GreatshotApplication {
    pub fn new(application_id: &str, flags: &gio::ApplicationFlags) -> Self {
        glib::Object::builder()
            .property("application-id", application_id)
            .property("flags", flags)
            .property("resource-base-path", "/io/github/syed/greatshot")
            .build()
    }

    fn setup_gactions(&self) {
        let quit_action = gio::ActionEntry::builder("quit")
            .activate(move |app: &Self, _, _| app.quit())
            .build();
        let about_action = gio::ActionEntry::builder("about")
            .activate(move |app: &Self, _, _| app.show_about())
            .build();
        self.add_action_entries([quit_action, about_action]);
    }

    fn show_about(&self) {
        let window = self.active_window().unwrap();
        let about = adw::AboutDialog::builder()
            .application_name("greatshot")
            .application_icon("io.github.syed.greatshot")
            .developer_name("Furqan Ali Shah")
            .version(VERSION)
            .developers(vec!["Furqan Ali Shah"])
            .translator_credits(&gettext("translator-credits"))
            .copyright("© 2025 Furqan Ali Shah")
            .build();
        about.present(Some(&window));
    }
}

use std::{
    f32::consts::PI,
    sync::{Arc, Mutex, MutexGuard},
    time::Duration,
};

use gtk::{
    prelude::WidgetExtManual,
    traits::{GtkWindowExt, IconThemeExt, WidgetExt},
    ApplicationWindow, IconLookupFlags,
};

use gdk::{gdk_pixbuf::Pixbuf, prelude::GdkContextExt, Screen};

use cairo::Context;

use glib::SourceId;

use std::cmp::min;

pub struct Overlay {
    base_width: i32,
    base_height: i32,
    base_font_size: i32,
    base_line_width: i32,
    screen_margin: i32,
    base_padding: i32,
    bg_opacity: f32,
    bg_corner_radius: i32,
    mute_opacity: f32,
    text_opacity: f32,
    num_bars: i32,
    opacity: f32,

    compositing: bool,
    fadeout_timeout: Option<SourceId>,
    hide_timeout: Option<SourceId>,

    window: ApplicationWindow,
    value: Option<String>,
    text: Option<String>,
    icon: Option<String>,
}

impl Overlay {
    pub fn new(
        application: &gtk::Application,
        value: Option<String>,
        text: Option<String>,
        icon: Option<String>,
    ) -> Arc<Mutex<Overlay>> {
        let overlay = Arc::new(Mutex::new(Overlay {
            window: ApplicationWindow::new(application),
            base_width: 200,
            base_height: 200,
            base_font_size: 42,
            base_line_width: 5,
            screen_margin: 64,
            base_padding: 24,
            bg_opacity: 0.85,
            bg_corner_radius: 8,
            mute_opacity: 1.0,
            text_opacity: 0.8,
            num_bars: 16,
            opacity: 1.0,
            compositing: false,
            fadeout_timeout: None,
            hide_timeout: None,
            value,
            text,
            icon,
        }));
        Overlay::init(&overlay);
        overlay
    }

    fn init(overlay: &Arc<Mutex<Overlay>>) {
        let osd_for_draw = overlay.clone();
        let osd_for_draw2 = overlay.clone();
        let osd_for_draw3 = overlay.clone();
        {
            let mut osd = overlay.lock().unwrap();
            osd.window.set_default_size(osd.base_width, osd.base_height);
            osd.window.set_decorated(false);
            osd.window.set_resizable(false);
            osd.window.set_title("OSD");
            osd.window.stick();
            osd.window.set_skip_taskbar_hint(false);
            osd.window.set_skip_pager_hint(false);
            osd.window.set_keep_above(true);
            osd.window
                .set_type_hint(gtk::gdk::WindowTypeHint::Notification);
            osd.window.set_resizable(false);

            if let Some(screen) = gdk::Screen::default() {
                let visua = screen.rgba_visual();
                if let Some(visual) = visua {
                    if screen.is_composited() {
                        println!("COMPOSITED!!");
                        osd.compositing = true;
                        osd.window.set_visual(Some(&visual));
                    }
                }
            }

            osd.window.set_app_paintable(true);
            osd.window.connect_draw(move |_, cont| {
                Overlay::draw_osd(Arc::clone(&osd_for_draw), cont);
                gtk::Inhibit(true)
            });
            osd.window.realize();
            osd.window.window().unwrap().set_override_redirect(true);
            osd.move_to_position();
            osd.window.show();
            osd.hide_timeout = Some(glib::timeout_add_local(
                Duration::from_millis(1000),
                move || {
                    Overlay::cb_hide_timeout(Arc::clone(&osd_for_draw2));
                    glib::Continue(false)
                },
            ));
            osd.fadeout_timeout = Some(glib::timeout_add_local(
                Duration::from_millis(30),
                move || glib::Continue(Overlay::cb_fadeout_timeout(Arc::clone(&osd_for_draw3))),
            ));
        }
    }

    fn move_to_position(&self) {
        if let Some(screen) = Screen::default() {
            let marginx = -self.screen_margin;
            let marginy = -self.screen_margin;
            let geometry = screen.monitor_geometry(screen.primary_monitor());
            let mut xpos = geometry.x();
            let mut ypos = geometry.y();
            let swidth = geometry.width();
            let sheight = geometry.height();
            xpos = marginx + xpos + swidth - self.base_width; // right
            ypos = marginy + ypos + sheight - self.base_height; // bottom
            self.window.move_(xpos, ypos);
        }
    }

    fn draw_osd(overlay: Arc<Mutex<Overlay>>, context: &Context) {
        let osd = overlay.lock().unwrap();
        let xcenter = osd.base_width / 2;
        let deg = PI / 180.0;
        context.new_sub_path();
        context.arc(
            (osd.base_width - osd.bg_corner_radius).into(),
            osd.bg_corner_radius.into(),
            osd.bg_corner_radius.into(),
            (-90.0 * deg).into(),
            0.0,
        );
        context.arc(
            (osd.base_width - osd.bg_corner_radius).into(),
            (osd.base_height - osd.bg_corner_radius).into(),
            osd.bg_corner_radius.into(),
            0.0,
            (90.0 * deg).into(),
        );
        context.arc(
            osd.bg_corner_radius.into(),
            (osd.base_height - osd.bg_corner_radius).into(),
            osd.bg_corner_radius.into(),
            (90.0 * deg).into(),
            (180.0 * deg).into(),
        );
        context.arc(
            osd.bg_corner_radius.into(),
            osd.bg_corner_radius.into(),
            osd.bg_corner_radius.into(),
            (180.0 * deg).into(),
            (270.0 * deg).into(),
        );
        context.close_path();

        context.set_source_rgba(0.1, 0.1, 0.1, (osd.bg_opacity * osd.opacity).into());
        context.set_operator(cairo::Operator::Source);
        if let Result::Err(err) = context.fill() {
            println!("Error on fill: {}", err.to_string());
            return;
        }
        context.set_operator(cairo::Operator::Over);

        context.set_source_rgba(
            1.0,
            1.0,
            1.0,
            (osd.text_opacity * osd.mute_opacity * osd.opacity).into(),
        );
        let mut text_height = 10.0;
        if let Some(text) = &osd.text {
            context.select_font_face(
                "sans-serif",
                cairo::FontSlant::Normal,
                cairo::FontWeight::Normal,
            );
            context.set_font_size(osd.base_font_size.into());
            let extents = context.text_extents(&text);
            if let Result::Ok(ext) = extents {
                context.move_to(
                    xcenter as f64 - ext.width / 2.0,
                    (osd.base_height - osd.base_padding).into(),
                );
                if let Result::Err(err) = context.show_text(&text) {
                    println!("Error on text: {}", err.to_string());
                }
                text_height = ext.height;
            } else if let Result::Err(err) = extents {
                println!("Error on extents: {}", err.to_string());
            }
        }

        if let Some(value) = &osd.value {
            if let Ok(value) = value.parse::<i32>() {
                let ind_height = (osd.base_height - 3 * osd.base_padding) as f64 - text_height;
                let outer_radius = ind_height / 2.0;
                let inner_radius = outer_radius / 1.5;
                let bars = min(
                    (osd.num_bars as f64 * value as f64 / 100.0).floor() as i32,
                    osd.num_bars,
                );
                context.set_line_width(osd.base_line_width.into());
                context.set_line_cap(cairo::LineCap::Round);
                for i in 0..bars {
                    context.identity_matrix();
                    context.translate(xcenter.into(), osd.base_padding as f64 + ind_height / 2.0);
                    context.rotate((PI + 2.0 * PI / osd.num_bars as f32 * i as f32).into());
                    context.move_to(0.0, -inner_radius);
                    context.line_to(0.0, -outer_radius);
                    if let Err(err) = context.stroke() {
                        println!("Error on stroke: {}", err.to_string());
                        return;
                    }
                }
            }
        }

        if let Some(icon) = &osd.icon {
            context.identity_matrix();
            context.translate((osd.base_width / 2).into(), (osd.base_height / 2).into());
            context.translate(0.0, -text_height + osd.bg_corner_radius as f64 / 2.0);
            let draw = |img: Pixbuf| {
                context.scale(48.0 / img.width() as f64, 48.0 / img.height() as f64);
                context.translate((-img.width() / 2).into(), (-img.height() / 2).into());
                context.set_source_pixbuf(&img, 0.0, 0.0);
                if let Err(err) = context.paint_with_alpha(osd.opacity.into()) {
                    println!("Error on paint: {}", err.to_string());
                }
            };

            if let Some(icon_theme) = gtk::IconTheme::default() {
                if let Some(icon_info) = icon_theme.lookup_icon(icon, 48, IconLookupFlags::empty())
                {
                    if let Ok(image) = icon_info.load_icon() {
                        draw(image);
                    }
                }
            } else {
                if let Ok(image) = Pixbuf::from_file(icon) {
                    draw(image);
                }
            }
        }
    }

    fn hide(osd: &mut MutexGuard<Overlay>) {
        if osd.compositing {
            osd.compositing = false;
        } else {
            unsafe { osd.window.destroy() };
        }
    }

    fn cb_hide_timeout(overlay: Arc<Mutex<Overlay>>) {
        let mut osd = overlay.lock().unwrap();
        if let Some(timeout) = osd.hide_timeout.take() {
            glib::SourceId::remove(timeout);
        }
        Overlay::hide(&mut osd);
    }

    fn cb_fadeout_timeout(overlay: Arc<Mutex<Overlay>>) -> bool {
        let mut osd = overlay.lock().unwrap();
        if osd.compositing {
            return true;
        }
        osd.opacity -= 0.05;
        osd.window.queue_draw();
        if osd.opacity >= 0.0 {
            return true;
        }
        osd.opacity = 0.0;
        if let Some(timeout) = osd.fadeout_timeout.take() {
            glib::SourceId::remove(timeout);
        }
        unsafe {
            osd.window.destroy();
        }
        return false;
    }
}

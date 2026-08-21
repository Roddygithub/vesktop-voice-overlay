use gtk4::prelude::*;
use gtk4::DrawingArea;
use std::rc::Rc;
use std::sync::Mutex;

pub struct SpeakingIndicator {
    drawing_area: DrawingArea,
    speaking: Rc<Mutex<bool>>,
}

impl SpeakingIndicator {
    pub fn new(speaking: bool) -> Self {
        let drawing_area = DrawingArea::new();
        drawing_area.set_size_request(8, 8);
        drawing_area.add_css_class("speaking-indicator");

        let speaking = Rc::new(Mutex::new(speaking));
        if *speaking.lock().unwrap() {
            drawing_area.add_css_class("active");
        }
        let speaking_clone = speaking.clone();

        drawing_area.set_draw_func(move |_, cr, _, _| {
            let is_speaking = *speaking_clone.lock().unwrap();
            let width = 6.0_f64;
            let height = 6.0_f64;
            let radius = width.min(height) / 2.0 - 1.0;
            let center_x = width / 2.0;
            let center_y = height / 2.0;

            cr.arc(center_x, center_y, radius, 0.0, 2.0 * std::f64::consts::PI);
            cr.set_source_rgba(0.0, 1.0, 0.0, if is_speaking { 1.0 } else { 0.0 });
            cr.fill().ok();
        });

        Self {
            drawing_area,
            speaking,
        }
    }

    pub fn widget(&self) -> &DrawingArea {
        &self.drawing_area
    }

    pub fn set_speaking(&self, speaking: bool) {
        *self.speaking.lock().unwrap() = speaking;
        if speaking {
            self.drawing_area.add_css_class("active");
        } else {
            self.drawing_area.remove_css_class("active");
        }
        self.drawing_area.queue_draw();
    }
}

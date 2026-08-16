use gtk4::prelude::*;
use gtk4::{DrawingArea, Widget};
use std::sync::Arc;

pub struct SpeakingIndicator {
    drawing_area: DrawingArea,
    speaking: Arc<std::sync::Mutex<bool>>,
}

impl SpeakingIndicator {
    pub fn new(speaking: bool) -> Self {
        let drawing_area = DrawingArea::new();
        drawing_area.set_size_request(8, 8);
        drawing_area.add_css_class("speaking-indicator");
        
        let speaking = Arc::new(std::sync::Mutex::new(speaking));
        let speaking_clone = speaking.clone();
        let drawing_area_clone = drawing_area.clone();
        
        drawing_area.set_draw_func(move |_, cr, _, _| {
            let is_speaking = *speaking_clone.lock().unwrap();
            if is_speaking {
                drawing_area_clone.add_css_class("active");
            } else {
                drawing_area_clone.remove_css_class("active");
            }
            
            // Draw a simple circle
            let width = drawing_area_clone.width() as f64;
            let height = drawing_area_clone.height() as f64;
            let radius = width.min(height) / 2.0 - 1.0;
            let center_x = width / 2.0;
            let center_y = height / 2.0;
            
            cr.arc(center_x, center_y, radius, 0.0, 2.0 * std::f64::consts::PI);
            cr.set_source_rgba(0.0, 1.0, 0.0, if is_speaking { 1.0 } else { 0.0 });
            cr.fill().ok();
        });

        Self { drawing_area, speaking }
    }

    pub fn widget(&self) -> &DrawingArea {
        &self.drawing_area
    }

    pub fn set_speaking(&self, speaking: bool) {
        *self.speaking.lock().unwrap() = speaking;
        self.drawing_area.queue_draw();
    }
}

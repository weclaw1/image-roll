use gtk::EventControllerScrollFlags;

#[derive(Clone)]
pub struct Controllers {
    window_key_event_controller: gtk::EventControllerKey,
    image_click_gesture: gtk::GestureClick,
    image_motion_event_controller: gtk::EventControllerMotion,
    image_zoom_gesture: gtk::GestureZoom,
    image_scrolled_window_scroll_controller: gtk::EventControllerScroll,
}

impl Controllers {
    pub fn init() -> Self {
        Self {
            window_key_event_controller: gtk::EventControllerKey::new(),
            image_click_gesture: gtk::GestureClick::new(),
            image_motion_event_controller: gtk::EventControllerMotion::new(),
            image_zoom_gesture: gtk::GestureZoom::new(),
            image_scrolled_window_scroll_controller: gtk::EventControllerScroll::new(
                EventControllerScrollFlags::BOTH_AXES,
            ),
        }
    }

    pub fn image_click_gesture(&self) -> &gtk::GestureClick {
        &self.image_click_gesture
    }

    pub fn image_motion_event_controller(&self) -> &gtk::EventControllerMotion {
        &self.image_motion_event_controller
    }

    pub fn image_zoom_gesture(&self) -> &gtk::GestureZoom {
        &self.image_zoom_gesture
    }

    pub fn window_key_event_controller(&self) -> &gtk::EventControllerKey {
        &self.window_key_event_controller
    }

    pub fn image_scrolled_window_scroll_controller(&self) -> &gtk::EventControllerScroll {
        &self.image_scrolled_window_scroll_controller
    }
}

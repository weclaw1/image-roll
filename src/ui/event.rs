use gtk::{gdk::Rectangle, gio, glib};
use std::path::PathBuf;

use crate::{image::PreviewSize, image_operation::ImageOperation};

#[derive(Debug)]
pub enum Event {
    OpenFile(gio::File),
    LoadImage(Option<PathBuf>),
    ImageViewportResize(Rectangle),
    RefreshPreview(PreviewSize),
    ChangePreviewSize(PreviewSize),
    ImageEdit(ImageOperation),
    StartSelection((u32, u32)),
    DragSelection((u32, u32)),
    SaveCurrentImage(Option<PathBuf>),
    DeleteCurrentImage,
    EndSelection,
    PreviewSmaller,
    PreviewLarger,
    PreviewFitScreen,
    NextImage,
    PreviousImage,
    RefreshFileList,
    ResizePopoverDisplayed,
    UpdateResizePopoverWidth,
    UpdateResizePopoverHeight,
    UndoOperation,
    RedoOperation,
    Print,
    DisplayError(anyhow::Error),
    HideErrorPanel,
}

pub fn post_event(sender: &glib::Sender<Event>, action: Event) {
    if let Err(err) = sender.send(action) {
        error!("Send error: {}", err);
    }
}

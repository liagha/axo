#[cfg(feature = "dialog")]
use axo::dialog::Dialog;
use axo::internal::Session;

fn main() {
    let session = Session::new().compile();

    if !session.has_input() {
        #[cfg(feature = "dialog")]
        Dialog::start(session);
    }
}

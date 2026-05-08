use axo::internal::Session;
#[cfg(feature = "dialog")]
use axo::dialog::Dialog;

fn main() {
    let session = Session::new().compile();

    if !session.has_input() {
        #[cfg(feature = "dialog")]
        Dialog::start(session);
    }
}

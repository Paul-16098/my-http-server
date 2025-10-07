use std::sync::{ Mutex, OnceLock };

// A global mutex to serialize tests that touch ./meta and env vars.
pub(crate) fn meta_mutex() -> &'static Mutex<()> {
  static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
  LOCK.get_or_init(|| Mutex::new(()))
}

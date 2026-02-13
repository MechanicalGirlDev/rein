//! Camera controls
//!
//! Provides various camera control schemes for interactive 3D viewing.

mod first_person;
mod fly;
mod orbit;

pub use first_person::FirstPersonControl;
pub use fly::FlyControl;
pub use orbit::OrbitControl;

//! Simple math operations missing in `stylus_sdk::storage`.
mod checked;
mod unchecked;

pub(crate) use checked::AddAssignChecked;
pub(crate) use unchecked::{AddAssignUnchecked, SubAssignUnchecked};

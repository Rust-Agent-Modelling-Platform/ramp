use uuid::Uuid;
use std::fmt;

#[derive(Debug)]
pub enum Action {
    Death(Uuid),
    Procreation(Uuid, Uuid),
    Migration(Uuid),
    Meeting(Uuid, Uuid),
    None(Uuid)
}


impl fmt::Display for Action {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Action::Death(ref id) => write!(f, "  Die({})", &id.to_string()[..5]),
            Action::Procreation(ref id1, ref id2) => write!(f, "  Procreate({}, {})", &id1.to_string()[..5], &id2.to_string()[..5]),
            Action::Migration(ref id) => write!(f, "  Migrate({})", &id.to_string()[..5]),
            Action::Meeting(ref id1, ref id2) => write!(f, "  Meet({}, {})", &id1.to_string()[..5], &id2.to_string()[..5]),
            Action::None(ref id) => write!(f, "  None({})", &id.to_string()[..5]),
        }
    }
}
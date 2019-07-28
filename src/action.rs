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
            Action::Death(ref id) => write!(f, "  Death({})", &id.to_string()[..5]),
            Action::Procreation(ref id1, ref id2) => write!(f, "  Procreation({}, {})", &id1.to_string()[..5], &id2.to_string()[..5]),
            Action::Migration(ref id) => write!(f, "  Migration({})", &id.to_string()[..5]),
            Action::Meeting(ref id1, ref id2) => write!(f, "  Meeting({}, {})", &id1.to_string()[..5], &id2.to_string()[..5]),
            Action::None(ref id) => write!(f, "  None({})", &id.to_string()[..5]),
        }
    }
}

//non-ownership-taking implementation
impl PartialEq for Action {
    fn eq(&self, other: &Self) -> bool {
        use Action::*;
        match (self, other) {
            (&Death(ref uuid1), Death(ref uuid2)) => uuid1 == uuid2,
            (&Migration(ref uuid1), &Migration(ref uuid2)) => uuid1 == uuid2,
            (&None(ref uuid1), &None(ref uuid2)) => uuid1 == uuid2,
            (&Meeting(ref uuid1, ref uuid2), &Meeting(ref uuid3, ref uuid4)) => uuid1 == uuid3 && uuid2 == uuid4,
            (&Procreation(ref uuid1, ref uuid2), &Procreation(ref uuid3, ref uuid4)) => uuid1 == uuid3 && uuid2 == uuid4,
            _ => false
        }
    }
}
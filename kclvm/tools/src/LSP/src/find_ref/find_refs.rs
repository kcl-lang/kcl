use crate::find_ref;
use crate::find_ref::go_to_def::go_to_def;
use kclvm_error::Position;

/// Find all references of the item at the cursor location.
pub fn find_refs(path: String, pos: Position) -> Vec<Position> {
    let declaration = go_to_def(pos.clone());
    let search = {
        move |decl: Position| {
            let name = find_ref::word_at_pos(pos);
            if name.is_none() {
                return vec![];
            }
            // Get identifiers with same name
            let candidates = find_ref::match_word(path, name.unwrap());
            // Check if the definition of candidate and declartion are the same
            let refs: Vec<Position> = candidates
                .into_iter()
                .filter(|x| go_to_def(x.clone()).as_ref() == Some(&decl))
                .collect();
            refs
        }
    };
    match declaration {
        Some(decl) => search(decl),
        None => Vec::new(),
    }
}

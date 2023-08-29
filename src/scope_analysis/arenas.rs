use id_arena::Arena;

use super::{scope::_Scope, definition::_Definition};

#[derive(Default)]
pub struct AllArenas<'a> {
    scopes: Arena<_Scope<'a>>,
    definitions: Arena<_Definition<'a>>,
}

use id_arena::Arena;

use super::{scope::_Scope, definition::_Definition};

#[derive(Default)]
pub struct AllArenas<'a> {
    pub scopes: Arena<_Scope<'a>>,
    pub definitions: Arena<_Definition<'a>>,
}

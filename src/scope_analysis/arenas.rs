use id_arena::Arena;

use super::{scope::_Scope, definition::_Definition, variable::_Variable, reference::_Reference};

#[derive(Default)]
pub struct AllArenas<'a> {
    pub scopes: Arena<_Scope<'a>>,
    pub definitions: Arena<_Definition<'a>>,
    pub variables: Arena<_Variable<'a>>,
    pub references: Arena<_Reference<'a>>,
}

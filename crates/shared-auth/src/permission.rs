pub enum Permission {
    ReadOnly,
    ReadWrite,
}

impl Permission {
    pub fn contains(&self, other: &Permission) -> bool {
        match (self, other) {
            (Permission::ReadWrite, _) => true,
            (Permission::ReadOnly, Permission::ReadOnly) => true,
            (Permission::ReadOnly, _) => false,
        }
    }
}

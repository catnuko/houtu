use bevy::render::define_atomic_id;

define_atomic_id!(CreditId);
pub struct Credit {
    pub id: CreditId,
    pub html: &'static str,
    pub show: bool,
}
impl Default for Credit {
    fn default() -> Self {
        Self {
            id: CreditId::new(),
            html: "",
            show: true,
        }
    }
}
impl Credit {
    
}

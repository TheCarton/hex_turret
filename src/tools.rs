use bevy::ecs::{entity::Entity, system::Query};

pub(crate) fn flatten_option_entity(
    option_entity: Option<Entity>,
    query: Query<Entity>,
) -> Option<Entity> {
    if let Some(entity) = option_entity {
        if query.get(entity).is_ok() {
            return Some(entity);
        }
    }
    None
}

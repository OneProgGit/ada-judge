use ada_judge_public_models::users::AdminLevel;

pub fn is_allowed(user_id: i64, owner_id: Option<i64>, admin_level: &AdminLevel) -> bool {
    if admin_level == &AdminLevel::Owner {
        return true;
    }
    if owner_id.is_none() {
        return false;
    }
    if let Some(owner_id) = owner_id
        && owner_id != user_id
    {
        return false;
    }
    return true;
}
